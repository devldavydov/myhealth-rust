use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Backup {
    #[serde(rename = "timestamp")]
    pub timestamp: i64,
    #[serde(rename = "weight")]
    pub weight: Vec<WeightBackup>,
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
