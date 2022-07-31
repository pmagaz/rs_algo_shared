use crate::helpers::date::*;
use crate::models::backtest_strategy::*;
use crate::models::market::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TradeDirection {
    Long,
    Short,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TradeType {
    EntryLong,
    ExitLong,
    EntryShort,
    ExitShort,
    StopLoss,
    TakeProfit,
    None,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct BackTestInstrumentResult {
    #[serde(rename = "_id", skip_serializing)]
    pub id: bson::oid::ObjectId,
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
}

pub enum TradeResult {
    TradeIn(TradeIn),
    TradeOut(TradeOut),
    None,
}

pub enum BackTestResult {
    BackTestInstrumentResult(BackTestInstrumentResult),
    None,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TradeIn {
    pub index_in: usize,
    pub price_in: f64,
    pub stop_loss: f64,
    pub date_in: DbDateTime,
    pub trade_type: TradeType,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TradeOut {
    pub trade_type: TradeType,
    pub index_in: usize,
    pub price_in: f64,
    pub date_in: DbDateTime,
    pub index_out: usize,
    pub price_out: f64,
    pub date_out: DbDateTime,
    pub profit: f64,
    pub profit_per: f64,
    pub run_up: f64,
    pub run_up_per: f64,
    pub draw_down: f64,
    pub draw_down_per: f64,
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
