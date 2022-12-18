use std::cmp::Ordering;

use crate::helpers::date::{DateTime, Duration, Local};

use chrono::Timelike;
use serde::{Deserialize, Serialize};

type DOHLC = (DateTime<Local>, f64, f64, f64, f64, f64);
type DOHLCC = (DateTime<Local>, f64, f64, f64, f64, f64, bool);
type VEC_DOHLC = Vec<DOHLC>;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TimeFrameType {
    MN,
    W,
    D,
    H4,
    H1,
    M30,
    M15,
    M5,
    M1,
    ERR,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeFrame {}

impl TimeFrame {
    pub fn new(time_frame: &str) -> TimeFrameType {
        match time_frame {
            "M5" => TimeFrameType::M5,
            "M1" => TimeFrameType::M1,
            "M15" => TimeFrameType::M15,
            "M30" => TimeFrameType::M30,
            "H1" => TimeFrameType::H1,
            "H4" => TimeFrameType::H4,
            "D" => TimeFrameType::D,
            "W" => TimeFrameType::W,
            "MN" => TimeFrameType::MN,
            &_ => TimeFrameType::ERR,
        }
    }
}

impl TimeFrameType {
    pub fn from_number(time_frame: usize) -> TimeFrameType {
        match time_frame {
            1 => TimeFrameType::M1,
            5 => TimeFrameType::M5,
            15 => TimeFrameType::M15,
            30 => TimeFrameType::M30,
            60 => TimeFrameType::H1,
            240 => TimeFrameType::H4,
            1440 => TimeFrameType::D,
            10080 => TimeFrameType::W,
            43200 => TimeFrameType::MN,
            _ => TimeFrameType::ERR,
        }
    }

    pub fn from_str(time_frame: &str) -> TimeFrameType {
        match time_frame {
            "M1" => TimeFrameType::M1,
            "M5" => TimeFrameType::M5,
            "M15" => TimeFrameType::M15,
            "M30" => TimeFrameType::M30,
            "H1" => TimeFrameType::H1,
            "H4" => TimeFrameType::H4,
            "D" => TimeFrameType::D,
            "W" => TimeFrameType::W,
            "MN" => TimeFrameType::MN,
            _ => TimeFrameType::ERR,
        }
    }

    pub fn to_number(&self) -> i64 {
        match *self {
            TimeFrameType::ERR => 0,
            TimeFrameType::M1 => 1,
            TimeFrameType::M5 => 5,
            TimeFrameType::M15 => 15,
            TimeFrameType::M30 => 30,
            TimeFrameType::H1 => 60,
            TimeFrameType::H4 => 240,
            TimeFrameType::D => 1440,
            TimeFrameType::W => 10080,
            TimeFrameType::MN => 43200,
        }
    }

    pub fn is_minutely_time_frame(&self) -> bool {
        if self == &TimeFrameType::M1
            || self == &TimeFrameType::M5
            || self == &TimeFrameType::M15
            || self == &TimeFrameType::M30
        {
            true
        } else {
            false
        }
    }

    pub fn is_hourly_time_frame(&self) -> bool {
        if self == &TimeFrameType::H1 || self == &TimeFrameType::H4 {
            true
        } else {
            false
        }
    }

    pub fn is_daily_time_frame(&self) -> bool {
        if self == &TimeFrameType::D {
            true
        } else {
            false
        }
    }

    pub fn max_bars(&self) -> i64 {
        match *self {
            TimeFrameType::ERR => 0,
            TimeFrameType::M1 => 1,
            TimeFrameType::M5 => 5,
            TimeFrameType::M15 => 15,
            TimeFrameType::M30 => 30,
            TimeFrameType::H1 => 60,
            TimeFrameType::H4 => 240,
            TimeFrameType::D => 1440,
            TimeFrameType::W => 10080,
            TimeFrameType::MN => 43200,
        }
    }

    pub fn prev_candles(&self) -> i64 {
        self.to_number()
    }

    pub fn closing_time(&self) -> Vec<i64> {
        //BRUTE FORZE XDDDDD
        match *self {
            TimeFrameType::ERR => vec![0],
            TimeFrameType::M1 => vec![
                0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22,
                23, 24, 25, 26, 27, 28, 29, 30, 31, 32, 33, 34, 35, 36, 37, 38, 39, 40, 41, 42, 43,
                44, 45, 46, 47, 48, 49, 50, 51, 52, 53, 54, 55, 56, 57, 58, 59,
            ],
            TimeFrameType::M5 => vec![0, 5, 10, 15, 20, 25, 30, 35, 40, 45, 50, 55],
            TimeFrameType::M15 => vec![0, 15, 30, 45],
            TimeFrameType::M30 => vec![0, 30],
            TimeFrameType::H1 => vec![
                0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22,
                23, 24,
            ],
            //FIXME when it starts?
            TimeFrameType::H4 => vec![0, 4, 8, 12, 16, 20, 24],
            TimeFrameType::D => vec![1],
            TimeFrameType::W => vec![1],
            TimeFrameType::MN => vec![1],
        }
    }
}

impl std::fmt::Display for TimeFrame {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl std::fmt::Display for TimeFrameType {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

pub fn get_open_until(data: DOHLC, time_frame: &TimeFrameType, next: bool) -> DateTime<Local> {
    let date = data.0;
    let minutes = date.minute() as i64 + 1;
    let hours = date.hour() as i64 + 1;
    let minutes_interval = time_frame.max_bars().clone();

    let comparator = match time_frame.is_minutely_time_frame() {
        true => minutes,
        false => hours,
    };

    let open_until = match next {
        true => {
            let next_close_idx = time_frame
                .closing_time()
                .iter()
                .enumerate()
                .filter(|(i, val)| comparator > **val)
                .map(|(i, _val)| i)
                .last()
                .unwrap();

            let closing_time = time_frame.closing_time().clone();
            let next_close = match closing_time.get(next_close_idx + 1) {
                Some(val) => val,
                _ => closing_time.first().unwrap(),
            };

            match time_frame.is_minutely_time_frame() {
                true => date + Duration::minutes(next_close - minutes + 1),
                false => date + Duration::minutes(minutes_interval - minutes + 1),
            }
        }
        false => date + Duration::minutes(minutes_interval),
    };

    open_until
}

pub fn get_open_from(data: DOHLC, time_frame: &TimeFrameType, next: bool) -> DateTime<Local> {
    let minutes_interval = time_frame.max_bars().clone();
    get_open_until(data, time_frame, next) - Duration::minutes(minutes_interval)
}

pub fn adapt_to_time_frame(data: DOHLC, time_frame: &TimeFrameType, next: bool) -> DOHLCC {
    let date = data.0;
    let now = Local::now();
    let minutes = date.minute() as i64;
    let minutes_interval = time_frame.max_bars().clone();

    println!("222222 {} {}", minutes, minutes_interval);
    let open_until = match next {
        true => match time_frame.closing_time().contains(&minutes) {
            true => {
                println!("true");
                get_open_until(data, time_frame, next) - Duration::minutes(minutes_interval)
            }
            false => {
                println!("false");
                get_open_until(data, time_frame, next)
            }
        },
        false => get_open_until(data, time_frame, next),
    };

    println!("3333333 {:?}", date);

    let open_from = open_until - Duration::minutes(minutes_interval);
    let is_closed = match next {
        true => date == open_until,
        false => now >= open_until,
    };

    let adapted: DOHLCC = match next {
        true => (open_from, data.1, data.2, data.3, data.4, data.5, is_closed),
        false => (data.0, data.1, data.2, data.3, data.4, data.5, is_closed),
    };

    println!(
        "444444 {:?} - {:?} - {:?} {} - {:?}",
        date, open_from, open_until, is_closed, adapted.0
    );

    adapted
}
