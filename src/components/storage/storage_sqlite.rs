use anyhow::Result;

use super::model::{Storage, UserSettings, Food, Bundle, Weight};
use crate::components::types::timestamp::Timestamp;

pub struct StorageSqlite;

impl StorageSqlite {
    pub fn new() -> Result<Self> {
        Ok(Self {})
    }
}

impl Storage for StorageSqlite {
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

    fn get_weight_list(&self, user_id: i64, from: Timestamp, to: Timestamp) -> Result<Vec<Weight>> {
        todo!()
    }

    fn set_weight(&self, user_id: i64, weight: &Weight) -> Result<()> {
        todo!()
    }

    fn delete_weight(&self, user_id: i64, timestamp: Timestamp) -> Result<()> {
        todo!()
    }

    fn get_user_settings(&self, user_id: i64) -> Result<UserSettings> {
        todo!()
    }

    fn set_user_settings(&self, user_id: i64, settings: &UserSettings) -> Result<()> {
        todo!()
    }
}