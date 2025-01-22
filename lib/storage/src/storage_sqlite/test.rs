use std::vec;

use super::*;
use anyhow::Result;
use model::backup::{FoodBackup, UserSettingsBackup, WeightBackup};
use tempfile::NamedTempFile;

//
// Migrations
//

#[test]
fn test_migrations_apply() -> Result<()> {
    let db_file = NamedTempFile::new()?;
    let stg = StorageSqlite::new(db_file.path())?;

    assert_eq!(6, stg.get_last_migration_id().unwrap());

    Ok(())
}

//
// Weight
//

#[test]
fn test_get_weight_list() -> Result<()> {
    let db_file = NamedTempFile::new()?;
    let stg = StorageSqlite::new(db_file.path())?;

    // Check EmptyList error
    let res = stg.get_weight_list(
        1,
        Timestamp::from_unix_millis(0).unwrap(),
        Timestamp::from_unix_millis(10).unwrap(),
    );

    assert!(stg.is_storage_error(StorageError::EmptyList, &res.unwrap_err()));

    // Add test data
    stg.raw_execute(
        "
            INSERT INTO weight(user_id, timestamp, value)
            VALUES 
                (1, 1, 1.1),
                (1, 2, 2.2),
                (1, 3, 3.3),
                (2, 4, 4.4)
            ;
        ",
        false,
        params![],
    )?;

    // Check weight list for user 1
    let res = stg.get_weight_list(
        1,
        Timestamp::from_unix_millis(0).unwrap(),
        Timestamp::from_unix_millis(10).unwrap(),
    );
    assert_eq!(
        vec![
            Weight {
                timestamp: Timestamp::from_unix_millis(1).unwrap(),
                value: 1.1
            },
            Weight {
                timestamp: Timestamp::from_unix_millis(2).unwrap(),
                value: 2.2
            },
            Weight {
                timestamp: Timestamp::from_unix_millis(3).unwrap(),
                value: 3.3
            },
        ],
        res.unwrap()
    );

    // Check weight list for user 2
    let res = stg.get_weight_list(
        2,
        Timestamp::from_unix_millis(0).unwrap(),
        Timestamp::from_unix_millis(10).unwrap(),
    );
    assert_eq!(
        vec![Weight {
            timestamp: Timestamp::from_unix_millis(4).unwrap(),
            value: 4.4
        },],
        res.unwrap()
    );

    Ok(())
}

#[test]
fn test_delete_weight() -> Result<()> {
    let db_file = NamedTempFile::new()?;
    let stg = StorageSqlite::new(db_file.path())?;

    // Add test data
    stg.raw_execute(
        "
            INSERT INTO weight(user_id, timestamp, value)
            VALUES 
                (1, 1, 1.1),
                (2, 4, 4.4)
            ;
        ",
        false,
        params![],
    )?;

    // Delete for user 2
    stg.delete_weight(2, Timestamp::from_unix_millis(4).unwrap())?;
    let res = stg.get_weight_list(
        2,
        Timestamp::from_unix_millis(0).unwrap(),
        Timestamp::from_unix_millis(10).unwrap(),
    );
    assert!(stg.is_storage_error(StorageError::EmptyList, &res.unwrap_err()));

    // Delete for user 1 record that not exists (timestamp=4)
    stg.delete_weight(1, Timestamp::from_unix_millis(4).unwrap())?;
    assert_eq!(
        1,
        stg.get_weight_list(
            1,
            Timestamp::from_unix_millis(0).unwrap(),
            Timestamp::from_unix_millis(10).unwrap(),
        )?
        .len()
    );

    Ok(())
}

#[test]
fn test_set_weight() -> Result<()> {
    let db_file = NamedTempFile::new()?;
    let stg = StorageSqlite::new(db_file.path())?;

    // Set invalid weight
    let res = stg.set_weight(
        1,
        &Weight {
            timestamp: Timestamp::from_unix_millis(1734876557).unwrap(),
            value: -1.1,
        },
    );
    assert!(stg.is_storage_error(StorageError::InvalidWeight, &res.unwrap_err()));

    // Set weight
    stg.set_weight(
        1,
        &Weight {
            timestamp: Timestamp::from_unix_millis(1734876557).unwrap(),
            value: 1.1,
        },
    )?;

    // Check in DB
    let res = stg.raw_query(
        "SELECT timestamp, value FROM weight WHERE user_id = 1",
        params![],
    )?;

    assert_eq!(1, res.len());
    assert_eq!(
        Timestamp::from_unix_millis(1734876557).unwrap(),
        StorageSqlite::get_timestamp(res.first().unwrap(), "timestamp").unwrap()
    );
    assert_eq!(
        1.1,
        StorageSqlite::get_float(res.first().unwrap(), "value").unwrap()
    );

    // Update weight
    stg.set_weight(
        1,
        &Weight {
            timestamp: Timestamp::from_unix_millis(1734876557).unwrap(),
            value: 2.2,
        },
    )?;

    // Check in DB
    let res = stg.raw_query(
        "SELECT timestamp, value FROM weight WHERE user_id = 1",
        params![],
    )?;

    assert_eq!(1, res.len());
    assert_eq!(
        Timestamp::from_unix_millis(1734876557).unwrap(),
        StorageSqlite::get_timestamp(res.first().unwrap(), "timestamp").unwrap()
    );
    assert_eq!(
        2.2,
        StorageSqlite::get_float(res.first().unwrap(), "value").unwrap()
    );

    Ok(())
}

//
// Food
//

#[test]
fn test_set_food() -> Result<()> {
    let db_file = NamedTempFile::new()?;
    let stg = StorageSqlite::new(db_file.path())?;

    // Set invalid food
    let res = stg.set_food(&Food {
        key: "".into(),
        name: "name".into(),
        brand: "brand".into(),
        cal100: 1.1,
        prot100: 2.2,
        fat100: 3.3,
        carb100: 4.4,
        comment: "comment".into(),
    });
    assert!(stg.is_storage_error(StorageError::InvalidFood, &res.unwrap_err()));

    // Set food
    stg.set_food(&Food {
        key: "key".into(),
        name: "name".into(),
        brand: "brand".into(),
        cal100: 1.1,
        prot100: 2.2,
        fat100: 3.3,
        carb100: 4.4,
        comment: "comment".into(),
    })?;

    // Check in DB
    let res = stg.raw_query(
        r#"
            SELECT
                key, name, brand, cal100,
                prot100, fat100, carb100, comment
            FROM food
        "#,
        params![],
    )?;

    assert_eq!(1, res.len());
    assert_eq!(
        String::from("key"),
        StorageSqlite::get_string(res.first().unwrap(), "key").unwrap()
    );
    assert_eq!(
        String::from("name"),
        StorageSqlite::get_string(res.first().unwrap(), "name").unwrap()
    );
    assert_eq!(
        String::from("brand"),
        StorageSqlite::get_string(res.first().unwrap(), "brand").unwrap()
    );
    assert_eq!(
        1.1,
        StorageSqlite::get_float(res.first().unwrap(), "cal100").unwrap()
    );
    assert_eq!(
        2.2,
        StorageSqlite::get_float(res.first().unwrap(), "prot100").unwrap()
    );
    assert_eq!(
        3.3,
        StorageSqlite::get_float(res.first().unwrap(), "fat100").unwrap()
    );
    assert_eq!(
        4.4,
        StorageSqlite::get_float(res.first().unwrap(), "carb100").unwrap()
    );
    assert_eq!(
        String::from("comment"),
        StorageSqlite::get_string(res.first().unwrap(), "comment").unwrap()
    );

    // Update food
    stg.set_food(&Food {
        key: "key".into(),
        name: "name".into(),
        brand: "".into(),
        cal100: 5.5,
        prot100: 6.6,
        fat100: 7.7,
        carb100: 8.8,
        comment: "".into(),
    })?;

    // Check in DB
    let res = stg.raw_query(
        r#"
            SELECT
                key, name, brand, cal100,
                prot100, fat100, carb100, comment
            FROM food
        "#,
        params![],
    )?;

    assert_eq!(1, res.len());
    assert_eq!(
        String::from("key"),
        StorageSqlite::get_string(res.first().unwrap(), "key").unwrap()
    );
    assert_eq!(
        String::from("name"),
        StorageSqlite::get_string(res.first().unwrap(), "name").unwrap()
    );
    assert_eq!(
        String::from(""),
        StorageSqlite::get_string(res.first().unwrap(), "brand").unwrap()
    );
    assert_eq!(
        5.5,
        StorageSqlite::get_float(res.first().unwrap(), "cal100").unwrap()
    );
    assert_eq!(
        6.6,
        StorageSqlite::get_float(res.first().unwrap(), "prot100").unwrap()
    );
    assert_eq!(
        7.7,
        StorageSqlite::get_float(res.first().unwrap(), "fat100").unwrap()
    );
    assert_eq!(
        8.8,
        StorageSqlite::get_float(res.first().unwrap(), "carb100").unwrap()
    );
    assert_eq!(
        String::from(""),
        StorageSqlite::get_string(res.first().unwrap(), "comment").unwrap()
    );

    Ok(())
}

#[test]
fn test_get_food() -> Result<()> {
    let db_file = NamedTempFile::new()?;
    let stg = StorageSqlite::new(db_file.path())?;

    // Get food that not exists
    let res = stg.get_food("key");
    assert!(stg.is_storage_error(StorageError::NotFound, &res.unwrap_err()));

    // Set food
    let f = Food {
        key: "key".into(),
        name: "name".into(),
        brand: "brand".into(),
        cal100: 1.1,
        prot100: 2.2,
        fat100: 3.3,
        carb100: 4.4,
        comment: "comment".into(),
    };
    stg.set_food(&f)?;

    // Get food
    assert_eq!(f, stg.get_food("key").unwrap());

    Ok(())
}

#[test]
fn test_get_food_list() -> Result<()> {
    let db_file = NamedTempFile::new()?;
    let stg = StorageSqlite::new(db_file.path())?;

    // Get empty food list
    let res = stg.get_food_list();
    assert!(stg.is_storage_error(StorageError::EmptyList, &res.unwrap_err()));

    // Set food
    let f1 = Food {
        key: "key1".into(),
        name: "name1".into(),
        brand: "brand".into(),
        cal100: 1.1,
        prot100: 2.2,
        fat100: 3.3,
        carb100: 4.4,
        comment: "comment".into(),
    };
    stg.set_food(&f1)?;

    let f2 = Food {
        key: "key2".into(),
        name: "name2".into(),
        brand: "brand".into(),
        cal100: 1.1,
        prot100: 2.2,
        fat100: 3.3,
        carb100: 4.4,
        comment: "comment".into(),
    };
    stg.set_food(&f2)?;

    // Get food list
    assert_eq!(vec![f1, f2], stg.get_food_list().unwrap());

    Ok(())
}

#[test]
fn test_delete_food() -> Result<()> {
    let db_file = NamedTempFile::new()?;
    let stg = StorageSqlite::new(db_file.path())?;

    // Set food
    let f1 = Food {
        key: "key1".into(),
        name: "name1".into(),
        brand: "brand".into(),
        cal100: 1.1,
        prot100: 2.2,
        fat100: 3.3,
        carb100: 4.4,
        comment: "comment".into(),
    };
    stg.set_food(&f1)?;

    let f2 = Food {
        key: "key2".into(),
        name: "name2".into(),
        brand: "brand".into(),
        cal100: 1.1,
        prot100: 2.2,
        fat100: 3.3,
        carb100: 4.4,
        comment: "comment".into(),
    };
    stg.set_food(&f2)?;

    // Get food list
    assert_eq!(vec![f1, f2.clone()], stg.get_food_list().unwrap());

    // Delete food1
    stg.delete_food("key1")?;

    // Get food list
    assert_eq!(vec![f2], stg.get_food_list().unwrap());

    // Delete food2
    stg.delete_food("key2")?;

    // Get food list
    let res = stg.get_food_list();
    assert!(stg.is_storage_error(StorageError::EmptyList, &res.unwrap_err()));

    Ok(())
}

#[test]
fn test_delete_food_with_bundle() -> Result<()> {
    let db_file = NamedTempFile::new()?;
    let stg = StorageSqlite::new(db_file.path())?;

    // Set food
    stg.set_food(&Food {
        key: "key1".into(),
        name: "name1".into(),
        brand: "brand".into(),
        cal100: 1.1,
        prot100: 2.2,
        fat100: 3.3,
        carb100: 4.4,
        comment: "comment".into(),
    })?;

    stg.set_food(&Food {
        key: "key2".into(),
        name: "name2".into(),
        brand: "brand".into(),
        cal100: 1.1,
        prot100: 2.2,
        fat100: 3.3,
        carb100: 4.4,
        comment: "comment".into(),
    })?;

    // Set bundle
    stg.set_bundle(
        1,
        &Bundle {
            key: "bndl_key".into(),
            data: HashMap::from([("key1".into(), 123.123)]),
        },
    )?;

    // Check delete food, that is used in bundle
    let res = stg.delete_food("key1");
    assert!(stg.is_storage_error(StorageError::FoodIsUsed, &res.unwrap_err()));

    // Delete food that not used
    stg.delete_food("key2")?;

    Ok(())
}

#[test]
fn test_find_food() -> Result<()> {
    let db_file = NamedTempFile::new()?;
    let stg = StorageSqlite::new(db_file.path())?;

    // Find empty result
    let res = stg.find_food("some food");
    assert!(stg.is_storage_error(StorageError::EmptyList, &res.unwrap_err()));

    // Set food
    let f1 = Food {
        key: "key1".into(),
        name: "name1".into(),
        brand: "brand".into(),
        cal100: 1.1,
        prot100: 2.2,
        fat100: 3.3,
        carb100: 4.4,
        comment: "comment".into(),
    };
    stg.set_food(&f1)?;

    let f2 = Food {
        key: "key2".into(),
        name: "name2".into(),
        brand: "brand".into(),
        cal100: 1.1,
        prot100: 2.2,
        fat100: 3.3,
        carb100: 4.4,
        comment: "comment".into(),
    };
    stg.set_food(&f2)?;

    let f3 = Food {
        key: "key3".into(),
        name: "Сырок Дружба".into(),
        brand: "Вкусвилл".into(),
        cal100: 1.1,
        prot100: 2.2,
        fat100: 3.3,
        carb100: 4.4,
        comment: "Вкусный".into(),
    };
    stg.set_food(&f3)?;

    // Find food
    assert_eq!(
        vec![f1.clone(), f2.clone(), f3.clone()],
        stg.find_food("kEy").unwrap()
    );
    assert_eq!(vec![f2], stg.find_food("NAMe2").unwrap());
    assert_eq!(vec![f3.clone()], stg.find_food("дружба").unwrap());
    assert_eq!(vec![f3.clone()], stg.find_food("вкусВиЛЛ").unwrap());
    assert_eq!(vec![f3.clone()], stg.find_food("нЫЙ").unwrap());

    Ok(())
}

//
// Sport
//

#[test]
fn test_set_sport() -> Result<()> {
    let db_file = NamedTempFile::new()?;
    let stg = StorageSqlite::new(db_file.path())?;

    // Set invalid sport
    let res = stg.set_sport(&Sport {
        key: "".into(),
        name: "name".into(),
        comment: "comment".into(),
    });
    assert!(stg.is_storage_error(StorageError::InvalidSport, &res.unwrap_err()));

    // Set sport
    stg.set_sport(&Sport {
        key: "key".into(),
        name: "name".into(),
        comment: "comment".into(),
    })?;

    // Check in DB
    let res = stg.raw_query(
        r#"
            SELECT
                key, name, comment
            FROM sport
        "#,
        params![],
    )?;

    assert_eq!(1, res.len());
    assert_eq!(
        String::from("key"),
        StorageSqlite::get_string(res.first().unwrap(), "key").unwrap()
    );
    assert_eq!(
        String::from("name"),
        StorageSqlite::get_string(res.first().unwrap(), "name").unwrap()
    );
    assert_eq!(
        String::from("comment"),
        StorageSqlite::get_string(res.first().unwrap(), "comment").unwrap()
    );

    // Update sport
    stg.set_sport(&Sport {
        key: "key".into(),
        name: "name".into(),
        comment: "".into(),
    })?;

    // Check in DB
    let res = stg.raw_query(
        r#"
            SELECT
                key, name, comment
            FROM sport
        "#,
        params![],
    )?;

    assert_eq!(1, res.len());
    assert_eq!(
        String::from("key"),
        StorageSqlite::get_string(res.first().unwrap(), "key").unwrap()
    );
    assert_eq!(
        String::from("name"),
        StorageSqlite::get_string(res.first().unwrap(), "name").unwrap()
    );
    assert_eq!(
        String::from(""),
        StorageSqlite::get_string(res.first().unwrap(), "comment").unwrap()
    );

    Ok(())
}

#[test]
fn test_get_sport() -> Result<()> {
    let db_file = NamedTempFile::new()?;
    let stg = StorageSqlite::new(db_file.path())?;

    // Get sport that not exists
    let res = stg.get_sport("key");
    assert!(stg.is_storage_error(StorageError::NotFound, &res.unwrap_err()));

    // Set sport
    let s = Sport {
        key: "key".into(),
        name: "name".into(),
        comment: "comment".into(),
    };
    stg.set_sport(&s)?;

    // Get sport
    assert_eq!(s, stg.get_sport("key").unwrap());

    Ok(())
}

#[test]
fn test_get_sport_list() -> Result<()> {
    let db_file = NamedTempFile::new()?;
    let stg = StorageSqlite::new(db_file.path())?;

    // Get empty sport list
    let res = stg.get_sport_list();
    assert!(stg.is_storage_error(StorageError::EmptyList, &res.unwrap_err()));

    // Set sport
    let s1 = Sport {
        key: "key1".into(),
        name: "name1".into(),
        comment: "comment".into(),
    };
    stg.set_sport(&s1)?;

    let s2 = Sport {
        key: "key2".into(),
        name: "name2".into(),
        comment: "comment".into(),
    };
    stg.set_sport(&s2)?;

    // Get sport list
    assert_eq!(vec![s1, s2], stg.get_sport_list().unwrap());

    Ok(())
}

#[test]
fn test_delete_sport() -> Result<()> {
    let db_file = NamedTempFile::new()?;
    let stg = StorageSqlite::new(db_file.path())?;

    // Set sport
    let s1 = Sport {
        key: "key1".into(),
        name: "name1".into(),
        comment: "comment".into(),
    };
    stg.set_sport(&s1)?;

    let s2 = Sport {
        key: "key2".into(),
        name: "name2".into(),
        comment: "comment".into(),
    };
    stg.set_sport(&s2)?;

    // Get sport list
    assert_eq!(vec![s1, s2.clone()], stg.get_sport_list().unwrap());

    // Delete sport1
    stg.delete_sport("key1")?;

    // Get sport list
    assert_eq!(vec![s2], stg.get_sport_list().unwrap());

    // Delete sport2
    stg.delete_sport("key2")?;

    // Get sport list
    let res = stg.get_sport_list();
    assert!(stg.is_storage_error(StorageError::EmptyList, &res.unwrap_err()));

    Ok(())
}

//
// Sport activity
//

#[test]
fn test_set_sport_activity() -> Result<()> {
    let db_file = NamedTempFile::new()?;
    let stg = StorageSqlite::new(db_file.path())?;

    // Set invalid sport activity
    let res = stg.set_sport_activity(
        1,
        &SportActivity {
            sport_key: "test".into(),
            timestamp: Timestamp::now(),
            sets: vec![],
        },
    );
    assert!(stg.is_storage_error(StorageError::InvalidSportActivity, &res.unwrap_err()));

    // Set sport activity for sport that not exists
    let res = stg.set_sport_activity(
        1,
        &SportActivity {
            sport_key: "test".into(),
            timestamp: Timestamp::now(),
            sets: vec![1, 2, 3],
        },
    );
    assert!(stg.is_storage_error(StorageError::InvalidSport, &res.unwrap_err()));

    // Set sport
    stg.set_sport(&Sport {
        key: "test".into(),
        name: "test".into(),
        comment: "".into(),
    })?;

    // Set sport activity
    stg.set_sport_activity(
        1,
        &SportActivity {
            sport_key: "test".into(),
            timestamp: Timestamp::from_unix_millis(1).unwrap(),
            sets: vec![1],
        },
    )?;

    // Check in DB
    let res = stg.raw_query(
        r#"
            SELECT
                timestamp, sport_key, sets
            FROM sport_activity
            WHERE user_id = 1
        "#,
        params![],
    )?;

    assert_eq!(1, res.len());
    assert_eq!(
        Timestamp::from_unix_millis(1).unwrap(),
        StorageSqlite::get_timestamp(res.first().unwrap(), "timestamp").unwrap()
    );
    assert_eq!(
        String::from("test"),
        StorageSqlite::get_string(res.first().unwrap(), "sport_key").unwrap()
    );
    assert_eq!(
        String::from("[1]"),
        StorageSqlite::get_string(res.first().unwrap(), "sets").unwrap()
    );

    // Update sport activity
    stg.set_sport_activity(
        1,
        &SportActivity {
            sport_key: "test".into(),
            timestamp: Timestamp::from_unix_millis(1).unwrap(),
            sets: vec![1, 2, 3],
        },
    )?;

    // Check in DB
    let res = stg.raw_query(
        r#"
            SELECT
                timestamp, sport_key, sets
            FROM sport_activity
            WHERE user_id = 1
        "#,
        params![],
    )?;

    assert_eq!(1, res.len());
    assert_eq!(
        Timestamp::from_unix_millis(1).unwrap(),
        StorageSqlite::get_timestamp(res.first().unwrap(), "timestamp").unwrap()
    );
    assert_eq!(
        String::from("test"),
        StorageSqlite::get_string(res.first().unwrap(), "sport_key").unwrap()
    );
    assert_eq!(
        String::from("[1,2,3]"),
        StorageSqlite::get_string(res.first().unwrap(), "sets").unwrap()
    );

    Ok(())
}

#[test]
fn test_get_sport_activity_report() -> Result<()> {
    let db_file = NamedTempFile::new()?;
    let stg = StorageSqlite::new(db_file.path())?;

    // Get empty report
    let res = stg.get_sport_activity_report(
        1,
        Timestamp::from_unix_millis(1).unwrap(),
        Timestamp::from_unix_millis(2).unwrap(),
    );
    assert!(stg.is_storage_error(StorageError::EmptyList, &res.unwrap_err()));

    // Set data
    stg.set_sport(&Sport {
        key: "sport1".into(),
        name: "Sport 1".into(),
        comment: "".into(),
    })?;
    stg.set_sport(&Sport {
        key: "sport2".into(),
        name: "Sport 2".into(),
        comment: "".into(),
    })?;

    stg.set_sport_activity(
        1,
        &SportActivity {
            sport_key: "sport2".into(),
            timestamp: Timestamp::from_unix_millis(1).unwrap(),
            sets: vec![1],
        },
    )?;
    stg.set_sport_activity(
        1,
        &SportActivity {
            sport_key: "sport1".into(),
            timestamp: Timestamp::from_unix_millis(1).unwrap(),
            sets: vec![1, 2],
        },
    )?;
    stg.set_sport_activity(
        1,
        &SportActivity {
            sport_key: "sport1".into(),
            timestamp: Timestamp::from_unix_millis(3).unwrap(),
            sets: vec![1, 2, 3],
        },
    )?;

    // Get report
    let res = stg.get_sport_activity_report(
        1,
        Timestamp::from_unix_millis(1).unwrap(),
        Timestamp::from_unix_millis(3).unwrap(),
    )?;

    assert_eq!(
        vec![
            SportActivityReport {
                sport_name: "Sport 1".into(),
                timestamp: Timestamp::from_unix_millis(1).unwrap(),
                sets: vec![1, 2],
            },
            SportActivityReport {
                sport_name: "Sport 2".into(),
                timestamp: Timestamp::from_unix_millis(1).unwrap(),
                sets: vec![1],
            },
            SportActivityReport {
                sport_name: "Sport 1".into(),
                timestamp: Timestamp::from_unix_millis(3).unwrap(),
                sets: vec![1, 2, 3],
            }
        ],
        res
    );

    Ok(())
}

#[test]
fn test_delete_sport_activity() -> Result<()> {
    let db_file = NamedTempFile::new()?;
    let stg = StorageSqlite::new(db_file.path())?;

    // Set data
    stg.set_sport(&Sport {
        key: "sport1".into(),
        name: "Sport 1".into(),
        comment: "".into(),
    })?;

    stg.set_sport_activity(
        1,
        &SportActivity {
            sport_key: "sport1".into(),
            timestamp: Timestamp::from_unix_millis(1).unwrap(),
            sets: vec![1],
        },
    )?;

    // Check sport activity report
    let res = stg.get_sport_activity_report(
        1,
        Timestamp::from_unix_millis(1).unwrap(),
        Timestamp::from_unix_millis(3).unwrap(),
    )?;
    assert_eq!(
        vec![SportActivityReport {
            sport_name: "Sport 1".into(),
            timestamp: Timestamp::from_unix_millis(1).unwrap(),
            sets: vec![1],
        }],
        res
    );

    // Delete sport activity
    stg.delete_sport_activity(1, Timestamp::from_unix_millis(1).unwrap(), "sport1")?;

    // Check empty report
    let res = stg.get_sport_activity_report(
        1,
        Timestamp::from_unix_millis(1).unwrap(),
        Timestamp::from_unix_millis(2).unwrap(),
    );
    assert!(stg.is_storage_error(StorageError::EmptyList, &res.unwrap_err()));

    Ok(())
}

#[test]
fn test_delete_sport_with_activity() -> Result<()> {
    let db_file = NamedTempFile::new()?;
    let stg = StorageSqlite::new(db_file.path())?;

    // Set data
    stg.set_sport(&Sport {
        key: "sport1".into(),
        name: "Sport 1".into(),
        comment: "".into(),
    })?;

    stg.set_sport_activity(
        1,
        &SportActivity {
            sport_key: "sport1".into(),
            timestamp: Timestamp::from_unix_millis(1).unwrap(),
            sets: vec![1],
        },
    )?;

    // Delet sport
    let res = stg.delete_sport("sport1");
    assert!(stg.is_storage_error(StorageError::SportIsUsedViolation, &res.unwrap_err()));

    Ok(())
}

//
// User settings
//

#[test]
fn set_user_settings() -> Result<()> {
    let db_file = NamedTempFile::new()?;
    let stg = StorageSqlite::new(db_file.path())?;

    // Set invalid user settings
    let res = stg.set_user_settings(1, &UserSettings { cal_limit: 0.0 });
    assert!(stg.is_storage_error(StorageError::InvalidUserSettings, &res.unwrap_err()));

    // Set user settings
    stg.set_user_settings(1, &UserSettings { cal_limit: 100.0 })?;

    // Check in DB
    let res = stg.raw_query(
        r#"
            SELECT cal_limit
            FROM user_settings
            WHERE user_id = 1
        "#,
        params![],
    )?;

    assert_eq!(1, res.len());
    assert_eq!(
        100.0,
        StorageSqlite::get_float(res.first().unwrap(), "cal_limit").unwrap()
    );

    // Upser user settings
    stg.set_user_settings(1, &UserSettings { cal_limit: 200.0 })?;

    // Check in DB
    let res = stg.raw_query(
        r#"
            SELECT cal_limit
            FROM user_settings
            WHERE user_id = 1
        "#,
        params![],
    )?;

    assert_eq!(1, res.len());
    assert_eq!(
        200.0,
        StorageSqlite::get_float(res.first().unwrap(), "cal_limit").unwrap()
    );

    Ok(())
}

#[test]
fn get_user_settings() -> Result<()> {
    let db_file = NamedTempFile::new()?;
    let stg = StorageSqlite::new(db_file.path())?;

    // Get settings that not exists
    let res = stg.get_user_settings(1);
    assert!(stg.is_storage_error(StorageError::NotFound, &res.unwrap_err()));

    // Set settings
    let s = UserSettings { cal_limit: 200.0 };
    stg.set_user_settings(1, &s)?;

    // Get settings
    let res = stg.get_user_settings(1)?;
    assert_eq!(s, res);

    Ok(())
}

//
// Bundle
//

#[test]
fn test_get_bundle() -> Result<()> {
    let db_file = NamedTempFile::new()?;
    let stg = StorageSqlite::new(db_file.path())?;

    // Get not existing bundle
    let res = stg.get_bundle(1, "test");
    assert!(stg.is_storage_error(StorageError::NotFound, &res.unwrap_err()));

    // Add bundle to DB
    stg.raw_execute(
        r#"
            INSERT INTO bundle(user_id, key, data)
            VALUES 
                (1, 'test', '{"bundle1": 0, "food1": 1.1}')
            ;
        "#,
        false,
        params![],
    )?;

    // Get bundle
    let res = stg.get_bundle(1, "test")?;
    assert_eq!(
        Bundle {
            key: "test".into(),
            data: HashMap::from([("bundle1".into(), 0.0), ("food1".into(), 1.1)]),
        },
        res
    );

    Ok(())
}

#[test]
fn test_get_bundle_list() -> Result<()> {
    let db_file = NamedTempFile::new()?;
    let stg = StorageSqlite::new(db_file.path())?;

    // Get empty bundle list
    let res = stg.get_bundle_list(1);
    assert!(stg.is_storage_error(StorageError::EmptyList, &res.unwrap_err()));

    // Add bundle to DB
    stg.raw_execute(
        r#"
            INSERT INTO bundle(user_id, key, data)
            VALUES 
                (1, 'test', '{"bundle1": 0, "food1": 1.1}'),
                (1, 'test2', '{"bundle2": 0}')
            ;
        "#,
        false,
        params![],
    )?;

    // Get bundle list
    let res = stg.get_bundle_list(1)?;
    assert_eq!(
        vec![
            Bundle {
                key: "test".into(),
                data: HashMap::from([("bundle1".into(), 0.0), ("food1".into(), 1.1)]),
            },
            Bundle {
                key: "test2".into(),
                data: HashMap::from([("bundle2".into(), 0.0)]),
            }
        ],
        res
    );

    Ok(())
}

#[test]
fn test_set_bundle() -> Result<()> {
    let db_file = NamedTempFile::new()?;
    let stg = StorageSqlite::new(db_file.path())?;

    // Check invalid bundle
    for b in [
        &Bundle {
            key: "".into(),
            data: HashMap::new(),
        },
        &Bundle {
            key: "key".into(),
            data: HashMap::new(),
        },
        &Bundle {
            key: "key".into(),
            data: HashMap::from([("food1".into(), -1.0)]),
        },
    ] {
        let res = stg.set_bundle(1, b);
        assert!(stg.is_storage_error(StorageError::InvalidBundle, &res.unwrap_err()));
    }

    // Check errors
    let res = stg.set_bundle(
        1,
        &Bundle {
            key: "bndl_key".into(),
            data: HashMap::from([("bndl_key".into(), 0.0)]),
        },
    );
    assert!(stg.is_storage_error(StorageError::BundleDepRecursive, &res.unwrap_err()));

    let res = stg.set_bundle(
        1,
        &Bundle {
            key: "bndl_key".into(),
            data: HashMap::from([("bndl_key2".into(), 0.0)]),
        },
    );
    assert!(stg.is_storage_error(StorageError::BundleDepBundleNotFound, &res.unwrap_err()));

    let res = stg.set_bundle(
        1,
        &Bundle {
            key: "bndl_key".into(),
            data: HashMap::from([("food_key".into(), 1.0)]),
        },
    );
    assert!(stg.is_storage_error(StorageError::BundleDepFoodNotFound, &res.unwrap_err()));

    // Set initial data
    stg.set_food(&Food {
        key: "food_key".into(),
        name: "name".into(),
        brand: "brand".into(),
        cal100: 1.1,
        prot100: 2.2,
        fat100: 3.3,
        carb100: 4.4,
        comment: "comment".into(),
    })?;

    // Set bundle
    stg.set_bundle(
        1,
        &Bundle {
            key: "bndl_key".into(),
            data: HashMap::from([("food_key".into(), 123.123)]),
        },
    )?;

    // Set another bundle
    stg.set_bundle(
        1,
        &Bundle {
            key: "bndl_key_2".into(),
            data: HashMap::from([("food_key".into(), 123.123)]),
        },
    )?;

    // Update bundle
    stg.set_bundle(
        1,
        &Bundle {
            key: "bndl_key".into(),
            data: HashMap::from([("food_key".into(), 123.123), ("bndl_key_2".into(), 0.0)]),
        },
    )?;

    // Get bundle list
    let res = stg.get_bundle_list(1)?;
    assert_eq!(
        vec![
            Bundle {
                key: "bndl_key".into(),
                data: HashMap::from([("food_key".into(), 123.123), ("bndl_key_2".into(), 0.0)]),
            },
            Bundle {
                key: "bndl_key_2".into(),
                data: HashMap::from([("food_key".into(), 123.123)]),
            }
        ],
        res
    );

    Ok(())
}

#[test]
fn test_delete_bundle() -> Result<()> {
    let db_file = NamedTempFile::new()?;
    let stg = StorageSqlite::new(db_file.path())?;

    // Set initial data
    stg.set_food(&Food {
        key: "food_key".into(),
        name: "name".into(),
        brand: "brand".into(),
        cal100: 1.1,
        prot100: 2.2,
        fat100: 3.3,
        carb100: 4.4,
        comment: "comment".into(),
    })?;

    stg.set_bundle(
        1,
        &Bundle {
            key: "bndl_key_2".into(),
            data: HashMap::from([("food_key".into(), 123.123)]),
        },
    )?;

    stg.set_bundle(
        1,
        &Bundle {
            key: "bndl_key".into(),
            data: HashMap::from([("food_key".into(), 123.123), ("bndl_key_2".into(), 0.0)]),
        },
    )?;

    // Try delete when used
    let res = stg.delete_bundle(1, "bndl_key_2");
    assert!(stg.is_storage_error(StorageError::BundleIsUsed, &res.unwrap_err()));

    // Delete correct
    stg.delete_bundle(1, "bndl_key")?;
    stg.delete_bundle(1, "bndl_key_2")?;

    Ok(())
}

//
// Restore/backup
//

#[test]
fn test_restore() -> Result<()> {
    let db_file = NamedTempFile::new()?;
    let stg = StorageSqlite::new(db_file.path())?;

    // Do restore
    stg.restore(&Backup {
        timestamp: 1,
        weight: vec![
            WeightBackup {
                timestamp: 1,
                user_id: 1,
                value: 1.1,
            },
            WeightBackup {
                timestamp: 2,
                user_id: 1,
                value: 2.2,
            },
            WeightBackup {
                timestamp: 3,
                user_id: 1,
                value: 3.3,
            },
            WeightBackup {
                timestamp: 4,
                user_id: 2,
                value: 4.4,
            },
        ],
        food: vec![
            FoodBackup {
                key: "key2".into(),
                name: "Food 2".into(),
                brand: "Brand2".into(),
                cal100: 5.5,
                prot100: 6.6,
                fat100: 7.7,
                carb100: 8.8,
                comment: "Comment2".into(),
            },
            FoodBackup {
                key: "key1".into(),
                name: "Food 1".into(),
                brand: "Brand 1".into(),
                cal100: 1.1,
                prot100: 2.2,
                fat100: 3.3,
                carb100: 4.4,
                comment: "Comment1".into(),
            },
            FoodBackup {
                key: "key4".into(),
                name: "Еда 4".into(),
                brand: "Брэнд 4".into(),
                cal100: 100.100,
                prot100: 200.200,
                fat100: 300.300,
                carb100: 400.400,
                comment: "Комментарий 4".into(),
            },
            FoodBackup {
                key: "key3".into(),
                name: "Еда 3".into(),
                brand: "Брэнд 3".into(),
                cal100: 10.10,
                prot100: 20.20,
                fat100: 30.30,
                carb100: 40.40,
                comment: "Комментарий 3".into(),
            },
        ],
        user_settings: vec![
            UserSettingsBackup {
                user_id: 1,
                cal_limit: 1.0,
            },
            UserSettingsBackup {
                user_id: 2,
                cal_limit: 2.0,
            },
        ],
    })?;

    // Check weight list for user 1
    let res = stg.get_weight_list(
        1,
        Timestamp::from_unix_millis(0).unwrap(),
        Timestamp::from_unix_millis(10).unwrap(),
    )?;
    assert_eq!(
        vec![
            Weight {
                timestamp: Timestamp::from_unix_millis(1).unwrap(),
                value: 1.1
            },
            Weight {
                timestamp: Timestamp::from_unix_millis(2).unwrap(),
                value: 2.2
            },
            Weight {
                timestamp: Timestamp::from_unix_millis(3).unwrap(),
                value: 3.3
            },
        ],
        res
    );

    // Check weight list for user 2
    let res = stg.get_weight_list(
        2,
        Timestamp::from_unix_millis(0).unwrap(),
        Timestamp::from_unix_millis(10).unwrap(),
    )?;
    assert_eq!(
        vec![Weight {
            timestamp: Timestamp::from_unix_millis(4).unwrap(),
            value: 4.4
        },],
        res
    );

    // Check food
    let res = stg.get_food_list()?;
    assert_eq!(
        vec![
            Food {
                key: "key1".into(),
                name: "Food 1".into(),
                brand: "Brand 1".into(),
                cal100: 1.1,
                prot100: 2.2,
                fat100: 3.3,
                carb100: 4.4,
                comment: "Comment1".into(),
            },
            Food {
                key: "key2".into(),
                name: "Food 2".into(),
                brand: "Brand2".into(),
                cal100: 5.5,
                prot100: 6.6,
                fat100: 7.7,
                carb100: 8.8,
                comment: "Comment2".into(),
            },
            Food {
                key: "key3".into(),
                name: "Еда 3".into(),
                brand: "Брэнд 3".into(),
                cal100: 10.10,
                prot100: 20.20,
                fat100: 30.30,
                carb100: 40.40,
                comment: "Комментарий 3".into(),
            },
            Food {
                key: "key4".into(),
                name: "Еда 4".into(),
                brand: "Брэнд 4".into(),
                cal100: 100.100,
                prot100: 200.200,
                fat100: 300.300,
                carb100: 400.400,
                comment: "Комментарий 4".into(),
            }
        ],
        res
    );

    // Check user settings
    let res = stg.get_user_settings(1)?;
    assert_eq!(1.0, res.cal_limit);

    let res = stg.get_user_settings(2)?;
    assert_eq!(2.0, res.cal_limit);

    Ok(())
}
