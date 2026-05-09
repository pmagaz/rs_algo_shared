use std::env;

use crate::helpers::date;
use crate::helpers::date::{DateTime, Duration, Local, Timelike};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Market {
    Stock,
    Forex,
    Crypto,
    Default,
}

impl Market {
    pub fn from_str(s: &str) -> Self {
        match s {
            "Forex" => Market::Forex,
            "Crypto" => Market::Crypto,
            "Stock" => Market::Stock,
            _ => Market::Default,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MarketSessions {
    NewYork,
    London,
    Tokyo,
    Sydney,
    // Frankfurt,
    // HongKongSingapore,
    // Toronto,
    // Zurich,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketSessionHours {
    pub session: MarketSessions,
    pub from: u32,
    pub to: u32,
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
    session_hours: Vec<MarketSessionHours>,
}

impl Default for MarketHours {
    fn default() -> Self {
        MarketHours {
            retry_after: 0,
            symbol: "".to_string(),
            data: vec![],
            session_hours: vec![],
        }
    }
}

impl MarketHours {
    pub fn new(symbol: String, data: Vec<MarketHour>) -> Self {
        let session_hours = Self::default_session_hours();

        MarketHours {
            symbol,
            data,
            retry_after: 0,
            session_hours,
        }
    }

    /// Build market hours for the given market type.
    /// Brokers call this instead of hardcoding schedules.
    pub fn for_market(market: &Market, symbol: &str) -> Self {
        let data = match market {
            Market::Forex => vec![
                MarketHour { day: 1, from: 0, to: 23 },
                MarketHour { day: 2, from: 0, to: 23 },
                MarketHour { day: 3, from: 0, to: 23 },
                MarketHour { day: 4, from: 0, to: 23 },
                MarketHour { day: 5, from: 0, to: 22 },
                MarketHour { day: 7, from: 22, to: 23 },
            ],
            _ => vec![
                MarketHour { day: 1, from: 9, to: 17 },
                MarketHour { day: 2, from: 9, to: 17 },
                MarketHour { day: 3, from: 9, to: 17 },
                MarketHour { day: 4, from: 9, to: 17 },
                MarketHour { day: 5, from: 9, to: 17 },
            ],
        };
        Self::new(symbol.to_string(), data)
    }

    pub fn symbol(&self) -> String {
        self.symbol.to_owned()
    }
    pub fn data(&self) -> &Vec<MarketHour> {
        &self.data
    }

    fn default_session_hours() -> Vec<MarketSessionHours> {
        let now = Local::now();
        let dst_adjustment = if date::is_dst(&now) { 1 } else { 0 };

        //GMT +1
        vec![
            MarketSessionHours {
                session: MarketSessions::NewYork,
                from: 14 + dst_adjustment, // Adjust for DST
                to: 23 + dst_adjustment,
            },
            MarketSessionHours {
                session: MarketSessions::London,
                from: 9 + dst_adjustment, // Adjust for DST
                to: 18 + dst_adjustment,
            },
            // Tokyo and Sydney times remain constant as they are not affected by GMT+1/GMT+2 changes
            MarketSessionHours {
                session: MarketSessions::Tokyo,
                from: 1,
                to: 10,
            },
            MarketSessionHours {
                session: MarketSessions::Sydney,
                from: 21,
                to: 5,
            },
        ]
    }

    pub fn is_trading_time(&self) -> bool {
        let current_date = Local::now();
        let current_hours = current_date.hour();
        let week_day = date::get_week_day(current_date);
        let mut open = false;

        for key in &self.data {
            if key.day == week_day && week_day != 6 {
                open = current_hours >= key.from && current_hours < key.to;
                break;
            } else {
                open = false;
            }
        }
        open
    }

    pub fn is_week_close(&self) -> bool {
        let current_date = Local::now();
        let current_hours = current_date.hour();
        let week_day = date::get_week_day(current_date);
        let mut open = false;

        for key in &self.data {
            if key.day == week_day && week_day == 5 {
                open = current_hours >= key.from && current_hours < key.to;
                break;
            } else {
                open = false;
            }
        }
        open
    }

    pub fn current_session(&self, date: DateTime<Local>) -> Option<MarketSessions> {
        let current_hour = date.hour();

        for session_hour in &self.session_hours {
            if (session_hour.from < session_hour.to
                && current_hour >= session_hour.from
                && current_hour < session_hour.to)
                || (session_hour.from > session_hour.to
                    && (current_hour >= session_hour.from || current_hour < session_hour.to))
            {
                return Some(session_hour.session.clone());
            }
        }

        None
    }

    pub fn wait_until(&self) -> DateTime<Local> {
        let current_date = Local::now();
        let weekday = date::get_week_day(current_date);

        let sunday = match self.data.last() {
            Some(d) => d,
            None => return current_date + Duration::seconds(60),
        };

        let opening_hours = sunday.from;
        let diff_days = 7 - weekday as i64;
        let mut opening_date = current_date + Duration::days(diff_days);
        let mut secs = env::var("DISCONNECTED_RETRY")
            .unwrap()
            .parse::<u64>()
            .unwrap();

        let ten_random_secs = (std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("Time went backwards")
            .subsec_nanos()
            % 11) as u64;

        secs += ten_random_secs;

        opening_date = opening_date
            .with_hour(opening_hours)
            .unwrap()
            .with_minute(5)
            .unwrap()
            .with_second(secs as u32)
            .unwrap();

        opening_date
    }
}
