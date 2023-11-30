use crate::{helpers::calc::*, models::market::Market, scanner::instrument::Instrument};

use serde::{Deserialize, Serialize};

use super::trade::{TradeIn, TradeOut};

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

pub fn is_mtf_strategy(strategy_type: &StrategyType) -> bool {
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

pub fn calculate_strategy_stats(
    instrument: &Instrument,
    trades_in: &Vec<TradeIn>,
    trades_out: &Vec<TradeOut>,
    equity: f64,
    commission: f64,
) -> StrategyStats {
    log::info!("Calculating stats");
    let _size = 1.;
    let data = &instrument.data;
    if !trades_out.is_empty() {
        let current_candle = data.last().unwrap();
        let current_price = current_candle.close;

        let w_trades: Vec<&TradeOut> = trades_out.iter().filter(|x| x.profit >= 0.).collect();
        let l_trades: Vec<&TradeOut> = trades_out.iter().filter(|x| x.profit < 0.).collect();
        let wining_trades = w_trades.len();
        let losing_trades = l_trades.len();
        let trades = wining_trades + losing_trades;
        let won_per_trade_per = avg_per_trade(&w_trades);
        let lost_per_trade_per = avg_per_trade(&l_trades);
        let stop_losses = trades_out.iter().filter(|x| x.trade_type.is_stop()).count();
        let gross_profits = total_gross(&w_trades);
        let gross_loses = total_gross(&l_trades);
        let gross_profit = gross_profits + gross_loses;
        let commissions = total_commissions(trades, commission);
        let net_profit = gross_profit - commissions;
        let first = trades_in.first().unwrap();

        let initial_order_amount = (first.price_in * first.size).ceil();
        let profit_factor = total_profit_factor(gross_profits, gross_loses);
        let net_profit_per = total_profit_per(trades_out);
        let profitable_trades = total_profitable_trades(wining_trades, trades);

        let max_drawdown = match instrument.market() {
            Market::Forex => total_drawdown(trades_out, equity) * 10.,
            _ => total_drawdown(trades_out, equity),
        };

        let max_runup = total_runup(trades_out, equity);
        let strategy_start_price = match instrument.data.first().map(|x| x.open) {
            Some(open) => open,
            _ => 0.,
        };

        let buy_hold =
            calculate_buy_hold(strategy_start_price, initial_order_amount, current_price);
        let annual_return = 100.;

        StrategyStats {
            trades,
            wining_trades,
            losing_trades,
            won_per_trade_per,
            lost_per_trade_per,
            stop_losses,
            gross_profit,
            commissions,
            net_profit,
            net_profit_per,
            profitable_trades,
            profit_factor,
            max_runup,
            max_drawdown,
            buy_hold,
            annual_return,
        }
    } else {
        StrategyStats::new()
    }
}
