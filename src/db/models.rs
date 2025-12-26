use serde::{Deserialize, Serialize};
use time::PrimitiveDateTime;

#[derive(Clone, Copy, Debug, PartialEq, Eq, sqlx::Type)]
#[sqlx(type_name = "TEXT", rename_all = "snake_case")]
pub enum AlertKind {
    SoilMoistureLow,
    SoilMoistureHigh,
    TemperatureHigh,
    TemperatureLow,
    WaterLevelLow,
}

pub struct AlertState {
    pub active: bool,
    pub last_sent_at: Option<PrimitiveDateTime>,
}

#[derive(Clone, Copy, Default, Serialize, Deserialize)]
pub struct SensorData {
    pub temperature: f32,
    pub humidity: f32,
    pub pressure: f32,
    pub soil_moisture: f32,
    pub water_level: f32,
}

pub struct DailyStats {
    pub min_temp: f32,
    pub max_temp: f32,
    pub avg_temp: f32,
    pub min_humidity: f32,
    pub max_humidity: f32,
}

#[allow(dead_code)]
pub struct PowerOutage {
    pub id: i32,
    pub started_at: PrimitiveDateTime,
    pub ended_at: Option<PrimitiveDateTime>,
    pub duration_minutes: Option<i32>,
}

pub struct LastSensorTime {
    pub created_at: PrimitiveDateTime,
}

#[derive(Clone, Debug)]
pub struct NotificationSettings {
    #[allow(dead_code)]
    pub telegram_user_id: i64,
    pub soil_moisture_alerts: bool,
    pub temperature_alerts: bool,
    pub power_alerts: bool,
    pub quiet_hours_enabled: bool,
    pub quiet_hours_start: i16,
    pub quiet_hours_end: i16,
}

impl Default for NotificationSettings {
    fn default() -> Self {
        Self {
            telegram_user_id: 0,
            soil_moisture_alerts: true,
            temperature_alerts: true,
            power_alerts: true,
            quiet_hours_enabled: false,
            quiet_hours_start: 23,
            quiet_hours_end: 7,
        }
    }
}
