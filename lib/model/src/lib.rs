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
        !self.key.is_empty()
            && !self.name.is_empty()
            && self.cal100 >= 0.0
            && self.prot100 >= 0.0
            && self.fat100 >= 0.0
            && self.carb100 >= 0.0
    }
}

impl Weight {
    pub fn validate(&self) -> bool {
        self.value > 0.0
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
        !self.food_key.is_empty() && self.food_weight > 0.0
    }
}

impl UserSettings {
    pub fn validate(&self) -> bool {
        self.cal_limit > 0.0
    }
}

impl Bundle {
    pub fn validate(&self) -> bool {
        if self.key.is_empty() || self.data.is_empty() {
            return false;
        }

        for v in self.data.values() {
            if *v < 0.0 {
                return false;
            }
        }

        true
    }
}

#[cfg(test)]
mod test {
    use std::collections::HashMap;

    use types::timestamp::Timestamp;

    use crate::{Bundle, Food, Journal, Meal, UserSettings, Weight};

    #[test]
    fn test_validate_food() {
        for t in vec![
            (
                Food {
                    key: "".into(),
                    name: "".into(),
                    brand: "brand".into(),
                    cal100: -1.0,
                    prot100: -1.0,
                    fat100: -1.0,
                    carb100: -1.0,
                    comment: "".into(),
                },
                false,
            ),
            (
                Food {
                    key: "key".into(),
                    name: "".into(),
                    brand: "brand".into(),
                    cal100: -1.0,
                    prot100: -1.0,
                    fat100: -1.0,
                    carb100: -1.0,
                    comment: "".into(),
                },
                false,
            ),
            (
                Food {
                    key: "key".into(),
                    name: "name".into(),
                    brand: "brand".into(),
                    cal100: -1.0,
                    prot100: -1.0,
                    fat100: -1.0,
                    carb100: -1.0,
                    comment: "".into(),
                },
                false,
            ),
            (
                Food {
                    key: "key".into(),
                    name: "name".into(),
                    brand: "brand".into(),
                    cal100: 1.0,
                    prot100: -1.0,
                    fat100: -1.0,
                    carb100: -1.0,
                    comment: "".into(),
                },
                false,
            ),
            (
                Food {
                    key: "key".into(),
                    name: "name".into(),
                    brand: "brand".into(),
                    cal100: 1.0,
                    prot100: 1.0,
                    fat100: -1.0,
                    carb100: -1.0,
                    comment: "".into(),
                },
                false,
            ),
            (
                Food {
                    key: "key".into(),
                    name: "name".into(),
                    brand: "brand".into(),
                    cal100: 1.0,
                    prot100: 1.0,
                    fat100: 1.0,
                    carb100: -1.0,
                    comment: "".into(),
                },
                false,
            ),
            (
                Food {
                    key: "key".into(),
                    name: "name".into(),
                    brand: "brand".into(),
                    cal100: 1.0,
                    prot100: 1.0,
                    fat100: 1.0,
                    carb100: 1.0,
                    comment: "".into(),
                },
                true,
            ),
        ] {
            assert_eq!(t.0.validate(), t.1);
        }
    }

    #[test]
    fn test_validate_weight() {
        assert!(!Weight {
            timestamp: Timestamp::now(),
            value: 0.0
        }
        .validate());
        assert!(Weight {
            timestamp: Timestamp::now(),
            value: 1.0
        }
        .validate());
    }

    #[test]
    fn test_validate_journal() {
        for t in vec![
            (
                Journal {
                    timestamp: Timestamp::now(),
                    meal: Meal::Breakfast,
                    food_key: "".into(),
                    food_weight: 0.0,
                },
                false,
            ),
            (
                Journal {
                    timestamp: Timestamp::now(),
                    meal: Meal::Breakfast,
                    food_key: "key".into(),
                    food_weight: 0.0,
                },
                false,
            ),
            (
                Journal {
                    timestamp: Timestamp::now(),
                    meal: Meal::Breakfast,
                    food_key: "key".into(),
                    food_weight: 1.0,
                },
                true,
            ),
        ] {
            assert_eq!(t.0.validate(), t.1);
        }
    }

    #[test]
    fn test_validate_user_settings() {
        assert!(!UserSettings { cal_limit: 0.0 }.validate());
        assert!(UserSettings { cal_limit: 1.0 }.validate());
    }

    #[test]
    fn test_validate_bundle() {
        for t in vec![
            (
                Bundle {
                    key: "".into(),
                    data: HashMap::default(),
                },
                false,
            ),
            (
                Bundle {
                    key: "key".into(),
                    data: HashMap::default(),
                },
                false,
            ),
            (
                Bundle {
                    key: "key".into(),
                    data: HashMap::default(),
                },
                false,
            ),
            (
                Bundle {
                    key: "key".into(),
                    data: HashMap::from_iter(vec![("bundle".into(), 0.0), ("food".into(), -1.0)]),
                },
                false,
            ),
            (
                Bundle {
                    key: "key".into(),
                    data: HashMap::from_iter(vec![("bundle".into(), 0.0), ("food".into(), 1.0)]),
                },
                true,
            ),
        ] {
            assert_eq!(t.0.validate(), t.1);
        }
    }
}
