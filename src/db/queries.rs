use super::models::{
    AlertKind, AlertState, DailyStats, LastSensorTime, NotificationSettings, PowerOutage,
    SensorData,
};
use super::Db;

impl Db {
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

    pub async fn get_sensor_data_by_id(&self, id: i32) -> sqlx::Result<Option<SensorData>> {
        sqlx::query_as!(
            SensorData,
            r#"
            SELECT temperature as "temperature: f32",
                   humidity as "humidity: f32",
                   pressure as "pressure: f32",
                   soil_moisture as "soil_moisture: f32",
                   water_level as "water_level: f32"
            FROM sensor_data WHERE id = $1
            "#,
            id
        )
        .fetch_optional(&self.pool)
        .await
    }

    pub async fn get_latest_sensor_data(&self) -> sqlx::Result<Option<SensorData>> {
        sqlx::query_as!(
            SensorData,
            r#"
            SELECT temperature as "temperature: f32",
                   humidity as "humidity: f32",
                   pressure as "pressure: f32",
                   soil_moisture as "soil_moisture: f32",
                   water_level as "water_level: f32"
            FROM sensor_data
            ORDER BY created_at DESC
            LIMIT 1
            "#
        )
        .fetch_optional(&self.pool)
        .await
    }

    pub async fn get_pressure_hours_ago(&self, hours: i32) -> sqlx::Result<Option<f32>> {
        sqlx::query_scalar!(
            r#"
            SELECT pressure as "pressure: f32"
            FROM sensor_data
            WHERE created_at <= NOW() - make_interval(hours => $1)
            ORDER BY created_at DESC
            LIMIT 1
            "#,
            hours
        )
        .fetch_optional(&self.pool)
        .await
    }

    pub async fn get_daily_stats(&self) -> sqlx::Result<Option<DailyStats>> {
        sqlx::query_as!(
            DailyStats,
            r#"
            SELECT
                MIN(temperature)::real as "min_temp!: f32",
                MAX(temperature)::real as "max_temp!: f32",
                AVG(temperature)::real as "avg_temp!: f32",
                MIN(humidity)::real as "min_humidity!: f32",
                MAX(humidity)::real as "max_humidity!: f32"
            FROM sensor_data
            WHERE created_at >= CURRENT_DATE
            "#
        )
        .fetch_optional(&self.pool)
        .await
    }

    pub async fn is_user_authorized(&self, telegram_user_id: i64) -> sqlx::Result<bool> {
        let result = sqlx::query_scalar!(
            r#"SELECT EXISTS(SELECT 1 FROM authorized_users WHERE telegram_user_id = $1) as "exists!""#,
            telegram_user_id
        )
        .fetch_one(&self.pool)
        .await?;
        Ok(result)
    }

    pub async fn authorize_user(
        &self,
        telegram_user_id: i64,
        username: Option<&str>,
    ) -> sqlx::Result<()> {
        sqlx::query!(
            r#"
            INSERT INTO authorized_users (telegram_user_id, username)
            VALUES ($1, $2)
            ON CONFLICT (telegram_user_id) DO NOTHING
            "#,
            telegram_user_id,
            username
        )
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    #[allow(dead_code)]
    pub async fn get_authorized_user_ids(&self) -> sqlx::Result<Vec<i64>> {
        let rows = sqlx::query_scalar!(r#"SELECT telegram_user_id FROM authorized_users"#)
            .fetch_all(&self.pool)
            .await?;
        Ok(rows)
    }

    pub async fn get_alert_state(&self, kind: AlertKind) -> sqlx::Result<Option<AlertState>> {
        sqlx::query_as!(
            AlertState,
            r#"SELECT active, last_sent_at FROM alert_states WHERE alert_kind = $1"#,
            kind as AlertKind
        )
        .fetch_optional(&self.pool)
        .await
    }

    pub async fn set_alert_state(
        &self,
        kind: AlertKind,
        active: bool,
        update_last_sent: bool,
    ) -> sqlx::Result<()> {
        if update_last_sent {
            sqlx::query!(
                r#"
                INSERT INTO alert_states (alert_kind, active, last_sent_at)
                VALUES ($1, $2, NOW())
                ON CONFLICT (alert_kind) DO UPDATE SET active = $2, last_sent_at = NOW()
                "#,
                kind as AlertKind,
                active
            )
            .execute(&self.pool)
            .await?;
        } else {
            sqlx::query!(
                r#"
                INSERT INTO alert_states (alert_kind, active)
                VALUES ($1, $2)
                ON CONFLICT (alert_kind) DO UPDATE SET active = $2
                "#,
                kind as AlertKind,
                active
            )
            .execute(&self.pool)
            .await?;
        }
        Ok(())
    }

    pub async fn get_last_sensor_time(&self) -> sqlx::Result<Option<LastSensorTime>> {
        sqlx::query_as!(
            LastSensorTime,
            r#"SELECT created_at FROM sensor_data ORDER BY created_at DESC LIMIT 1"#
        )
        .fetch_optional(&self.pool)
        .await
    }

    pub async fn get_active_outage(&self) -> sqlx::Result<Option<PowerOutage>> {
        sqlx::query_as!(
            PowerOutage,
            r#"
            SELECT id, started_at, ended_at, duration_minutes
            FROM power_outages
            WHERE ended_at IS NULL
            ORDER BY started_at DESC
            LIMIT 1
            "#
        )
        .fetch_optional(&self.pool)
        .await
    }

    pub async fn start_outage(&self) -> sqlx::Result<()> {
        sqlx::query!(
            r#"INSERT INTO power_outages (started_at) VALUES (NOW())"#
        )
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn end_outage(&self) -> sqlx::Result<Option<i32>> {
        let result = sqlx::query_scalar!(
            r#"
            UPDATE power_outages
            SET ended_at = NOW(),
                duration_minutes = EXTRACT(EPOCH FROM (NOW() - started_at))::integer / 60
            WHERE ended_at IS NULL
            RETURNING duration_minutes
            "#
        )
        .fetch_optional(&self.pool)
        .await?;
        Ok(result.flatten())
    }

    pub async fn get_recent_outages(&self, limit: i64) -> sqlx::Result<Vec<PowerOutage>> {
        sqlx::query_as!(
            PowerOutage,
            r#"
            SELECT id, started_at, ended_at, duration_minutes
            FROM power_outages
            ORDER BY started_at DESC
            LIMIT $1
            "#,
            limit
        )
        .fetch_all(&self.pool)
        .await
    }

    pub async fn get_notification_settings(
        &self,
        user_id: i64,
    ) -> sqlx::Result<NotificationSettings> {
        let settings = sqlx::query_as!(
            NotificationSettings,
            r#"
            SELECT telegram_user_id, soil_moisture_alerts, temperature_alerts,
                   power_alerts, quiet_hours_enabled, quiet_hours_start, quiet_hours_end
            FROM notification_settings
            WHERE telegram_user_id = $1
            "#,
            user_id
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(settings.unwrap_or_else(|| NotificationSettings {
            telegram_user_id: user_id,
            ..Default::default()
        }))
    }

    pub async fn ensure_notification_settings(&self, user_id: i64) -> sqlx::Result<()> {
        sqlx::query!(
            r#"
            INSERT INTO notification_settings (telegram_user_id)
            VALUES ($1)
            ON CONFLICT (telegram_user_id) DO NOTHING
            "#,
            user_id
        )
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn toggle_soil_alerts(&self, user_id: i64) -> sqlx::Result<bool> {
        let result = sqlx::query_scalar!(
            r#"
            UPDATE notification_settings
            SET soil_moisture_alerts = NOT soil_moisture_alerts
            WHERE telegram_user_id = $1
            RETURNING soil_moisture_alerts
            "#,
            user_id
        )
        .fetch_one(&self.pool)
        .await?;
        Ok(result)
    }

    pub async fn toggle_temperature_alerts(&self, user_id: i64) -> sqlx::Result<bool> {
        let result = sqlx::query_scalar!(
            r#"
            UPDATE notification_settings
            SET temperature_alerts = NOT temperature_alerts
            WHERE telegram_user_id = $1
            RETURNING temperature_alerts
            "#,
            user_id
        )
        .fetch_one(&self.pool)
        .await?;
        Ok(result)
    }

    pub async fn toggle_power_alerts(&self, user_id: i64) -> sqlx::Result<bool> {
        let result = sqlx::query_scalar!(
            r#"
            UPDATE notification_settings
            SET power_alerts = NOT power_alerts
            WHERE telegram_user_id = $1
            RETURNING power_alerts
            "#,
            user_id
        )
        .fetch_one(&self.pool)
        .await?;
        Ok(result)
    }

    pub async fn toggle_quiet_hours(&self, user_id: i64) -> sqlx::Result<bool> {
        let result = sqlx::query_scalar!(
            r#"
            UPDATE notification_settings
            SET quiet_hours_enabled = NOT quiet_hours_enabled
            WHERE telegram_user_id = $1
            RETURNING quiet_hours_enabled
            "#,
            user_id
        )
        .fetch_one(&self.pool)
        .await?;
        Ok(result)
    }

    pub async fn get_users_for_alert(&self, kind: AlertKind) -> sqlx::Result<Vec<i64>> {
        let column = match kind {
            AlertKind::SoilMoistureLow | AlertKind::SoilMoistureHigh => "soil_moisture_alerts",
            AlertKind::TemperatureHigh | AlertKind::TemperatureLow => "temperature_alerts",
            AlertKind::WaterLevelLow => "soil_moisture_alerts",
        };

        let query = format!(
            r#"
            SELECT au.telegram_user_id
            FROM authorized_users au
            LEFT JOIN notification_settings ns ON au.telegram_user_id = ns.telegram_user_id
            WHERE COALESCE(ns.{}, true) = true
            "#,
            column
        );

        let rows = sqlx::query_scalar::<_, i64>(&query)
            .fetch_all(&self.pool)
            .await?;
        Ok(rows)
    }

    pub async fn get_users_for_power_alert(&self) -> sqlx::Result<Vec<i64>> {
        let rows = sqlx::query_scalar!(
            r#"
            SELECT au.telegram_user_id
            FROM authorized_users au
            LEFT JOIN notification_settings ns ON au.telegram_user_id = ns.telegram_user_id
            WHERE COALESCE(ns.power_alerts, true) = true
            "#
        )
        .fetch_all(&self.pool)
        .await?;
        Ok(rows.into_iter().flatten().collect())
    }
}
