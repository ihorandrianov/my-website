use crate::config::{pressure, soil, water};
use crate::db::SensorData;

#[derive(Debug, Clone, Copy)]
pub enum Status {
    Critical,
    Warning,
    Good,
    High,
}

impl Status {
    pub fn emoji(&self) -> &'static str {
        match self {
            Status::Critical => "🔴",
            Status::Warning => "🟡",
            Status::Good => "🟢",
            Status::High => "🔵",
        }
    }

    #[allow(dead_code)]
    pub fn as_str(&self) -> &'static str {
        match self {
            Status::Critical => "Critical",
            Status::Warning => "Warning",
            Status::Good => "Good",
            Status::High => "High",
        }
    }
}

pub struct SoilAnalysis {
    pub status: Status,
    pub message: &'static str,
}

pub fn analyze_soil_moisture(value: f32) -> SoilAnalysis {
    if value < soil::VERY_DRY {
        SoilAnalysis {
            status: Status::Critical,
            message: "Very dry - water now!",
        }
    } else if value < soil::DRY {
        SoilAnalysis {
            status: Status::Warning,
            message: "Getting dry",
        }
    } else if value < soil::WET {
        SoilAnalysis {
            status: Status::Good,
            message: "Good",
        }
    } else {
        SoilAnalysis {
            status: Status::High,
            message: "Very wet",
        }
    }
}

pub struct WaterAnalysis {
    pub status: Status,
    pub message: &'static str,
}

pub fn analyze_water_level(value: f32) -> WaterAnalysis {
    if value < water::LOW {
        WaterAnalysis {
            status: Status::Critical,
            message: "Low - refill needed",
        }
    } else if value < water::MEDIUM {
        WaterAnalysis {
            status: Status::Warning,
            message: "Getting low",
        }
    } else {
        WaterAnalysis {
            status: Status::Good,
            message: "OK",
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum PressureTrend {
    FallingFast,
    Falling,
    Stable,
    Rising,
    RisingFast,
}

impl PressureTrend {
    pub fn symbol(&self) -> &'static str {
        match self {
            PressureTrend::FallingFast => "↓↓",
            PressureTrend::Falling => "↓",
            PressureTrend::Stable => "→",
            PressureTrend::Rising => "↑",
            PressureTrend::RisingFast => "↑↑",
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            PressureTrend::FallingFast => "falling fast",
            PressureTrend::Falling => "falling",
            PressureTrend::Stable => "stable",
            PressureTrend::Rising => "rising",
            PressureTrend::RisingFast => "rising fast",
        }
    }
}

pub struct WeatherForecast {
    pub emoji: &'static str,
    pub message: &'static str,
}

pub struct PressureAnalysis {
    pub trend: PressureTrend,
    pub delta: f32,
    pub forecast: WeatherForecast,
}

pub fn analyze_pressure(current: f32, past: f32) -> PressureAnalysis {
    let delta = current - past;

    let trend = if delta < pressure::FALLING_FAST {
        PressureTrend::FallingFast
    } else if delta < pressure::FALLING {
        PressureTrend::Falling
    } else if delta > pressure::RISING_FAST {
        PressureTrend::RisingFast
    } else if delta > pressure::RISING {
        PressureTrend::Rising
    } else {
        PressureTrend::Stable
    };

    let forecast = if delta < pressure::STORM_THRESHOLD {
        WeatherForecast {
            emoji: "⛈",
            message: "Storm likely",
        }
    } else if delta < pressure::RAIN_THRESHOLD {
        WeatherForecast {
            emoji: "🌧",
            message: "Rain possible",
        }
    } else if delta > pressure::CLEAR_THRESHOLD {
        WeatherForecast {
            emoji: "☀️",
            message: "Clear weather",
        }
    } else {
        WeatherForecast {
            emoji: "🌤",
            message: "No significant change",
        }
    };

    PressureAnalysis {
        trend,
        delta,
        forecast,
    }
}

pub fn should_alert_soil_low(data: &SensorData) -> bool {
    data.soil_moisture < soil::ALERT_LOW
}

pub fn should_alert_temp_high(data: &SensorData) -> bool {
    data.temperature > crate::config::temperature::ALERT_HIGH
}
