use std::env;
use std::fmt::Display;

use super::mode::{self, ExecutionMode};
use super::order::{Order, OrderType};
use super::tick::InstrumentTick;
use crate::helpers::calc::*;
use crate::helpers::date::*;
use crate::helpers::uuid;
use crate::helpers::{calc, date};
use crate::scanner::candle::Candle;
use crate::scanner::instrument::*;

use serde::{Deserialize, Serialize};

pub trait Trade {
    fn get_id(&self) -> &usize;
    fn get_date(&self) -> &DbDateTime;
    fn get_chrono_date(&self) -> DateTime<Local>;
    fn get_price_in(&self) -> &f64;
    fn get_type(&self) -> &TradeType;
    fn get_price_out(&self) -> &f64;
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TradeDirection {
    Long,
    Short,
    None,
}
impl TradeDirection {
    pub fn is_long(&self) -> bool {
        match *self {
            TradeDirection::Long => true,
            _ => false,
        }
    }

    pub fn is_short(&self) -> bool {
        match *self {
            TradeDirection::Short => true,
            _ => false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TradeType {
    MarketInLong,
    MarketOutLong,
    MarketInShort,
    MarketOutShort,
    OrderInLong,
    OrderOutLong,
    OrderInShort,
    OrderOutShort,
    StopLossLong,
    StopLossShort,
    None,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Position {
    MarketIn(Option<Vec<OrderType>>),
    MarketOut(Option<Vec<OrderType>>),
    MarketInOrder(Order),
    MarketOutOrder(Order),
    Order(Vec<OrderType>),
    None,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PositionResult {
    MarketIn(TradeResult, Option<Vec<Order>>),
    MarketOut(TradeResult),
    PendingOrder(Vec<Order>),
    MarketInOrder(TradeResult, Order),
    MarketOutOrder(TradeResult, Order),
    None,
}

impl TradeType {
    pub fn is_entry(&self) -> bool {
        match *self {
            TradeType::MarketInLong
            | TradeType::MarketInShort
            | TradeType::OrderInLong
            | TradeType::OrderInShort => true,
            _ => false,
        }
    }

    pub fn is_exit(&self) -> bool {
        match *self {
            TradeType::MarketOutLong
            | TradeType::MarketOutShort
            | TradeType::OrderOutLong
            | TradeType::StopLossLong
            | TradeType::StopLossShort
            | TradeType::OrderOutShort => true,
            _ => false,
        }
    }

    pub fn is_long(&self) -> bool {
        match *self {
            TradeType::MarketInLong
            | TradeType::MarketOutLong
            | TradeType::StopLossLong
            | TradeType::OrderInLong
            | TradeType::OrderOutLong => true,
            _ => false,
        }
    }

    pub fn is_long_entry(&self) -> bool {
        match *self {
            TradeType::MarketInLong | TradeType::OrderInLong => true,
            _ => false,
        }
    }

    pub fn is_short(&self) -> bool {
        match *self {
            TradeType::MarketInShort
            | TradeType::MarketOutShort
            | TradeType::OrderInShort
            | TradeType::OrderOutShort => true,
            _ => false,
        }
    }

    pub fn is_short_entry(&self) -> bool {
        match *self {
            TradeType::MarketInShort | TradeType::OrderInShort => true,
            _ => false,
        }
    }

    pub fn is_order(&self) -> bool {
        match *self {
            TradeType::OrderInLong
            | TradeType::OrderOutLong
            | TradeType::OrderInShort
            | TradeType::OrderOutShort
            | TradeType::StopLossLong
            | TradeType::StopLossShort => true,
            _ => false,
        }
    }

    pub fn is_stop(&self) -> bool {
        match *self {
            TradeType::StopLossLong | TradeType::StopLossShort => true,
            _ => false,
        }
    }

    pub fn to_str(&self) -> &str {
        match self {
            TradeType::MarketInLong => "MarketInLong",
            TradeType::MarketOutLong => "MarketOutLong",
            TradeType::MarketInShort => "MarketInShort",
            TradeType::MarketOutShort => "MarketOutShort",
            TradeType::OrderInLong => "OrderInLong",
            TradeType::OrderOutLong => "OrderOutLong",
            TradeType::OrderInShort => "OrderInShort",
            TradeType::OrderOutShort => "OrderOutShort",
            TradeType::StopLossLong => "StopLossLong",
            TradeType::StopLossShort => "StopLossShort",
            TradeType::None => "None",
        }
    }
}

pub fn type_from_str(trade_type: &str) -> TradeType {
    match trade_type {
        "MarketInLong" => TradeType::MarketInLong,
        "MarketOutLong" => TradeType::MarketOutLong,
        "MarketInShort" => TradeType::MarketInShort,
        "MarketOutShort" => TradeType::MarketOutShort,
        "OrderInLong" => TradeType::OrderInLong,
        "OrderOutLong" => TradeType::OrderOutLong,
        "OrderInShort" => TradeType::OrderInShort,
        "OrderOutShort" => TradeType::OrderOutShort,
        "StopLossLong" => TradeType::StopLossLong,
        "StopLossShort" => TradeType::StopLossShort,
        _ => TradeType::None,
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TradeResult {
    TradeIn(TradeIn),
    TradeOut(TradeOut),
    None,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TradeIn {
    pub id: usize,
    pub index_in: usize,
    pub size: f64,
    pub origin_price: f64,
    pub price_in: f64,
    pub ask: f64,
    pub spread: f64,
    pub date_in: DbDateTime,
    pub trade_type: TradeType,
}

impl Default for TradeIn {
    fn default() -> Self {
        TradeIn {
            id: 0,
            index_in: 0,
            size: 0.0,
            origin_price: 0.0,
            price_in: 0.0,
            ask: 0.0,
            spread: 0.0,
            date_in: to_dbtime(Local::now()),
            trade_type: TradeType::MarketInLong,
        }
    }
}

impl Trade for TradeIn {
    fn get_id(&self) -> &usize {
        &self.id
    }
    fn get_date(&self) -> &DbDateTime {
        &self.date_in
    }
    fn get_type(&self) -> &TradeType {
        &self.trade_type
    }
    fn get_chrono_date(&self) -> DateTime<Local> {
        from_dbtime(&self.date_in)
    }
    fn get_price_in(&self) -> &f64 {
        &self.price_in
    }
    fn get_price_out(&self) -> &f64 {
        &self.price_in
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TradeOut {
    pub id: usize,
    pub trade_type: TradeType,
    pub index_in: usize,
    pub price_in: f64,
    pub size: f64,
    pub ask: f64,
    pub spread_in: f64,
    pub date_in: DbDateTime,
    pub index_out: usize,
    pub price_origin: f64,
    pub price_out: f64,
    pub bid: f64,
    pub spread_out: f64,
    pub date_out: DbDateTime,
    pub profit: f64,
    pub profit_per: f64,
    pub run_up: f64,
    pub run_up_per: f64,
    pub draw_down: f64,
    pub draw_down_per: f64,
}

impl Trade for TradeOut {
    fn get_id(&self) -> &usize {
        &self.id
    }
    fn get_date(&self) -> &DbDateTime {
        &self.date_out
    }
    fn get_type(&self) -> &TradeType {
        &self.trade_type
    }
    fn get_chrono_date(&self) -> DateTime<Local> {
        from_dbtime(&self.date_out)
    }
    fn get_price_in(&self) -> &f64 {
        &self.price_in
    }
    fn get_price_out(&self) -> &f64 {
        &self.price_out
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

pub fn resolve_trade_in(
    index: usize,
    trade_size: f64,
    instrument: &Instrument,
    trade_type: &TradeType,
    order: Option<&Order>,
    tick: &InstrumentTick,
) -> TradeResult {
    let execution_mode = mode::from_str(&env::var("EXECUTION_MODE").unwrap());
    let order_engine = &env::var("ORDER_ENGINE").unwrap();
    let activation_source = &env::var("ORDER_ACTIVATION_SOURCE").unwrap();
    let index = calculate_trade_index(index, order, &execution_mode);
    let size = trade_size;

    if trade_type.is_entry() {
        let spread = tick.spread();

        let current_candle = match execution_mode.is_back_test() {
            true => instrument.data().get(index).unwrap(),
            false => instrument.data().last().unwrap(),
        };

        let current_date = current_candle.date();
        let id = uuid::generate_ts_id(current_date);

        let price = match order_engine.as_ref() {
            "broker" => match order {
                Some(order) => order.target_price,
                None => current_candle.open(),
            },
            "bot" | _ => match order {
                Some(order) => match activation_source.as_ref() {
                    "close" => current_candle.open(),
                    "highs_lows" | _ => order.target_price,
                },
                None => current_candle.open(),
            },
        };

        let ask = match trade_type.is_long() {
            true => price + spread,
            false => price,
        };

        let price_in = match trade_type.is_long() {
            true => ask,
            false => price,
        };

        let index_in = match execution_mode.is_back_test() {
            true => index,
            false => id,
        };

        TradeResult::TradeIn(TradeIn {
            id,
            index_in,
            origin_price: price,
            price_in,
            ask,
            spread,
            size,
            date_in: to_dbtime(current_date),
            trade_type: trade_type.clone(),
        })
    } else {
        TradeResult::None
    }
}

pub fn resolve_trade_out(
    index: usize,
    instrument: &Instrument,
    trade_in: &TradeIn,
    trade_type: &TradeType,
    order: Option<&Order>,
    tick: &InstrumentTick,
) -> TradeResult {
    let _size = trade_in.size;
    let data = &instrument.data;
    let spread = tick.spread();
    let trade_in_type = &trade_in.trade_type;
    let index_in = trade_in.index_in;
    let spread_in = trade_in.spread;
    let execution_mode = mode::from_str(&env::var("EXECUTION_MODE").unwrap());
    let non_profitable_outs = &env::var("NON_PROFITABLE_OUTS")
        .unwrap()
        .parse::<bool>()
        .unwrap();
    let order_engine = &env::var("ORDER_ENGINE").unwrap();

    let leverage = env::var("LEVERAGE").unwrap().parse::<f64>().unwrap();
    let size = trade_in.size;

    let index = calculate_trade_index(index, order, &execution_mode);
    let current_candle = instrument.data.get(index).unwrap();
    let current_date = current_candle.date();
    let price_origin = *trade_in.get_price_in();

    let close_trade_price = match trade_type {
        TradeType::StopLossLong | TradeType::StopLossShort => order.unwrap().target_price,
        _ => current_candle.open(),
    };

    let price_out = match order_engine.as_ref() {
        "broker" => match order {
            Some(order) => order.target_price,
            None => close_trade_price,
        },
        _ => close_trade_price,
    };

    let (price_in, price_out) = match execution_mode.is_back_test() {
        true => match trade_in_type.is_long() {
            true => (trade_in.price_in, price_out),
            false => (trade_in.price_in, price_out + spread),
        },
        false => (trade_in.price_in, price_out),
    };

    let bid = match trade_type.is_long() {
        true => price_out + spread,
        false => price_out,
    };
    let index_out = index;

    let profit = match trade_in_type.is_long() {
        true => price_out - price_in,
        false => price_in - price_out,
    };

    let is_profitable = match profit {
        _ if profit > 0. => true,
        _ => false,
    };

    let date_out = to_dbtime(current_candle.date());

    if trade_type.is_stop() && profit > 0. {
        log::error!(
            "Profitable stop loss! {} @ {:?} {} ",
            index,
            (price_in, price_out),
            profit
        );
        //panic!();
    }

    // let profit_check = match non_profitable_outs {
    //     true => true || trade_type.is_stop(),
    //     false => is_profitable || trade_type.is_stop(),
    // };

    let profit_check = if *non_profitable_outs {
        true
    } else {
        is_profitable || trade_type.is_stop()
    };

    if profit_check {
        let date_in = match execution_mode.is_back_test() {
            true => to_dbtime(instrument.data.get(index_in).unwrap().date()),
            false => to_dbtime(current_date),
        };

        let profit = match execution_mode.is_back_test() {
            true => calc::calculate_profit(size, price_in, price_out, leverage, trade_in_type),
            false => 0.,
        };

        let profit_per = match execution_mode.is_back_test() {
            true => calc::calculate_profit_per(profit, size, price_in, leverage),
            false => 0.,
        };

        let run_up = match execution_mode.is_back_test() {
            true => calc::calculate_runup(data, price_in, index_in, index, leverage, trade_in_type),
            false => 0.,
        };

        let run_up_per = match execution_mode.is_back_test() {
            true => calc::calculate_runup_per(run_up, price_in, trade_in_type),
            false => 0.,
        };

        let draw_down = match execution_mode.is_back_test() {
            true => calc::calculate_drawdown(data, price_in, index_in, index, trade_in_type),
            false => 0.,
        };

        let draw_down_per = match execution_mode.is_back_test() {
            true => calc::calculate_drawdown_per(draw_down, price_in, trade_in_type),
            false => 0.,
        };

        TradeResult::TradeOut(TradeOut {
            id: trade_in.id,
            index_in,
            price_in,
            size,
            trade_type: trade_type.clone(),
            date_in,
            spread_in,
            ask: price_in,
            index_out,
            price_origin,
            price_out,
            bid,
            spread_out: spread,
            date_out,
            profit,
            profit_per,
            run_up,
            run_up_per,
            draw_down,
            draw_down_per,
        })
    } else {
        log::warn!("Non profitable {:?} exit", trade_type);
        TradeResult::None
    }
}

pub fn wait_for_new_trade(
    index: usize,
    instrument: &Instrument,
    trades_out: &Vec<TradeOut>,
) -> bool {
    let wait_for_new_entry = env::var("WAIT_FOR_NEW_ENTRY")
        .unwrap()
        .parse::<bool>()
        .unwrap();

    match wait_for_new_entry {
        true => {
            let execution_mode = mode::from_str(&env::var("EXECUTION_MODE").unwrap());

            let candles_until_new_operation = env::var("CANDLES_UNTIL_NEW_ENTRY")
                .unwrap()
                .parse::<i64>()
                .unwrap();

            let time_frame = instrument.time_frame();

            let current_date = match execution_mode.is_back_test() {
                true => instrument.data().get(index).unwrap().date(),
                false => Local::now(),
            };

            match trades_out.last() {
                Some(trade_out) => {
                    let next_entry_date = match instrument.time_frame().is_minutely_time_frame() {
                        true => {
                            date::from_dbtime(&trade_out.date_out)
                                + date::Duration::minutes(
                                    candles_until_new_operation * time_frame.to_minutes(),
                                )
                        }
                        false => {
                            date::from_dbtime(&trade_out.date_out)
                                + date::Duration::hours(
                                    candles_until_new_operation * time_frame.to_hours(),
                                )
                        }
                    };

                    current_date < next_entry_date
                }
                None => false,
            }
        }
        false => false,
    }
}

pub fn wait_for_closing_trade(index: usize, instrument: &Instrument, trade_in: &TradeIn) -> bool {
    let wait_for_new_exit = env::var("WAIT_FOR_NEW_EXIT")
        .unwrap()
        .parse::<bool>()
        .unwrap();

    match wait_for_new_exit {
        true => {
            let execution_mode = mode::from_str(&env::var("EXECUTION_MODE").unwrap());

            let candles_until_new_operation = env::var("CANDLES_UNTIL_NEW_ENTRY")
                .unwrap()
                .parse::<i64>()
                .unwrap();

            let time_frame = instrument.time_frame();

            let current_date = match execution_mode.is_back_test() {
                true => instrument.data().get(index).unwrap().date(),
                false => Local::now(),
            };

            let next_entry_date = match instrument.time_frame().is_minutely_time_frame() {
                true => {
                    date::from_dbtime(&trade_in.date_in)
                        + date::Duration::minutes(
                            candles_until_new_operation * time_frame.to_minutes(),
                        )
                }
                false => {
                    date::from_dbtime(&trade_in.date_in)
                        + date::Duration::hours(candles_until_new_operation * time_frame.to_hours())
                }
            };
            next_entry_date <= current_date
        }
        false => true,
    }
}

pub fn calculate_trade_index(
    index: usize,
    _order: Option<&Order>,
    execution_mode: &ExecutionMode,
) -> usize {
    match execution_mode.is_back_test() {
        true => index + 1,
        _ => index,
    }
}

pub fn calculate_trade_stats(
    trade_in: &TradeIn,
    trade_out: &TradeOut,
    data: &Vec<Candle>,
) -> TradeOut {
    let execution_mode = mode::from_str(&env::var("EXECUTION_MODE").unwrap());

    let leverage = env::var("LEVERAGE").unwrap().parse::<f64>().unwrap();

    let _trade_type = &trade_in.trade_type;
    let _date_out = match execution_mode {
        mode::ExecutionMode::Bot => trade_out.date_out,
        _ => to_dbtime(data.last().unwrap().date()),
    };

    let trade_type = &trade_in.trade_type;
    let price_in = trade_in.price_in;
    let price_out = trade_out.price_out;
    let size = trade_in.size;

    let quantity = calculate_quantity(size, price_in);
    let profit = calculate_trade_profit(quantity, price_in, price_out, leverage, trade_type);
    let profit_per = calculate_trade_profit_per(profit, size, price_in, leverage);

    let run_up = calculate_trade_runup(data, price_in, trade_type);
    let run_up_per = calculate_trade_runup_per(run_up, price_in, trade_type);
    let draw_down = calculate_trade_drawdown(data, price_in, trade_type);
    let draw_down_per = calculate_trade_drawdown_per(draw_down, price_in, trade_type);

    TradeOut {
        id: trade_out.id,
        index_in: trade_in.index_in,
        price_in: trade_in.price_in,
        size: trade_in.size,
        ask: trade_in.ask,
        spread_in: trade_in.spread,
        trade_type: trade_out.trade_type.clone(),
        date_in: trade_in.date_in,
        index_out: trade_out.index_out,
        price_origin: trade_out.price_origin,
        price_out: trade_out.price_out,
        bid: trade_out.bid,
        spread_out: trade_in.spread,
        date_out: trade_out.date_out,
        profit,
        profit_per,
        run_up,
        run_up_per,
        draw_down,
        draw_down_per,
    }
}

pub fn trade_exists<T: Trade>(trades: &[T], search_id: usize) -> bool {
    trades.iter().any(|order| order.get_id() == &search_id)
}

pub fn update_trades<T>(trades: &mut Vec<T>, new_trade: T) -> bool
where
    T: Trade + Clone,
{
    let mut updated = false;

    if let Some(trade) = trades
        .iter_mut()
        .find(|trade| trade.get_id() == new_trade.get_id())
    {
        *trade = new_trade;
        updated = true;
    }

    updated
}
