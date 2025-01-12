use teloxide::{
    prelude::*,
    types::{ChatId, ParseMode},
    Bot,
};

use crate::{messages::ERR_WRONG_COMMAND, HandlerResult};

pub async fn process_cal_calc_command(bot: Bot, chat_id: ChatId, args: Vec<&str>) -> HandlerResult {
    if args.len() != 4 {
        log::error!("wrong args count");
        bot.send_message(chat_id, ERR_WRONG_COMMAND).await?;
        return Ok(());
    }

    let gender = args.first().unwrap();
    if !(*gender == "m" || *gender == "f") {
        log::error!("wrong gender");
        bot.send_message(chat_id, ERR_WRONG_COMMAND).await?;
        return Ok(());
    }

    let weight = match args.get(1).unwrap().parse::<f64>() {
        Ok(v) => v,
        Err(err) => {
            log::error!("parse weight error: {err}");
            bot.send_message(chat_id, ERR_WRONG_COMMAND).await?;
            return Ok(());
        }
    };
    let height = match args.get(2).unwrap().parse::<f64>() {
        Ok(v) => v,
        Err(err) => {
            log::error!("parse height error: {err}");
            bot.send_message(chat_id, ERR_WRONG_COMMAND).await?;
            return Ok(());
        }
    };
    let age = match args.get(3).unwrap().parse::<f64>() {
        Ok(v) => v,
        Err(err) => {
            log::error!("parse age error: {err}");
            bot.send_message(chat_id, ERR_WRONG_COMMAND).await?;
            return Ok(());
        }
    };

    if weight <= 0.0 || height <= 0.0 || age <= 0.0 {
        log::error!("error <= 0: weight={weight}, height={height}, age={age}");
        bot.send_message(chat_id, ERR_WRONG_COMMAND).await?;
        return Ok(());
    }

    let mut ubm = 10.0 * weight + 6.25 * height - 5.0 * age;
    if *gender == "m" {
        ubm += 5.0
    } else {
        ubm -= 161.0
    }

    let mut res = String::new();
    res.push_str("<b>Уровень Базального Метаболизма (УБМ)</b>\n");
    res.push_str(&format!("{} ккал\n\n", ubm as i64));

    res.push_str("<b>Усредненные значения по активностям</b>\n\n");
    for (name, k) in [
        ("Сидячая активность", 1.2),
        ("Легкая активность", 1.375),
        ("Средняя активность", 1.55),
        ("Полноценная активность", 1.725),
        ("Супер активность", 1.9),
    ] {
        res.push_str(&format!("<b>{}</b>\n", name));
        let norm = (ubm * k) as i64;
        res.push_str(&format!("ККал: {norm}\n\n"));
    }

    bot.send_message(chat_id, res)
        .parse_mode(ParseMode::Html)
        .await?;

    Ok(())
}
