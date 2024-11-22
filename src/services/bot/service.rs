use std::sync::Arc;
use std::error::Error;

use teloxide::prelude::*;
use crate::components::storage::{model::Storage, storage_sqlite::StorageSqlite};

use super::cmd_proc::CommandProcessor;
use anyhow::Result;

pub type HandlerResult = Result<(), Box<dyn Error + Send + Sync>>;

pub struct Config {
    pub token: String,
    pub allowed_user_ids: Arc<Vec<u64>>
}

pub struct Service {
    config: Config,
}

impl Service {
    pub fn new(config: Config) -> Self {
        Self { config }
    }

    pub async fn run(&self) -> Result<()> {
        let bot = Bot::new(self.config.token.clone());

        let handler = dptree::entry()           
            .branch(Update::filter_message()
                .filter(Service::filter_allowed_users)
                .endpoint(CommandProcessor::process_command));

        let stg: Arc<Box<dyn Storage>> = Arc::new(Box::new(StorageSqlite::new()?));

        Dispatcher::builder(bot, handler)
            .dependencies(dptree::deps![stg, self.config.allowed_user_ids.clone()])
            .enable_ctrlc_handler()
            .build()
            .dispatch()
            .await;

        Ok(())
    }

    fn filter_allowed_users(msg: Message, allowed_users_ids: Arc<Vec<u64>>) -> bool {
        allowed_users_ids.contains(&msg.from.unwrap().id.0)
    }
}