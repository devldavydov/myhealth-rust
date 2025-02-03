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
    ORDER BY
        timestamp
";

pub const SELECT_WEIGHT_FOR_BACKUP: &str = "
    SELECT user_id, timestamp, value
    FROM weight
    ORDER BY user_id, timestamp
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

pub const SELECT_FOOD: &str = "
    SELECT 
        key, name, brand, cal100,
        prot100, fat100, carb100, comment
    FROM food
    WHERE key = ?1
";

pub const SELECT_FOOD_LIST: &str = "
    SELECT 
        key, name, brand, cal100,
        prot100, fat100, carb100, comment
    FROM food
    ORDER BY name, key
";

pub const SELECT_FOOD_FOR_BACKUP: &str = "
    SELECT 
        key, name, brand, cal100,
        prot100, fat100, carb100, comment
    FROM food
    ORDER BY key
";

pub const DELETE_FOOD: &str = "
    DELETE FROM food
    WHERE key = ?1
";

pub const FIND_FOOD: &str = "
    SELECT 
        key, name, brand, cal100,
        prot100, fat100, carb100, comment
    FROM food
    WHERE
        r_upper(key)     LIKE '%' || ?1 || '%' OR
        r_upper(name)    LIKE '%' || ?1 || '%' OR
        r_upper(brand)   LIKE '%' || ?1 || '%' OR
        r_upper(comment) LIKE '%' || ?1 || '%'
    ORDER BY name
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

pub const UPSERT_JOURNAL: &str = "
    INSERT INTO journal (
        user_id, timestamp, meal, foodkey, foodweight
    )
    VALUES (?1, ?2, ?3, ?4, ?5)
    ON CONFLICT (user_id, timestamp, meal, foodkey) DO
    UPDATE SET
        foodweight = ?5
";

pub const DELETE_JOURNAL: &str = "
    DELETE FROM journal
    WHERE user_id = ?1 AND
          timestamp = ?2 AND
          meal = ?3 AND
          foodkey = ?4
";

pub const DELETE_JOURNAL_MEAL: &str = "
    DELETE FROM journal
    WHERE user_id = ?1 AND
          timestamp = ?2 AND
          meal = ?3
";

pub const JOURNAL_REPORT: &str = "
    SELECT
        j.timestamp,
        j.meal,
        j.foodkey,
        f.name AS foodname,
        f.brand AS foodbrand,
        j.foodweight,
        j.foodweight / 100 * f.cal100 AS cal,
        j.foodweight / 100 * f.prot100 AS prot,
        j.foodweight / 100 * f.fat100 AS fat,
        j.foodweight / 100 * f.carb100 AS carb
    FROM journal j, food f
    WHERE
        j.foodkey = f.key AND
        j.user_id = ?1 AND
        j.timestamp >= ?2 AND
        j.timestamp <= ?3
    ORDER BY
        j.timestamp,
        j.meal,
        f.name
";

pub const JOURNAL_FOOD_AVG_WEIGHT: &str = "
    SELECT coalesce(avg(foodweight), 0.0) AS avg_food_weight
    FROM journal j
    WHERE
        j.user_id = ?1 AND
        j.foodkey = ?2 AND
        j.timestamp >= ?3 AND
        j.timestamp <= ?4
";

pub const SELECT_JOURNAL_FOR_BACKUP: &str = "
    SELECT user_id, timestamp, meal, foodkey, foodweight
    from journal
    ORDER BY user_id, timestamp, meal, foodkey
";

//
// Bundle
//

pub const CREATE_TABLE_BUNDLE: &str = "
    CREATE TABLE bundle (
        user_id    INTEGER NOT NULL,
        key        TEXT    NOT NULL, 
        data       TEXT    NOT NULL,  
        PRIMARY KEY (user_id, key)
    )
";

pub const SELECT_BUNDLE: &str = "
    SELECT key, data
    FROM bundle
    WHERE user_id = ?1 AND key = ?2
";

pub const SELECT_BUNDLE_LIST: &str = "
    SELECT key, data
    FROM bundle
    WHERE user_id = ?1
    ORDER by key
";

pub const SELECT_ALL_BUNDLES: &str = "
    SELECT key, data
    FROM bundle
";

pub const SELECT_BUNDLES_FOR_BACKUP: &str = "
    SELECT user_id, key, data
    FROM bundle
    ORDER BY user_id, key
";

pub const UPSERT_BUNDLE: &str = "
    INSERT INTO bundle (
        user_id, key, data
    )
    VALUES (?1, ?2, ?3)
    ON CONFLICT (user_id, key) DO
    UPDATE SET
        data = ?3
";

pub const DELETE_BUNDLE: &str = "
    DELETE FROM bundle
    WHERE user_id = ?1 AND key = ?2
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

pub const SELECT_USER_SETTINGS: &str = "
    SELECT cal_limit
    FROM user_settings
    WHERE user_id = ?1
";

pub const SELECT_USER_SETTINGS_FOR_BACKUP: &str = "
    SELECT user_id, cal_limit
    FROM user_settings
    ORDER BY user_id
";

pub const UPSERT_USER_SETTINGS: &str = "
    INSERT INTO user_settings (
        user_id, cal_limit
    )
    VALUES (?1, ?2)
    ON CONFLICT (user_id) DO
    UPDATE SET
        cal_limit = ?2
";

//
// Sport
//

pub const CREATE_TABLE_SPORT: &str = "
    CREATE TABLE sport (
        key     TEXT NOT NULL PRIMARY KEY,
        name    TEXT NOT NULL,
        comment TEXT NULL
    )
";

pub const SELECT_SPORT: &str = "
    SELECT 
        key, name, comment
    FROM sport
    WHERE key = ?1
";

pub const SELECT_SPORT_LIST: &str = "
    SELECT 
        key, name, comment
    FROM sport
    ORDER BY name
";

pub const SELECT_SPORT_FOR_BACKUP: &str = "
    SELECT 
        key, name, comment
    FROM sport
    ORDER BY key
";

pub const DELETE_SPORT: &str = "
    DELETE FROM sport
    WHERE key = ?1
";

pub const UPSERT_SPORT: &str = "
    INSERT INTO sport (
        key, name, comment
    )
    VALUES (?1, ?2, ?3)
    ON CONFLICT (key) DO
    UPDATE SET
        name = ?2, comment = ?3
";

//
// Sport activity
//

pub const CREATE_TABLE_SPORT_ACTIVITY: &str = "
    CREATE TABLE sport_activity (
        user_id   INTEGER NOT NULL,
        timestamp INTEGER NOT NULL,
        sport_key TEXT NOT NULL,
        sets      TEXT NOT NULL,
        PRIMARY KEY (user_id, timestamp, sport_key),
        FOREIGN KEY (sport_key) REFERENCES sport(key) ON DELETE RESTRICT
    )
";

pub const UPSERT_SPORT_ACTIVITY: &str = "
    INSERT INTO sport_activity (
        user_id, timestamp, sport_key, sets
    )
    VALUES (?1, ?2, ?3, ?4)
    ON CONFLICT (user_id, timestamp, sport_key) DO
    UPDATE SET
        sets = ?4
";

pub const SELECT_SPORT_ACTIVITY_REPORT: &str = "
    SELECT sa.timestamp, s.name as sport_name, sa.sets
    FROM
        sport_activity sa,
        sport s
    WHERE
        sa.sport_key = s.key AND
        user_id = ?1 AND
        timestamp >= ?2 AND
        timestamp <= ?3
    ORDER BY
        sa.timestamp,
        s.name
";

pub const SELECT_SPORT_ACTIVITY_FOR_BACKUP: &str = "
    SELECT user_id, timestamp, sport_key, sets
    FROM sport_activity 
    ORDER BY user_id, timestamp, sport_key
";

pub const DELETE_SPORT_ACTIVITY: &str = "
    DELETE FROM sport_activity
    WHERE
        user_id = ?1 AND
        timestamp = ?2 AND
        sport_key = ?3
";
