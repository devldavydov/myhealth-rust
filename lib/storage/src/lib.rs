use anyhow::Result;
use model::{
    backup::Backup, Bundle, Food, Sport, SportActivity, SportActivityReport, UserSettings, Weight,
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
    fn restore(&self, backup: &Backup) -> Result<()>;

    // Error
    fn is_storage_error(&self, stg_err: StorageError, err: &anyhow::Error) -> bool;
}

#[derive(Error, Debug, PartialEq, Default)]
pub enum StorageError {
    #[error("unknown")]
    #[default]
    Unknown,
    #[error("not found")]
    NotFound,
    #[error("empty list")]
    EmptyList,
    #[error("invalid weight")]
    InvalidWeight,
    #[error("invalid food")]
    InvalidFood,
    #[error("invalid sport")]
    InvalidSport,
    #[error("sport is used in activity")]
    SportIsUsedViolation,
    #[error("invalid sport activity")]
    InvalidSportActivity,
    #[error("invalid user settings")]
    InvalidUserSettings,
    #[error("invalid bundle")]
    InvalidBundle,
    #[error("dependent food not found")]
    BundleDepFoodNotFound,
    #[error("dependent bundle not found")]
    BundleDepBundleNotFound,
    #[error("dependent recursive bundle not allowed")]
    BundleDepRecursive,
    #[error("bundle is used")]
    BundleIsUsed,
}
