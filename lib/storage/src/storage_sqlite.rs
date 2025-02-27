use std::collections::HashMap;
use std::path::Path;
use std::sync::Mutex;

use crate::{Storage, StorageError};
use anyhow::{anyhow, bail, ensure, Context, Error, Result};
use model::{
    backup::{
        Backup, BundleBackup, FoodBackup, JournalBackup, SportActivityBackup, SportBackup,
        UserSettingsBackup, WeightBackup,
    },
    Bundle, Food, Journal, JournalReport, Meal, Sport, SportActivity, SportActivityReport,
    UserSettings, Weight,
};
use rusqlite::{
    functions::FunctionFlags, params, types::Value, Connection, Error::SqliteFailure, Params,
    Transaction,
};
use serde_json::json;
use types::timestamp::Timestamp;

mod migrations;
mod queries;

#[cfg(test)]
mod test;

pub struct StorageSqlite {
    conn: Mutex<Connection>,
}

impl StorageSqlite {
    pub fn new(db_file: &Path) -> Result<Self> {
        let conn = Connection::open(format!(
            "file:{}?mode=rwc&_timeout=5000&_fk=1&_sync=1&_journal=wal",
            db_file.to_str().unwrap(),
        ))
        .context("open db connection")?;

        Self::add_custom_functions(&conn).context("add custom functions")?;

        let s = Self {
            conn: Mutex::new(conn),
        };

        s.init().context("storage init")?;
        s.apply_migrations().context("storage apply migrations")?;

        Ok(s)
    }

    fn init(&self) -> Result<()> {
        // Create system table if not exists
        self.raw_execute(queries::CREATE_TABLE_SYSTEM, false, params![])
            .context("exec create system table")
    }

    fn apply_migrations(&self) -> Result<()> {
        let last_migration_id = self
            .get_last_migration_id()
            .context("get last migration id")?;

        let mut conn = self.conn.lock().unwrap();
        migrations::apply(&mut conn, last_migration_id).context("apply list of migrations")
    }

    fn raw_query<P>(&self, query: &str, params: P) -> Result<Vec<HashMap<String, Value>>>
    where
        P: Params,
    {
        let mut conn = self.conn.lock().unwrap();
        let tx = conn
            .transaction()
            .context("failed to get transaction")
            .unwrap();

        // tx when dropped is rollbacked - it's Ok for query
        Self::raw_query_tx(&tx, query, params)
    }

    fn raw_execute<P>(&self, query: &str, batch: bool, params: P) -> Result<()>
    where
        P: Params,
    {
        let mut conn = self.conn.lock().unwrap();
        let tx = conn
            .transaction()
            .context("failed to get transaction")
            .unwrap();

        Self::raw_execute_tx(&tx, query, batch, params)?;
        tx.commit().context("failed to commit transaction")?;

        Ok(())
    }

    fn get_last_migration_id(&self) -> Result<i64> {
        let res = self
            .raw_query(queries::SELECT_MIGRATION_ID, params![])
            .context("query last migration")?;
        if res.is_empty() {
            return Ok(0);
        }

        let row = res.first().unwrap();
        if let Some(Value::Integer(val)) = row.get("migration_id") {
            return Ok(*val);
        }

        bail!("migration_id not found");
    }

    fn raw_query_tx<P>(
        tx: &Transaction,
        query: &str,
        params: P,
    ) -> Result<Vec<HashMap<String, Value>>>
    where
        P: Params,
    {
        let mut stmt = tx.prepare(query).context("prepare raw query")?;
        let mut rows = stmt.query(params).context("quering raw query")?;

        let mut col_names: Vec<String> = Default::default();
        let mut col_cnt: usize = Default::default();
        let mut first = true;

        let mut res = Vec::new();
        while let Some(row) = rows.next().context("get next row")? {
            if first {
                col_names = row
                    .as_ref()
                    .column_names()
                    .iter()
                    .map(|&s| s.to_string())
                    .collect();
                col_cnt = col_names.len();
                first = false;
            }

            let mut res_row = HashMap::with_capacity(col_cnt);
            for i in 0..col_cnt {
                res_row.insert(
                    col_names.get(i).unwrap().clone(),
                    Value::from(
                        row.get_ref(i)
                            .with_context(|| anyhow!(format!("get column {i}")))?,
                    ),
                );
            }
            res.push(res_row);
        }
        Ok(res)
    }

    fn raw_execute_tx<P>(tx: &Transaction, query: &str, batch: bool, params: P) -> Result<()>
    where
        P: Params,
    {
        if !batch {
            tx.execute(query, params).context("raw execute query")?;
        } else {
            tx.execute_batch(query).context("raw execute batch query")?;
        }

        Ok(())
    }

    fn add_custom_functions(conn: &Connection) -> Result<()> {
        conn.create_scalar_function(
            "r_upper",
            1,
            FunctionFlags::SQLITE_UTF8 | FunctionFlags::SQLITE_DETERMINISTIC,
            move |ctx| {
                assert_eq!(ctx.len(), 1, "called with unexpected number of arguments");
                Ok(ctx.get_raw(0).as_str()?.to_uppercase())
            },
        )
        .map_err(|e| anyhow!(e))
    }

    fn get_timestamp(row: &HashMap<String, Value>, field: &str) -> Result<Timestamp> {
        let Some(Value::Integer(ts)) = row.get(field) else {
            bail!("failed to get \"{field}\" field");
        };

        let Some(ts) = Timestamp::from_unix_millis(*ts) else {
            bail!("failed to parse \"{field}\" field");
        };

        Ok(ts)
    }

    fn get_float(row: &HashMap<String, Value>, field: &str) -> Result<f64> {
        let Some(Value::Real(val)) = row.get(field) else {
            bail!("failed to get \"{field}\" field")
        };

        Ok(*val)
    }

    fn get_integer(row: &HashMap<String, Value>, field: &str) -> Result<i64> {
        let Some(Value::Integer(val)) = row.get(field) else {
            bail!("failed to get \"{field}\" field")
        };

        Ok(*val)
    }

    fn get_string(row: &HashMap<String, Value>, field: &str) -> Result<String> {
        let Some(Value::Text(val)) = row.get(field) else {
            bail!("failed to get \"{field}\" field")
        };

        Ok(val.clone())
    }

    fn get_bundle_food_items(
        tx: &Transaction,
        user_id: i64,
        bndl_key: &str,
    ) -> Result<HashMap<String, f64>> {
        let mut bundles: Vec<String> = vec![bndl_key.into()];
        let mut res = HashMap::new();
        let mut i = 0;

        while i < bundles.len() {
            // Get next bundle
            let db_res = Self::raw_query_tx(
                tx,
                queries::SELECT_BUNDLE,
                params![user_id, bundles.get(i).unwrap()],
            )
            .context("get bundle query")?;

            if db_res.is_empty() {
                bail!(StorageError::BundleNotFound)
            }

            // Parse bundle data
            let json_data = Self::get_string(db_res.first().unwrap(), "data")
                .context("get bundle data field")?;
            let data: HashMap<String, f64> =
                serde_json::from_str(&json_data).context("convert bundle data from JSON")?;

            for (k, v) in &data {
                if *v == 0.0 {
                    // Add bundle next bundle
                    bundles.push(k.clone());
                    continue;
                }

                // Check if food exists add to result map
                let db_res = Self::raw_query_tx(tx, queries::SELECT_FOOD, params![k])
                    .context("get food query")?;

                if db_res.is_empty() {
                    bail!(StorageError::FoodNotFound)
                }

                res.insert(k.clone(), *v);
            }

            i += 1;
        }

        Ok(res)
    }
}

impl Storage for StorageSqlite {
    //
    // Food
    //

    fn get_food(&self, key: &str) -> Result<Food> {
        let db_res = self
            .raw_query(queries::SELECT_FOOD, params![key])
            .context("get food query")?;

        ensure!(!db_res.is_empty(), StorageError::FoodNotFound);

        let row = db_res.first().unwrap();

        Ok(Food {
            key: Self::get_string(row, "key").context("get food key field")?,
            name: Self::get_string(row, "name").context("get food name field")?,
            brand: Self::get_string(row, "brand").context("get food brand field")?,
            cal100: Self::get_float(row, "cal100").context("get food cal100 field")?,
            prot100: Self::get_float(row, "prot100").context("get food prot100 field")?,
            fat100: Self::get_float(row, "fat100").context("get food fat100 field")?,
            carb100: Self::get_float(row, "carb100").context("get food carb100 field")?,
            comment: Self::get_string(row, "comment").context("get food comment field")?,
        })
    }

    fn get_food_list(&self) -> Result<Vec<Food>> {
        let db_res = self
            .raw_query(queries::SELECT_FOOD_LIST, params![])
            .context("get food list query")?;

        ensure!(!db_res.is_empty(), StorageError::EmptyResult);

        let mut food_list = Vec::with_capacity(db_res.len());
        for row in &db_res {
            food_list.push(Food {
                key: Self::get_string(row, "key").context("get food key field")?,
                name: Self::get_string(row, "name").context("get food name field")?,
                brand: Self::get_string(row, "brand").context("get food brand field")?,
                cal100: Self::get_float(row, "cal100").context("get food cal100 field")?,
                prot100: Self::get_float(row, "prot100").context("get food prot100 field")?,
                fat100: Self::get_float(row, "fat100").context("get food fat100 field")?,
                carb100: Self::get_float(row, "carb100").context("get food carb100 field")?,
                comment: Self::get_string(row, "comment").context("get food comment field")?,
            });
        }

        Ok(food_list)
    }

    fn set_food(&self, food: &Food) -> Result<()> {
        ensure!(food.validate(), StorageError::FoodInvalid);

        self.raw_execute(
            queries::UPSERT_FOOD,
            false,
            params![
                food.key,
                food.name,
                food.brand,
                food.cal100,
                food.prot100,
                food.fat100,
                food.carb100,
                food.comment
            ],
        )
        .context("exec upsert food")
    }

    fn find_food(&self, pattern: &str) -> Result<Vec<Food>> {
        let db_res = self
            .raw_query(queries::FIND_FOOD, params![pattern.to_uppercase()])
            .context("find food list query")?;

        ensure!(!db_res.is_empty(), StorageError::EmptyResult);

        let mut food_list = Vec::with_capacity(db_res.len());
        for row in &db_res {
            food_list.push(Food {
                key: Self::get_string(row, "key").context("get food key field")?,
                name: Self::get_string(row, "name").context("get food name field")?,
                brand: Self::get_string(row, "brand").context("get food brand field")?,
                cal100: Self::get_float(row, "cal100").context("get food cal100 field")?,
                prot100: Self::get_float(row, "prot100").context("get food prot100 field")?,
                fat100: Self::get_float(row, "fat100").context("get food fat100 field")?,
                carb100: Self::get_float(row, "carb100").context("get food carb100 field")?,
                comment: Self::get_string(row, "comment").context("get food comment field")?,
            });
        }

        Ok(food_list)
    }

    fn delete_food(&self, key: &str) -> Result<()> {
        let mut conn = self.conn.lock().unwrap();
        let tx = conn.transaction().context("failed to get transaction")?;

        // Check that food not used in bundle
        let db_res = Self::raw_query_tx(&tx, queries::SELECT_ALL_BUNDLES, params![])
            .context("get all bundles query")?;

        for row in &db_res {
            let json_data = Self::get_string(row, "data").context("get bundle data field")?;
            let data: HashMap<String, f64> =
                serde_json::from_str(&json_data).context("convert bundle data from JSON")?;

            for (k, v) in &data {
                if *v > 0.0 && k == key {
                    bail!(StorageError::FoodIsUsed)
                }
            }
        }

        // Delete food
        if let Err(err) = Self::raw_execute_tx(&tx, queries::DELETE_FOOD, false, params![key])
            .context("exec delete food")
        {
            for cause in err.chain() {
                if let Some(SqliteFailure(_, Some(val))) = cause.downcast_ref::<rusqlite::Error>() {
                    if val == "FOREIGN KEY constraint failed" {
                        bail!(StorageError::FoodIsUsed)
                    };

                    bail!(err);
                }
            }

            bail!(err);
        };

        tx.commit().context("failed to commit transaction")?;

        Ok(())
    }

    //
    // Bundle
    //

    fn get_bundle(&self, user_id: i64, key: &str) -> Result<Bundle> {
        let db_res = self
            .raw_query(queries::SELECT_BUNDLE, params![user_id, key])
            .context("get bundle query")?;

        ensure!(!db_res.is_empty(), StorageError::BundleNotFound);

        let row = db_res.first().unwrap();

        let json_data = Self::get_string(row, "data").context("get bundle data field")?;
        let data: HashMap<String, f64> =
            serde_json::from_str(&json_data).context("convert bundle data from JSON")?;

        Ok(Bundle {
            key: Self::get_string(row, "key").context("get bundle key field")?,
            data,
        })
    }

    fn get_bundle_list(&self, user_id: i64) -> Result<Vec<Bundle>> {
        let db_res = self
            .raw_query(queries::SELECT_BUNDLE_LIST, params![user_id])
            .context("get bundle list query")?;

        ensure!(!db_res.is_empty(), StorageError::EmptyResult);

        let mut res = Vec::with_capacity(db_res.len());
        for row in &db_res {
            let json_data = Self::get_string(row, "data").context("get bundle data field")?;
            let data: HashMap<String, f64> =
                serde_json::from_str(&json_data).context("convert bundle data from JSON")?;

            res.push(Bundle {
                key: Self::get_string(row, "key").context("get bundle key field")?,
                data,
            });
        }

        Ok(res)
    }

    fn set_bundle(&self, user_id: i64, bndl: &Bundle) -> Result<()> {
        ensure!(bndl.validate(), StorageError::BundleInvalid);

        let mut conn = self.conn.lock().unwrap();
        let tx = conn.transaction().context("failed to get transaction")?;

        // Check bundle data
        for (k, v) in &bndl.data {
            if *v == 0.0 {
                // Dependent bundle
                if *k == bndl.key {
                    bail!(StorageError::BundleDepRecursive)
                }

                let db_res = Self::raw_query_tx(&tx, queries::SELECT_BUNDLE, params![user_id, k])
                    .context("get bundle query")?;

                if db_res.is_empty() {
                    bail!(StorageError::BundleDepBundleNotFound)
                }
            } else {
                // Dependent food
                let db_res = Self::raw_query_tx(&tx, queries::SELECT_FOOD, params![k])
                    .context("get food query")?;

                if db_res.is_empty() {
                    bail!(StorageError::BundleDepFoodNotFound)
                }
            }
        }

        // Set bundle
        let data =
            serde_json::to_string(&json!(bndl.data)).context("convert bundle data to JSON")?;

        Self::raw_execute_tx(
            &tx,
            queries::UPSERT_BUNDLE,
            false,
            params![user_id, bndl.key, data],
        )?;
        tx.commit().context("failed to commit transaction")?;

        Ok(())
    }

    fn delete_bundle(&self, user_id: i64, key: &str) -> Result<()> {
        let mut conn = self.conn.lock().unwrap();
        let tx = conn.transaction().context("failed to get transaction")?;

        // Check that bundle not used in other bundles
        let db_res = Self::raw_query_tx(&tx, queries::SELECT_BUNDLE_LIST, params![user_id])
            .context("get bundle list query")?;

        for row in &db_res {
            let json_data = Self::get_string(row, "data").context("get bundle data field")?;
            let data: HashMap<String, f64> =
                serde_json::from_str(&json_data).context("convert bundle data from JSON")?;

            for (k, v) in &data {
                if *v == 0.0 && k == key {
                    bail!(StorageError::BundleIsUsed)
                }
            }
        }

        // Delete bundle
        Self::raw_execute_tx(&tx, queries::DELETE_BUNDLE, false, params![user_id, key])?;
        tx.commit().context("failed to commit transaction")?;

        Ok(())
    }

    //
    // Weight
    //

    fn get_weight_list(&self, user_id: i64, from: Timestamp, to: Timestamp) -> Result<Vec<Weight>> {
        let db_res = self
            .raw_query(
                queries::SELECT_WEIGHT_LIST,
                params![user_id, from.unix_millis(), to.unix_millis()],
            )
            .context("weight list query")?;

        ensure!(!db_res.is_empty(), StorageError::EmptyResult);

        let mut res = Vec::with_capacity(db_res.len());
        for row in &db_res {
            res.push(Weight {
                timestamp: Self::get_timestamp(row, "timestamp")
                    .context("get weight timestamp field")?,
                value: Self::get_float(row, "value").context("get weight value field")?,
            });
        }

        Ok(res)
    }

    fn set_weight(&self, user_id: i64, weight: &Weight) -> Result<()> {
        ensure!(weight.validate(), StorageError::WeightInvalid);

        self.raw_execute(
            queries::UPSERT_WEIGHT,
            false,
            params![user_id, weight.timestamp.unix_millis(), weight.value],
        )
        .context("exec upsert weight")
    }

    fn delete_weight(&self, user_id: i64, timestamp: Timestamp) -> Result<()> {
        self.raw_execute(
            queries::DELETE_WEIGHT,
            false,
            params![user_id, timestamp.unix_millis()],
        )
        .context("exec delete weight")
    }

    //
    // User settings
    //

    fn get_user_settings(&self, user_id: i64) -> Result<UserSettings> {
        let db_res = self
            .raw_query(queries::SELECT_USER_SETTINGS, params![user_id])
            .context("get user settings query")?;

        ensure!(!db_res.is_empty(), StorageError::UserSettingsNotFound);

        let row = db_res.first().unwrap();

        Ok(UserSettings {
            cal_limit: Self::get_float(row, "cal_limit").context("get cal_limit field")?,
        })
    }

    fn set_user_settings(&self, user_id: i64, settings: &UserSettings) -> Result<()> {
        ensure!(settings.validate(), StorageError::UserSettingsInvalid);

        self.raw_execute(
            queries::UPSERT_USER_SETTINGS,
            false,
            params![user_id, settings.cal_limit],
        )
        .context("exec upsert user settings")
    }

    //
    // Journal
    //

    fn set_journal(&self, user_id: i64, journal: &Journal) -> Result<()> {
        ensure!(journal.validate(), StorageError::JournalInvalid);

        match self.raw_execute(
            queries::UPSERT_JOURNAL,
            false,
            params![
                user_id,
                journal.timestamp.unix_millis(),
                u8::from(journal.meal),
                journal.food_key,
                journal.food_weight,
            ],
        ) {
            Err(err) => {
                for cause in err.chain() {
                    if let Some(SqliteFailure(_, Some(val))) =
                        cause.downcast_ref::<rusqlite::Error>()
                    {
                        if val == "FOREIGN KEY constraint failed" {
                            bail!(StorageError::FoodNotFound)
                        };

                        bail!(err);
                    }
                }

                bail!(err);
            }
            _ => Ok(()),
        }
    }

    fn set_journal_bundle(
        &self,
        user_id: i64,
        timestamp: Timestamp,
        meal: Meal,
        bndl_key: &str,
    ) -> Result<()> {
        let mut conn = self.conn.lock().unwrap();
        let tx = conn.transaction().context("failed to get transaction")?;

        let food_items = Self::get_bundle_food_items(&tx, user_id, bndl_key)?;
        for (k, v) in food_items {
            Self::raw_execute_tx(
                &tx,
                queries::UPSERT_JOURNAL,
                false,
                params![user_id, timestamp.unix_millis(), u8::from(meal), k, v],
            )?;
        }

        tx.commit().context("failed to commit transaction")
    }

    fn delete_journal(
        &self,
        user_id: i64,
        timestamp: Timestamp,
        meal: Meal,
        food_key: &str,
    ) -> Result<()> {
        self.raw_execute(
            queries::DELETE_JOURNAL,
            false,
            params![user_id, timestamp.unix_millis(), u8::from(meal), food_key],
        )
        .context("exec delete journal")
    }

    fn delete_journal_meal(&self, user_id: i64, timestamp: Timestamp, meal: Meal) -> Result<()> {
        self.raw_execute(
            queries::DELETE_JOURNAL_MEAL,
            false,
            params![user_id, timestamp.unix_millis(), u8::from(meal)],
        )
        .context("exec delete journal")
    }

    fn get_journal_report(
        &self,
        user_id: i64,
        from: Timestamp,
        to: Timestamp,
    ) -> Result<Vec<JournalReport>> {
        let db_res = self
            .raw_query(
                queries::JOURNAL_REPORT,
                params![user_id, from.unix_millis(), to.unix_millis()],
            )
            .context("get journal reportl query")?;

        ensure!(!db_res.is_empty(), StorageError::EmptyResult);

        let mut report = Vec::with_capacity(db_res.len());
        for row in &db_res {
            report.push(JournalReport {
                timestamp: Self::get_timestamp(row, "timestamp").context("get timestamp field")?,
                meal: Meal::new(Self::get_integer(row, "meal").context("get meal field")? as u8)
                    .context("wrong meal")?,
                food_key: Self::get_string(row, "foodkey").context("get foodkey field")?,
                food_name: Self::get_string(row, "foodname").context("get foodname field")?,
                food_brand: Self::get_string(row, "foodbrand").context("get foodbrand field")?,
                food_weight: Self::get_float(row, "foodweight").context("get foodweight field")?,
                cal: Self::get_float(row, "cal").context("get cal field")?,
                prot: Self::get_float(row, "prot").context("get prot field")?,
                fat: Self::get_float(row, "fat").context("get fat field")?,
                carb: Self::get_float(row, "carb").context("get carb field")?,
            });
        }

        Ok(report)
    }

    fn get_journal_food_avg_weight(
        &self,
        user_id: i64,
        food_key: &str,
        from: Timestamp,
        to: Timestamp,
    ) -> Result<f64> {
        let db_res = self
            .raw_query(
                queries::JOURNAL_FOOD_AVG_WEIGHT,
                params![user_id, food_key, from.unix_millis(), to.unix_millis()],
            )
            .context("get journal food avg weight query")?;

        Self::get_float(db_res.first().unwrap(), "avg_food_weight")
            .context("get avg_food_weight field")
    }

    //
    // Sport
    //

    fn get_sport(&self, key: &str) -> Result<Sport> {
        let db_res = self
            .raw_query(queries::SELECT_SPORT, params![key])
            .context("get sport query")?;

        ensure!(!db_res.is_empty(), StorageError::SportNotFound);

        let row = db_res.first().unwrap();

        Ok(Sport {
            key: Self::get_string(row, "key").context("get sport key field")?,
            name: Self::get_string(row, "name").context("get food sport field")?,
            comment: Self::get_string(row, "comment").context("get sport comment field")?,
        })
    }

    fn get_sport_list(&self) -> Result<Vec<Sport>> {
        let db_res = self
            .raw_query(queries::SELECT_SPORT_LIST, params![])
            .context("get sport list query")?;

        ensure!(!db_res.is_empty(), StorageError::EmptyResult);

        let mut sport_list = Vec::with_capacity(db_res.len());
        for row in &db_res {
            sport_list.push(Sport {
                key: Self::get_string(row, "key").context("get sport key field")?,
                name: Self::get_string(row, "name").context("get sport name field")?,
                comment: Self::get_string(row, "comment").context("get sport comment field")?,
            });
        }

        Ok(sport_list)
    }

    fn set_sport(&self, sport: &Sport) -> Result<()> {
        ensure!(sport.validate(), StorageError::SportInvalid);

        self.raw_execute(
            queries::UPSERT_SPORT,
            false,
            params![sport.key, sport.name, sport.comment],
        )
        .context("exec upsert sport")
    }

    fn delete_sport(&self, key: &str) -> Result<()> {
        match self
            .raw_execute(queries::DELETE_SPORT, false, params![key])
            .context("exec delete sport")
        {
            Err(err) => {
                for cause in err.chain() {
                    if let Some(SqliteFailure(_, Some(val))) =
                        cause.downcast_ref::<rusqlite::Error>()
                    {
                        if val == "FOREIGN KEY constraint failed" {
                            bail!(StorageError::SportIsUsedViolation)
                        };

                        bail!(err);
                    }
                }

                bail!(err);
            }
            _ => Ok(()),
        }
    }

    //
    // Sport activity
    //

    fn set_sport_activity(&self, user_id: i64, act: &SportActivity) -> Result<()> {
        ensure!(act.validate(), StorageError::SportActivityInvalid);

        // Convert sets to JSON array
        let str_sets = serde_json::to_string(&json!(act.sets))
            .context("convert sport activity sets to JSON")?;

        match self.raw_execute(
            queries::UPSERT_SPORT_ACTIVITY,
            false,
            params![
                user_id,
                act.timestamp.unix_millis(),
                act.sport_key,
                str_sets
            ],
        ) {
            Err(err) => {
                for cause in err.chain() {
                    if let Some(SqliteFailure(_, Some(val))) =
                        cause.downcast_ref::<rusqlite::Error>()
                    {
                        if val == "FOREIGN KEY constraint failed" {
                            bail!(StorageError::SportInvalid)
                        };

                        bail!(err);
                    }
                }

                bail!(err);
            }
            _ => Ok(()),
        }
    }

    fn delete_sport_activity(
        &self,
        user_id: i64,
        timestamp: Timestamp,
        sport_key: &str,
    ) -> Result<()> {
        self.raw_execute(
            queries::DELETE_SPORT_ACTIVITY,
            false,
            params![user_id, timestamp.unix_millis(), sport_key],
        )
        .context("exec delete sport activity")
    }

    fn get_sport_activity_report(
        &self,
        user_id: i64,
        from: Timestamp,
        to: Timestamp,
    ) -> Result<Vec<SportActivityReport>> {
        let db_res = self
            .raw_query(
                queries::SELECT_SPORT_ACTIVITY_REPORT,
                params![user_id, from.unix_millis(), to.unix_millis()],
            )
            .context("sport activity report query")?;

        ensure!(!db_res.is_empty(), StorageError::EmptyResult);

        let mut res = Vec::with_capacity(db_res.len());
        for row in &db_res {
            let json_sets = Self::get_string(row, "sets").context("get sets field")?;
            let sets: Vec<i64> =
                serde_json::from_str(&json_sets).context("convert sets from JSON")?;

            res.push(SportActivityReport {
                sport_name: Self::get_string(row, "sport_name").context("get sport name field")?,
                timestamp: Self::get_timestamp(row, "timestamp").context("get timestamp field")?,
                sets,
            });
        }

        Ok(res)
    }

    //
    // Backup/Restore
    //

    fn backup(&self, user_id: i64) -> Result<Backup> {
        // Weight
        let db_res = self
            .raw_query(queries::SELECT_WEIGHT_FOR_BACKUP, params![])
            .context("select weight backup query")?;

        let mut weight_backup = Vec::with_capacity(db_res.len());
        for row in db_res {
            weight_backup.push(WeightBackup {
                user_id: Self::get_integer(&row, "user_id").context("get user_id field")?,
                timestamp: Self::get_timestamp(&row, "timestamp")
                    .context("get timestamp field")?
                    .unix_millis(),
                value: Self::get_float(&row, "value").context("get value field")?,
            });
        }

        // Food
        let db_res = self
            .raw_query(queries::SELECT_FOOD_FOR_BACKUP, params![])
            .context("select food backup query")?;

        let mut food_backup = Vec::with_capacity(db_res.len());
        for row in db_res {
            food_backup.push(FoodBackup {
                user_id,
                key: Self::get_string(&row, "key").context("get key field")?,
                name: Self::get_string(&row, "name").context("get name field")?,
                brand: Self::get_string(&row, "brand").context("get brand field")?,
                cal100: Self::get_float(&row, "cal100").context("get cal100 field")?,
                prot100: Self::get_float(&row, "prot100").context("get prot100 field")?,
                fat100: Self::get_float(&row, "fat100").context("get fat100 field")?,
                carb100: Self::get_float(&row, "carb100").context("get carb100 field")?,
                comment: Self::get_string(&row, "comment").context("get comment field")?,
            });
        }

        // User settings
        let db_res = self
            .raw_query(queries::SELECT_USER_SETTINGS_FOR_BACKUP, params![])
            .context("select user settings backup query")?;

        let mut us_backup = Vec::with_capacity(db_res.len());
        for row in db_res {
            us_backup.push(UserSettingsBackup {
                user_id: Self::get_integer(&row, "user_id").context("get user_id field")?,
                cal_limit: Self::get_float(&row, "cal_limit").context("get cal_limit field")?,
            });
        }

        // Bundles
        let db_res: Vec<HashMap<String, Value>> = self
            .raw_query(queries::SELECT_BUNDLES_FOR_BACKUP, params![])
            .context("select bundles backup query")?;

        let mut bundle_backup = Vec::with_capacity(db_res.len());
        for row in db_res {
            bundle_backup.push(BundleBackup {
                user_id: Self::get_integer(&row, "user_id").context("get user_id field")?,
                key: Self::get_string(&row, "key").context("get key field")?,
                data: Self::get_string(&row, "data").context("get bundle data field")?,
            });
        }

        // Journal
        let db_res: Vec<HashMap<String, Value>> = self
            .raw_query(queries::SELECT_JOURNAL_FOR_BACKUP, params![])
            .context("select journal backup query")?;

        let mut journal_backup = Vec::with_capacity(db_res.len());
        for row in db_res {
            journal_backup.push(JournalBackup {
                user_id: Self::get_integer(&row, "user_id").context("get user_id field")?,
                timestamp: Self::get_timestamp(&row, "timestamp")
                    .context("get timestamp field")?
                    .unix_millis(),
                meal: Self::get_integer(&row, "meal").context("get meal field")? as u8,
                food_key: Self::get_string(&row, "foodkey").context("get foodkey field")?,
                food_weight: Self::get_float(&row, "foodweight").context("get foodweight field")?,
            });
        }

        // Sport
        let db_res: Vec<HashMap<String, Value>> = self
            .raw_query(queries::SELECT_SPORT_FOR_BACKUP, params![])
            .context("select sport backup query")?;

        let mut sport_backup = Vec::with_capacity(db_res.len());
        for row in db_res {
            sport_backup.push(SportBackup {
                user_id,
                key: Self::get_string(&row, "key").context("get key field")?,
                name: Self::get_string(&row, "name").context("get name field")?,
                comment: Self::get_string(&row, "comment").context("get comment field")?,
            });
        }

        // Sport activity
        let db_res: Vec<HashMap<String, Value>> = self
            .raw_query(queries::SELECT_SPORT_ACTIVITY_FOR_BACKUP, params![])
            .context("select sport activity backup query")?;

        let mut sa_backup = Vec::with_capacity(db_res.len());
        for row in db_res {
            sa_backup.push(SportActivityBackup {
                user_id: Self::get_integer(&row, "user_id").context("get user_id field")?,
                timestamp: Self::get_timestamp(&row, "timestamp")
                    .context("get timestamp field")?
                    .unix_millis(),
                sport_key: Self::get_string(&row, "sport_key").context("get sport_key field")?,
                sets: Self::get_string(&row, "sets").context("get sets field")?,
            });
        }

        Ok(Backup {
            timestamp: Timestamp::now().unix_millis(),
            food: food_backup,
            weight: weight_backup,
            user_settings: us_backup,
            bundle: bundle_backup,
            journal: journal_backup,
            sport: sport_backup,
            sport_activity: sa_backup,
        })
    }

    fn restore(&self, backup: &Backup) -> Result<()> {
        for w in &backup.weight {
            self.raw_execute(
                queries::UPSERT_WEIGHT,
                false,
                params![w.user_id, w.timestamp, w.value],
            )
            .context("exec upsert backup weight")?;
        }

        for f in &backup.food {
            self.raw_execute(
                queries::UPSERT_FOOD,
                false,
                params![
                    f.key, f.name, f.brand, f.cal100, f.prot100, f.fat100, f.carb100, f.comment
                ],
            )
            .context("exec upsert backup food")?;
        }

        for us in &backup.user_settings {
            self.raw_execute(
                queries::UPSERT_USER_SETTINGS,
                false,
                params![us.user_id, us.cal_limit],
            )
            .context("exec upsert backup user settings")?;
        }

        for b in &backup.bundle {
            self.raw_execute(
                queries::UPSERT_BUNDLE,
                false,
                params![b.user_id, b.key, b.data],
            )
            .context("exec upsert backup bundle")?;
        }

        for j in &backup.journal {
            self.raw_execute(
                queries::UPSERT_JOURNAL,
                false,
                params![j.user_id, j.timestamp, j.meal, j.food_key, j.food_weight],
            )
            .context("exec upsert backup journal")?;
        }

        for s in &backup.sport {
            self.raw_execute(
                queries::UPSERT_SPORT,
                false,
                params![s.key, s.name, s.comment],
            )
            .context("exec upsert backup sport")?;
        }

        for sa in &backup.sport_activity {
            self.raw_execute(
                queries::UPSERT_SPORT_ACTIVITY,
                false,
                params![sa.user_id, sa.timestamp, sa.sport_key, sa.sets],
            )
            .context("exec upsert backup sport activity")?;
        }

        Ok(())
    }

    //
    // Error
    //

    fn is_storage_error(&self, stg_err: StorageError, err: &Error) -> bool {
        stg_err
            == *err
                .root_cause()
                .downcast_ref::<StorageError>()
                .unwrap_or(&StorageError::default())
    }
}
