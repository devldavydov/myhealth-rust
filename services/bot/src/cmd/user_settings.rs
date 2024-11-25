use std::sync::Arc;
use storage::Storage;
use teloxide::prelude::*;

use crate::HandlerResult;

pub async fn process_user_settings_command(
    bot: Bot,
    chat_id: ChatId,
    args: Vec<&str>,
    stg: Arc<Box<dyn Storage>>,
) -> HandlerResult {
    bot.send_message(
        chat_id,
        "Управление пользовательскими настройками в разработке",
    )
    .await?;
    Ok(())
}
