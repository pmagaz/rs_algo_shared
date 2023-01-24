use std::env::current_exe;

use super::order::{Order, OrderCondition, OrderType};
use super::pricing::Pricing;
use crate::helpers::calc::*;
use crate::helpers::date::*;
use crate::helpers::uuid;
use crate::models::stop_loss::*;
use crate::scanner::instrument::*;

use round::round;
use serde::{Deserialize, Serialize};

pub trait Trade {
    fn get_date(&self) -> &DbDateTime;
    fn get_chrono_date(&self) -> DateTime<Local>;
    fn get_price_in(&self) -> &f64;
    fn get_price_out(&self) -> &f64;
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TradeDirection {
    Long,
    Short,
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
    StopLoss,
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
            | TradeType::StopLoss
            | TradeType::OrderOutShort => true,
            _ => false,
        }
    }

    pub fn is_long(&self) -> bool {
        match *self {
            TradeType::MarketInLong
            | TradeType::MarketOutLong
            | TradeType::OrderInLong
            | TradeType::OrderOutLong => true,
            _ => false,
        }
    }

    pub fn is_stop(&self) -> bool {
        match *self {
            TradeType::StopLoss => true,
            _ => false,
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
        "StopLoss" => TradeType::StopLoss,
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
    pub quantity: f64,
    pub origin_price: f64,
    pub price_in: f64,
    pub ask: f64,
    pub spread: f64,
    pub date_in: DbDateTime,
    pub trade_type: TradeType,
}

impl Trade for TradeIn {
    fn get_date(&self) -> &DbDateTime {
        &self.date_in
    }
    fn get_chrono_date(&self) -> DateTime<Local> {
        fom_dbtime(&self.date_in)
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
    fn get_date(&self) -> &DbDateTime {
        &self.date_out
    }
    fn get_chrono_date(&self) -> DateTime<Local> {
        fom_dbtime(&self.date_out)
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
    pricing: &Pricing,
    entry_type: &TradeType,
) -> TradeResult {
    let spread = pricing.spread();

    if entry_type.is_entry() {
        let nex_candle_index = index + 1;
        let next_day_candle = instrument.data.get(nex_candle_index);
        let next_candle_price = match next_day_candle {
            Some(candle) => candle.open(),
            None => -100.,
        };

        let current_date = next_day_candle.unwrap().date;

        let ask = match entry_type.is_long() {
            true => next_candle_price + spread,
            false => next_candle_price,
        };

        let price_in = match entry_type.is_long() {
            true => ask,
            false => next_candle_price,
        };

        let quantity = round(trade_size / next_candle_price, 3);
        log::info!(
            "TRADEIN PREPARING {} @@@ {:?} ",
            index,
            (
                next_candle_price,
                instrument.data.get(index).unwrap().close()
            )
        );
        TradeResult::TradeIn(TradeIn {
            id: uuid::generate_ts_id(current_date),
            index_in: nex_candle_index,
            origin_price: next_candle_price,
            price_in,
            ask,
            spread,
            quantity,
            date_in: to_dbtime(current_date),
            trade_type: entry_type.clone(),
        })
    } else {
        TradeResult::None
    }
}

pub fn resolve_trade_out(
    index: usize,
    instrument: &Instrument,
    pricing: &Pricing,
    trade_in: &TradeIn,
    trade_type: &TradeType,
) -> TradeResult {
    let quantity = trade_in.quantity;
    let data = &instrument.data;
    let nex_candle_index = index + 1;
    let index_in = trade_in.index_in;
    let spread = pricing.spread();
    let trade_in_type = &trade_in.trade_type;

    //Stop loss resolved same day
    let current_candle = match trade_type {
        TradeType::StopLoss => data.get(index).unwrap(),
        _ => data.get(nex_candle_index).unwrap(),
    };

    let close_price = match trade_type {
        TradeType::StopLoss => match trade_in_type.is_long() {
            true => current_candle.low(),
            false => current_candle.high(),
        },
        _ => current_candle.open(),
    };

    let current_date = current_candle.date();
    let price_origin = close_price;
    let (price_in, price_out) = match trade_in_type.is_long() {
        true => (trade_in.price_in, close_price),
        false => (trade_in.price_in, close_price + spread),
    };

    let profit = match trade_in_type.is_long() {
        true => price_out - price_in,
        false => price_in - price_out,
    };

    let is_profitable = match profit {
        _ if profit > 0. => true,
        _ => false,
    };

    log::info!(
        "111111111111111111 {:?}",
        (
            trade_type,
            close_price,
            price_in,
            price_out,
            profit,
            is_profitable
        )
    );
    if trade_type == &TradeType::StopLoss && profit > 0. {
        panic!(
            "[PANIC] Profitable stop loss! {} @ {:?} {} ",
            index,
            (price_in, price_out),
            profit
        )
    }

    if index > trade_in.index_in
        && ((is_profitable && trade_type.is_exit()) || trade_type == &TradeType::StopLoss)
    {
        let date_in = instrument.data.get(index_in).unwrap().date();
        let date_out = current_candle.date();
        let profit = calculate_profit(quantity, price_in, price_out, trade_in_type);
        let profit_per = calculate_profit_per(price_in, price_out, trade_in_type);
        let run_up = calculate_runup(data, price_in, index_in, nex_candle_index, trade_in_type);
        let run_up_per = calculate_runup_per(run_up, price_in, trade_in_type);
        let draw_down =
            calculate_drawdown(data, price_in, index_in, nex_candle_index, trade_in_type);
        let draw_down_per = calculate_drawdown_per(draw_down, price_in, trade_in_type);

        TradeResult::TradeOut(TradeOut {
            id: uuid::generate_ts_id(current_date),
            index_in,
            price_in,
            trade_type: trade_type.clone(),
            date_in: to_dbtime(date_in),
            spread_in: trade_in.spread,
            ask: price_in,
            index_out: nex_candle_index,
            price_origin,
            price_out: price_out,
            bid: price_out,
            spread_out: trade_in.spread,
            date_out: to_dbtime(date_out),
            profit,
            profit_per,
            run_up,
            run_up_per,
            draw_down,
            draw_down_per,
        })
    } else {
        TradeResult::None
    }
}

pub fn resolve_bot_trade_in(
    trade_size: f64,
    instrument: &Instrument,
    entry_type: TradeType,
    stop_loss: &StopLoss,
) -> TradeResult {
    if entry_type.is_entry() {
        let candle = instrument.data.last().unwrap();
        let current_date = candle.date();
        let close_price = candle.close();
        let quantity = round(trade_size / close_price, 3);
        let id = uuid::generate_ts_id(current_date);

        TradeResult::TradeIn(TradeIn {
            id,
            index_in: id,
            origin_price: close_price,
            price_in: close_price,
            spread: close_price,
            ask: close_price,
            quantity,
            //stop_loss: create_bot_stop_loss(&entry_type, instrument, index, stop_loss),
            date_in: to_dbtime(current_date),
            trade_type: entry_type,
        })
    } else {
        TradeResult::None
    }
}

pub fn resolve_bot_trade_out(
    instrument: &Instrument,
    trade_in: TradeIn,
    exit_type: TradeType,
    spread: f64,
) -> TradeResult {
    let data = &instrument.data;

    let _quantity = trade_in.quantity;
    let index_in = trade_in.index_in;
    let price_in = trade_in.price_in;

    let spread_in = trade_in.spread;
    let ask = trade_in.ask;
    let date_in = trade_in.date_in;
    let candle = data.last().unwrap();
    let date_out = candle.date();
    let close_price = candle.close();
    let id = uuid::generate_ts_id(date_out);
    let price_origin = close_price;

    let ask = trade_in.ask;
    let price_origin = close_price;
    let price_in = trade_in.price_in;

    let bid = match trade_in.trade_type.is_long() {
        true => close_price,
        false => close_price - spread,
    };

    let price_out = match trade_in.trade_type.is_long() {
        true => bid,
        false => close_price,
    };

    let profit = price_out - price_in;
    let is_profitable = match profit {
        _ if profit > 0. => true,
        _ => false,
    };

    if is_profitable && exit_type.is_exit() {
        log::info!("Executing {:?}", exit_type);

        let trade_type = exit_type;

        TradeResult::TradeOut(TradeOut {
            id,
            index_in,
            price_in,
            ask,
            spread_in,
            trade_type,
            date_in,
            index_out: id,
            price_origin,
            price_out,
            bid,
            spread_out: spread_in,
            date_out: to_dbtime(date_out),
            profit: 0.,
            profit_per: 0.,
            run_up: 0.,
            run_up_per: 0.,
            draw_down: 0.,
            draw_down_per: 0.,
        })
    } else {
        TradeResult::None
    }
}
