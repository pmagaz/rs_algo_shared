use crate::helpers::date::{DateTime, Local};
use serde::{Deserialize, Serialize};

pub type LECHES = (DateTime<Local>, f64, f64, f64, f64, f64, f64);

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TimeFrameType {
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

pub fn parse_data_timeframe(data: LECHES, time_frame: TimeFrameType) -> LECHES {
    data
}
