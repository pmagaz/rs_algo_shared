use super::order::{Order, OrderCondition, OrderType};
use crate::helpers::calc::*;
use crate::helpers::date::*;
use crate::helpers::uuid;
use crate::models::stop_loss::*;
use crate::scanner::instrument::*;

use round::round;
use serde::{Deserialize, Serialize};

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
pub enum Operation {
    MarketIn(Option<Vec<OrderType>>),
    MarketOut(Option<Vec<OrderType>>),
    MarketInOrder(Order),
    MarketOutOrder(Order),
    Order(Vec<OrderType>),
    None,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OperationResult {
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
    pub price_in: f64,
    pub ask: f64,
    pub spread: f64,
    //pub stop_loss: StopLoss,
    pub date_in: DbDateTime,
    pub trade_type: TradeType,
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
    entry_type: &TradeType,
    spread: f64,
    //stop_loss: &StopLoss,
) -> TradeResult {
    if entry_type.is_entry() {
        let nex_candle_index = index + 1;
        let next_day_candle = instrument.data.get(nex_candle_index);
        let next_day_price = match next_day_candle {
            Some(candle) => candle.open,
            None => -100.,
        };
        let current_date = next_day_candle.unwrap().date;

        let ask = next_day_price + spread;

        let quantity = round(trade_size / next_day_price, 3);

        TradeResult::TradeIn(TradeIn {
            id: uuid::generate_ts_id(current_date),
            index_in: nex_candle_index,
            price_in: next_day_price,
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
    trade_in: &TradeIn,
    exit_type: &TradeType,
) -> TradeResult {
    let quantity = trade_in.quantity;
    let data = &instrument.data;
    let nex_candle_index = index + 1;
    let index_in = trade_in.index_in;

    let current_candle = data.get(nex_candle_index);
    let current_date = current_candle.unwrap().date;
    let ask = trade_in.ask;
    let bid = match current_candle {
        Some(candle) => candle.open,
        None => -100.,
    };

    let profit = bid - ask;

    let is_profitable = match profit {
        _ if profit > 0. => true,
        _ => false,
    };

    if index > trade_in.index_in
        && ((exit_type.is_exit() && is_profitable) || exit_type == &TradeType::StopLoss)
    {
        let date_in = instrument.data.get(index_in).unwrap().date;
        let date_out = current_candle.unwrap().date;
        let profit = calculate_profit(quantity, ask, bid);
        let profit_per = calculate_profit_per(ask, bid);
        let run_up = calculate_runup(data, ask, index_in, nex_candle_index);
        let run_up_per = calculate_runup_per(run_up, ask);
        let draw_down = calculate_drawdown(data, ask, index_in, nex_candle_index);
        let draw_down_per = calculate_drawdown_per(draw_down, ask);

        // let trade_type = match stop_loss_activated {
        //     true => TradeType::StopLoss,
        //     false => exit_type,
        // };
        TradeResult::TradeOut(TradeOut {
            id: uuid::generate_ts_id(current_date),
            index_in,
            price_in: ask,
            trade_type: exit_type.clone(),
            date_in: to_dbtime(date_in),
            spread_in: trade_in.spread,
            ask: ask,
            index_out: nex_candle_index,
            price_out: bid,
            bid: bid,
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
    let bid = candle.close();
    let id = uuid::generate_ts_id(date_out);

    let stop_loss_price = match exit_type.is_long() {
        true => candle.low,
        false => candle.high,
    };

    let profit = bid - ask;

    let is_profitable = match profit {
        _ if profit > 0. => true,
        _ => false,
    };

    //let stop_loss_activated = resolve_stop_loss(stop_loss_price, &trade_in);

    if is_profitable && exit_type.is_exit()
    //|| stop_loss_activated
    {
        log::info!("Executing {:?}", exit_type);

        // let trade_type = match stop_loss_activated {
        //     true => {
        //         log::info!("Stop loss activated");
        //         TradeType::StopLoss
        //     }
        //     false => exit_type,
        // };

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
            price_out: candle.close(),
            bid: stop_loss_price,
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
