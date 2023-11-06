use crate::helpers::date::{DateTime, Duration, Local, Timelike};
use serde::{Deserialize, Serialize};

use crate::helpers::date;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Market {
    Stock,
    Forex,
    Crypto,
    Default,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketHour {
    pub day: u32,
    pub from: u32,
    pub to: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketHours {
    retry_after: isize,
    symbol: String,
    data: Vec<MarketHour>,
}

impl MarketHours {
    pub fn new(symbol: String, data: Vec<MarketHour>) -> Self {
        MarketHours {
            symbol,
            data,
            retry_after: 0,
        }
    }

    // pub fn open(&self) -> bool {
    //     self.open
    // }
    pub fn symbol(&self) -> String {
        self.symbol.to_owned()
    }
    pub fn data(&self) -> &Vec<MarketHour> {
        &self.data
    }

    pub fn is_trading_time(&self) -> bool {
        let current_date = Local::now();
        let current_hours = current_date.hour();
        let week_day = date::get_week_day(current_date);
        let mut open = false;
        for key in &self.data {
            if key.day == week_day {
                open = current_hours >= key.from && current_hours <= key.to;
            } else {
                if week_day == 6 {
                    open = false;
                }
            }
        }
        open
    }

    pub fn wait_until(&self) -> DateTime<Local> {
        let current_date = Local::now();
        let weekday = date::get_week_day(current_date);

        //let friday = &self.data.get(4).unwrap();
        let sunday = &self.data.last().unwrap();

        let opening_hours = sunday.from;
        let diff_days = 7 - weekday as i64;
        let mut opening_date = current_date + Duration::days(diff_days);
        opening_date = opening_date
            .with_hour(opening_hours)
            .unwrap()
            .with_minute(5)
            .unwrap()
            .with_second(5)
            .unwrap();

        opening_date
    }
}
