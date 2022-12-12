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

    pub fn closing_time(&self) -> Vec<u32> {
        match *self {
            TimeFrameType::ERR => vec![0],
            TimeFrameType::M1 => vec![0],
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

pub fn adapt_to_time_frame(data: DOHLC, time_frame: &TimeFrameType) -> DOHLCC {
    let date = data.0;
    let current_minute = date.minute() + 1;
    let current_hour = date.hour() + 1;

    let add_minutes = time_frame.max_bars();
    let open_until = date + Duration::minutes(add_minutes);
    let still_open = Local::now() <= open_until && date <= open_until;
    let is_closed = !still_open;
    let mut adapted: DOHLCC = (data.0, data.1, data.2, data.3, data.4, data.5, is_closed);

    match time_frame {
        TimeFrameType::M5 => {
            if TimeFrameType::M5.closing_time().contains(&current_minute) && still_open {
                log::info!("Candle {} closed ", time_frame);
                adapted.6 = true;
            }
            adapted
        }
        TimeFrameType::M15 => {
            if TimeFrameType::M15.closing_time().contains(&current_minute) && still_open {
                log::info!("Candle {} closed ", time_frame);
                adapted.6 = true;
            }
            adapted
        }
        TimeFrameType::M30 => {
            if TimeFrameType::M30.closing_time().contains(&current_minute) && still_open {
                log::info!("Candle {} closed ", time_frame);
                adapted.6 = true;
            }
            adapted
        }
        TimeFrameType::H1 => {
            if TimeFrameType::M1.closing_time().contains(&current_minute) && still_open {
                log::info!("Candle {} closed ", time_frame);
                adapted.6 = true;
            }
            adapted
        }
        TimeFrameType::H4 => {
            if TimeFrameType::H4.closing_time().contains(&current_hour)
                && TimeFrameType::M1.closing_time().contains(&current_minute)
                && still_open
            {
                log::info!("Candle {} closed ", time_frame);
                adapted.6 = true;
            }
            adapted
        }
        //M1
        _ => {
            log::info!("Candle {} closed ", time_frame);
            adapted.6 = true;
            adapted
        }
    };
    adapted
}
