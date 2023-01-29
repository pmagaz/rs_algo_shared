use crate::helpers::{date::*, uuid};
use crate::models::market::*;
use crate::models::strategy::*;
pub use bson::Uuid;
use serde::{Deserialize, Serialize};

use super::time_frame::TimeFrameType;

impl std::fmt::Display for StrategyType {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct BackTestStrategyResult {
    #[serde(rename = "_id")]
    pub uuid: Uuid,
    pub strategy: String,
    pub strategy_type: StrategyType,
    pub time_frame: TimeFrameType,
    pub market: Market,
    pub date: DbDateTime,
    pub avg_sessions: usize,
    pub avg_trades: usize,
    pub avg_wining_trades: usize,
    pub avg_won_per_trade: f64,
    pub avg_lost_per_trade: f64,
    pub avg_losing_trades: usize,
    pub avg_stop_losses: usize,
    pub avg_gross_profit: f64,
    pub avg_commissions: f64,
    pub avg_net_profit: f64,
    pub avg_net_profit_per: f64,
    pub avg_profitable_trades: f64,
    pub avg_profit_factor: f64,
    pub avg_max_runup: f64,
    pub avg_max_drawdown: f64,
    pub avg_buy_hold: f64,
    pub avg_annual_return: f64,
}
