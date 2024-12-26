use std::{fs, io::Read, path::Path, sync::Arc};

use env_logger::{Builder, Env};
use flate2::read::GzDecoder;
use model::backup::Backup;
use storage::{storage_sqlite::StorageSqlite, Storage};
use teloxide::prelude::*;

use super::args::ArgsCli;
use super::cmd;
use super::config::Config;
use anyhow::{Context, Result};
use chrono_tz::Tz;

const BACKUP_FILE: &str = "backup.json.gz";

pub struct App {
    config: Config,
}

impl App {
    pub fn new(args: ArgsCli) -> Self {
        Self {
            config: Config::new(args),
        }
    }

    fn init_logging(&self) {
        let mut log_builder = Builder::from_env(Env::default());
        log_builder
            .format(|buf, record| {
                use std::io::Write;
                writeln!(
                    buf,
                    "[{} {} {}:{}] > {}",
                    buf.timestamp(),
                    record.level(),
                    record.target(),
                    record.line().unwrap_or_default(),
                    record.args()
                )
            })
            .init();
    }

    fn filter_allowed_users(msg: Message, allowed_users_ids: Arc<Vec<u64>>) -> bool {
        match msg.from {
            Some(usr) => allowed_users_ids.contains(&usr.id.0),
            _ => false,
        }
    }

    fn try_get_backup(&self) -> Result<Option<Backup>> {
        if !fs::exists(BACKUP_FILE).context("check backup file exists")? {
            return Ok(None);
        }

        let f_data = fs::read(BACKUP_FILE).context("read backup file")?;
        let mut gz = GzDecoder::new(&f_data[..]);
        let mut json_data = Vec::new();
        gz.read_to_end(&mut json_data).context("gunzip data")?;

        let backup: Backup = serde_json::from_slice(&json_data[..]).context("json decode")?;

        Ok(Some(backup))
    }
}

impl service::Service for App {
    fn run(&mut self) -> Result<()> {
        self.init_logging();

        log::info!("Starting MyHealth bot...");

        let bot = Bot::new(self.config.token.clone());

        let handler = dptree::entry().branch(
            Update::filter_message()
                .filter(App::filter_allowed_users)
                .endpoint(cmd::process_command),
        );

        let tz: Tz = self.config.tz.parse().context("tz parse")?;

        // Init storage
        let stg: Arc<Box<dyn Storage>> = Arc::new(Box::new(
            StorageSqlite::new(Path::new(&self.config.db_file_path))
                .context("new sqlite storage")?,
        ));

        if let Some(backup) = self.try_get_backup().context("try get backup")? {
            stg.restore(&backup).context("storage backup")?;
        }

        // Init runtime

        let runtime = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .context("build tokio runtime")?;

        runtime.block_on(async {
            Dispatcher::builder(bot, handler)
                .dependencies(dptree::deps![
                    stg.clone(),
                    self.config.allowed_user_ids.clone(),
                    tz
                ])
                .enable_ctrlc_handler()
                .build()
                .dispatch()
                .await;

            Ok(())
        })
    }
}
