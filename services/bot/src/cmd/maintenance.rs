use std::sync::Arc;

use chrono_tz::Tz;
use storage::Storage;
use teloxide::{prelude::*, types::ChatId, Bot};

use crate::{messages::ERR_WRONG_COMMAND, HandlerResult};

pub async fn process_maintenance(
    bot: Bot,
    chat_id: ChatId,
    args: Vec<&str>,
    stg: Arc<Box<dyn Storage>>,
    tz: Tz,
) -> HandlerResult {
    if args.is_empty() {
        log::error!("empty args");
        bot.send_message(chat_id, ERR_WRONG_COMMAND).await?;
        return Ok(());
    }

    match *args.first().unwrap() {
        "backup" => {
            backup(bot, chat_id, args[1..].to_vec(), stg, tz).await?;
        }
        _ => {
            log::error!("unknown command");
            bot.send_message(chat_id, ERR_WRONG_COMMAND).await?;
        }
    };

    Ok(())
}

async fn backup(
    bot: Bot,
    chat_id: ChatId,
    args: Vec<&str>,
    stg: Arc<Box<dyn Storage>>,
    tz: Tz,
) -> HandlerResult {
    // Get storage data for backup

    Ok(())
}
