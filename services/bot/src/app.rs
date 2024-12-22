use std::{path::Path, sync::Arc};

use env_logger::{Builder, Env};
use storage::{storage_sqlite::StorageSqlite, storage_sqlite::DB_FILE, Storage};
use teloxide::prelude::*;

use super::args::ArgsCli;
use super::cmd;
use super::config::Config;
use anyhow::{Context, Result};
use chrono_tz::Tz;

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

        let stg: Arc<Box<dyn Storage>> = Arc::new(Box::new(
            StorageSqlite::new(Path::new(DB_FILE)).context("new sqlite storage")?,
        ));

        let tz: Tz = self.config.tz.parse()?;

        let runtime = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()?;

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
