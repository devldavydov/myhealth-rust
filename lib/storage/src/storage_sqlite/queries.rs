//
// System
//

pub const CREATE_TABLE_SYSTEM: &str = "
    CREATE TABLE IF NOT EXISTS system (
        migration_id INTEGER
    )    
";

pub const INSERT_INITIAL_MIGRATION_ID: &str = "
    INSERT INTO system(migration_id) VALUES(0)
";

pub const SELECT_MIGRATION_ID: &str = "
    SELECT migration_id FROM system LIMIT 1
";

pub const UPDATE_MIGRATION_ID: &str = "
    UPDATE system SET migration_id = ?1
";

//
// Weight
//

pub const CREATE_TABLE_WEIGHT: &str = "
    CREATE TABLE weight (
        user_id   INTEGER NOT NULL,
        timestamp INTEGER NOT NULL,
        value     REAL    NOT NULL,
        PRIMARY KEY (user_id, timestamp)
    )
";

pub const SELECT_WEIGHT_LIST: &str = "
    SELECT timestamp, value
    FROM weight
    WHERE
        user_id = ?1 AND
        timestamp >= ?2 AND
        timestamp <= ?3
";

pub const DELETE_WEIGHT: &str = "
    DELETE FROM weight
    WHERE
        user_id = ?1 AND
        timestamp = ?2
";

pub const UPSERT_WEIGHT: &str = "
    INSERT INTO weight (user_id, timestamp, value)
    VALUES (?1, ?2, ?3)
    ON CONFLICT (user_id, timestamp) DO
    UPDATE SET value = ?3
";

//
// Food
//

pub const CREATE_TABLE_FOOD: &str = "
    CREATE TABLE food (
        key     TEXT NOT NULL PRIMARY KEY,
        name    TEXT NOT NULL,
        brand   TEXT NULL,
        cal100  REAL NOT NULL,
        prot100 REAL NOT NULL, 
        fat100  REAL NOT NULL,
        carb100 REAL NOT NULL,
        comment TEXT NULL
    )
";

pub const UPSERT_FOOD: &str = "
    INSERT INTO food (
        key, name, brand, cal100,
        prot100, fat100, carb100, comment
    )
    VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
    ON CONFLICT (key) DO
    UPDATE SET
        name = ?2, brand = ?3, cal100 = ?4,
        prot100 = ?5, fat100 = ?6, carb100 = ?7,
        comment = ?8
";

//
// Journal
//

pub const CREATE_TABLE_JOURNAL: &str = "
    CREATE TABLE journal (
        user_id    INTEGER NOT NULL,
        timestamp  INTEGER NOT NULL,
        meal       INTEGER NOT NULL,
        foodkey    TEXT NOT NULL,
        foodweight REAL NOT NULL,
        PRIMARY KEY (user_id, timestamp, meal, foodkey),
        FOREIGN KEY (foodkey) REFERENCES food(key) ON DELETE RESTRICT
    )
";

//
// Bundle
//

pub const CREATE_TABLE_BUNDLE: &str = "
    CREATE TABLE bundle (
        user_id    INTEGER NOT NULL,
        key        TEXT    NOT NULL,        
        PRIMARY KEY (user_id, key)
    )
";

pub const CREATE_TABLE_BUNDLE_FOOD_ITEMS: &str = "
    CREATE TABLE bundle_food_items (
        bundle_key  INTEGER NOT NULL,
        food_key    INTEGER NOT NULL,
        food_weight REAL    NOT NULL,
        PRIMARY KEY (bundle_key, food_key),
        FOREIGN KEY (bundle_key) REFERENCES bundle(key) ON DELETE RESTRICT,
        FOREIGN KEY (food_key) REFERENCES food(key) ON DELETE RESTRICT
    )
";

pub const CREATE_TABLE_BUNDLE_BUNDLE_ITEMS: &str = "
    CREATE TABLE bundle_bundle_items (
        bundle_key       INTEGER NOT NULL,
        child_bundle_key INTEGER NOT NULL,
        PRIMARY KEY (bundle_key, child_bundle_key),
        FOREIGN KEY (bundle_key) REFERENCES bundle(key) ON DELETE RESTRICT,
        FOREIGN KEY (child_bundle_key) REFERENCES bundle(key) ON DELETE RESTRICT
    )
";

//
// User settings
//

pub const CREATE_TABLE_USER_SETTINGS: &str = "
    CREATE TABLE user_settings (
        user_id   INTEGER NOT NULL PRIMARY KEY,
        cal_limit REAL    NOT NULL
    )
";
