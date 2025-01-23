use html::{
    attrs::Attrs,
    div::Div,
    h::H,
    s::S,
    table::{Table, Td, Tr},
};
use std::sync::Arc;
use storage::{Storage, StorageError};
use teloxide::{prelude::*, types::InputFile};

use crate::{
    messages::{ERR_BUNDLE_IS_USED, ERR_EMPTY, ERR_INTERNAL, ERR_WRONG_COMMAND, OK},
    HandlerResult,
};

pub async fn process_bundle_command(
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
            bundle_set(bot, user_id, chat_id, args[1..].to_vec(), stg).await?;
        }
        "st" => {
            bundle_set_template(bot, user_id, chat_id, args[1..].to_vec(), stg).await?;
        }
        "list" => {
            bundle_list(bot, user_id, chat_id, stg).await?;
        }
        "del" => {
            bundle_del(bot, user_id, chat_id, args[1..].to_vec(), stg).await?;
        }
        _ => {
            log::error!("unknown command");
            bot.send_message(chat_id, ERR_WRONG_COMMAND).await?;
        }
    };

    Ok(())
}

async fn bundle_set(
    bot: Bot,
    user_id: i64,
    chat_id: ChatId,
    args: Vec<&str>,
    stg: Arc<Box<dyn Storage>>,
) -> HandlerResult {
    Ok(())
}

async fn bundle_set_template(
    bot: Bot,
    user_id: i64,
    chat_id: ChatId,
    args: Vec<&str>,
    stg: Arc<Box<dyn Storage>>,
) -> HandlerResult {
    Ok(())
}

async fn bundle_list(
    bot: Bot,
    user_id: i64,
    chat_id: ChatId,
    stg: Arc<Box<dyn Storage>>,
) -> HandlerResult {
    // Call storage
    let b_lst = match stg.get_bundle_list(user_id) {
        Err(err) => {
            log::error!("bundle list error: {err}");
            if stg.is_storage_error(StorageError::EmptyList, &err) {
                bot.send_message(chat_id, ERR_EMPTY).await?;
            } else {
                bot.send_message(chat_id, ERR_INTERNAL).await?;
            }
            return Ok(());
        }
        Ok(lst) => lst,
    };

    let mut doc = html::Builder::new("Список бандлов");
    let mut tbl = Table::new(vec![
        "Ключ бандла".into(),
        "Еда/Ключ дочернего бандла".into(),
        "Вес еды, г.".into(),
    ]);

    for b in &b_lst {
        for (i, (k, v)) in b.data.iter().enumerate() {
            let mut tr = Tr::new();
            if i == 0 {
                tr = tr.add_td(Td::new(S::create(&b.key)).set_attrs(Attrs::from_items(
                    vec![("rowspan", b.data.len().to_string().as_str())].into_iter(),
                )));
            }
            tr = tr.add_td(Td::new(S::create(k)));
            if *v > 0.0 {
                tr = tr.add_td(Td::new(S::create(&format!("{:.1}", v))));
            } else {
                tr = tr.add_td(Td::new(S::create("")));
            }
            tbl.add_row(tr);
        }
    }

    doc = doc.add_element(Box::new(
        Div::new_container()
            .add_element(Box::new(
                H::new("Список бандлов", 5)
                    .set_attr(Attrs::from_items(vec![("align", "center")].into_iter())),
            ))
            .add_element(Box::new(tbl)),
    ));

    bot.send_document(
        chat_id,
        InputFile::memory(doc.build()).file_name("bundles.html"),
    )
    .await?;

    Ok(())
}

async fn bundle_del(
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

    // Call storage
    if let Err(err) = stg.delete_bundle(user_id, args.first().unwrap()) {
        log::error!("del bundle error: {err}");
        if stg.is_storage_error(StorageError::BundleIsUsed, &err) {
            bot.send_message(chat_id, ERR_BUNDLE_IS_USED).await?;
        } else {
            bot.send_message(chat_id, ERR_INTERNAL).await?;
        }
        return Ok(());
    };

    bot.send_message(chat_id, OK).await?;

    Ok(())
}
