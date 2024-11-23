use crate::Storage;
use model::{Food, Bundle, Weight, UserSettings};
use types::timestamp::Timestamp;
use anyhow::Result;

pub struct StorageSqlite;


impl StorageSqlite {
    pub fn new() -> Result<Self> {
        Ok(Self {})
    }
}

impl Storage for StorageSqlite {
    fn get_food(&self, key: &str) -> anyhow::Result<Food> {
        todo!()
    }

    fn get_food_list(&self) -> anyhow::Result<Vec<Food>> {
        todo!()
    }

    fn set_food(&self, food: &Food) -> anyhow::Result<()> {
        todo!()
    }

    fn find_food(&self, pattern: String) -> anyhow::Result<Vec<Food>> {
        todo!()
    }

    fn delete_food(&self, key: &str) -> anyhow::Result<()> {
        todo!()
    }

    fn get_bundle(&self, user_id: i64, key: &str) -> anyhow::Result<Bundle> {
        todo!()
    }

    fn get_bundle_list(&self, user_id: i64) -> anyhow::Result<Vec<Bundle>> {
        todo!()
    }

    fn set_bundle(&self, user_id: i64, bndl: &Bundle) -> anyhow::Result<()> {
        todo!()
    }

    fn delete_bundle(&self, user_id: i64, key: &str) -> anyhow::Result<()> {
        todo!()
    }

    fn get_weight_list(&self, user_id: i64, from: Timestamp, to: Timestamp) -> anyhow::Result<Vec<Weight>> {
        todo!()
    }

    fn set_weight(&self, user_id: i64, weight: &Weight) -> anyhow::Result<()> {
        todo!()
    }

    fn delete_weight(&self, user_id: i64, timestamp: Timestamp) -> anyhow::Result<()> {
        todo!()
    }

    fn get_user_settings(&self, user_id: i64) -> anyhow::Result<UserSettings> {
        todo!()
    }

    fn set_user_settings(&self, user_id: i64, settings: &UserSettings) -> anyhow::Result<()> {
        todo!()
    }
}