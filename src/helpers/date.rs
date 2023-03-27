pub use bson::DateTime as DbDateTime;
pub use chrono::offset::{Local, TimeZone, Utc};
use chrono::Offset;

pub use chrono::{DateTime, Datelike, Duration, NaiveDateTime, NaiveTime, Timelike};

pub fn parse_time(date: i64) -> DateTime<Local> {
    let ts = chrono::NaiveDateTime::from_timestamp(date, 0);
    Local.from_utc_datetime(&ts)
}

pub fn to_dbtime(date: DateTime<Local>) -> DbDateTime {
    let offset_str = date.offset().to_string();
    let offset_seconds = date.offset().local_minus_utc() as i64;
    match offset_str.contains("+") {
        true => DbDateTime::from_chrono(date + Duration::seconds(offset_seconds)),
        false => DbDateTime::from_chrono(date),
    }
}

pub fn from_dbtime(date: &DbDateTime) -> DateTime<Local> {
    let chrono_date = date.to_chrono();
    let offset_str = chrono_date.offset().to_string();
    let offset_seconds = chrono_date.offset().fix().utc_minus_local() as i64;
    let db_date_time: DateTime<Local> = match offset_str.contains("UTC") {
        true => DateTime::from(chrono_date - Duration::seconds(offset_seconds)),
        false => DateTime::from(chrono_date),
    };
    db_date_time
}

pub fn get_week_day(date: DateTime<Local>) -> u32 {
    date.weekday().number_from_monday()
}
