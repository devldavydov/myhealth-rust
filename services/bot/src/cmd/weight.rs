use chrono_tz::Tz;
use model::Weight;
use std::sync::Arc;
use storage::Storage;
use teloxide::prelude::*;

use crate::{
    messages::{ERR_INTERNAL, ERR_WRONG_COMMAND, OK},
    HandlerResult,
};

use super::parse_timestamp;

pub async fn process_weight_command(
    bot: Bot,
    user_id: i64,
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
        "set" => {
            weight_set(bot, user_id, chat_id, args[1..].to_vec(), stg, tz).await?;
        }
        "del" => {
            weight_del(bot, user_id, chat_id, args[1..].to_vec(), stg, tz).await?;
        }
        "list" => {
            weight_list(bot, user_id, chat_id, args[1..].to_vec(), stg, tz).await?;
        }
        _ => {
            log::error!("unknown command");
            bot.send_message(chat_id, ERR_WRONG_COMMAND).await?;
        }
    };

    Ok(())
}

async fn weight_set(
    bot: Bot,
    user_id: i64,
    chat_id: ChatId,
    args: Vec<&str>,
    stg: Arc<Box<dyn Storage>>,
    tz: Tz,
) -> HandlerResult {
    if args.len() != 2 {
        log::error!("wrong args count");
        bot.send_message(chat_id, ERR_WRONG_COMMAND).await?;
        return Ok(());
    }

    // Parse args
    let timestamp = match parse_timestamp(args.first().unwrap(), tz) {
        Ok(v) => v,
        Err(err) => {
            log::error!("parse timestamp error: {err}");
            bot.send_message(chat_id, ERR_WRONG_COMMAND).await?;
            return Ok(());
        }
    };

    let value = match args.get(1).unwrap().parse::<f64>() {
        Ok(v) => v,
        Err(err) => {
            log::error!("parse timestamp error: {err}");
            bot.send_message(chat_id, ERR_WRONG_COMMAND).await?;
            return Ok(());
        }
    };

    // Validate weight
    let w = Weight { timestamp, value };
    if !w.validate() {
        log::error!("invalid weight value: {:#?}", w);
        bot.send_message(chat_id, ERR_WRONG_COMMAND).await?;
        return Ok(());
    }

    // Call storage
    if let Err(err) = stg.set_weight(user_id, &w) {
        log::error!("set weight error: {err}");
        bot.send_message(chat_id, ERR_INTERNAL).await?;
    } else {
        bot.send_message(chat_id, OK).await?;
    }

    Ok(())
}

async fn weight_del(
    bot: Bot,
    user_id: i64,
    chat_id: ChatId,
    args: Vec<&str>,
    stg: Arc<Box<dyn Storage>>,
    tz: Tz,
) -> HandlerResult {
    if args.len() != 1 {
        log::error!("wrong args count");
        bot.send_message(chat_id, ERR_WRONG_COMMAND).await?;
        return Ok(());
    }

    // Parse args
    let timestamp = match parse_timestamp(args.first().unwrap(), tz) {
        Ok(v) => v,
        Err(err) => {
            log::error!("parse timestamp error: {err}");
            bot.send_message(chat_id, ERR_WRONG_COMMAND).await?;
            return Ok(());
        }
    };

    // Call storage
    if let Err(err) = stg.delete_weight(user_id, timestamp) {
        log::error!("delete weight error: {err}");
        bot.send_message(chat_id, ERR_INTERNAL).await?;
    } else {
        bot.send_message(chat_id, OK).await?;
    }

    Ok(())
}

async fn weight_list(
    bot: Bot,
    user_id: i64,
    chat_id: ChatId,
    args: Vec<&str>,
    stg: Arc<Box<dyn Storage>>,
    tz: Tz,
) -> HandlerResult {
    Ok(())
}
