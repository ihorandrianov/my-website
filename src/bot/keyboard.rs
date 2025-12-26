use teloxide::types::{
    InlineKeyboardButton, InlineKeyboardMarkup, KeyboardButton, KeyboardMarkup,
};

use crate::db::NotificationSettings;

pub fn main_keyboard() -> KeyboardMarkup {
    KeyboardMarkup::new(vec![
        vec![
            KeyboardButton::new("📊 Status"),
            KeyboardButton::new("🌤 Weather"),
        ],
        vec![
            KeyboardButton::new("🌱 Garden"),
            KeyboardButton::new("📈 Stats"),
        ],
        vec![
            KeyboardButton::new("⚡ Power"),
            KeyboardButton::new("⚙️ Settings"),
        ],
    ])
    .resize_keyboard()
    .persistent()
}

pub fn settings_keyboard(settings: &NotificationSettings) -> InlineKeyboardMarkup {
    let soil_icon = if settings.soil_moisture_alerts { "✅" } else { "❌" };
    let temp_icon = if settings.temperature_alerts { "✅" } else { "❌" };
    let power_icon = if settings.power_alerts { "✅" } else { "❌" };
    let quiet_icon = if settings.quiet_hours_enabled { "✅" } else { "❌" };

    InlineKeyboardMarkup::new(vec![
        vec![InlineKeyboardButton::callback(
            format!("{} Soil moisture", soil_icon),
            "toggle_soil",
        )],
        vec![InlineKeyboardButton::callback(
            format!("{} Temperature", temp_icon),
            "toggle_temp",
        )],
        vec![InlineKeyboardButton::callback(
            format!("{} Power outage", power_icon),
            "toggle_power",
        )],
        vec![InlineKeyboardButton::callback(
            format!("{} Quiet hours (23-07)", quiet_icon),
            "toggle_quiet",
        )],
        vec![InlineKeyboardButton::callback("« Back", "back")],
    ])
}
