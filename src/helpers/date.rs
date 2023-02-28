pub use bson::DateTime as DbDateTime;
pub use chrono::offset::{Local, TimeZone, Utc};

pub use chrono::{DateTime, Datelike, Duration, NaiveDateTime, NaiveTime, Timelike};

pub fn parse_time(date: i64) -> DateTime<Local> {
    let ts = chrono::NaiveDateTime::from_timestamp(date, 0);
    Local.from_utc_datetime(&ts)
}

pub fn to_dbtime(date: DateTime<Local>) -> DbDateTime {
    let offset = date.offset().to_string();
    //let offset = date.offset().to_string()[2..3].parse::<i64>().unwrap();
    let db_date_time = match offset.contains("+01") {
        true => DbDateTime::from_chrono(date + Duration::hours(1)),
        false => DbDateTime::from_chrono(date),
    };
    db_date_time
}

pub fn fom_dbtime(date: &DbDateTime) -> DateTime<Local> {
    let date_time: DateTime<Local> = DateTime::from(date.to_chrono());
    date_time
}
