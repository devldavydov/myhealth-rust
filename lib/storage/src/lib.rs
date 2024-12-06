use anyhow::Result;
use model::{Bundle, Food, UserSettings, Weight};
use types::timestamp::Timestamp;

pub mod storage_sqlite;

pub trait Storage: Send + Sync {
    // Food
    fn get_food(&self, key: &str) -> Result<Food>;
    fn get_food_list(&self) -> Result<Vec<Food>>;
    fn set_food(&self, food: &Food) -> Result<()>;
    fn find_food(&self, pattern: String) -> Result<Vec<Food>>;
    fn delete_food(&self, key: &str) -> Result<()>;
    // Bundle
    fn get_bundle(&self, user_id: i64, key: &str) -> Result<Bundle>;
    fn get_bundle_list(&self, user_id: i64) -> Result<Vec<Bundle>>;
    fn set_bundle(&self, user_id: i64, bndl: &Bundle) -> Result<()>;
    fn delete_bundle(&self, user_id: i64, key: &str) -> Result<()>;
    // Weight
    fn get_weight_list(&self, user_id: i64, from: Timestamp, to: Timestamp) -> Result<Vec<Weight>>;
    fn set_weight(&self, user_id: i64, weight: &Weight) -> Result<()>;
    fn delete_weight(&self, user_id: i64, timestamp: Timestamp) -> Result<()>;
    // Journal
    // UserSettings
    fn get_user_settings(&self, user_id: i64) -> Result<UserSettings>;
    fn set_user_settings(&self, user_id: i64, settings: &UserSettings) -> Result<()>;
}
