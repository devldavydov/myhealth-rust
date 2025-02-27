use anyhow::Result;
use model::{
    backup::Backup, Bundle, Food, Journal, JournalReport, Meal, Sport, SportActivity,
    SportActivityReport, UserSettings, Weight,
};
use thiserror::Error;
use types::timestamp::Timestamp;

pub mod storage_sqlite;

pub trait Storage: Send + Sync {
    // Food
    fn get_food(&self, key: &str) -> Result<Food>;
    fn get_food_list(&self) -> Result<Vec<Food>>;
    fn set_food(&self, food: &Food) -> Result<()>;
    fn find_food(&self, pattern: &str) -> Result<Vec<Food>>;
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
    fn set_journal(&self, user_id: i64, journal: &Journal) -> Result<()>;
    fn set_journal_bundle(
        &self,
        user_id: i64,
        timestamp: Timestamp,
        meal: Meal,
        bndl_key: &str,
    ) -> Result<()>;
    fn delete_journal(
        &self,
        user_id: i64,
        timestamp: Timestamp,
        meal: Meal,
        food_key: &str,
    ) -> Result<()>;
    fn delete_journal_meal(&self, user_id: i64, timestamp: Timestamp, meal: Meal) -> Result<()>;
    fn get_journal_report(
        &self,
        user_id: i64,
        from: Timestamp,
        to: Timestamp,
    ) -> Result<Vec<JournalReport>>;
    fn get_journal_food_avg_weight(
        &self,
        user_id: i64,
        food_key: &str,
        from: Timestamp,
        to: Timestamp,
    ) -> Result<f64>;

    // UserSettings
    fn get_user_settings(&self, user_id: i64) -> Result<UserSettings>;
    fn set_user_settings(&self, user_id: i64, settings: &UserSettings) -> Result<()>;

    // Sport
    fn get_sport(&self, key: &str) -> Result<Sport>;
    fn get_sport_list(&self) -> Result<Vec<Sport>>;
    fn set_sport(&self, sport: &Sport) -> Result<()>;
    fn delete_sport(&self, key: &str) -> Result<()>;

    // SportActivity
    fn set_sport_activity(&self, user_id: i64, act: &SportActivity) -> Result<()>;
    fn delete_sport_activity(
        &self,
        user_id: i64,
        timestamp: Timestamp,
        sport_key: &str,
    ) -> Result<()>;
    fn get_sport_activity_report(
        &self,
        user_id: i64,
        from: Timestamp,
        to: Timestamp,
    ) -> Result<Vec<SportActivityReport>>;

    // Backup/Restore
    fn backup(&self, user_id: i64) -> Result<Backup>;
    fn restore(&self, backup: &Backup) -> Result<()>;

    // Error
    fn is_storage_error(&self, stg_err: StorageError, err: &anyhow::Error) -> bool;
}

#[derive(Error, Debug, PartialEq, Default)]
pub enum StorageError {
    #[error("unknown")]
    #[default]
    Unknown,
    #[error("empty result")]
    EmptyResult,
    // Weight
    #[error("weight invalid")]
    WeightInvalid,
    // Food
    #[error("food invalid")]
    FoodInvalid,
    #[error("food is used")]
    FoodIsUsed,
    #[error("food not found")]
    FoodNotFound,
    // Sport
    #[error("sport invalid")]
    SportInvalid,
    #[error("sport not found")]
    SportNotFound,
    #[error("sport is used in activity")]
    SportIsUsedViolation,
    // Sport activity
    #[error("sport activity invalid")]
    SportActivityInvalid,
    // User settings
    #[error("user settings invalid")]
    UserSettingsInvalid,
    #[error("user settings not found")]
    UserSettingsNotFound,
    // Bundle
    #[error("bundle invalid")]
    BundleInvalid,
    #[error("dependent food not found")]
    BundleDepFoodNotFound,
    #[error("dependent bundle not found")]
    BundleDepBundleNotFound,
    #[error("dependent recursive bundle not allowed")]
    BundleDepRecursive,
    #[error("bundle is used")]
    BundleIsUsed,
    #[error("bundle not found")]
    BundleNotFound,
    // Journal
    #[error("journal invalid")]
    JournalInvalid,
}
