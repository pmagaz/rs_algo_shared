use crate::{
    helpers::{
        calc::get_prev_index,
        date::{DateTime, Duration, Local},
    },
    scanner::instrument::{HTFInstrument, Instrument},
};

use chrono::Timelike;
use serde::{Deserialize, Serialize};

use super::{
    mode::{self, ExecutionMode},
    trade::TradeDirection,
};

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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
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

    pub fn get_starting_bar(
        num_bars: i64,
        time_frame: &TimeFrameType,
        execution_mode: &ExecutionMode,
    ) -> DateTime<Local> {
        // let bars_time_frame = match execution_mode {
        //     mode::ExecutionMode::Scanner | mode::ExecutionMode::ScannerBackTest => num_bars,
        //     _ => num_bars, // * time_frame.to_minutes(),
        // };

        match time_frame {
            TimeFrameType::D | TimeFrameType::W => Local::now() - Duration::days(num_bars),
            TimeFrameType::H4 => Local::now() - Duration::minutes(num_bars),
            TimeFrameType::H1 => Local::now() - Duration::minutes(num_bars),
            TimeFrameType::M30 => Local::now() - Duration::minutes(num_bars),
            TimeFrameType::M15 => Local::now() - Duration::minutes(num_bars),
            TimeFrameType::M5 => Local::now() - Duration::minutes(num_bars),
            TimeFrameType::M1 => Local::now() - Duration::minutes(num_bars),
            _ => Local::now() - Duration::days(num_bars),
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

    pub fn is_base_time_frame(&self) -> bool {
        self == &TimeFrameType::M1
    }

    pub fn is_minutely_time_frame(&self) -> bool {
        self == &TimeFrameType::M1
            || self == &TimeFrameType::M5
            || self == &TimeFrameType::M15
            || self == &TimeFrameType::M30
    }

    pub fn is_hourly_time_frame(&self) -> bool {
        self == &TimeFrameType::H1 || self == &TimeFrameType::H4
    }

    pub fn is_daily_time_frame(&self) -> bool {
        self == &TimeFrameType::D
    }

    pub fn to_minutes(&self) -> i64 {
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

    pub fn to_hours(&self) -> i64 {
        match *self {
            TimeFrameType::ERR => 0,
            TimeFrameType::H1 => 1,
            TimeFrameType::H4 => 4,
            TimeFrameType::D => 24,
            TimeFrameType::W => 168,
            TimeFrameType::MN => 672,
            _ => 0,
        }
    }

    pub fn prev_candles(&self) -> i64 {
        self.to_number()
    }

    pub fn closing_minutes(&self) -> Vec<i64> {
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
            TimeFrameType::H1 => vec![0],
            TimeFrameType::H4 => vec![0],
            TimeFrameType::D => vec![0],
            TimeFrameType::W => vec![0],
            TimeFrameType::MN => vec![0],
        }
    }

    pub fn closing_hours(&self) -> Vec<i64> {
        match *self {
            TimeFrameType::ERR => vec![],
            TimeFrameType::M1 => vec![],
            TimeFrameType::M5 => vec![],
            TimeFrameType::M15 => vec![],
            TimeFrameType::M30 => vec![],
            TimeFrameType::H1 => vec![
                0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22,
                23,
            ],
            TimeFrameType::H4 => vec![0, 4, 8, 12, 16, 20],
            TimeFrameType::D => vec![0],
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
    let candle_minute = date.minute() as i64 + 1;
    let candle_hour = date.hour() as i64 + 1;
    let num_minutes = time_frame.to_minutes();
    let num_hours = time_frame.to_hours();

    let open_until = match next {
        true => match time_frame.is_minutely_time_frame() {
            true => {
                let closing_idx = time_frame
                    .closing_minutes()
                    .iter()
                    .enumerate()
                    .filter(|(_i, val)| candle_minute > **val)
                    .map(|(i, _val)| i)
                    .last()
                    .unwrap();

                let closing_minutes = time_frame.closing_minutes();

                let next_close_mins = match closing_minutes.get(closing_idx + 1) {
                    Some(val) => *val,
                    None => time_frame.closing_minutes().first().unwrap() + 60,
                };

                date + Duration::minutes(next_close_mins - candle_minute + 1)
            }
            false => {
                let closing_idx = time_frame
                    .closing_hours()
                    .iter()
                    .enumerate()
                    .filter(|(_i, val)| candle_hour > **val)
                    .map(|(i, _val)| i)
                    .last()
                    .unwrap();

                let closing_hours = time_frame.closing_hours();

                let next_close_hours = match closing_hours.get(closing_idx + 1) {
                    Some(val) => *val,
                    None => time_frame.closing_hours().first().unwrap() + 24,
                };

                date + Duration::hours(next_close_hours - candle_hour + 1)
                    - Duration::minutes(candle_minute - 1)
            }
        },
        false => match time_frame.is_minutely_time_frame() {
            true => date + Duration::minutes(num_minutes),
            false => date + Duration::hours(num_hours),
        },
    };

    open_until
}

pub fn get_open_from(data: DOHLC, time_frame: &TimeFrameType, next: bool) -> DateTime<Local> {
    let minutes_interval = time_frame.to_minutes();
    get_open_until(data, time_frame, next) - Duration::minutes(minutes_interval)
}

pub fn adapt_to_time_frame(data: DOHLC, time_frame: &TimeFrameType, next: bool) -> DOHLCC {
    let date = data.0;
    let now = Local::now();
    let minutes = date.minute() as i64;
    let num_minutes = time_frame.to_minutes();
    let num_hours = time_frame.to_hours();

    let open_until = match next {
        true => {
            if time_frame.is_minutely_time_frame() {
                match time_frame.closing_minutes().contains(&minutes) {
                    true => {
                        //"2023-02-23T23:00:00+01:00"
                        //  1111111 2023-02-23T23:05:00+01:00
                        // 777777777 (2023-02-23T23:00:00+01:00, 2023-02-23T22:55:00+01:00)
                        //log::info!("1111111 {:?}", get_open_until(data, time_frame, next));
                        get_open_until(data, time_frame, next) - Duration::minutes(num_minutes)
                    }
                    false => get_open_until(data, time_frame, next),
                }
            } else if time_frame.is_hourly_time_frame() {
                let hours = date.hour() as i64;
                match time_frame.closing_hours().contains(&hours)
                    && time_frame.closing_minutes().contains(&minutes)
                {
                    true => {
                        //"2023-02-23T23:00:00+01:00"
                        //22222222 2023-02-24T00:00:00+01:00
                        // 777777777 (2023-02-23T23:00:00+01:00, 2023-02-23T22:00:00+01:00)
                        get_open_until(data, time_frame, next) - Duration::hours(num_hours)
                    }
                    false => get_open_until(data, time_frame, next),
                }
            } else {
                get_open_until(data, time_frame, next)
            }
        }
        false => get_open_until(data, time_frame, next),
    };

    let open_from = match time_frame.is_minutely_time_frame() {
        true => open_until - Duration::minutes(num_minutes),
        false => open_until - Duration::hours(num_hours),
    };

    let is_closed = match next {
        true => date == open_until,
        false => now >= open_until,
    };

    let adapted: DOHLCC = match next {
        true => (open_from, data.1, data.2, data.3, data.4, data.5, is_closed),
        false => (data.0, data.1, data.2, data.3, data.4, data.5, is_closed),
    };

    adapted
}

pub fn get_htf_data<F>(
    index: usize,
    instrument: &Instrument,
    htf_instrument: &HTFInstrument,
    mut callback: F,
) -> bool
where
    F: Send + FnMut((usize, usize, &Instrument)) -> bool,
{
    let base_date = &instrument.data.get(index).unwrap().date;
    let upper_tf_data = match htf_instrument {
        HTFInstrument::HTFInstrument(htf_instrument) => {
            let upper_indexes: Vec<usize> = htf_instrument
                .data
                .iter()
                .enumerate()
                .filter(|(_id, x)| &x.date <= base_date)
                .map(|(id, _x)| id)
                .collect();

            let upper_tf_indx = match upper_indexes.last() {
                Some(val) => *val,
                _ => 0,
            };

            let prev_upper_tf_indx = get_prev_index(upper_tf_indx);

            (upper_tf_indx, prev_upper_tf_indx, htf_instrument)
        }
        _ => (0, 0, instrument),
    };
    callback(upper_tf_data)
}

pub fn get_htf_trading_direction<F>(
    index: usize,
    instrument: &Instrument,
    htf_instrument: &HTFInstrument,
    mut callback: F,
) -> TradeDirection
where
    F: Send + FnMut((usize, usize, &Instrument)) -> TradeDirection,
{
    let base_date = &instrument.data.get(index).unwrap().date;
    let upper_tf_data = match htf_instrument {
        HTFInstrument::HTFInstrument(htf_instrument) => {
            let upper_indexes: Vec<usize> = htf_instrument
                .data
                .iter()
                .enumerate()
                .filter(|(_id, x)| &x.date <= base_date)
                .map(|(id, _x)| id)
                .collect();

            let upper_tf_indx = match upper_indexes.last() {
                Some(val) => *val,
                _ => 0,
            };

            let prev_upper_tf_indx = get_prev_index(upper_tf_indx);

            (upper_tf_indx, prev_upper_tf_indx, htf_instrument)
        }
        _ => (0, 0, instrument),
    };
    callback(upper_tf_data)
}
// pub fn get_bot_htf_data<F>(
//     instrument: &Instrument,
//     htf_instrument: &HTFInstrument,
//     mut callback: F,
// ) -> bool
// where
//     F: Send + FnMut((usize, usize, &Instrument)) -> bool,
// {
//     let base_date = &instrument.data.last().unwrap().date;
//     let upper_tf_data = match htf_instrument {
//         HTFInstrument::HTFInstrument(htf_instrument) => {
//             let upper_indexes: Vec<usize> = htf_instrument
//                 .data
//                 .iter()
//                 .enumerate()
//                 .filter(|(_id, x)| &x.date <= base_date)
//                 .map(|(id, _x)| id)
//                 .collect();

//             let upper_tf_indx = match upper_indexes.last() {
//                 Some(val) => *val,
//                 _ => 0,
//             };

//             let prev_upper_tf_indx = get_prev_index(upper_tf_indx);

//             (upper_tf_indx, prev_upper_tf_indx, htf_instrument)
//         }
//         _ => (0, 0, instrument),
//     };
//     callback(upper_tf_data)
// }
