use std::time::Duration;

use time::OffsetDateTime;
use tokio::time::interval;

use crate::alerter::Alerter;
use crate::config::power;
use crate::db::Db;
use crate::services::{format_kyiv, now_kyiv};

pub fn spawn_power_monitor(db: Db, alerter: Alerter) {
    tokio::spawn(async move {
        let mut interval = interval(Duration::from_secs(power::CHECK_INTERVAL_SECS));

        loop {
            interval.tick().await;

            if let Err(e) = check_power_status(&db, &alerter).await {
                eprintln!("Power monitor error: {}", e);
            }
        }
    });
}

async fn check_power_status(db: &Db, alerter: &Alerter) -> anyhow::Result<()> {
    let last_sensor = db.get_last_sensor_time().await?;
    let active_outage = db.get_active_outage().await?;

    let Some(last) = last_sensor else {
        return Ok(());
    };

    let now = OffsetDateTime::now_utc();
    let last_utc = last.created_at.assume_utc();
    let elapsed_secs = (now - last_utc).whole_seconds();

    let is_power_down = elapsed_secs > power::OUTAGE_THRESHOLD_SECS;

    match (is_power_down, active_outage.is_some()) {
        (true, false) => {
            db.start_outage().await?;
            let time_str = format_kyiv(last.created_at);
            alerter
                .broadcast_power_alert(&format!("⚡ Power outage detected!\nLast data: {}", time_str))
                .await?;
        }
        (false, true) => {}
        _ => {}
    }

    Ok(())
}

/// Call this when new sensor data arrives to check if power was restored
pub async fn check_power_restored(db: &Db, alerter: &Alerter) -> anyhow::Result<()> {
    let active_outage = db.get_active_outage().await?;

    if active_outage.is_some() {
        // Power is back!
        if let Some(duration) = db.end_outage().await? {
            let now = now_kyiv();
            let time_str = now
                .format(&time::format_description::parse("[day].[month] [hour]:[minute]").unwrap())
                .unwrap_or_else(|_| "??".to_string());

            let duration_str = crate::services::format_duration_minutes(duration);

            alerter
                .broadcast_power_alert(&format!(
                    "✅ Power restored at {}\nOutage duration: {}",
                    time_str, duration_str
                ))
                .await?;
        }
    }

    Ok(())
}
