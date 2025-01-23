use std::collections::HashMap;

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
}

#[derive(Serialize, Deserialize)]
pub struct WeightBackup {
    #[serde(rename = "user_id")]
    pub user_id: i64,
    #[serde(rename = "timestamp")]
    pub timestamp: i64,
    #[serde(rename = "value")]
    pub value: f64,
}

#[derive(Serialize, Deserialize)]
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

#[derive(Serialize, Deserialize)]
pub struct UserSettingsBackup {
    #[serde(rename = "user_id")]
    pub user_id: i64,
    #[serde(rename = "cal_limit")]
    pub cal_limit: f64,
}

#[derive(Serialize, Deserialize)]
pub struct BundleBackup {
    #[serde(rename = "user_id")]
    pub user_id: i64,
    #[serde(rename = "key")]
    pub key: String,
    #[serde(rename = "data")]
    pub data: HashMap<String, f64>,
}
