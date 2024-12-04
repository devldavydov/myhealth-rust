use super::{functions::update_migration_id, queries};
use anyhow::{Context, Result};
use rusqlite::{Connection, Transaction};

type MigrationFn = fn(&Transaction) -> Result<()>;
type Migrations = Vec<(i32, MigrationFn)>;

pub fn apply(conn: &mut Connection, last_migration_id: i32) -> Result<()> {
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

fn get_all_migrations() -> Migrations {
    vec![(1, create_initial_tables)]
}

fn create_initial_tables(tx: &Transaction) -> Result<()> {
    if let Err(err) = tx.execute(queries::CREATE_WEIGHT_TABLE, []) {
        anyhow::bail!(err);
    }

    Ok(())
}
