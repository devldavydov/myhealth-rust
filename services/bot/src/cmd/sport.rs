use chrono_tz::Tz;
use html::{
    attrs::Attrs,
    div::Div,
    h::H,
    s::S,
    table::{Table, Td, Tr},
};
use model::{Sport, SportActivity};
use std::sync::Arc;
use storage::{Storage, StorageError};
use teloxide::{prelude::*, types::InputFile};

use crate::{
    messages::{
        ERR_EMPTY, ERR_INTERNAL, ERR_SPORT_IS_USED, ERR_SPORT_NOT_FOUND, ERR_WRONG_COMMAND, OK,
    },
    HandlerResult,
};

use super::parse_timestamp;

pub async fn process_sport_command(
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
        // Sport
        "set" => {
            sport_set(bot, chat_id, args[1..].to_vec(), stg).await?;
        }
        "st" => {
            sport_set_template(bot, chat_id, args[1..].to_vec(), stg).await?;
        }
        "list" => {
            sport_list(bot, chat_id, stg).await?;
        }
        "del" => {
            sport_del(bot, chat_id, args[1..].to_vec(), stg).await?;
        }
        // Sport activity
        "as" => {
            sport_activity_set(bot, user_id, chat_id, args[1..].to_vec(), stg, tz).await?;
        }
        "ad" => {
            sport_activity_del(bot, user_id, chat_id, args[1..].to_vec(), stg, tz).await?;
        }
        _ => {
            log::error!("unknown command");
            bot.send_message(chat_id, ERR_WRONG_COMMAND).await?;
        }
    };

    Ok(())
}

async fn sport_set(
    bot: Bot,
    chat_id: ChatId,
    args: Vec<&str>,
    stg: Arc<Box<dyn Storage>>,
) -> HandlerResult {
    if args.len() != 3 {
        log::error!("wrong args count");
        bot.send_message(chat_id, ERR_WRONG_COMMAND).await?;
        return Ok(());
    }

    let key = args.first().unwrap().to_string();
    let name = args.get(1).unwrap().to_string();
    let comment = args.get(2).unwrap().to_string();

    // Call storage
    if let Err(err) = stg.set_sport(&Sport { key, name, comment }) {
        log::error!("set sport error: {err}");
        if stg.is_storage_error(StorageError::InvalidSport, &err) {
            bot.send_message(chat_id, ERR_WRONG_COMMAND).await?;
        } else {
            bot.send_message(chat_id, ERR_INTERNAL).await?;
        }
    } else {
        bot.send_message(chat_id, OK).await?;
    }

    Ok(())
}

async fn sport_set_template(
    bot: Bot,
    chat_id: ChatId,
    args: Vec<&str>,
    stg: Arc<Box<dyn Storage>>,
) -> HandlerResult {
    if args.len() != 1 {
        log::error!("wrong args count");
        bot.send_message(chat_id, ERR_WRONG_COMMAND).await?;
        return Ok(());
    }

    // Call storage
    let sport = match stg.get_sport(args.first().unwrap()) {
        Err(err) => {
            log::error!("get sport error: {err}");
            if stg.is_storage_error(StorageError::NotFound, &err) {
                bot.send_message(chat_id, ERR_SPORT_NOT_FOUND).await?;
            } else {
                bot.send_message(chat_id, ERR_INTERNAL).await?;
            }
            return Ok(());
        }
        Ok(f) => f,
    };

    bot.send_message(
        chat_id,
        format!("s,set,{},{},{}", sport.key, sport.name, sport.comment),
    )
    .await?;

    Ok(())
}

async fn sport_list(bot: Bot, chat_id: ChatId, stg: Arc<Box<dyn Storage>>) -> HandlerResult {
    // Call storage
    let f_lst = match stg.get_sport_list() {
        Err(err) => {
            log::error!("sport list error: {err}");
            if stg.is_storage_error(StorageError::EmptyList, &err) {
                bot.send_message(chat_id, ERR_EMPTY).await?;
            } else {
                bot.send_message(chat_id, ERR_INTERNAL).await?;
            }
            return Ok(());
        }
        Ok(lst) => lst,
    };

    let mut doc = html::Builder::new("Список спорта");
    let mut tbl = Table::new(vec![
        "Ключ".into(),
        "Наименование".into(),
        "Комментарий".into(),
    ]);

    for f in &f_lst {
        tbl.add_row(
            Tr::new()
                .add_td(Td::new(S::create(&f.key)))
                .add_td(Td::new(S::create(&f.name)))
                .add_td(Td::new(S::create(&f.comment))),
        );
    }

    doc = doc.add_element(Box::new(
        Div::new_container()
            .add_element(Box::new(
                H::new("Список спорта", 5)
                    .set_attr(Attrs::from_items(vec![("align", "center")].into_iter())),
            ))
            .add_element(Box::new(tbl)),
    ));

    bot.send_document(
        chat_id,
        InputFile::memory(doc.build()).file_name("sport.html"),
    )
    .await?;

    Ok(())
}

async fn sport_del(
    bot: Bot,
    chat_id: ChatId,
    args: Vec<&str>,
    stg: Arc<Box<dyn Storage>>,
) -> HandlerResult {
    if args.len() != 1 {
        log::error!("wrong args count");
        bot.send_message(chat_id, ERR_WRONG_COMMAND).await?;
        return Ok(());
    }

    // Call storage
    if let Err(err) = stg.delete_sport(args.first().unwrap()) {
        log::error!("del sport error: {err}");
        if stg.is_storage_error(StorageError::SportIsUsedViolation, &err) {
            bot.send_message(chat_id, ERR_SPORT_IS_USED).await?;
        } else {
            bot.send_message(chat_id, ERR_INTERNAL).await?;
        }
        return Ok(());
    };

    bot.send_message(chat_id, OK).await?;

    Ok(())
}

async fn sport_activity_set(
    bot: Bot,
    user_id: i64,
    chat_id: ChatId,
    args: Vec<&str>,
    stg: Arc<Box<dyn Storage>>,
    tz: Tz,
) -> HandlerResult {
    if args.len() < 3 {
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

    let sport_key = args.get(1).unwrap().to_string();

    let mut sets = Vec::new();
    for i in 2..args.len() {
        let set = match args.get(i).unwrap().parse::<i64>() {
            Ok(v) => v,
            Err(err) => {
                log::error!("parse sport activity arg {i}: {err}");
                bot.send_message(chat_id, ERR_WRONG_COMMAND).await?;
                return Ok(());
            }
        };

        sets.push(set);
    }

    // Call storage
    if let Err(err) = stg.set_sport_activity(
        user_id,
        &SportActivity {
            sport_key,
            sets,
            timestamp,
        },
    ) {
        log::error!("set sport activity error: {err}");
        if stg.is_storage_error(StorageError::InvalidSport, &err) {
            bot.send_message(chat_id, ERR_SPORT_NOT_FOUND).await?;
        } else {
            bot.send_message(chat_id, ERR_INTERNAL).await?;
        }
        return Ok(());
    }

    bot.send_message(chat_id, OK).await?;

    Ok(())
}

async fn sport_activity_del(
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

    let sport_key = args.get(1).unwrap().to_string();

    // Call storage
    if let Err(err) = stg.delete_sport_activity(user_id, timestamp, &sport_key) {
        log::error!("set sport activity error: {err}");
        bot.send_message(chat_id, ERR_INTERNAL).await?;
        return Ok(());
    }

    bot.send_message(chat_id, OK).await?;

    Ok(())
}
