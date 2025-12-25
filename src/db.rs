use serde::{Deserialize, Serialize};

#[derive(Clone, Debug)]
pub struct Db {
    pub pool: sqlx::Pool<sqlx::Postgres>,
}

#[derive(Clone, Copy, Default, Serialize, Deserialize)]
pub struct SensorData {
    pub temperature: f32,
    pub humidity: f32,
    pub pressure: f32,
    pub soil_moisture: f32,
    pub water_level: f32,
}

impl Db {
    pub fn new(pool: sqlx::Pool<sqlx::Postgres>) -> Self {
        Self { pool }
    }

    pub async fn write_sensor_data(&self, data: SensorData) -> sqlx::Result<()> {
        sqlx::query!(
            r#"
        INSERT INTO sensor_data
        (temperature, humidity, pressure, soil_moisture, water_level)
        VALUES ($1, $2, $3, $4, $5)
            "#,
            data.temperature as f64,
            data.humidity as f64,
            data.pressure as f64,
            data.soil_moisture as f64,
            data.water_level as f64
        )
        .execute(&self.pool)
        .await?;
        Ok(())
    }
}
