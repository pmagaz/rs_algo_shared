pub use bson::DateTime as DbDateTime;
pub use chrono::offset::{Local, TimeZone, Utc};
use chrono::Offset;
use regex::Regex;

pub use chrono::{DateTime, Datelike, Duration, NaiveDateTime, NaiveTime, Timelike};

pub fn parse_time(date: i64) -> DateTime<Local> {
    let ts = chrono::NaiveDateTime::from_timestamp(date, 0);
    Local.from_utc_datetime(&ts)
}

pub fn utc_reg() -> Regex {
    Regex::new(r"\+0[1-9]").unwrap()
}

pub fn to_dbtime(date: DateTime<Local>) -> DbDateTime {
    let offset_str = date.offset().to_string();
    let offset_seconds = date.offset().local_minus_utc() as i64;
    match utc_reg().find(&offset_str) {
        Some(_) => DbDateTime::from_chrono(date + Duration::seconds(offset_seconds)),
        None => DbDateTime::from_chrono(date),
    }
}

pub fn from_dbtime(date: &DbDateTime) -> DateTime<Local> {
    let date: DateTime<Local> = DateTime::from(date.to_chrono());
    let offset_str = date.offset().to_string();
    let offset_seconds = date.offset().fix().utc_minus_local() as i64;

    let db_date_time = match utc_reg().find(&offset_str) {
        Some(_) => date + Duration::seconds(offset_seconds),
        None => date,
    };

    db_date_time
}

pub fn get_week_day(date: DateTime<Local>) -> u32 {
    date.weekday().number_from_monday()
}
