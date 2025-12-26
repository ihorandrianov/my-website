mod models;
mod queries;

pub use models::{AlertKind, DailyStats, NotificationSettings, SensorData};

#[derive(Clone, Debug)]
pub struct Db {
    pub pool: sqlx::Pool<sqlx::Postgres>,
}

impl Db {
    pub fn new(pool: sqlx::Pool<sqlx::Postgres>) -> Self {
        Self { pool }
    }
}
