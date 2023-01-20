pub use bson::{Bson, DateTime as DbDateTime};
pub use chrono::offset::{Local, TimeZone, Utc};
pub use chrono::{Date, DateTime, Datelike, Duration, NaiveDateTime, NaiveTime, Timelike};

pub fn parse_time(date: i64) -> DateTime<Local> {
    let ts = chrono::NaiveDateTime::from_timestamp(date, 0);
    Local.from_utc_datetime(&ts)
}

pub fn to_dbtime(date: DateTime<Local>) -> DbDateTime {
    DbDateTime::from_chrono(date + Duration::hours(2))
}

pub fn fom_dbtime(date: DbDateTime) -> DateTime<Local> {
    date.to_chrono().into()
}
