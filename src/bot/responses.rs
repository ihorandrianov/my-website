use crate::config::pressure;
use crate::db::{DailyStats, Db, SensorData};
use crate::services::{
    analyze_pressure, analyze_soil_moisture, analyze_water_level, format_duration_minutes,
    format_kyiv,
};

pub async fn build_status(db: &Db) -> String {
    let Some(data) = db.get_latest_sensor_data().await.ok().flatten() else {
        return "No sensor data available".to_string();
    };

    format_status(&data)
}

pub fn format_status(data: &SensorData) -> String {
    format!(
        "📊 Current Status\n\n\
         🌡 Temperature: {:.1}°C\n\
         💧 Humidity: {:.1}%\n\
         🌪 Pressure: {:.1} hPa\n\
         🌱 Soil moisture: {:.1}%\n\
         💦 Water level: {:.1}%",
        data.temperature, data.humidity, data.pressure, data.soil_moisture, data.water_level
    )
}

pub async fn build_weather(db: &Db) -> String {
    let Some(current) = db.get_latest_sensor_data().await.ok().flatten() else {
        return "No sensor data available".to_string();
    };

    let pressure_past = db
        .get_pressure_hours_ago(pressure::TREND_HOURS)
        .await
        .ok()
        .flatten();

    let trend_str = match pressure_past {
        Some(past) => {
            let analysis = analyze_pressure(current.pressure, past);
            format!(
                "📉 {}h trend: {} {} ({:+.1} hPa)\n\n\
                 {} {}",
                pressure::TREND_HOURS,
                analysis.trend.symbol(),
                analysis.trend.label(),
                analysis.delta,
                analysis.forecast.emoji,
                analysis.forecast.message
            )
        }
        None => "📉 Trend: -- no history yet".to_string(),
    };

    format!(
        "🌤 Weather\n\n\
         🌡 Temperature: {:.1}°C\n\
         💧 Humidity: {:.1}%\n\
         🌪 Pressure: {:.1} hPa\n\n\
         {}",
        current.temperature, current.humidity, current.pressure, trend_str
    )
}

pub async fn build_garden(db: &Db) -> String {
    let Some(data) = db.get_latest_sensor_data().await.ok().flatten() else {
        return "No sensor data available".to_string();
    };

    let soil = analyze_soil_moisture(data.soil_moisture);
    let water = analyze_water_level(data.water_level);

    format!(
        "🌱 Garden Status\n\n\
         🌱 Soil moisture: {:.1}%\n\
         {} {}\n\n\
         💦 Water level: {:.1}%\n\
         {} {}",
        data.soil_moisture,
        soil.status.emoji(),
        soil.message,
        data.water_level,
        water.status.emoji(),
        water.message
    )
}

pub async fn build_stats(db: &Db) -> String {
    let Some(stats) = db.get_daily_stats().await.ok().flatten() else {
        return "No data for today".to_string();
    };

    format_stats(&stats)
}

pub fn format_stats(stats: &DailyStats) -> String {
    format!(
        "📈 Today's Stats\n\n\
         🌡 Temperature:\n\
           Min: {:.1}°C\n\
           Max: {:.1}°C\n\
           Avg: {:.1}°C\n\n\
         💧 Humidity:\n\
           Min: {:.1}%\n\
           Max: {:.1}%",
        stats.min_temp, stats.max_temp, stats.avg_temp, stats.min_humidity, stats.max_humidity
    )
}

pub async fn build_power_history(db: &Db) -> String {
    let active = db.get_active_outage().await.ok().flatten();
    let recent = db.get_recent_outages(5).await.unwrap_or_default();

    let mut result = String::from("⚡ Power History\n\n");

    if let Some(outage) = active {
        let started = format_kyiv(outage.started_at);
        result.push_str(&format!("🔴 Current outage since {}\n\n", started));
    } else {
        result.push_str("🟢 Power is OK\n\n");
    }

    if recent.is_empty() {
        result.push_str("No recent outages");
    } else {
        result.push_str("Recent outages:\n");
        for outage in recent.iter().filter(|o| o.ended_at.is_some()) {
            let started = format_kyiv(outage.started_at);
            let duration = outage
                .duration_minutes
                .map(format_duration_minutes)
                .unwrap_or_else(|| "?".to_string());
            result.push_str(&format!("• {} ({})\n", started, duration));
        }
    }

    result
}
