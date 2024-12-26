use chart::{get_chart_snippet, ChartData, ChartDataset, CHART_COLOR_BLUE};
use chrono_tz::Tz;
use html::accordion::{Accordion, AccordionItem};
use html::canvas::Canvas;
use html::div::Div;
use html::s::S;
use html::script::Script;
use html::table::Table;
use html::table::{Td, Tr};
use html::{JS_BOOTSTRAP_URL, JS_CHART_URL};
use model::Weight;
use std::sync::Arc;
use storage::{Storage, StorageError};
use teloxide::{prelude::*, types::InputFile};

use crate::{
    messages::{ERR_EMPTY, ERR_INTERNAL, ERR_WRONG_COMMAND, OK},
    HandlerResult,
};

use super::{format_timestamp, parse_timestamp};

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
    if args.len() != 2 {
        log::error!("wrong args count");
        bot.send_message(chat_id, ERR_WRONG_COMMAND).await?;
        return Ok(());
    }

    // Parse args
    let ts_from = match parse_timestamp(args.first().unwrap(), tz) {
        Ok(v) => v,
        Err(err) => {
            log::error!("parse timestamp from error: {err}");
            bot.send_message(chat_id, ERR_WRONG_COMMAND).await?;
            return Ok(());
        }
    };

    let ts_to = match parse_timestamp(args.get(1).unwrap(), tz) {
        Ok(v) => v,
        Err(err) => {
            log::error!("parse timestamp to error: {err}");
            bot.send_message(chat_id, ERR_WRONG_COMMAND).await?;
            return Ok(());
        }
    };

    // Call storage
    let w_lst = match stg.get_weight_list(user_id, ts_from.clone(), ts_to.clone()) {
        Err(err) => {
            log::error!("delete weight error: {err}");
            if stg.is_storage_error(StorageError::EmptyList, &err) {
                bot.send_message(chat_id, ERR_EMPTY).await?;
            } else {
                bot.send_message(chat_id, ERR_INTERNAL).await?;
            }
            return Ok(());
        }
        Ok(lst) => lst,
    };

    // Generate HTML
    let ts_from = format_timestamp(&ts_from, "%d.%m.%Y", tz);
    let ts_to = format_timestamp(&ts_to, "%d.%m.%Y", tz);

    let mut doc = html::Builder::new("Таблица веса");
    let mut accrd = Accordion::new("accordionWeight");

    // Table
    let mut tbl = Table::new(vec!["Дата".into(), "Вес".into()]);

    let mut x_labels = Vec::with_capacity(w_lst.len());
    let mut data = Vec::with_capacity(w_lst.len());

    for w in &w_lst {
        tbl.add_row(
            Tr::new()
                .add_td(Td::new(S::create( &format_timestamp(&w.timestamp, "%d.%m.%Y", tz))))
                .add_td(Td::new(S::create(&format!("{:.1}", w.value)))),
        );
        x_labels.push(format_timestamp(&w.timestamp, "%d.%m.%Y", tz));
        data.push(w.value);
    }

    accrd.add_item(AccordionItem::new(
        "tbl",
        &format!("Таблица веса за {} - {}", &ts_from, &ts_to),
        Box::new(tbl),
    ));

    // Chart
    accrd.add_item(AccordionItem::new(
        "graph",
        &format!("График веса за {} - {}", &ts_from, &ts_to),
        Canvas::create("chart"),
    ));

    let chart_snip = match get_chart_snippet(ChartData {
        elem_id: "chart".into(),
        x_labels: x_labels,
        ctype: "line".into(),
        datasets: vec![ChartDataset {
            data: data,
            label: "Вес".into(),
            color: CHART_COLOR_BLUE.into(),
        }],
    }) {
        Err(err) => {
            log::error!("chart snippet error: {err}");
            bot.send_message(chat_id, ERR_INTERNAL).await?;
            return Ok(());
        }
        Ok(snip) => snip,
    };

    // Doc
    doc = doc
        .add_element(Box::new(Div::new_container().add_element(Box::new(accrd))))
        .add_element(Script::create(JS_BOOTSTRAP_URL))
        .add_element(Script::create(JS_CHART_URL))
        .add_element(S::create(&chart_snip));

    bot.send_document(
        chat_id,
        InputFile::memory(doc.build()).file_name(format!("weight_{}_{}.html", &ts_from, &ts_to)),
    )
    .await?;

    Ok(())
}
