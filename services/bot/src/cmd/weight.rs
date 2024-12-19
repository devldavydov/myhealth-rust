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
    user_id: u64,
    chat_id: ChatId,
    args: Vec<&str>,
    stg: Arc<Box<dyn Storage>>,
) -> HandlerResult {
    if args.is_empty() {
        bot.send_message(chat_id, ERR_WRONG_COMMAND).await?;
        return Ok(());
    }

    match *args.first().unwrap() {
        "set" => {
            weight_set(bot, user_id, chat_id, args[1..].to_vec(), stg).await?;
        }
        "del" => {
            weight_del(bot, user_id, chat_id, args[1..].to_vec(), stg).await?;
        }
        "list" => {
            weight_list(bot, user_id, chat_id, args[1..].to_vec(), stg).await?;
        }
        _ => {
            bot.send_message(chat_id, ERR_WRONG_COMMAND).await?;
        }
    };

    Ok(())
}

async fn weight_set(
    bot: Bot,
    user_id: u64,
    chat_id: ChatId,
    args: Vec<&str>,
    stg: Arc<Box<dyn Storage>>,
) -> HandlerResult {
    if args.len() != 2 {
        bot.send_message(chat_id, ERR_WRONG_COMMAND).await?;
        return Ok(());
    }

    // Parse args
    let timestamp = match parse_timestamp(args.first().unwrap()) {
        Ok(v) => v,
        Err(_) => {
            bot.send_message(chat_id, ERR_WRONG_COMMAND).await?;
            return Ok(());
        }
    };

    let value = match args.get(1).unwrap().parse::<f64>() {
        Ok(v) => v,
        Err(_) => {
            bot.send_message(chat_id, ERR_WRONG_COMMAND).await?;
            return Ok(());
        }
    };

    // Call storage
    if let Err(_) = stg.set_weight(user_id, &Weight { timestamp, value }) {
        bot.send_message(chat_id, ERR_INTERNAL).await?;
    } else {
        bot.send_message(chat_id, OK).await?;
    }

    Ok(())
}

async fn weight_del(
    bot: Bot,
    user_id: u64,
    chat_id: ChatId,
    args: Vec<&str>,
    stg: Arc<Box<dyn Storage>>,
) -> HandlerResult {
    Ok(())
}

async fn weight_list(
    bot: Bot,
    user_id: u64,
    chat_id: ChatId,
    args: Vec<&str>,
    stg: Arc<Box<dyn Storage>>,
) -> HandlerResult {
    Ok(())
}
