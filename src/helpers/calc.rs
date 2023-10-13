use crate::helpers::comp::*;
use crate::models::pricing::Pricing;
use crate::models::trade::*;
use crate::scanner::candle::Candle;
use round::round;

use std::cmp::Ordering;

pub fn get_min_price(data: &Vec<Candle>, index_in: usize, index_out: usize) -> f64 {
    data.iter()
        .enumerate()
        .filter(|(index, _x)| index >= &index_in && index <= &index_out)
        .map(|(_i, x)| x.low)
        .min_by(|a, b| a.partial_cmp(b).unwrap())
        .unwrap()
}

pub fn get_max_price(data: &Vec<Candle>, index_in: usize, index_out: usize) -> f64 {
    data.iter()
        .enumerate()
        .filter(|(index, _x)| index >= &index_in && index <= &index_out)
        .map(|(_i, x)| x.high)
        .max_by(|a, b| a.partial_cmp(b).unwrap())
        .unwrap()
}

pub fn calculate_profit(size: f64, price_in: f64, price_out: f64, trade_type: &TradeType) -> f64 {
    match trade_type.is_long() {
        true => size * (price_out - price_in),
        false => size * (price_in - price_out),
    }
}

pub fn to_pips(pips: f64, pricing: &Pricing) -> f64 {
    pricing.pip_size() * pips
}

pub fn from_pips(pips: f64, pricing: &Pricing) -> f64 {
    pricing.pip_size() / pips
}

pub fn calculate_profit_per(price_in: f64, price_out: f64, trade_type: &TradeType) -> f64 {
    match trade_type.is_long() {
        true => ((price_out - price_in) / price_in) * 100.,
        false => ((price_in - price_out) / price_out) * 100.,
    }
}

pub fn calculate_cum_profit(
    size: f64,
    price_in: f64,
    price_out: f64,
    trade_type: &TradeType,
) -> f64 {
    match trade_type.is_long() {
        true => size * ((price_out - price_in) / price_in),
        false => size * ((price_in - price_out) / price_out),
    }
}

pub fn calculate_cum_profit_per(size: f64, price_in: f64, price_out: f64) -> f64 {
    (size * ((price_out - price_in) / price_in)) * 100.
}

pub fn calculate_runup(
    data: &Vec<Candle>,
    price_in: f64,
    index_in: usize,
    index_out: usize,
    trade_type: &TradeType,
) -> f64 {
    match trade_type.is_long() {
        true => {
            let max_price = get_max_price(data, index_in, index_out);
            (max_price - price_in).abs() * 100.
        }
        false => {
            let min_price = get_min_price(data, index_in, index_out);
            (min_price + price_in).abs() * 100.
        }
    }
}

pub fn calculate_drawdown(
    data: &Vec<Candle>,
    price_in: f64,
    index_in: usize,
    index_out: usize,
    trade_type: &TradeType,
) -> f64 {
    match trade_type.is_long() {
        true => {
            let min_price = get_min_price(data, index_in, index_out);
            (price_in - min_price).abs()
        }
        false => {
            let max_price = get_max_price(data, index_in, index_out);
            (price_in + max_price).abs()
        }
    }
}

pub fn calculate_drawdown_per(draw_down: f64, price_in: f64, _trade_type: &TradeType) -> f64 {
    (draw_down / price_in) * 100.
}

pub fn calculate_runup_per(run_up: f64, price_in: f64, _trade_type: &TradeType) -> f64 {
    (run_up / price_in).abs() * 100.
}

pub fn total_gross(trades_out: &Vec<&TradeOut>) -> f64 {
    trades_out.iter().map(|trade| trade.profit).sum()
}

pub fn avg_per_trade(trades_out: &Vec<&TradeOut>) -> f64 {
    if trades_out.is_empty() {
        0.01
    } else {
        let profit_per_trade: f64 = trades_out.iter().map(|trade| trade.profit_per).sum();
        profit_per_trade / trades_out.len() as f64
    }
}

pub fn total_drawdown(trades_out: &Vec<TradeOut>, equity: f64) -> f64 {
    let mut max_acc_equity = equity;
    let mut equity_curve: Vec<f64> = vec![];

    for trade in trades_out.iter() {
        max_acc_equity += trade.profit;
        equity_curve.push(max_acc_equity);
    }

    let mut min_equity_peak = equity_curve
        .iter()
        .enumerate()
        .filter(|(i, x)| {
            if i > &0 {
                match equity_curve.get(*i - 1) {
                    Some(prev) => match prev < x {
                        true => false,
                        false => true,
                    },
                    None => true,
                }
            } else {
                false
            }
        })
        .map(|(_i, x)| *x)
        .fold(f64::NAN, f64::min);

    let min_equity_index = equity_curve
        .iter()
        .position(|&r| r == min_equity_peak)
        .unwrap_or(0);

    let mut max_equity_peak = equity_curve
        .iter()
        .enumerate()
        .filter(|(i, _x)| i <= &min_equity_index)
        .map(|(_i, x)| *x)
        .fold(f64::NAN, f64::max);

    if min_equity_peak.is_nan() || max_equity_peak.is_nan() {
        max_equity_peak = equity;
        min_equity_peak = equity;
    }

    ((min_equity_peak - max_equity_peak) / max_equity_peak * 100.).abs()
}

pub fn total_runup(trades_out: &Vec<TradeOut>, equity: f64) -> f64 {
    let mut max_acc_equity = equity;
    let mut min_acc_equity = equity;
    let max_equity = trades_out
        .iter()
        .enumerate()
        .map(|(_i, x)| {
            max_acc_equity += x.profit;
            max_acc_equity
        })
        .fold(f64::NEG_INFINITY, f64::max);
    //.fold(0. / 0., f64::max);

    let min_equity = trades_out
        .iter()
        .enumerate()
        .map(|(_i, x)| {
            min_acc_equity += x.profit;
            min_acc_equity
        })
        .fold(f64::NEG_INFINITY, f64::min);
    // .fold(0. / 0., f64::min);

    ((max_equity - min_equity) / min_equity * 100.).abs() * 100.
}

pub fn calculate_buy_hold(bought_at: f64, initial_equity: f64, current_price: f64) -> f64 {
    let size = initial_equity / bought_at;
    let sold_at = size * current_price;
    percentage_change(initial_equity, sold_at)
}

pub fn total_commissions(num_trades: usize, commission: f64) -> f64 {
    num_trades as f64 * commission
}

pub fn total_profitable_trades(winning_trades: usize, total_trades: usize) -> f64 {
    ((winning_trades as f64 / total_trades as f64) * 100.).abs()
}

pub fn total_profit_per(
    equity: f64,
    _net_profit: f64,
    _trades_in: &Vec<TradeIn>,
    trades_out: &Vec<TradeOut>,
) -> f64 {
    let _initial_value = equity;
    trades_out.iter().map(|trade| trade.profit_per).sum()
}

pub fn total_profit_factor(gross_profits: f64, gross_loses: f64) -> f64 {
    match gross_loses {
        0.0 => 0.,
        _ => (gross_profits / gross_loses).abs(),
    }
}

pub fn get_prev_index(index: usize) -> usize {
    match index.cmp(&0) {
        Ordering::Greater => index - 1,
        Ordering::Equal => index,
        Ordering::Less => index,
    }
}

pub fn get_trade_min_price(data: &Vec<Candle>) -> f64 {
    data.iter()
        .map(|x| x.low)
        .min_by(|x, y| x.partial_cmp(y).unwrap())
        .unwrap()
}

pub fn calculate_trade_drawdown(data: &Vec<Candle>, price_in: f64, trade_type: &TradeType) -> f64 {
    match trade_type.is_long() {
        true => {
            let min_price = get_trade_min_price(data);
            (price_in - min_price).abs()
        }
        false => {
            let max_price = get_trade_max_price(data);
            (price_in + max_price).abs()
        }
    }
}

pub fn calculate_trade_runup(data: &Vec<Candle>, price_in: f64, trade_type: &TradeType) -> f64 {
    match trade_type.is_long() {
        true => {
            let max_price = get_trade_max_price(data);
            (max_price - price_in).abs() * 100.
        }
        false => {
            let min_price = get_trade_min_price(data);
            (min_price + price_in).abs() * 100.
        }
    }
}

pub fn calculate_trade_drawdown_per(draw_down: f64, price_in: f64, trade_type: &TradeType) -> f64 {
    calculate_drawdown_per(draw_down, price_in, trade_type)
}

pub fn calculate_trade_runup_per(run_up: f64, price_in: f64, trade_type: &TradeType) -> f64 {
    calculate_runup_per(run_up, price_in, trade_type)
}

pub fn get_trade_max_price(data: &Vec<Candle>) -> f64 {
    data.iter()
        .map(|x| x.high)
        .max_by(|x, y| x.partial_cmp(y).unwrap())
        .unwrap()
}

pub fn calculate_trade_profit(
    size: f64,
    price_in: f64,
    price_out: f64,
    trade_type: &TradeType,
) -> f64 {
    calculate_profit(size, price_in, price_out, trade_type)
}

pub fn calculate_trade_profit_per(price_in: f64, price_out: f64, trade_type: &TradeType) -> f64 {
    calculate_profit_per(price_in, price_out, trade_type)
}

pub fn calculate_quantity(order_size: f64, price: f64) -> f64 {
    round(order_size / price, 3)
}
