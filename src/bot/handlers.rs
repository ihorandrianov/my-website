use teloxide::{dispatching::dialogue::InMemStorage, prelude::*, utils::command::BotCommands};

use super::keyboard::{main_keyboard, settings_keyboard};
use super::responses;
use crate::db::Db;

#[derive(BotCommands, Clone)]
#[command(rename_rule = "lowercase", description = "Available commands:")]
pub enum Command {
    #[command(description = "Show main menu")]
    Start,
    #[command(description = "Show this help message")]
    Help,
    #[command(description = "Notification settings")]
    Settings,
}

#[derive(Clone, Default)]
pub enum State {
    #[default]
    Unauthorized,
    Authorized,
}

pub type BotDialogue = Dialogue<State, InMemStorage<State>>;

#[derive(Clone)]
pub struct BotState {
    pub db: Db,
    pub bot_secret: String,
}

pub async fn handle_unauthorized(
    bot: Bot,
    msg: Message,
    dialogue: BotDialogue,
    state: BotState,
) -> ResponseResult<()> {
    let Some(user) = msg.from.as_ref() else {
        return Ok(());
    };

    let user_id = user.id.0 as i64;
    let username = user.username.as_deref();

    if state.db.is_user_authorized(user_id).await.unwrap_or(false) {
        let _ = dialogue.update(State::Authorized).await;
        bot.send_message(msg.chat.id, "Welcome back!")
            .reply_markup(main_keyboard())
            .await?;
        return Ok(());
    }

    let Some(text) = msg.text() else {
        bot.send_message(msg.chat.id, "Please enter the secret word to continue.")
            .await?;
        return Ok(());
    };

    if text.trim() == state.bot_secret {
        if let Err(e) = state.db.authorize_user(user_id, username).await {
            eprintln!("Failed to authorize user: {}", e);
            bot.send_message(msg.chat.id, "Authorization failed. Please try again.")
                .await?;
        } else {
            let _ = state.db.ensure_notification_settings(user_id).await;
            let _ = dialogue.update(State::Authorized).await;
            bot.send_message(msg.chat.id, "You are now authorized!")
                .reply_markup(main_keyboard())
                .await?;
        }
    } else {
        bot.send_message(msg.chat.id, "Incorrect. Please enter the secret word.")
            .await?;
    }

    Ok(())
}

pub async fn handle_command(
    bot: Bot,
    msg: Message,
    cmd: Command,
    state: BotState,
) -> ResponseResult<()> {
    match cmd {
        Command::Start => {
            bot.send_message(msg.chat.id, "Main menu:")
                .reply_markup(main_keyboard())
                .await?;
        }
        Command::Help => {
            bot.send_message(msg.chat.id, Command::descriptions().to_string())
                .await?;
        }
        Command::Settings => {
            let user_id = msg.from.as_ref().map(|u| u.id.0 as i64).unwrap_or(0);
            let _ = state.db.ensure_notification_settings(user_id).await;
            let settings = state
                .db
                .get_notification_settings(user_id)
                .await
                .unwrap_or_default();

            bot.send_message(msg.chat.id, "⚙️ Notification Settings")
                .reply_markup(settings_keyboard(&settings))
                .await?;
        }
    }
    Ok(())
}

pub async fn handle_message(bot: Bot, msg: Message, state: BotState) -> ResponseResult<()> {
    let Some(text) = msg.text() else {
        return Ok(());
    };

    let user_id = msg.from.as_ref().map(|u| u.id.0 as i64).unwrap_or(0);

    let response = match text {
        "📊 Status" => responses::build_status(&state.db).await,
        "🌤 Weather" => responses::build_weather(&state.db).await,
        "🌱 Garden" => responses::build_garden(&state.db).await,
        "📈 Stats" => responses::build_stats(&state.db).await,
        "⚡ Power" => responses::build_power_history(&state.db).await,
        "⚙️ Settings" => {
            let _ = state.db.ensure_notification_settings(user_id).await;
            let settings = state.db.get_notification_settings(user_id).await.ok();
            let settings = settings.unwrap_or_default();

            bot.send_message(msg.chat.id, "⚙️ Notification Settings")
                .reply_markup(settings_keyboard(&settings))
                .await?;
            return Ok(());
        }
        _ => {
            bot.send_message(msg.chat.id, "Use the menu below:")
                .reply_markup(main_keyboard())
                .await?;
            return Ok(());
        }
    };

    bot.send_message(msg.chat.id, response).await?;
    Ok(())
}

pub async fn handle_callback(bot: Bot, q: CallbackQuery, state: BotState) -> ResponseResult<()> {
    let Some(data) = q.data.as_ref() else {
        return Ok(());
    };

    let user_id = q.from.id.0 as i64;
    if !state.db.is_user_authorized(user_id).await.unwrap_or(false) {
        bot.answer_callback_query(q.id.clone())
            .text("Not authorized")
            .await?;
        return Ok(());
    }

    let Some(ref msg) = q.message else {
        return Ok(());
    };

    if data.starts_with("toggle_") {
        handle_toggle(&bot, &q, &state, user_id, data, msg).await?;
        return Ok(());
    }

    if data == "back" {
        bot.answer_callback_query(q.id.clone()).await?;
        bot.delete_message(msg.chat().id, msg.id()).await?;
    }

    Ok(())
}

async fn handle_toggle(
    bot: &Bot,
    q: &CallbackQuery,
    state: &BotState,
    user_id: i64,
    data: &str,
    msg: &teloxide::types::MaybeInaccessibleMessage,
) -> ResponseResult<()> {
    let result = match data {
        "toggle_soil" => state.db.toggle_soil_alerts(user_id).await,
        "toggle_temp" => state.db.toggle_temperature_alerts(user_id).await,
        "toggle_power" => state.db.toggle_power_alerts(user_id).await,
        "toggle_quiet" => state.db.toggle_quiet_hours(user_id).await,
        _ => return Ok(()),
    };

    let notification = match result {
        Ok(enabled) => {
            let name = match data {
                "toggle_soil" => "Soil alerts",
                "toggle_temp" => "Temperature alerts",
                "toggle_power" => "Power alerts",
                "toggle_quiet" => "Quiet hours",
                _ => "Setting",
            };
            format!("{} {}", name, if enabled { "enabled" } else { "disabled" })
        }
        Err(_) => "Failed to update".to_string(),
    };

    bot.answer_callback_query(q.id.clone()).text(&notification).await?;

    if let Ok(settings) = state.db.get_notification_settings(user_id).await {
        bot.edit_message_text(msg.chat().id, msg.id(), "⚙️ Notification Settings")
            .reply_markup(settings_keyboard(&settings))
            .await?;
    }

    Ok(())
}
