use std::collections::HashMap;
use std::path::Path;
use std::sync::Mutex;

use crate::{Storage, StorageError};
use anyhow::{anyhow, bail, ensure, Context, Error, Result};
use model::{backup::Backup, Bundle, Food, Sport, UserSettings, Weight};
use rusqlite::{functions::FunctionFlags, params, types::Value, Connection, Params};
use types::timestamp::Timestamp;

mod migrations;
mod queries;

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
        let conn = self.conn.lock().unwrap();

        let mut stmt = conn.prepare(query).context("prepare raw query")?;
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

    fn raw_execute<P>(&self, query: &str, batch: bool, params: P) -> Result<()>
    where
        P: Params,
    {
        let conn = self.conn.lock().unwrap();

        if !batch {
            conn.execute(query, params).context("raw execute query")?;
        } else {
            conn.execute_batch(query)
                .context("raw execute batch query")?;
        }

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

    fn get_string(row: &HashMap<String, Value>, field: &str) -> Result<String> {
        let Some(Value::Text(val)) = row.get(field) else {
            bail!("failed to get \"{field}\" field")
        };

        Ok(val.clone())
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

        ensure!(!db_res.is_empty(), StorageError::NotFound);

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

        ensure!(!db_res.is_empty(), StorageError::EmptyList);

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
        ensure!(food.validate(), StorageError::InvalidFood);

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

        ensure!(!db_res.is_empty(), StorageError::EmptyList);

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
        self.raw_execute(queries::DELETE_FOOD, false, params![key])
            .context("exec delete food")
    }

    //
    // Bundle
    //

    fn get_bundle(&self, user_id: i64, key: &str) -> Result<Bundle> {
        todo!()
    }

    fn get_bundle_list(&self, user_id: i64) -> Result<Vec<Bundle>> {
        todo!()
    }

    fn set_bundle(&self, user_id: i64, bndl: &Bundle) -> Result<()> {
        todo!()
    }

    fn delete_bundle(&self, user_id: i64, key: &str) -> Result<()> {
        todo!()
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

        ensure!(!db_res.is_empty(), StorageError::EmptyList);

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
        ensure!(weight.validate(), StorageError::InvalidWeight);

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
        todo!()
    }

    fn set_user_settings(&self, user_id: i64, settings: &UserSettings) -> Result<()> {
        todo!()
    }

    //
    // Sport
    //

    fn get_sport_list(&self) -> Result<Vec<Sport>> {
        let db_res = self
            .raw_query(queries::SELECT_SPORT_LIST, params![])
            .context("get sport list query")?;

        ensure!(!db_res.is_empty(), StorageError::EmptyList);

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
        ensure!(sport.validate(), StorageError::InvalidSport);

        self.raw_execute(
            queries::UPSERT_SPORT,
            false,
            params![sport.key, sport.name, sport.comment],
        )
        .context("exec upsert sport")
    }

    fn delete_sport(&self, key: &str) -> Result<()> {
        self.raw_execute(queries::DELETE_SPORT, false, params![key])
            .context("exec delete sport")
    }

    //
    // Backup/Restore
    //

    fn restore(&self, backup: &Backup) -> Result<()> {
        for w in &backup.weight {
            self.raw_execute(
                queries::UPSERT_WEIGHT,
                false,
                params![w.user_id, w.timestamp, w.value],
            )
            .context("upsert backup weight query")?;
        }

        for f in &backup.food {
            self.raw_execute(
                queries::UPSERT_FOOD,
                false,
                params![
                    f.key, f.name, f.brand, f.cal100, f.prot100, f.fat100, f.carb100, f.comment
                ],
            )
            .context("upsert backup food query")?;
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

#[cfg(test)]
mod test {
    use super::*;
    use anyhow::Result;
    use model::backup::{FoodBackup, WeightBackup};
    use tempfile::NamedTempFile;

    //
    // Migrations
    //

    #[test]
    fn test_migrations_apply() -> Result<()> {
        let db_file = NamedTempFile::new()?;
        let stg = StorageSqlite::new(db_file.path())?;

        assert_eq!(2, stg.get_last_migration_id().unwrap());

        Ok(())
    }

    //
    // Weight
    //

    #[test]
    fn test_get_weight_list() -> Result<()> {
        let db_file = NamedTempFile::new()?;
        let stg = StorageSqlite::new(db_file.path())?;

        // Check EmptyList error
        let res = stg.get_weight_list(
            1,
            Timestamp::from_unix_millis(0).unwrap(),
            Timestamp::from_unix_millis(10).unwrap(),
        );

        assert!(stg.is_storage_error(StorageError::EmptyList, &res.unwrap_err()));

        // Add test data
        stg.raw_execute(
            "
            INSERT INTO weight(user_id, timestamp, value)
            VALUES 
                (1, 1, 1.1),
                (1, 2, 2.2),
                (1, 3, 3.3),
                (2, 4, 4.4)
            ;
        ",
            false,
            params![],
        )?;

        // Check weight list for user 1
        let res = stg.get_weight_list(
            1,
            Timestamp::from_unix_millis(0).unwrap(),
            Timestamp::from_unix_millis(10).unwrap(),
        );
        assert_eq!(
            vec![
                Weight {
                    timestamp: Timestamp::from_unix_millis(1).unwrap(),
                    value: 1.1
                },
                Weight {
                    timestamp: Timestamp::from_unix_millis(2).unwrap(),
                    value: 2.2
                },
                Weight {
                    timestamp: Timestamp::from_unix_millis(3).unwrap(),
                    value: 3.3
                },
            ],
            res.unwrap()
        );

        // Check weight list for user 2
        let res = stg.get_weight_list(
            2,
            Timestamp::from_unix_millis(0).unwrap(),
            Timestamp::from_unix_millis(10).unwrap(),
        );
        assert_eq!(
            vec![Weight {
                timestamp: Timestamp::from_unix_millis(4).unwrap(),
                value: 4.4
            },],
            res.unwrap()
        );

        Ok(())
    }

    #[test]
    fn test_delete_weight() -> Result<()> {
        let db_file = NamedTempFile::new()?;
        let stg = StorageSqlite::new(db_file.path())?;

        // Add test data
        stg.raw_execute(
            "
            INSERT INTO weight(user_id, timestamp, value)
            VALUES 
                (1, 1, 1.1),
                (2, 4, 4.4)
            ;
        ",
            false,
            params![],
        )?;

        // Delete for user 2
        stg.delete_weight(2, Timestamp::from_unix_millis(4).unwrap())?;
        let res = stg.get_weight_list(
            2,
            Timestamp::from_unix_millis(0).unwrap(),
            Timestamp::from_unix_millis(10).unwrap(),
        );
        assert!(stg.is_storage_error(StorageError::EmptyList, &res.unwrap_err()));

        // Delete for user 1 record that not exists (timestamp=4)
        stg.delete_weight(1, Timestamp::from_unix_millis(4).unwrap())?;
        assert_eq!(
            1,
            stg.get_weight_list(
                1,
                Timestamp::from_unix_millis(0).unwrap(),
                Timestamp::from_unix_millis(10).unwrap(),
            )?
            .len()
        );

        Ok(())
    }

    #[test]
    fn test_set_weight() -> Result<()> {
        let db_file = NamedTempFile::new()?;
        let stg = StorageSqlite::new(db_file.path())?;

        // Set invalid weight
        let res = stg.set_weight(
            1,
            &Weight {
                timestamp: Timestamp::from_unix_millis(1734876557).unwrap(),
                value: -1.1,
            },
        );
        assert!(stg.is_storage_error(StorageError::InvalidWeight, &res.unwrap_err()));

        // Set weight
        stg.set_weight(
            1,
            &Weight {
                timestamp: Timestamp::from_unix_millis(1734876557).unwrap(),
                value: 1.1,
            },
        )?;

        // Check in DB
        let res = stg.raw_query(
            "SELECT timestamp, value FROM weight WHERE user_id = 1",
            params![],
        )?;

        assert_eq!(1, res.len());
        assert_eq!(
            Timestamp::from_unix_millis(1734876557).unwrap(),
            StorageSqlite::get_timestamp(res.get(0).unwrap(), "timestamp").unwrap()
        );
        assert_eq!(
            1.1,
            StorageSqlite::get_float(res.get(0).unwrap(), "value").unwrap()
        );

        // Update weight
        stg.set_weight(
            1,
            &Weight {
                timestamp: Timestamp::from_unix_millis(1734876557).unwrap(),
                value: 2.2,
            },
        )?;

        // Check in DB
        let res = stg.raw_query(
            "SELECT timestamp, value FROM weight WHERE user_id = 1",
            params![],
        )?;

        assert_eq!(1, res.len());
        assert_eq!(
            Timestamp::from_unix_millis(1734876557).unwrap(),
            StorageSqlite::get_timestamp(res.get(0).unwrap(), "timestamp").unwrap()
        );
        assert_eq!(
            2.2,
            StorageSqlite::get_float(res.get(0).unwrap(), "value").unwrap()
        );

        Ok(())
    }

    //
    // Food
    //

    #[test]
    fn test_set_food() -> Result<()> {
        let db_file = NamedTempFile::new()?;
        let stg = StorageSqlite::new(db_file.path())?;

        // Set invalid food
        let res = stg.set_food(&Food {
            key: "".into(),
            name: "name".into(),
            brand: "brand".into(),
            cal100: 1.1,
            prot100: 2.2,
            fat100: 3.3,
            carb100: 4.4,
            comment: "comment".into(),
        });
        assert!(stg.is_storage_error(StorageError::InvalidFood, &res.unwrap_err()));

        // Set food
        stg.set_food(&Food {
            key: "key".into(),
            name: "name".into(),
            brand: "brand".into(),
            cal100: 1.1,
            prot100: 2.2,
            fat100: 3.3,
            carb100: 4.4,
            comment: "comment".into(),
        })?;

        // Check in DB
        let res = stg.raw_query(
            r#"
            SELECT
                key, name, brand, cal100,
                prot100, fat100, carb100, comment
            FROM food
        "#,
            params![],
        )?;

        assert_eq!(1, res.len());
        assert_eq!(
            String::from("key"),
            StorageSqlite::get_string(res.get(0).unwrap(), "key").unwrap()
        );
        assert_eq!(
            String::from("name"),
            StorageSqlite::get_string(res.get(0).unwrap(), "name").unwrap()
        );
        assert_eq!(
            String::from("brand"),
            StorageSqlite::get_string(res.get(0).unwrap(), "brand").unwrap()
        );
        assert_eq!(
            1.1,
            StorageSqlite::get_float(res.get(0).unwrap(), "cal100").unwrap()
        );
        assert_eq!(
            2.2,
            StorageSqlite::get_float(res.get(0).unwrap(), "prot100").unwrap()
        );
        assert_eq!(
            3.3,
            StorageSqlite::get_float(res.get(0).unwrap(), "fat100").unwrap()
        );
        assert_eq!(
            4.4,
            StorageSqlite::get_float(res.get(0).unwrap(), "carb100").unwrap()
        );
        assert_eq!(
            String::from("comment"),
            StorageSqlite::get_string(res.get(0).unwrap(), "comment").unwrap()
        );

        // Update food
        stg.set_food(&Food {
            key: "key".into(),
            name: "name".into(),
            brand: "".into(),
            cal100: 5.5,
            prot100: 6.6,
            fat100: 7.7,
            carb100: 8.8,
            comment: "".into(),
        })?;

        // Check in DB
        let res = stg.raw_query(
            r#"
            SELECT
                key, name, brand, cal100,
                prot100, fat100, carb100, comment
            FROM food
        "#,
            params![],
        )?;

        assert_eq!(1, res.len());
        assert_eq!(
            String::from("key"),
            StorageSqlite::get_string(res.get(0).unwrap(), "key").unwrap()
        );
        assert_eq!(
            String::from("name"),
            StorageSqlite::get_string(res.get(0).unwrap(), "name").unwrap()
        );
        assert_eq!(
            String::from(""),
            StorageSqlite::get_string(res.get(0).unwrap(), "brand").unwrap()
        );
        assert_eq!(
            5.5,
            StorageSqlite::get_float(res.get(0).unwrap(), "cal100").unwrap()
        );
        assert_eq!(
            6.6,
            StorageSqlite::get_float(res.get(0).unwrap(), "prot100").unwrap()
        );
        assert_eq!(
            7.7,
            StorageSqlite::get_float(res.get(0).unwrap(), "fat100").unwrap()
        );
        assert_eq!(
            8.8,
            StorageSqlite::get_float(res.get(0).unwrap(), "carb100").unwrap()
        );
        assert_eq!(
            String::from(""),
            StorageSqlite::get_string(res.get(0).unwrap(), "comment").unwrap()
        );

        Ok(())
    }

    #[test]
    fn test_get_food() -> Result<()> {
        let db_file = NamedTempFile::new()?;
        let stg = StorageSqlite::new(db_file.path())?;

        // Get food that not exists
        let res = stg.get_food("key");
        assert!(stg.is_storage_error(StorageError::NotFound, &res.unwrap_err()));

        // Set food
        let f = Food {
            key: "key".into(),
            name: "name".into(),
            brand: "brand".into(),
            cal100: 1.1,
            prot100: 2.2,
            fat100: 3.3,
            carb100: 4.4,
            comment: "comment".into(),
        };
        stg.set_food(&f)?;

        // Get food
        assert_eq!(f, stg.get_food("key").unwrap());

        Ok(())
    }

    #[test]
    fn test_get_food_list() -> Result<()> {
        let db_file = NamedTempFile::new()?;
        let stg = StorageSqlite::new(db_file.path())?;

        // Get empty food list
        let res = stg.get_food_list();
        assert!(stg.is_storage_error(StorageError::EmptyList, &res.unwrap_err()));

        // Set food
        let f1 = Food {
            key: "key1".into(),
            name: "name1".into(),
            brand: "brand".into(),
            cal100: 1.1,
            prot100: 2.2,
            fat100: 3.3,
            carb100: 4.4,
            comment: "comment".into(),
        };
        stg.set_food(&f1)?;

        let f2 = Food {
            key: "key2".into(),
            name: "name2".into(),
            brand: "brand".into(),
            cal100: 1.1,
            prot100: 2.2,
            fat100: 3.3,
            carb100: 4.4,
            comment: "comment".into(),
        };
        stg.set_food(&f2)?;

        // Get food list
        assert_eq!(vec![f1, f2], stg.get_food_list().unwrap());

        Ok(())
    }

    #[test]
    fn test_delete_food() -> Result<()> {
        let db_file = NamedTempFile::new()?;
        let stg = StorageSqlite::new(db_file.path())?;

        // Set food
        let f1 = Food {
            key: "key1".into(),
            name: "name1".into(),
            brand: "brand".into(),
            cal100: 1.1,
            prot100: 2.2,
            fat100: 3.3,
            carb100: 4.4,
            comment: "comment".into(),
        };
        stg.set_food(&f1)?;

        let f2 = Food {
            key: "key2".into(),
            name: "name2".into(),
            brand: "brand".into(),
            cal100: 1.1,
            prot100: 2.2,
            fat100: 3.3,
            carb100: 4.4,
            comment: "comment".into(),
        };
        stg.set_food(&f2)?;

        // Get food list
        assert_eq!(vec![f1, f2.clone()], stg.get_food_list().unwrap());

        // Delete food1
        stg.delete_food("key1")?;

        // Get food list
        assert_eq!(vec![f2], stg.get_food_list().unwrap());

        // Delete food2
        stg.delete_food("key2")?;

        // Get food list
        let res = stg.get_food_list();
        assert!(stg.is_storage_error(StorageError::EmptyList, &res.unwrap_err()));

        Ok(())
    }

    #[test]
    fn test_find_food() -> Result<()> {
        let db_file = NamedTempFile::new()?;
        let stg = StorageSqlite::new(db_file.path())?;

        // Find empty result
        let res = stg.find_food("some food");
        assert!(stg.is_storage_error(StorageError::EmptyList, &res.unwrap_err()));

        // Set food
        let f1 = Food {
            key: "key1".into(),
            name: "name1".into(),
            brand: "brand".into(),
            cal100: 1.1,
            prot100: 2.2,
            fat100: 3.3,
            carb100: 4.4,
            comment: "comment".into(),
        };
        stg.set_food(&f1)?;

        let f2 = Food {
            key: "key2".into(),
            name: "name2".into(),
            brand: "brand".into(),
            cal100: 1.1,
            prot100: 2.2,
            fat100: 3.3,
            carb100: 4.4,
            comment: "comment".into(),
        };
        stg.set_food(&f2)?;

        let f3 = Food {
            key: "key3".into(),
            name: "Сырок Дружба".into(),
            brand: "Вкусвилл".into(),
            cal100: 1.1,
            prot100: 2.2,
            fat100: 3.3,
            carb100: 4.4,
            comment: "Вкусный".into(),
        };
        stg.set_food(&f3)?;

        // Find food
        assert_eq!(
            vec![f1.clone(), f2.clone(), f3.clone()],
            stg.find_food("kEy").unwrap()
        );
        assert_eq!(vec![f2], stg.find_food("NAMe2").unwrap());
        assert_eq!(vec![f3.clone()], stg.find_food("дружба").unwrap());
        assert_eq!(vec![f3.clone()], stg.find_food("вкусВиЛЛ").unwrap());
        assert_eq!(vec![f3.clone()], stg.find_food("нЫЙ").unwrap());

        Ok(())
    }

    //
    // Sport
    //

    #[test]
    fn test_set_sport() -> Result<()> {
        let db_file = NamedTempFile::new()?;
        let stg = StorageSqlite::new(db_file.path())?;

        // Set invalid sport
        let res = stg.set_sport(&Sport {
            key: "".into(),
            name: "name".into(),
            comment: "comment".into(),
        });
        assert!(stg.is_storage_error(StorageError::InvalidSport, &res.unwrap_err()));

        // Set sport
        stg.set_sport(&Sport {
            key: "key".into(),
            name: "name".into(),
            comment: "comment".into(),
        })?;

        // Check in DB
        let res = stg.raw_query(
            r#"
            SELECT
                key, name, comment
            FROM sport
        "#,
            params![],
        )?;

        assert_eq!(1, res.len());
        assert_eq!(
            String::from("key"),
            StorageSqlite::get_string(res.get(0).unwrap(), "key").unwrap()
        );
        assert_eq!(
            String::from("name"),
            StorageSqlite::get_string(res.get(0).unwrap(), "name").unwrap()
        );       
        assert_eq!(
            String::from("comment"),
            StorageSqlite::get_string(res.get(0).unwrap(), "comment").unwrap()
        );

        // Update sport
        stg.set_sport(&Sport {
            key: "key".into(),
            name: "name".into(),
            comment: "".into(),
        })?;

        // Check in DB
        let res = stg.raw_query(
            r#"
            SELECT
                key, name, comment
            FROM sport
        "#,
            params![],
        )?;

        assert_eq!(1, res.len());
        assert_eq!(
            String::from("key"),
            StorageSqlite::get_string(res.get(0).unwrap(), "key").unwrap()
        );
        assert_eq!(
            String::from("name"),
            StorageSqlite::get_string(res.get(0).unwrap(), "name").unwrap()
        );       
        assert_eq!(
            String::from(""),
            StorageSqlite::get_string(res.get(0).unwrap(), "comment").unwrap()
        );

        Ok(())
    }

    #[test]
    fn test_get_sport_list() -> Result<()> {
        let db_file = NamedTempFile::new()?;
        let stg = StorageSqlite::new(db_file.path())?;

        // Get empty sport list
        let res = stg.get_sport_list();
        assert!(stg.is_storage_error(StorageError::EmptyList, &res.unwrap_err()));

        // Set sport
        let s1 = Sport {
            key: "key1".into(),
            name: "name1".into(),
            comment: "comment".into(),
        };
        stg.set_sport(&s1)?;

        let s2 = Sport {
            key: "key2".into(),
            name: "name2".into(),           
            comment: "comment".into(),
        };
        stg.set_sport(&s2)?;

        // Get sport list
        assert_eq!(vec![s1, s2], stg.get_sport_list().unwrap());

        Ok(())
    }

    #[test]
    fn test_delete_sport() -> Result<()> {
        let db_file = NamedTempFile::new()?;
        let stg = StorageSqlite::new(db_file.path())?;

        // Set sport
        let s1 = Sport {
            key: "key1".into(),
            name: "name1".into(),
            comment: "comment".into(),
        };
        stg.set_sport(&s1)?;

        let s2 = Sport {
            key: "key2".into(),
            name: "name2".into(),
            comment: "comment".into(),
        };
        stg.set_sport(&s2)?;

        // Get sport list
        assert_eq!(vec![s1, s2.clone()], stg.get_sport_list().unwrap());

        // Delete sport1
        stg.delete_sport("key1")?;

        // Get sport list
        assert_eq!(vec![s2], stg.get_sport_list().unwrap());

        // Delete sport2
        stg.delete_sport("key2")?;

        // Get sport list
        let res = stg.get_sport_list();
        assert!(stg.is_storage_error(StorageError::EmptyList, &res.unwrap_err()));

        Ok(())
    }

    //
    // Restore/backup
    //

    #[test]
    fn test_restore() -> Result<()> {
        let db_file = NamedTempFile::new()?;
        let stg = StorageSqlite::new(db_file.path())?;

        // Do restore
        stg.restore(&Backup {
            timestamp: 1,
            weight: vec![
                WeightBackup {
                    timestamp: 1,
                    user_id: 1,
                    value: 1.1,
                },
                WeightBackup {
                    timestamp: 2,
                    user_id: 1,
                    value: 2.2,
                },
                WeightBackup {
                    timestamp: 3,
                    user_id: 1,
                    value: 3.3,
                },
                WeightBackup {
                    timestamp: 4,
                    user_id: 2,
                    value: 4.4,
                },
            ],
            food: vec![
                FoodBackup {
                    key: "key2".into(),
                    name: "Food 2".into(),
                    brand: "Brand2".into(),
                    cal100: 5.5,
                    prot100: 6.6,
                    fat100: 7.7,
                    carb100: 8.8,
                    comment: "Comment2".into(),
                },
                FoodBackup {
                    key: "key1".into(),
                    name: "Food 1".into(),
                    brand: "Brand 1".into(),
                    cal100: 1.1,
                    prot100: 2.2,
                    fat100: 3.3,
                    carb100: 4.4,
                    comment: "Comment1".into(),
                },
                FoodBackup {
                    key: "key4".into(),
                    name: "Еда 4".into(),
                    brand: "Брэнд 4".into(),
                    cal100: 100.100,
                    prot100: 200.200,
                    fat100: 300.300,
                    carb100: 400.400,
                    comment: "Комментарий 4".into(),
                },
                FoodBackup {
                    key: "key3".into(),
                    name: "Еда 3".into(),
                    brand: "Брэнд 3".into(),
                    cal100: 10.10,
                    prot100: 20.20,
                    fat100: 30.30,
                    carb100: 40.40,
                    comment: "Комментарий 3".into(),
                },
            ],
        })?;

        // Check weight list for user 1
        let res = stg.get_weight_list(
            1,
            Timestamp::from_unix_millis(0).unwrap(),
            Timestamp::from_unix_millis(10).unwrap(),
        )?;
        assert_eq!(
            vec![
                Weight {
                    timestamp: Timestamp::from_unix_millis(1).unwrap(),
                    value: 1.1
                },
                Weight {
                    timestamp: Timestamp::from_unix_millis(2).unwrap(),
                    value: 2.2
                },
                Weight {
                    timestamp: Timestamp::from_unix_millis(3).unwrap(),
                    value: 3.3
                },
            ],
            res
        );

        // Check weight list for user 2
        let res = stg.get_weight_list(
            2,
            Timestamp::from_unix_millis(0).unwrap(),
            Timestamp::from_unix_millis(10).unwrap(),
        )?;
        assert_eq!(
            vec![Weight {
                timestamp: Timestamp::from_unix_millis(4).unwrap(),
                value: 4.4
            },],
            res
        );

        // Check food
        let res = stg.get_food_list()?;
        assert_eq!(
            vec![
                Food {
                    key: "key1".into(),
                    name: "Food 1".into(),
                    brand: "Brand 1".into(),
                    cal100: 1.1,
                    prot100: 2.2,
                    fat100: 3.3,
                    carb100: 4.4,
                    comment: "Comment1".into(),
                },
                Food {
                    key: "key2".into(),
                    name: "Food 2".into(),
                    brand: "Brand2".into(),
                    cal100: 5.5,
                    prot100: 6.6,
                    fat100: 7.7,
                    carb100: 8.8,
                    comment: "Comment2".into(),
                },
                Food {
                    key: "key3".into(),
                    name: "Еда 3".into(),
                    brand: "Брэнд 3".into(),
                    cal100: 10.10,
                    prot100: 20.20,
                    fat100: 30.30,
                    carb100: 40.40,
                    comment: "Комментарий 3".into(),
                },
                Food {
                    key: "key4".into(),
                    name: "Еда 4".into(),
                    brand: "Брэнд 4".into(),
                    cal100: 100.100,
                    prot100: 200.200,
                    fat100: 300.300,
                    carb100: 400.400,
                    comment: "Комментарий 4".into(),
                }
            ],
            res
        );

        Ok(())
    }
}
