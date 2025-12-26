CREATE TABLE notification_settings (
    telegram_user_id BIGINT PRIMARY KEY REFERENCES authorized_users(telegram_user_id),
    soil_moisture_alerts BOOLEAN NOT NULL DEFAULT TRUE,
    temperature_alerts BOOLEAN NOT NULL DEFAULT TRUE,
    power_alerts BOOLEAN NOT NULL DEFAULT TRUE,
    quiet_hours_enabled BOOLEAN NOT NULL DEFAULT FALSE,
    quiet_hours_start SMALLINT NOT NULL DEFAULT 23,
    quiet_hours_end SMALLINT NOT NULL DEFAULT 7
);
