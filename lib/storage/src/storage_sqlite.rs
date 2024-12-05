use std::{collections::HashMap, path::PathBuf};
use std::sync::Mutex;

use crate::Storage;
use anyhow::{Context, Result, anyhow};
use model::{Bundle, Food, UserSettings, Weight};
use rusqlite::{types::Value, Connection, Params};
use types::timestamp::Timestamp;

mod functions;
mod migrations;
mod queries;

use functions::get_last_migration_id;

pub const DB_FILE: &str = "myhealth.db";

pub struct StorageSqlite {
    conn: Mutex<Option<Connection>>,
}

impl StorageSqlite {
    pub fn new(db_file: PathBuf) -> Result<Self> {
        let conn = Connection::open(format!(
            "file:{}?mode=rwc&_timeout=5000&_fk=1&_sync=1&_journal=wal",
            db_file.to_str().unwrap(),
        ))?;

        let s = Self {
            conn: Mutex::new(Some(conn)),
        };

        s.apply_migrations()?;

        Ok(s)
    }

    fn raw_query<P>(&self, query: &str, params: P) -> Result<Vec<HashMap<String, Value>>>
    where P: Params
    {
        let mut guard = self.conn.lock().unwrap();
        let conn = guard.as_mut().unwrap();

        let mut stmt = conn.prepare(query).context("prepare raw query")?;
        let mut rows = stmt.query(params).context("quering raw query")?;

        let mut res = Vec::new();        
        while let Some(row) = rows.next().context("get next row")? {
            let col_names = row.as_ref().column_names();
            let col_cnt = col_names.len();

            let mut res_row = HashMap::with_capacity(col_cnt);
            for i in 0..col_cnt {
                res_row.insert(
                    (*col_names.get(i).unwrap()).into(),
                    Value::from(row.get_ref(i).with_context(|| anyhow!(format!("get column {i}")))?)
                );
            }
            res.push(res_row);
        }

        Ok(res)
    }
}

impl Storage for StorageSqlite {
    //
    // System
    //

    fn apply_migrations(&self) -> Result<()> {
        let mut guard = self.conn.lock().unwrap();
        let conn = guard.as_mut().unwrap();

        // Create system table if not exists
        conn.execute_batch(queries::CREATE_TABLE_SYSTEM)
            .context("create system table")?;

        let last_migration_id = get_last_migration_id(conn).context("get last migration id")?;
        migrations::apply(conn, last_migration_id)
    }

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

    fn get_weight_list(
        &self,
        user_id: i64,
        from: Timestamp,
        to: Timestamp,
    ) -> anyhow::Result<Vec<Weight>> {
        todo!()
    }

    fn set_weight(&self, user_id: i64, weight: &Weight) -> Result<()> {
        todo!()
    }

    fn delete_weight(&self, user_id: i64, timestamp: Timestamp) -> Result<()> {
        todo!()
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
        let stg = StorageSqlite::new(db_file.path().to_owned())?;

        let res = stg.raw_query("select migration_id from system", [])?;

        assert_eq!(Value::Integer(2), *res.get(0).unwrap().get("migration_id").unwrap());

        Ok(())
    }
}