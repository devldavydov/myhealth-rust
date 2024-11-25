use std::sync::Arc;
use storage::Storage;
use teloxide::prelude::*;
pub struct CommandProcessor;

impl CommandProcessor {
    pub async fn process_command(
        bot: Bot,
        msg: Message,
        stg: Arc<Box<dyn Storage>>,
    ) -> super::app::HandlerResult {
        bot.send_message(msg.chat.id, "В разработке").await?;
        Ok(())
    }
}
