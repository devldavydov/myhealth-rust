use chrono_tz::Tz;
use html::{
    attrs::Attrs,
    b::B,
    div::Div,
    h::H,
    s::S,
    span::Span,
    table::{Table, Td, Tr},
    Element,
};
use model::{Journal, Meal, UserSettings};
use std::sync::Arc;
use storage::{Storage, StorageError};
use teloxide::{
    prelude::*,
    types::{InputFile, ParseMode},
};

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
            journal_report_day(bot, user_id, chat_id, args[1..].to_vec(), stg, tz).await?;
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
    let rep = match stg.get_journal_report(user_id, timestamp.clone(), timestamp.clone()) {
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

    let us: Option<UserSettings> = match stg.get_user_settings(user_id) {
        Ok(v) => Some(v),
        Err(err) => {
            if stg.is_storage_error(StorageError::UserSettingsNotFound, &err) {
                None
            } else {
                bot.send_message(chat_id, ERR_INTERNAL).await?;
                return Ok(());
            }
        }
    };

    // Generate html
    let mut doc = html::Builder::new("Журнал приема пищи");
    let mut tbl = Table::new(vec![
        "Наименование".into(),
        "Вес".into(),
        "ККал".into(),
        "Белки".into(),
        "Жиры".into(),
        "Углеводы".into(),
    ]);
    let ts_str = format_timestamp(&timestamp, "%d.%m.%Y", tz);

    let (mut total_cal, mut total_prot, mut total_fat, mut total_carb) = (0.0, 0.0, 0.0, 0.0);
    let (mut sub_total_cal, mut sub_total_prot, mut sub_total_fat, mut sub_total_carb) =
        (0.0, 0.0, 0.0, 0.0);
    let mut last_meal: Option<Meal> = None;

    for i in 0..rep.len() {
        let jr = &rep[i];

        // Add meal divider
        if last_meal.is_none() || (last_meal.is_some() && jr.meal != last_meal.unwrap()) {
            tbl.add_row(
                Tr::new()
                    .set_attrs(Attrs::from_items(
                        vec![("class", "table-active")].into_iter(),
                    ))
                    .add_td(
                        Td::new(B::new(String::from(jr.meal).as_str()).as_box()).set_attrs(
                            Attrs::from_items(
                                vec![("colspan", "6"), ("align", "center")].into_iter(),
                            ),
                        ),
                    ),
            );

            last_meal = Some(jr.meal)
        }

        // Add meal rows
        let mut food_lbl = jr.food_name.clone();
        if !jr.food_brand.is_empty() {
            food_lbl.push_str(&format!(" - {}", jr.food_brand));
        }
        food_lbl.push_str(&format!(" [{}]", jr.food_key));

        tbl.add_row(
            Tr::new()
                .add_td(Td::new(S::create(&food_lbl)))
                .add_td(Td::new(S::create(&format!("{:.1}", jr.food_weight))))
                .add_td(Td::new(S::create(&format!("{:.2}", jr.cal))))
                .add_td(Td::new(S::create(&format!("{:.2}", jr.prot))))
                .add_td(Td::new(S::create(&format!("{:.2}", jr.fat))))
                .add_td(Td::new(S::create(&format!("{:.2}", jr.carb)))),
        );

        total_cal += jr.cal;
        total_prot += jr.prot;
        total_fat += jr.fat;
        total_carb += jr.carb;

        sub_total_cal += jr.cal;
        sub_total_prot += jr.prot;
        sub_total_fat += jr.fat;
        sub_total_carb += jr.carb;

        // Add subtotal row
        if i == rep.len() - 1 || rep[i + 1].meal != jr.meal {
            tbl.add_row(
                Tr::new()
                    .add_td(
                        Td::new(B::new("Всего").as_box()).set_attrs(Attrs::from_items(
                            vec![("colspan", "2"), ("align", "right")].into_iter(),
                        )),
                    )
                    .add_td(Td::new(S::create(&format!("{:.2}", sub_total_cal))))
                    .add_td(Td::new(S::create(&format!("{:.2}", sub_total_prot))))
                    .add_td(Td::new(S::create(&format!("{:.2}", sub_total_fat))))
                    .add_td(Td::new(S::create(&format!("{:.2}", sub_total_carb)))),
            );

            (sub_total_cal, sub_total_prot, sub_total_fat, sub_total_carb) = (0.0, 0.0, 0.0, 0.0);
        }
    }

    // Footer
    let total_pfc = total_prot + total_fat + total_carb;
    tbl.add_footer_element(
        Tr::new()
            .add_td(
                Td::new(Span::create(vec![
                    B::new("Всего потреблено, ккал: ").as_box(),
                    S::create(&format!("{:.2}", total_cal)),
                ]))
                .set_attrs(Attrs::from_items(vec![("colspan", "6")].into_iter())),
            )
            .as_box(),
    );

    if us.is_some() {
        let us = us.unwrap();
        tbl.add_footer_element(
            Tr::new()
                .add_td(
                    Td::new(Span::create(vec![
                        B::new("Лимит, ккал: ").as_box(),
                        S::create(&format!("{:.2}", us.cal_limit)),
                    ]))
                    .set_attrs(Attrs::from_items(vec![("colspan", "6")].into_iter())),
                )
                .as_box(),
        );
        tbl.add_footer_element(
            Tr::new()
                .add_td(
                    Td::new(Span::create(vec![
                        B::new("Разница, ккал: ").as_box(),
                        call_diff_snippet(us.cal_limit - total_cal),
                    ]))
                    .set_attrs(Attrs::from_items(vec![("colspan", "6")].into_iter())),
                )
                .as_box(),
        );
    }

    tbl.add_footer_element(
        Tr::new()
            .add_td(
                Td::new(Span::create(vec![
                    B::new("Всего, Б: ").as_box(),
                    pfc_snippet(total_prot, total_pfc),
                ]))
                .set_attrs(Attrs::from_items(vec![("colspan", "6")].into_iter())),
            )
            .as_box(),
    );
    tbl.add_footer_element(
        Tr::new()
            .add_td(
                Td::new(Span::create(vec![
                    B::new("Всего, Ж: ").as_box(),
                    pfc_snippet(total_fat, total_pfc),
                ]))
                .set_attrs(Attrs::from_items(vec![("colspan", "6")].into_iter())),
            )
            .as_box(),
    );
    tbl.add_footer_element(
        Tr::new()
            .add_td(
                Td::new(Span::create(vec![
                    B::new("Всего, У: ").as_box(),
                    pfc_snippet(total_carb, total_pfc),
                ]))
                .set_attrs(Attrs::from_items(vec![("colspan", "6")].into_iter())),
            )
            .as_box(),
    );

    doc = doc.add_element(
        Div::new_container()
            .add_element(
                H::new(&format!("Журнал приема пищи за {}", ts_str), 5)
                    .set_attr(Attrs::from_items(vec![("align", "center")].into_iter()))
                    .as_box(),
            )
            .add_element(tbl.as_box())
            .as_box(),
    );

    bot.send_document(
        chat_id,
        InputFile::memory(doc.build()).file_name(format!("report_{}.html", ts_str)),
    )
    .await?;

    Ok(())
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

fn call_diff_snippet(diff: f64) -> Box<dyn Element> {
    if diff < 0.0 && diff.abs() > 0.01 {
        B::new(&format!("{:.2}", diff))
            .set_attr(Attrs::from_items(
                vec![("class", "text-danger")].into_iter(),
            ))
            .as_box()
    } else if diff >= 0.0 && diff.abs() > 0.01 {
        B::new(&format!("+{:.2}", diff))
            .set_attr(Attrs::from_items(
                vec![("class", "text-success")].into_iter(),
            ))
            .as_box()
    } else {
        S::create(&format!("{:.2}", diff))
    }
}

fn pfc_snippet(val: f64, total: f64) -> Box<dyn Element> {
    if total == 0.0 {
        S::create(&format!("{:.2}", val))
    } else {
        S::create(&format!("{:.2} ({:.2}%)", val, val / total * 100.0))
    }
}