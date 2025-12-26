use sqlx::postgres::PgListener;
use sqlx::PgPool;

use crate::alerter::Alerter;
use crate::db::{AlertKind, Db};
use crate::power_monitor::check_power_restored;
use crate::services::{should_alert_soil_low, should_alert_temp_high};

pub async fn spawn_sensor_listener(pool: PgPool, alerter: Alerter) -> anyhow::Result<()> {
    let mut listener = PgListener::connect_with(&pool).await?;
    listener.listen("sensor_data").await?;

    let db = Db::new(pool);

    tokio::spawn(async move {
        loop {
            match listener.recv().await {
                Ok(notification) => {
                    let payload = notification.payload();
                    if let Ok(id) = payload.parse::<i32>() {
                        if let Err(e) = process_sensor_data(&db, &alerter, id).await {
                            eprintln!("Failed to process sensor data: {}", e);
                        }
                    }
                }
                Err(e) => {
                    eprintln!("Listener error: {}", e);
                    tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
                }
            }
        }
    });

    Ok(())
}

async fn process_sensor_data(db: &Db, alerter: &Alerter, id: i32) -> anyhow::Result<()> {
    check_power_restored(db, alerter).await?;

    let Some(data) = db.get_sensor_data_by_id(id).await? else {
        return Ok(());
    };

    alerter
        .check_and_alert(
            AlertKind::SoilMoistureLow,
            should_alert_soil_low(&data),
            &format!("⚠️ Low soil moisture: {:.1}%", data.soil_moisture),
        )
        .await?;

    alerter
        .check_and_alert(
            AlertKind::TemperatureHigh,
            should_alert_temp_high(&data),
            &format!("🔥 High temperature: {:.1}°C", data.temperature),
        )
        .await?;

    Ok(())
}
