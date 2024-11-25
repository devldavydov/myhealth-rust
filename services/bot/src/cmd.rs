mod bundle;
mod food;
mod journal;
mod user_settings;
mod weight;

use super::messages;
use std::sync::Arc;
use storage::Storage;
use teloxide::prelude::*;

use crate::HandlerResult;

pub async fn process_command(bot: Bot, msg: Message, stg: Arc<Box<dyn Storage>>) -> HandlerResult {
    match msg.text() {
        None => {
            bot.send_message(msg.chat.id, messages::ERR_WRONG_COMMAND)
                .await?;
        }
        Some(input) => {
            let parts: Vec<&str> = input.split(",").collect();

            if parts.is_empty() {
                bot.send_message(msg.chat.id, messages::ERR_WRONG_COMMAND)
                    .await?;
            } else {
                match parts[0] {
                    "b" => {
                        bundle::process_bundle_command(bot, msg.chat.id, parts[1..].to_vec(), stg)
                            .await?;
                    }
                    "f" => {
                        food::process_food_command(bot, msg.chat.id, parts[1..].to_vec(), stg)
                            .await?;
                    }
                    "j" => {
                        journal::process_journal_command(
                            bot,
                            msg.chat.id,
                            parts[1..].to_vec(),
                            stg,
                        )
                        .await?;
                    }
                    "u" => {
                        user_settings::process_user_settings_command(
                            bot,
                            msg.chat.id,
                            parts[1..].to_vec(),
                            stg,
                        )
                        .await?;
                    }
                    "w" => {
                        weight::process_weight_command(bot, msg.chat.id, parts[1..].to_vec(), stg)
                            .await?;
                    }
                    _ => {
                        bot.send_message(msg.chat.id, messages::ERR_WRONG_COMMAND)
                            .await?;
                    }
                }
            }
        }
    };

    Ok(())
}
