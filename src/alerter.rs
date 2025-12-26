use std::sync::Arc;

use teloxide::{prelude::*, types::ChatId};
use time::OffsetDateTime;
use time_tz::{timezones::db::europe::KYIV, OffsetDateTimeExt};

use crate::config::ALERT_COOLDOWN_SECS;
use crate::db::{AlertKind, Db};

#[derive(Clone)]
pub struct Alerter {
    bot: Arc<Bot>,
    db: Db,
}

impl Alerter {
    pub fn new(bot: Arc<Bot>, db: Db) -> Self {
        Self { bot, db }
    }

    pub async fn check_and_alert(
        &self,
        kind: AlertKind,
        triggered: bool,
        message: &str,
    ) -> anyhow::Result<()> {
        let state = self.db.get_alert_state(kind).await?;
        let was_active = state.as_ref().map(|s| s.active).unwrap_or(false);

        let should_send = if triggered && !was_active {
            true
        } else if triggered && was_active {
            match state.and_then(|s| s.last_sent_at) {
                Some(last) => {
                    let now = OffsetDateTime::now_utc();
                    let last_utc = last.assume_utc();
                    let elapsed = (now - last_utc).whole_seconds();
                    elapsed >= ALERT_COOLDOWN_SECS
                }
                None => true,
            }
        } else {
            false
        };

        self.db
            .set_alert_state(kind, triggered, should_send)
            .await?;

        if should_send {
            self.broadcast_alert(kind, message).await?;
        }

        Ok(())
    }

    /// Broadcast alert respecting user preferences
    async fn broadcast_alert(&self, kind: AlertKind, message: &str) -> anyhow::Result<()> {
        let user_ids = self.db.get_users_for_alert(kind).await?;

        for user_id in user_ids {
            if self.is_quiet_hours(user_id).await {
                continue;
            }
            if let Err(e) = self.bot.send_message(ChatId(user_id), message).await {
                eprintln!("Failed to send alert to {}: {}", user_id, e);
            }
        }

        Ok(())
    }

    /// Broadcast power alert respecting user preferences
    pub async fn broadcast_power_alert(&self, message: &str) -> anyhow::Result<()> {
        let user_ids = self.db.get_users_for_power_alert().await?;

        for user_id in user_ids {
            if self.is_quiet_hours(user_id).await {
                continue;
            }
            if let Err(e) = self.bot.send_message(ChatId(user_id), message).await {
                eprintln!("Failed to send power alert to {}: {}", user_id, e);
            }
        }

        Ok(())
    }

    /// Check if it's quiet hours for a user
    async fn is_quiet_hours(&self, user_id: i64) -> bool {
        let settings = match self.db.get_notification_settings(user_id).await {
            Ok(s) => s,
            Err(_) => return false,
        };

        if !settings.quiet_hours_enabled {
            return false;
        }

        let now = OffsetDateTime::now_utc().to_timezone(KYIV);
        let hour = now.hour() as i16;

        let start = settings.quiet_hours_start;
        let end = settings.quiet_hours_end;

        if start > end {
            hour >= start || hour < end
        } else {
            hour >= start && hour < end
        }
    }

    #[allow(dead_code)]
    pub async fn broadcast(&self, message: &str) -> anyhow::Result<()> {
        let user_ids = self.db.get_authorized_user_ids().await?;

        for user_id in user_ids {
            if let Err(e) = self.bot.send_message(ChatId(user_id), message).await {
                eprintln!("Failed to send alert to {}: {}", user_id, e);
            }
        }

        Ok(())
    }

    #[allow(dead_code)]
    pub async fn send_to(&self, user_id: i64, message: &str) -> anyhow::Result<()> {
        self.bot.send_message(ChatId(user_id), message).await?;
        Ok(())
    }
}
