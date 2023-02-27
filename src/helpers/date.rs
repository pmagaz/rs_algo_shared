pub use bson::{Bson, DateTime as DbDateTime};
pub use chrono::offset::{Local, TimeZone, Utc};
use chrono::FixedOffset;
pub use chrono::{DateTime, Datelike, Duration, NaiveDateTime, NaiveTime, Timelike};

pub fn parse_time(date: i64) -> DateTime<Local> {
    let ts = chrono::NaiveDateTime::from_timestamp(date, 0);
    Local.from_utc_datetime(&ts)
}

pub fn to_dbtime(date: DateTime<Local>) -> DbDateTime {
    let ts = date.timestamp_micros() / 1000;
    let db_time: bson::DateTime = DbDateTime::from_millis(ts);
    db_time
}

pub fn fom_dbtime(date: &DbDateTime) -> DateTime<Local> {
    let date_time: DateTime<Local> = DateTime::from(date.to_chrono());
    date_time
}
