use chrono_tz::Tz;
use model::{Bundle, Journal, Meal};
use std::sync::Arc;
use storage::{Storage, StorageError};
use teloxide::{prelude::*, types::ParseMode};

use crate::{
    messages::{
        ERR_BUNDLE_NOT_FOUND, ERR_EMPTY, ERR_FOOD_NOT_FOUND, ERR_INTERNAL, ERR_WRONG_COMMAND, OK,
    },
    HandlerResult,
};

use super::{format_timestamp, parse_timestamp};

pub async fn process_journal_command(
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
            journal_set(bot, user_id, chat_id, args[1..].to_vec(), stg, tz).await?;
        }
        "sb" => {
            journal_set_bundle(bot, user_id, chat_id, args[1..].to_vec(), stg, tz).await?;
        }
        "del" => {
            journal_del(bot, user_id, chat_id, args[1..].to_vec(), stg, tz).await?;
        }
        "dm" => {
            journal_del_meal(bot, user_id, chat_id, args[1..].to_vec(), stg, tz).await?;
        }
        "rd" => {
            journal_report_day(bot, user_id, chat_id, args[1..].to_vec(), stg).await?;
        }
        "tm" => {
            journal_template_meal(bot, user_id, chat_id, args[1..].to_vec(), stg, tz).await?;
        }
        _ => {
            log::error!("unknown command");
            bot.send_message(chat_id, ERR_WRONG_COMMAND).await?;
        }
    };

    Ok(())
}

async fn journal_set(
    bot: Bot,
    user_id: i64,
    chat_id: ChatId,
    args: Vec<&str>,
    stg: Arc<Box<dyn Storage>>,
    tz: Tz,
) -> HandlerResult {
    if args.len() != 4 {
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

    let meal = match Meal::new_str(args.get(1).unwrap()) {
        Ok(v) => v,
        Err(err) => {
            log::error!("parse meal error: {err}");
            bot.send_message(chat_id, ERR_WRONG_COMMAND).await?;
            return Ok(());
        }
    };

    let food_key = args.get(2).unwrap().to_string();

    let food_weight = match args.get(3).unwrap().parse::<f64>() {
        Ok(v) => v,
        Err(err) => {
            log::error!("parse food weight error: {err}");
            bot.send_message(chat_id, ERR_WRONG_COMMAND).await?;
            return Ok(());
        }
    };

    // Call storage
    match stg.set_journal(
        user_id,
        &Journal {
            timestamp,
            meal,
            food_key,
            food_weight,
        },
    ) {
        Ok(_) => {
            bot.send_message(chat_id, OK).await?;
        }
        Err(err) => {
            log::error!("set journal error: {err}");
            if stg.is_storage_error(StorageError::FoodNotFound, &err) {
                bot.send_message(chat_id, ERR_FOOD_NOT_FOUND).await?;
            } else {
                bot.send_message(chat_id, ERR_INTERNAL).await?;
            }
        }
    }

    Ok(())
}

async fn journal_set_bundle(
    bot: Bot,
    user_id: i64,
    chat_id: ChatId,
    args: Vec<&str>,
    stg: Arc<Box<dyn Storage>>,
    tz: Tz,
) -> HandlerResult {
    if args.len() != 3 {
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

    let meal = match Meal::new_str(args.get(1).unwrap()) {
        Ok(v) => v,
        Err(err) => {
            log::error!("parse meal error: {err}");
            bot.send_message(chat_id, ERR_WRONG_COMMAND).await?;
            return Ok(());
        }
    };

    let bnld_key = args.get(2).unwrap();

    // Call storage
    match stg.set_journal_bundle(user_id, timestamp, meal, bnld_key) {
        Ok(_) => {
            bot.send_message(chat_id, OK).await?;
        }
        Err(err) => {
            log::error!("set journal error: {err}");
            if stg.is_storage_error(StorageError::FoodNotFound, &err) {
                bot.send_message(chat_id, ERR_FOOD_NOT_FOUND).await?;
            } else if stg.is_storage_error(StorageError::BundleNotFound, &err) {
                bot.send_message(chat_id, ERR_BUNDLE_NOT_FOUND).await?;
            } else {
                bot.send_message(chat_id, ERR_INTERNAL).await?;
            }
        }
    }

    Ok(())
}

async fn journal_del(
    bot: Bot,
    user_id: i64,
    chat_id: ChatId,
    args: Vec<&str>,
    stg: Arc<Box<dyn Storage>>,
    tz: Tz,
) -> HandlerResult {
    if args.len() != 3 {
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

    let meal = match Meal::new_str(args.get(1).unwrap()) {
        Ok(v) => v,
        Err(err) => {
            log::error!("parse meal error: {err}");
            bot.send_message(chat_id, ERR_WRONG_COMMAND).await?;
            return Ok(());
        }
    };

    let food_key = args.get(2).unwrap();

    // Call storage
    if let Err(err) = stg.delete_journal(user_id, timestamp, meal, food_key) {
        log::error!("del journal error: {err}");
        bot.send_message(chat_id, ERR_INTERNAL).await?;
        return Ok(());
    }

    bot.send_message(chat_id, OK).await?;

    Ok(())
}

async fn journal_del_meal(
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

    let meal = match Meal::new_str(args.get(1).unwrap()) {
        Ok(v) => v,
        Err(err) => {
            log::error!("parse meal error: {err}");
            bot.send_message(chat_id, ERR_WRONG_COMMAND).await?;
            return Ok(());
        }
    };

    // Call storage
    if let Err(err) = stg.delete_journal_meal(user_id, timestamp, meal) {
        log::error!("del journal meal error: {err}");
        bot.send_message(chat_id, ERR_INTERNAL).await?;
        return Ok(());
    }

    bot.send_message(chat_id, OK).await?;

    Ok(())
}

async fn journal_report_day(
    bot: Bot,
    user_id: i64,
    chat_id: ChatId,
    args: Vec<&str>,
    stg: Arc<Box<dyn Storage>>,
) -> HandlerResult {
    todo!()
}

async fn journal_template_meal(
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

    let meal = match Meal::new_str(args.get(1).unwrap()) {
        Ok(v) => v,
        Err(err) => {
            log::error!("parse meal error: {err}");
            bot.send_message(chat_id, ERR_WRONG_COMMAND).await?;
            return Ok(());
        }
    };

    // Call storage
    let rep = match stg.get_journal_report(user_id, timestamp.clone(), timestamp) {
        Ok(v) => v,
        Err(err) => {
            log::error!("get journal report error: {err}");
            if stg.is_storage_error(StorageError::EmptyList, &err) {
                bot.send_message(chat_id, ERR_EMPTY).await?;
            } else {
                bot.send_message(chat_id, ERR_INTERNAL).await?;
            }
            return Ok(());
        }
    };

    let mut meal_rep = Vec::with_capacity(rep.len());
    for jr in rep {
        if jr.meal == meal {
            meal_rep.push(jr);
        }
    }

    if meal_rep.is_empty() {
        bot.send_message(chat_id, ERR_EMPTY).await?;
        return Ok(());
    }

    // Send response
    bot.send_message(chat_id, "<b>Изменение еды</b>")
        .parse_mode(ParseMode::Html)
        .await?;

    for jr in &meal_rep {
        bot.send_message(
            chat_id,
            format!(
                "j,set,{},{},{},{:.1}",
                format_timestamp(&jr.timestamp, "%d.%m.%Y", tz),
                String::from(jr.meal),
                jr.food_key,
                jr.food_weight
            ),
        )
        .await?;
    }

    bot.send_message(chat_id, "<b>Удаление еды</b>")
        .parse_mode(ParseMode::Html)
        .await?;

    for jr in &meal_rep {
        bot.send_message(
            chat_id,
            format!(
                "j,del,{},{},{}",
                format_timestamp(&jr.timestamp, "%d.%m.%Y", tz),
                String::from(jr.meal),
                jr.food_key,
            ),
        )
        .await?;
    }

    Ok(())
}
