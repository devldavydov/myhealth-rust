use super::queries;
use anyhow::{anyhow, Context, Result};
use rusqlite::{Connection, Transaction};

pub fn get_last_migration_id(conn: &mut Connection) -> Result<i32> {
    let mut stmt = conn
        .prepare(queries::SELECT_MIGRATION_ID)
        .context("prepare query for current migration")?;
    let mut rows = stmt
        .query_map([], |row| row.get(0))
        .context("quering current migration")?;

    if let Some(val) = rows.next() {
        return val.context("get current migration value");
    }

    anyhow::bail!("failed to get system migration_id");
}

pub fn update_migration_id(tx: &Transaction, migration_id: i32) -> Result<()> {
    match tx.execute(queries::UPDATE_MIGRATION_ID, [migration_id]) {
        Ok(_) => Ok(()),
        Err(err) => Err(anyhow!(err)),
    }
}
