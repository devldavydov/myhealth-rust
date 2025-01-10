use model::UserSettings;
use std::sync::Arc;
use storage::{Storage, StorageError};
use teloxide::{prelude::*, types::ParseMode};

use crate::{
    messages::{ERR_INTERNAL, ERR_USER_SETTINGS_NOT_FOUND, ERR_WRONG_COMMAND, OK},
    HandlerResult,
};

pub async fn process_user_settings_command(
    bot: Bot,
    user_id: i64,
    chat_id: ChatId,
    args: Vec<&str>,
    stg: Arc<Box<dyn Storage>>,
) -> HandlerResult {
    if args.is_empty() {
        log::error!("empty args");
        bot.send_message(chat_id, ERR_WRONG_COMMAND).await?;
        return Ok(());
    }

    match *args.first().unwrap() {
        "set" => {
            user_settings_set(bot, user_id, chat_id, args[1..].to_vec(), stg).await?;
        }
        "get" => {
            user_settings_get(bot, user_id, chat_id, stg).await?;
        }
        _ => {
            log::error!("unknown command");
            bot.send_message(chat_id, ERR_WRONG_COMMAND).await?;
        }
    };

    Ok(())
}

async fn user_settings_set(
    bot: Bot,
    user_id: i64,
    chat_id: ChatId,
    args: Vec<&str>,
    stg: Arc<Box<dyn Storage>>,
) -> HandlerResult {
    if args.len() != 1 {
        log::error!("wrong args count");
        bot.send_message(chat_id, ERR_WRONG_COMMAND).await?;
        return Ok(());
    }

    let cal_limit = match args.first().unwrap().parse::<f64>() {
        Ok(v) => v,
        Err(err) => {
            log::error!("parse cal_limit error: {err}");
            bot.send_message(chat_id, ERR_WRONG_COMMAND).await?;
            return Ok(());
        }
    };

    if let Err(err) = stg.set_user_settings(user_id, &UserSettings { cal_limit }) {
        log::error!("set sport error: {err}");
        bot.send_message(chat_id, ERR_INTERNAL).await?;
    } else {
        bot.send_message(chat_id, OK).await?;
    }

    Ok(())
}

async fn user_settings_get(
    bot: Bot,
    user_id: i64,
    chat_id: ChatId,
    stg: Arc<Box<dyn Storage>>,
) -> HandlerResult {
    // Call storage
    match stg.get_user_settings(user_id) {
        Err(err) => {
            log::error!("get user settings error: {err}");
            if stg.is_storage_error(StorageError::NotFound, &err) {
                bot.send_message(chat_id, ERR_USER_SETTINGS_NOT_FOUND)
                    .await?;
            } else {
                bot.send_message(chat_id, ERR_INTERNAL).await?;
            }
        }
        Ok(us) => {
            bot.send_message(chat_id, format!("<b>Лимит калорий:</b> {}", us.cal_limit))
                .parse_mode(ParseMode::Html)
                .await?;
        }
    };

    Ok(())
}
