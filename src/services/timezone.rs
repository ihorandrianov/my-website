use time::OffsetDateTime;
use time_tz::{timezones::db::europe::KYIV, OffsetDateTimeExt};

pub fn now_kyiv() -> OffsetDateTime {
    OffsetDateTime::now_utc().to_timezone(KYIV)
}

pub fn format_kyiv(dt: time::PrimitiveDateTime) -> String {
    let utc = dt.assume_utc();
    let kyiv = utc.to_timezone(KYIV);
    kyiv.format(
        &time::format_description::parse("[day].[month] [hour]:[minute]").unwrap()
    )
    .unwrap_or_else(|_| "??".to_string())
}

pub fn format_duration_minutes(minutes: i32) -> String {
    if minutes < 60 {
        format!("{} min", minutes)
    } else {
        let hours = minutes / 60;
        let mins = minutes % 60;
        if mins == 0 {
            format!("{} h", hours)
        } else {
            format!("{} h {} min", hours, mins)
        }
    }
}
