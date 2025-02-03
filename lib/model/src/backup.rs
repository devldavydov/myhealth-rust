use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Backup {
    #[serde(rename = "timestamp")]
    pub timestamp: i64,
    #[serde(rename = "weight")]
    pub weight: Vec<WeightBackup>,
    #[serde(rename = "food")]
    pub food: Vec<FoodBackup>,
    #[serde(rename = "user_settings")]
    pub user_settings: Vec<UserSettingsBackup>,
    #[serde(rename = "bundle")]
    pub bundle: Vec<BundleBackup>,
    #[serde(rename = "journal")]
    pub journal: Vec<JournalBackup>,
    #[serde(rename = "sport")]
    pub sport: Vec<SportBackup>,
    #[serde(rename = "sport_activity")]
    pub sport_activity: Vec<SportActivityBackup>,
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct WeightBackup {
    #[serde(rename = "user_id")]
    pub user_id: i64,
    #[serde(rename = "timestamp")]
    pub timestamp: i64,
    #[serde(rename = "value")]
    pub value: f64,
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct FoodBackup {
    #[serde(rename = "key")]
    pub key: String,
    #[serde(rename = "name")]
    pub name: String,
    #[serde(rename = "brand")]
    pub brand: String,
    #[serde(rename = "cal100")]
    pub cal100: f64,
    #[serde(rename = "prot100")]
    pub prot100: f64,
    #[serde(rename = "fat100")]
    pub fat100: f64,
    #[serde(rename = "carb100")]
    pub carb100: f64,
    #[serde(rename = "comment")]
    pub comment: String,
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct UserSettingsBackup {
    #[serde(rename = "user_id")]
    pub user_id: i64,
    #[serde(rename = "cal_limit")]
    pub cal_limit: f64,
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct BundleBackup {
    #[serde(rename = "user_id")]
    pub user_id: i64,
    #[serde(rename = "key")]
    pub key: String,
    #[serde(rename = "data")]
    pub data: String,
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct JournalBackup {
    #[serde(rename = "user_id")]
    pub user_id: i64,
    #[serde(rename = "timestamp")]
    pub timestamp: i64,
    #[serde(rename = "meal")]
    pub meal: u8,
    #[serde(rename = "food_key")]
    pub food_key: String,
    #[serde(rename = "food_weight")]
    pub food_weight: f64,
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct SportBackup {
    #[serde(rename = "key")]
    pub key: String,
    #[serde(rename = "name")]
    pub name: String,
    #[serde(rename = "comment")]
    pub comment: String,
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct SportActivityBackup {
    #[serde(rename = "user_id")]
    pub user_id: i64,
    #[serde(rename = "sport_key")]
    pub sport_key: String,
    #[serde(rename = "timestamp")]
    pub timestamp: i64,
    #[serde(rename = "sets")]
    pub sets: String,
}
