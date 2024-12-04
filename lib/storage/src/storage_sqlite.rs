use std::sync::Mutex;

use crate::Storage;
use anyhow::anyhow;
use anyhow::Result;
use model::{Bundle, Food, UserSettings, Weight};
use rusqlite::Connection;
use types::timestamp::Timestamp;

mod migrations;
mod queries;

use queries::CREATE_TABLE_SYSTEM;

const DB_FILE: &str = "myhealth.db";

pub struct StorageSqlite {
    conn: Mutex<Option<Connection>>,
}

impl StorageSqlite {
    pub fn new() -> Result<Self> {
        let conn = Connection::open(format!(
            "file:{}?mode=rwc&_timeout=5000&_fk=1&_sync=1&_journal=wal",
            DB_FILE
        ))?;

        let s = Self {
            conn: Mutex::new(Some(conn)),
        };

        s.apply_migrations()?;

        Ok(s)
    }
}

impl Storage for StorageSqlite {
    //
    // System
    //

    fn apply_migrations(&self) -> Result<()> {
        let mut guard = self.conn.lock().unwrap();
        let mut conn = guard.as_mut().unwrap();

        // Create system table if not exists
        conn.execute(CREATE_TABLE_SYSTEM, ())?;

        Ok(())
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

    fn close(&self) -> Result<()> {
        match self.conn.lock().unwrap().take() {
            Some(conn) => match conn.close() {
                Err((_, err)) => Err(anyhow!(err)),
                _ => Ok(()),
            },
            _ => Err(anyhow!("unexpected error")),
        }
    }
}
