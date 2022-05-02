use crate::helpers::date::*;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TradeDirection {
    Long,
    Short,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TradeType {
    Entry(TradeDirection),
    Exit(TradeDirection),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackTestResult {
    pub instrument: BackTestInstrument,
    pub strategy: String,
    pub trades: usize,
    pub net_profit: f64,
    pub profitable_per: f64,
    pub profit_factor: f64,
    pub max_runup: f64,
    pub max_drawdown: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackTestInstrument {
    pub symbol: String,
    pub trades_in: Vec<TradeIn>,
    pub trades_out: Vec<TradeOut>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TradeIn {
    pub index_in: usize,
    pub price_in: f64,
    pub stop_loss: f64,
    pub date_in: DbDateTime,
    pub direction: TradeDirection,
    pub trade_type: TradeType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TradeOut {
    pub index_in: usize,
    pub price_in: f64,
    pub date_in: DbDateTime,
    pub index_out: usize,
    pub price_out: f64,
    pub date_out: DbDateTime,
    pub profit: f64,
    pub profit_per: f64,
    pub cum_profit: f64,
    pub cum_profit_per: f64,
    pub run_up: f64,
    pub run_up_per: f64,
    pub draw_down: f64,
    pub draw_down_per: f64,
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
