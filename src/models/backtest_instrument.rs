use crate::helpers::date::*;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackTestInstrument {
    pub symbol: String,
    pub trades_in: Vec<TradeIn>,
    pub trades_out: Vec<TradeOut>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TradeIn {
    pub index_in: i32,
    pub price_in: f64,
    pub quantity: i32,
    pub date: DbDateTime,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TradeOut {
    pub index_in: i32,
    pub price_in: f64,
    pub quantity: i32,
    pub index_out: i32,
    pub price_out: f64,
    pub diff: f64,
}

impl std::fmt::Display for BackTestInstrument {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl std::fmt::Display for TradeIn {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl std::fmt::Display for TradeOut {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}
