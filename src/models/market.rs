use chrono::{DateTime, Local};
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
    pub day: isize,
    pub from: DateTime<Local>,
    pub to: DateTime<Local>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketHours {
    open: bool,
    retry_after: isize,
    symbol: String,
    data: Vec<MarketHour>,
}

impl MarketHours {
    pub fn new(open: bool, symbol: String, data: Vec<MarketHour>) -> Self {
        MarketHours {
            open,
            symbol,
            data,
            retry_after: 0,
        }
    }

    pub fn open(&self) -> bool {
        self.open
    }
    pub fn symbol(&self) -> String {
        self.symbol.to_owned()
    }
    pub fn data(&self) -> &Vec<MarketHour> {
        &self.data
    }
    pub fn is_open(&self) -> bool {
        let current_date = Local::now();
        let week_day = date::get_week_day(current_date) as isize;
        let mut open = false;
        for key in &self.data {
            if key.day == week_day {
                if current_date > key.from && current_date < key.to {
                    open = true
                } else {
                    open = false
                }
            }
        }
        open
    }
}
