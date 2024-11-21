use std::collections::HashMap;

use anyhow::{anyhow, Result};

use crate::components::types::timestamp::Timestamp;

pub struct System {
    pub migration_id: u32,
}

pub struct Food {
    pub key: String,
    pub name: String,
    pub brand: String,
    pub cal100: f64,
    pub prot100: f64,
    pub fat100: f64,
    pub carb100: f64,
    pub comment: String,
}

pub struct Weight {
    pub timestamp: Timestamp,
    pub value: f64,
}

pub enum Meal {
    Breakfast,
    FirstSnack,
    Dinner,
    SecondSnack,
    ThirdSnack,
    Supper,
}

pub struct Journal {
    pub timestamp: Timestamp,
    pub meal: Meal,
    pub food_key: String,
    pub food_weight: f64,
}

pub struct UserSettings {
    pub cal_limit: f64
}

pub struct Bundle {
    pub key: String,
    // Map of bundle data
	// Variants:
	// if food: food_key -> weight > 0
	// if bundle: bundle_key -> 0
    pub data: HashMap<String, f64>
}

pub trait Storage {
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

impl Food {
    pub fn validate(&self) -> bool {
        self.key != "" &&
            self.name != "" &&
            self.cal100 >= 0.0 && 
            self.prot100 >= 0.0 &&
            self.fat100 >= 0.0 &&
            self.carb100 >= 0.0
    }
}

impl Weight {
    pub fn validate(&self) -> bool {
        self.value >= 0.0
    }
}

impl Meal {
    fn new(v: u8) -> Result<Meal> {
        match v {
            0 => Ok(Meal::Breakfast),
            1 => Ok(Meal::FirstSnack),
            2 => Ok(Meal::Dinner),
            3 => Ok(Meal::SecondSnack),
            4 => Ok(Meal::ThirdSnack),
            5 => Ok(Meal::Supper),
            _ => Err(anyhow!("wrong meal")),
        }
    }
}

impl From<Meal> for String {
    fn from(value: Meal) -> Self {
        match value {
            Meal::Breakfast => "Завтрак".into(),
            Meal::FirstSnack => "До обеда".into(),
            Meal::Dinner => "Обед".into(),
            Meal::SecondSnack => "Полдник".into(),
            Meal::ThirdSnack => "До ужина".into(),
            Meal::Supper => "Ужин".into(),
        }
    }
}

impl Journal {
    pub fn validate(&self) -> bool {
        self.food_key != "" &&
            self.food_weight > 0.0
    }
}

impl UserSettings {
    pub fn validate(&self) -> bool {
        self.cal_limit > 0.0
    }
}

impl Bundle {
    pub fn validate(&self) -> bool {
        if self.key == "" || self.data.len() == 0 {
            return false;
        }

        for (_, v) in &self.data {
            if *v < 0.0 {
                return false;
            }
        }

        true
    }
}