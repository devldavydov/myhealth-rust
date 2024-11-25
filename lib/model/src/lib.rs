use anyhow::{anyhow, Result};
use std::collections::HashMap;
use types::timestamp::Timestamp;

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
    pub cal_limit: f64,
}

pub struct Bundle {
    pub key: String,
    // Map of bundle data
    // Variants:
    // if food: food_key -> weight > 0
    // if bundle: bundle_key -> 0
    pub data: HashMap<String, f64>,
}

impl Food {
    pub fn validate(&self) -> bool {
        self.key != ""
            && self.name != ""
            && self.cal100 >= 0.0
            && self.prot100 >= 0.0
            && self.fat100 >= 0.0
            && self.carb100 >= 0.0
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
        self.food_key != "" && self.food_weight > 0.0
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
