/// Soil moisture thresholds (%)
pub mod soil {
    pub const VERY_DRY: f32 = 20.0;
    pub const DRY: f32 = 40.0;
    pub const WET: f32 = 70.0;
    pub const ALERT_LOW: f32 = 30.0;
}

/// Water level thresholds (%)
pub mod water {
    pub const LOW: f32 = 20.0;
    pub const MEDIUM: f32 = 40.0;
}

/// Temperature thresholds (°C)
pub mod temperature {
    pub const ALERT_HIGH: f32 = 35.0;
    #[allow(dead_code)]
    pub const ALERT_LOW: f32 = 5.0;
}

/// Pressure trend thresholds (hPa)
pub mod pressure {
    pub const FALLING_FAST: f32 = -2.0;
    pub const FALLING: f32 = -0.5;
    pub const RISING_FAST: f32 = 2.0;
    pub const RISING: f32 = 0.5;
    pub const STORM_THRESHOLD: f32 = -3.0;
    pub const RAIN_THRESHOLD: f32 = -1.5;
    pub const CLEAR_THRESHOLD: f32 = 1.5;
    pub const TREND_HOURS: i32 = 3;
}

/// Alert cooldown (seconds)
pub const ALERT_COOLDOWN_SECS: i64 = 300;

/// Power outage detection
pub mod power {
    #[allow(dead_code)]
    pub const SENSOR_INTERVAL_SECS: i64 = 60;

    /// How long without data before considering power down (seconds)
    pub const OUTAGE_THRESHOLD_SECS: i64 = 150;

    /// How often to check for outages (seconds)
    pub const CHECK_INTERVAL_SECS: u64 = 120;
}
