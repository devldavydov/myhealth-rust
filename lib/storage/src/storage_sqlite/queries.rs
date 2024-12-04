pub const CREATE_TABLE_SYSTEM: &str = "
    CREATE TABLE IF NOT EXISTS system (
        migration_id INTEGER
    );
    INSERT INTO system(migration_id) VALUES(0);
";

pub const SELECT_MIGRATION_ID: &str = "
    SELECT migration_id FROM system LIMIT 1
";

pub const UPDATE_MIGRATION_ID: &str = "
    UPDATE system SET migration_id = ?1
";

pub const CREATE_WEIGHT_TABLE: &str = "
    CREATE TABLE weight (
        id        INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
        user_id   INTEGER NOT NULL,
        timestamp INTEGER NOT NULL,
        value     REAL    NOT NULL
    );
";