use teloxide::prelude::*;
use std::sync::Arc;

use crate::components::storage::model::Storage;

pub struct CommandProcessor;

impl CommandProcessor {
    pub async fn process_command(bot: Bot, msg: Message, stg: Arc<Box<dyn Storage>>) -> super::service::HandlerResult {
        bot.send_message(msg.chat.id, "В разработке").await?;
        Ok(())
    }
}