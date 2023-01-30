use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum StrategyType {
    OnlyLong,
    OnlyShort,
    LongShort,
    LongShortMTF,
    OnlyLongMTF,
    OnlyShortMTF,
}

impl StrategyType {
    pub fn is_long_only(&self) -> bool {
        match *self {
            StrategyType::OnlyLong => true,
            StrategyType::OnlyLongMTF => true,
            _ => false,
        }
    }

    pub fn is_multi_timeframe(&self) -> bool {
        match *self {
            StrategyType::OnlyLongMTF => true,
            StrategyType::LongShortMTF => true,
            StrategyType::OnlyShortMTF => true,
            _ => false,
        }
    }
}

pub fn from_str(strategy: &str) -> StrategyType {
    match strategy {
        "OnlyLong" => StrategyType::OnlyLong,
        "OnlyShort" => StrategyType::OnlyShort,
        "LongShort" => StrategyType::LongShort,
        "LongShortMTF" => StrategyType::LongShortMTF,
        "OnlyLongMTF" => StrategyType::OnlyLongMTF,
        "OnlyShortMTF" => StrategyType::OnlyShortMTF,
        _ => StrategyType::OnlyLong,
    }
}

pub fn is_multi_timeframe_strategy(strategy_type: &StrategyType) -> bool {
    match strategy_type {
        StrategyType::OnlyLongMTF => true,
        StrategyType::LongShortMTF => true,
        StrategyType::OnlyShortMTF => true,
        _ => false,
    }
}

pub fn is_long_only(strategy_type: &StrategyType) -> bool {
    match strategy_type {
        StrategyType::OnlyLong => true,
        StrategyType::OnlyLongMTF => true,
        _ => false,
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct StrategyStats {
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

impl StrategyStats {
    pub fn new() -> StrategyStats {
        StrategyStats {
            trades: 0,
            wining_trades: 0,
            losing_trades: 0,
            won_per_trade_per: 0.,
            lost_per_trade_per: 0.,
            stop_losses: 0,
            gross_profit: 0.,
            commissions: 0.,
            net_profit: 0.,
            net_profit_per: 0.,
            profitable_trades: 0.,
            profit_factor: 0.,
            max_runup: 0.,
            max_drawdown: 0.,
            buy_hold: 0.,
            annual_return: 0.,
        }
    }
}
