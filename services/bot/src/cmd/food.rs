use std::sync::Arc;
use storage::Storage;
use teloxide::prelude::*;

use crate::HandlerResult;

pub async fn process_food_command(
    bot: Bot,
    chat_id: ChatId,
    args: Vec<&str>,
    stg: Arc<Box<dyn Storage>>,
) -> HandlerResult {
    bot.send_message(chat_id, "Управление едой в разработке")
        .await?;
    Ok(())
}
