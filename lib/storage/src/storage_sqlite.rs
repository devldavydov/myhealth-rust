use std::collections::HashMap;
use std::path::Path;
use std::sync::Mutex;

use crate::{Storage, StorageError};
use anyhow::{anyhow, bail, ensure, Context, Error, Result};
use model::{Bundle, Food, UserSettings, Weight};
use rusqlite::{params, types::Value, Connection, Params};
use types::timestamp::Timestamp;

mod migrations;
mod queries;

pub const DB_FILE: &str = "myhealth.db";

pub struct StorageSqlite {
    conn: Mutex<Connection>,
}

impl StorageSqlite {
    pub fn new(db_file: &Path) -> Result<Self> {
        let conn = Connection::open(format!(
            "file:{}?mode=rwc&_timeout=5000&_fk=1&_sync=1&_journal=wal",
            db_file.to_str().unwrap(),
        ))?;

        let s = Self {
            conn: Mutex::new(conn),
        };

        s.init()?;
        s.apply_migrations()?;

        Ok(s)
    }

    pub fn is_storage_error(&self, stg_err: StorageError, err: &Error) -> bool {
        stg_err
            == *err
                .root_cause()
                .downcast_ref::<StorageError>()
                .unwrap_or(&StorageError::default())
    }

    fn init(&self) -> Result<()> {
        // Create system table if not exists
        self.raw_execute(queries::CREATE_TABLE_SYSTEM, false, params![])
    }

    fn apply_migrations(&self) -> Result<()> {
        let last_migration_id = self
            .get_last_migration_id()
            .context("get last migration id")?;

        let mut conn = self.conn.lock().unwrap();
        migrations::apply(&mut conn, last_migration_id)
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
            conn.execute_batch(query).context("raw execute query")?;
        }

        Ok(())
    }

    fn get_last_migration_id(&self) -> Result<i64> {
        let res = self.raw_query(queries::SELECT_MIGRATION_ID, params![])?;
        if res.is_empty() {
            return Ok(0);
        }

        let row = res.first().unwrap();
        if let Some(Value::Integer(val)) = row.get("migration_id") {
            return Ok(*val);
        }

        bail!("migration_id not found");
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
}

impl Storage for StorageSqlite {
    //
    // Food
    //

    fn get_food(&self, key: &str) -> Result<Food> {
        todo!()
    }

    fn get_food_list(&self) -> Result<Vec<Food>> {
        todo!()
    }

    fn set_food(&self, food: &Food) -> Result<()> {
        todo!()
    }

    fn find_food(&self, pattern: String) -> Result<Vec<Food>> {
        todo!()
    }

    fn delete_food(&self, key: &str) -> Result<()> {
        todo!()
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
        let db_res = self.raw_query(
            queries::SELECT_WEIGHT_LIST,
            params![user_id, from.millisecond(), to.millisecond()],
        )?;

        ensure!(!db_res.is_empty(), StorageError::EmptyList);

        let mut res = Vec::with_capacity(db_res.len());
        for row in &db_res {
            res.push(Weight {
                timestamp: Self::get_timestamp(row, "timestamp")?,
                value: Self::get_float(row, "value")?,
            });
        }

        Ok(res)
    }

    fn set_weight(&self, user_id: i64, weight: &Weight) -> Result<()> {
        self.raw_execute(
            queries::UPSERT_WEIGHT,
            false,
            params![user_id, weight.timestamp.millisecond(), weight.value],
        )
    }

    fn delete_weight(&self, user_id: i64, timestamp: Timestamp) -> Result<()> {
        self.raw_execute(
            queries::DELETE_WEIGHT,
            false,
            params![user_id, timestamp.millisecond()],
        )
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
}

#[cfg(test)]
mod test {
    use super::*;
    use anyhow::Result;
    use tempfile::NamedTempFile;

    #[test]
    fn test_migrations_apply() -> Result<()> {
        let db_file = NamedTempFile::new()?;
        let stg = StorageSqlite::new(db_file.path())?;

        assert_eq!(2, stg.get_last_migration_id().unwrap());

        Ok(())
    }

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
                    timestamp: Timestamp::from_unix_millis(3).unwrap(),
                    value: 3.3
                },
                Weight {
                    timestamp: Timestamp::from_unix_millis(2).unwrap(),
                    value: 2.2
                },
                Weight {
                    timestamp: Timestamp::from_unix_millis(1).unwrap(),
                    value: 1.1
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

        // Delete for user 1, that not exists
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

        // Set weight
        stg.set_weight(
            1,
            &Weight {
                timestamp: Timestamp::from_unix_millis(1).unwrap(),
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
            Value::Integer(1),
            *res.get(0).unwrap().get("timestamp").unwrap()
        );
        assert_eq!(Value::Real(1.1), *res.get(0).unwrap().get("value").unwrap());

        // Update weight
        stg.set_weight(
            1,
            &Weight {
                timestamp: Timestamp::from_unix_millis(1).unwrap(),
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
            Value::Integer(1),
            *res.get(0).unwrap().get("timestamp").unwrap()
        );
        assert_eq!(Value::Real(2.2), *res.get(0).unwrap().get("value").unwrap());

        Ok(())
    }
}
