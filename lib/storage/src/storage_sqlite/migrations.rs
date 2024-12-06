use super::queries;
use anyhow::{anyhow, Context, Result};
use rusqlite::{Connection, Transaction};

type MigrationFn = fn(&Transaction) -> Result<()>;
type Migrations = Vec<(i64, MigrationFn)>;

pub fn apply(conn: &mut Connection, last_migration_id: i64) -> Result<()> {
    for (id, f) in get_all_migrations() {
        if id <= last_migration_id {
            continue;
        }

        let tx = conn
            .transaction()
            .with_context(|| format!("start migration [{}] transaction", id))?;

        f(&tx).with_context(|| format!("exec migration [{}] transaction", id))?;

        update_migration_id(&tx, id)
            .with_context(|| format!("update migration_id for migration [{}]", id))?;

        tx.commit()
            .with_context(|| format!("commit migration [{}] transaction", id))?;
    }

    Ok(())
}

fn update_migration_id(tx: &Transaction, migration_id: i64) -> Result<()> {
    match tx.execute(queries::UPDATE_MIGRATION_ID, [migration_id]) {
        Ok(_) => Ok(()),
        Err(err) => Err(anyhow!(err)),
    }
}

fn get_all_migrations() -> Migrations {
    vec![(1, insert_initial_migration_id), (2, create_tables)]
}

fn insert_initial_migration_id(tx: &Transaction) -> Result<()> {
    tx.execute(queries::INSERT_INITIAL_MIGRATION_ID, [])
        .map_err(|e| anyhow!(e))?;

    Ok(())
}

fn create_tables(tx: &Transaction) -> Result<()> {
    tx.execute(queries::CREATE_TABLE_WEIGHT, [])
        .map_err(|e| anyhow!(e))?;
    tx.execute(queries::CREATE_TABLE_FOOD, [])
        .map_err(|e| anyhow!(e))?;
    tx.execute(queries::CREATE_TABLE_JOURNAL, [])
        .map_err(|e| anyhow!(e))?;
    tx.execute(queries::CREATE_TABLE_BUNDLE, [])
        .map_err(|e| anyhow!(e))?;
    tx.execute(queries::CREATE_TABLE_BUNDLE_FOOD_ITEMS, [])
        .map_err(|e| anyhow!(e))?;
    tx.execute(queries::CREATE_TABLE_BUNDLE_BUNDLE_ITEMS, [])
        .map_err(|e| anyhow!(e))?;
    tx.execute(queries::CREATE_TABLE_USER_SETTINGS, [])
        .map_err(|e| anyhow!(e))?;

    Ok(())
}
