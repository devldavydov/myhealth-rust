use super::queries;
use anyhow::{Context, Result};
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
            .with_context(|| format!("update migration id for migration [{}]", id))?;

        tx.commit()
            .with_context(|| format!("commit migration [{}] transaction", id))?;
    }

    Ok(())
}

fn update_migration_id(tx: &Transaction, migration_id: i64) -> Result<()> {
    tx.execute(queries::UPDATE_MIGRATION_ID, [migration_id])
        .context("exec update migration id query")?;
    Ok(())
}

fn get_all_migrations() -> Migrations {
    vec![
        (1, insert_initial_migration_id),
        (2, create_tables_weight_food),
        (3, create_table_sport),
        (4, create_table_sport_activity),
        (5, create_table_user_settings),
        (6, create_table_bundle),
        (7, create_table_journal),
    ]
}

fn insert_initial_migration_id(tx: &Transaction) -> Result<()> {
    tx.execute(queries::INSERT_INITIAL_MIGRATION_ID, [])
        .context("exec initial migration id query")?;
    Ok(())
}

fn create_tables_weight_food(tx: &Transaction) -> Result<()> {
    tx.execute(queries::CREATE_TABLE_WEIGHT, [])
        .context("exec create table weight")?;
    tx.execute(queries::CREATE_TABLE_FOOD, [])
        .context("exec create table food")?;

    Ok(())
}

fn create_table_sport(tx: &Transaction) -> Result<()> {
    tx.execute(queries::CREATE_TABLE_SPORT, [])
        .context("exec create table sport")?;

    Ok(())
}

fn create_table_sport_activity(tx: &Transaction) -> Result<()> {
    tx.execute(queries::CREATE_TABLE_SPORT_ACTIVITY, [])
        .context("exec create table sport activity")?;

    Ok(())
}

fn create_table_user_settings(tx: &Transaction) -> Result<()> {
    tx.execute(queries::CREATE_TABLE_USER_SETTINGS, [])
        .context("exec create table user settings")?;

    Ok(())
}

fn create_table_bundle(tx: &Transaction) -> Result<()> {
    tx.execute(queries::CREATE_TABLE_BUNDLE, [])
        .context("exec create table bundle")?;

    Ok(())
}

fn create_table_journal(tx: &Transaction) -> Result<()> {
    tx.execute(queries::CREATE_TABLE_JOURNAL, [])
        .context("exec create table journal")?;

    Ok(())
}
