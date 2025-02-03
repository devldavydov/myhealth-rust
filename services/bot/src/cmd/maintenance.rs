use std::{io::Read, sync::Arc};

use anyhow::Context;
use chrono_tz::Tz;
use flate2::{bufread::GzEncoder, Compression};
use serde_json::json;
use storage::Storage;
use teloxide::{
    prelude::*,
    types::{ChatId, InputFile},
    Bot,
};
use types::timestamp::Timestamp;

use crate::{
    messages::{ERR_INTERNAL, ERR_WRONG_COMMAND},
    HandlerResult,
};

use super::format_timestamp;

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
            backup(bot, chat_id, stg, tz).await?;
        }
        _ => {
            log::error!("unknown command");
            bot.send_message(chat_id, ERR_WRONG_COMMAND).await?;
        }
    };

    Ok(())
}

async fn backup(bot: Bot, chat_id: ChatId, stg: Arc<Box<dyn Storage>>, tz: Tz) -> HandlerResult {
    // Get storage data for backup
    let res = stg.backup();
    if let Err(err) = res {
        log::error!("backup error: {err}");
        bot.send_message(chat_id, ERR_INTERNAL).await?;
        return Ok(());
    }

    let data = serde_json::to_vec(&json!(res.unwrap())).context("failed to serde JSON backup")?;
    let mut gz = GzEncoder::new(&data[..], Compression::best());
    let mut out = Vec::new();
    gz.read_to_end(&mut out).context("failed to gzip backup")?;

    bot.send_document(
        chat_id,
        InputFile::memory(out).file_name(format!(
            "backup_{}.json.gz",
            format_timestamp(&Timestamp::now(), "%d.%m.%Y", tz)
        )),
    )
    .await?;

    Ok(())
}
