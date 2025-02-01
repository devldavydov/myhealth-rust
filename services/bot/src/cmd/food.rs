use html::{
    attrs::Attrs,
    div::Div,
    h::H,
    s::S,
    table::{Table, Td, Tr},
};
use model::Food;
use std::sync::Arc;
use storage::{Storage, StorageError};
use teloxide::{
    prelude::*,
    types::{InputFile, ParseMode},
};

use crate::{
    messages::{
        ERR_EMPTY, ERR_FOOD_IS_USED, ERR_FOOD_NOT_FOUND, ERR_INTERNAL, ERR_WRONG_COMMAND, OK,
    },
    HandlerResult,
};

pub async fn process_food_command(
    bot: Bot,
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
            food_set(bot, chat_id, args[1..].to_vec(), stg).await?;
        }
        "st" => {
            food_set_template(bot, chat_id, args[1..].to_vec(), stg).await?;
        }
        "list" => {
            food_list(bot, chat_id, stg).await?;
        }
        "find" => {
            food_find(bot, chat_id, args[1..].to_vec(), stg).await?;
        }
        "del" => {
            food_del(bot, chat_id, args[1..].to_vec(), stg).await?;
        }
        _ => {
            log::error!("unknown command");
            bot.send_message(chat_id, ERR_WRONG_COMMAND).await?;
        }
    };

    Ok(())
}

async fn food_set(
    bot: Bot,
    chat_id: ChatId,
    args: Vec<&str>,
    stg: Arc<Box<dyn Storage>>,
) -> HandlerResult {
    if args.len() != 8 {
        log::error!("wrong args count");
        bot.send_message(chat_id, ERR_WRONG_COMMAND).await?;
        return Ok(());
    }

    let key = args.first().unwrap().to_string();
    let name = args.get(1).unwrap().to_string();
    let brand = args.get(2).unwrap().to_string();
    let comment = args.get(7).unwrap().to_string();

    let cal100 = match args.get(3).unwrap().parse::<f64>() {
        Ok(v) => v,
        Err(err) => {
            log::error!("parse cal100 error: {err}");
            bot.send_message(chat_id, ERR_WRONG_COMMAND).await?;
            return Ok(());
        }
    };
    let prot100 = match args.get(4).unwrap().parse::<f64>() {
        Ok(v) => v,
        Err(err) => {
            log::error!("parse prot100 error: {err}");
            bot.send_message(chat_id, ERR_WRONG_COMMAND).await?;
            return Ok(());
        }
    };
    let fat100 = match args.get(5).unwrap().parse::<f64>() {
        Ok(v) => v,
        Err(err) => {
            log::error!("parse fat100 error: {err}");
            bot.send_message(chat_id, ERR_WRONG_COMMAND).await?;
            return Ok(());
        }
    };
    let carb100 = match args.get(6).unwrap().parse::<f64>() {
        Ok(v) => v,
        Err(err) => {
            log::error!("parse carb100 error: {err}");
            bot.send_message(chat_id, ERR_WRONG_COMMAND).await?;
            return Ok(());
        }
    };

    // Call storage
    if let Err(err) = stg.set_food(&Food {
        key,
        name,
        brand,
        cal100,
        prot100,
        fat100,
        carb100,
        comment,
    }) {
        log::error!("set food error: {err}");
        if stg.is_storage_error(StorageError::FoodInvalid, &err) {
            bot.send_message(chat_id, ERR_WRONG_COMMAND).await?;
        } else {
            bot.send_message(chat_id, ERR_INTERNAL).await?;
        }
    } else {
        bot.send_message(chat_id, OK).await?;
    }

    Ok(())
}

async fn food_set_template(
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
    let food = match stg.get_food(args.first().unwrap()) {
        Err(err) => {
            log::error!("get food error: {err}");
            if stg.is_storage_error(StorageError::FoodNotFound, &err) {
                bot.send_message(chat_id, ERR_FOOD_NOT_FOUND).await?;
            } else {
                bot.send_message(chat_id, ERR_INTERNAL).await?;
            }
            return Ok(());
        }
        Ok(f) => f,
    };

    bot.send_message(
        chat_id,
        format!(
            "f,set,{},{},{},{:.2},{:.2},{:.2},{:.2},{}",
            food.key,
            food.name,
            food.brand,
            food.cal100,
            food.prot100,
            food.fat100,
            food.carb100,
            food.comment
        ),
    )
    .await?;

    Ok(())
}

async fn food_list(bot: Bot, chat_id: ChatId, stg: Arc<Box<dyn Storage>>) -> HandlerResult {
    // Call storage
    let f_lst = match stg.get_food_list() {
        Err(err) => {
            log::error!("food list error: {err}");
            if stg.is_storage_error(StorageError::EmptyList, &err) {
                bot.send_message(chat_id, ERR_EMPTY).await?;
            } else {
                bot.send_message(chat_id, ERR_INTERNAL).await?;
            }
            return Ok(());
        }
        Ok(lst) => lst,
    };

    let mut doc = html::Builder::new("Список продуктов");
    let mut tbl = Table::new(vec![
        "Ключ".into(),
        "Наименование".into(),
        "Бренд".into(),
        "ККал в 100г.".into(),
        "Белки в 100г.".into(),
        "Жиры в 100г.".into(),
        "Углеводы в 100г.".into(),
        "Комментарий".into(),
    ]);

    for f in &f_lst {
        tbl.add_row(
            Tr::new()
                .add_td(Td::new(S::create(&f.key)))
                .add_td(Td::new(S::create(&f.name)))
                .add_td(Td::new(S::create(&f.brand)))
                .add_td(Td::new(S::create(&format!("{:.2}", f.cal100))))
                .add_td(Td::new(S::create(&format!("{:.2}", f.prot100))))
                .add_td(Td::new(S::create(&format!("{:.2}", f.fat100))))
                .add_td(Td::new(S::create(&format!("{:.2}", f.carb100))))
                .add_td(Td::new(S::create(&f.comment))),
        );
    }

    doc = doc.add_element(
        Div::new_container()
            .add_element(
                H::new("Список продуктов и энергетической ценности", 5)
                    .set_attr(Attrs::from_items(vec![("align", "center")].into_iter()))
                    .as_box(),
            )
            .add_element(tbl.as_box())
            .as_box(),
    );

    bot.send_document(
        chat_id,
        InputFile::memory(doc.build()).file_name("food.html"),
    )
    .await?;

    Ok(())
}

async fn food_find(
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
    let food = match stg.find_food(args.first().unwrap()) {
        Err(err) => {
            log::error!("find food error: {err}");
            if stg.is_storage_error(StorageError::EmptyList, &err) {
                bot.send_message(chat_id, ERR_EMPTY).await?;
            } else {
                bot.send_message(chat_id, ERR_INTERNAL).await?;
            }
            return Ok(());
        }
        Ok(f) => f,
    };

    let mut res = String::new();
    for (i, f) in food.iter().enumerate() {
        res.push_str(&format!("<b>Ключ: </b>{}\n", f.key));
        res.push_str(&format!("<b>Наименование:</b> {}\n", f.name));
        res.push_str(&format!("<b>Бренд:</b> {}\n", f.brand));
        res.push_str(&format!("<b>ККал100:</b> {:.2}\n", f.cal100));
        res.push_str(&format!("<b>Бел100:</b> {:.2}\n", f.prot100));
        res.push_str(&format!("<b>Жир100:</b> {:.2}\n", f.fat100));
        res.push_str(&format!("<b>Угл100:</b> {:.2}\n", f.carb100));
        res.push_str(&format!("<b>Комментарий:</b> {}\n", f.comment));

        if i != res.len() - 1 {
            res.push('\n');
        }
    }

    bot.send_message(chat_id, res)
        .parse_mode(ParseMode::Html)
        .await?;

    Ok(())
}

async fn food_del(
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
    if let Err(err) = stg.delete_food(args.first().unwrap()) {
        log::error!("del food error: {err}");
        if stg.is_storage_error(StorageError::FoodIsUsed, &err) {
            bot.send_message(chat_id, ERR_FOOD_IS_USED).await?;
        } else {
            bot.send_message(chat_id, ERR_INTERNAL).await?;
        }
        return Ok(());
    };

    bot.send_message(chat_id, OK).await?;

    Ok(())
}
