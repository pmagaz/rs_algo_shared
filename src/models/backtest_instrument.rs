use crate::helpers::date::*;
use crate::models::market::*;

use crate::models::strategy::*;
use crate::models::trade::*;

use serde::{Deserialize, Serialize};

use super::order::Order;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct BackTestInstrumentResult {
    pub instrument: BackTestInstrument,
    pub strategy: String,
    pub market: Market,
    pub strategy_type: StrategyType,
    pub date_start: DbDateTime,
    pub date_end: DbDateTime,
    pub sessions: usize,
    pub trades: usize,
    pub wining_trades: usize,
    pub losing_trades: usize,
    pub won_per_trade_per: f64,
    pub lost_per_trade_per: f64,
    pub stop_losses: usize,
    pub gross_profit: f64,
    pub commissions: f64,
    pub net_profit: f64,
    pub net_profit_per: f64,
    pub profitable_trades: f64,
    pub profit_factor: f64,
    pub max_runup: f64,
    pub max_drawdown: f64,
    pub buy_hold: f64,
    pub annual_return: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct BackTestInstrument {
    pub symbol: String,
    pub trades_in: Vec<TradeIn>,
    pub trades_out: Vec<TradeOut>,
    pub orders: Vec<Order>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackTestSpread {
    pub symbol: String,
    pub spread: f64,
}

pub enum BackTestResult {
    BackTestInstrumentResult(BackTestInstrumentResult),
    None,
}

impl std::fmt::Display for Market {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl std::fmt::Display for BackTestInstrument {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}
