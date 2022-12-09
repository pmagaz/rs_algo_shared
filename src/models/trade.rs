use crate::helpers::calc::*;
use crate::helpers::date::*;
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
    EntryLong,
    ExitLong,
    EntryShort,
    ExitShort,
    StopLoss,
    TakeProfit,
    None,
}

impl TradeType {
    pub fn is_entry(&self) -> bool {
        match *self {
            TradeType::EntryLong => true,
            TradeType::EntryShort => true,
            _ => false,
        }
    }

    pub fn is_exit(&self, trade_type: &TradeType) -> bool {
        match trade_type {
            TradeType::ExitLong => true,
            TradeType::ExitShort => true,
            _ => false,
        }
    }
}

pub fn type_from_str(trade_type: &str) -> TradeType {
    match trade_type {
        "EntryLong" => TradeType::EntryLong,
        "ExitLong" => TradeType::ExitLong,
        "EntryShort" => TradeType::EntryShort,
        "ExitShort" => TradeType::ExitShort,
        "StopLoss" => TradeType::StopLoss,
        "TakeProfit" => TradeType::TakeProfit,
        _ => TradeType::None,
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TradeIn {
    pub id: usize,
    pub index_in: usize,
    pub quantity: f64,
    pub price_in: f64,
    pub ask: f64,
    pub spread: f64,
    pub stop_loss: StopLoss,
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

#[derive(Debug)]
pub enum TradeResult {
    TradeIn(TradeIn),
    TradeOut(TradeOut),
    None,
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
    order_size: f64,
    instrument: &Instrument,
    entry_type: TradeType,
    stop_loss: &StopLoss,
) -> TradeResult {
    if entry_type == TradeType::EntryLong || entry_type == TradeType::EntryShort {
        let nex_candle_index = index + 1;
        let next_day_candle = instrument.data.get(nex_candle_index);
        let next_day_price = match next_day_candle {
            Some(candle) => candle.open,
            None => -100.,
        };
        let current_date = next_day_candle.unwrap().date;

        let quantity = round(order_size / next_day_price, 3);

        TradeResult::TradeIn(TradeIn {
            id: 0,
            index_in: nex_candle_index,
            price_in: next_day_price,
            ask: next_day_price,
            spread: next_day_price,
            quantity,
            stop_loss: create_stop_loss(&entry_type, instrument, nex_candle_index, stop_loss),
            date_in: to_dbtime(current_date),
            trade_type: entry_type,
        })
    } else {
        TradeResult::None
    }
}

pub fn resolve_trade_out(
    index: usize,
    instrument: &Instrument,
    trade_in: TradeIn,
    exit_type: TradeType,
) -> TradeResult {
    let quantity = trade_in.quantity;
    let data = &instrument.data;
    let nex_candle_index = index + 1;
    let index_in = trade_in.index_in;
    let price_in = trade_in.price_in;
    let current_candle = data.get(nex_candle_index);
    let current_price = match current_candle {
        Some(candle) => candle.open,
        None => -100.,
    };

    let date_in = instrument.data.get(index_in).unwrap().date;
    let date_out = current_candle.unwrap().date;
    let profit = calculate_profit(quantity, price_in, current_price);
    let profit_per = calculate_profit_per(price_in, current_price);
    let run_up = calculate_runup(data, price_in, index_in, nex_candle_index);
    let run_up_per = calculate_runup_per(run_up, price_in);
    let draw_down = calculate_drawdown(data, price_in, index_in, nex_candle_index);
    let draw_down_per = calculate_drawdown_per(draw_down, price_in);

    let stop_loss_activated = resolve_stop_loss(current_price, &trade_in);

    if index > trade_in.index_in
        && (exit_type == TradeType::ExitLong
            || exit_type == TradeType::ExitShort
            || stop_loss_activated)
    {
        log::info!("Executing tradeOut");

        let trade_type = match stop_loss_activated {
            true => TradeType::StopLoss,
            false => exit_type,
        };

        TradeResult::TradeOut(TradeOut {
            id: 0,
            index_in,
            price_in,
            trade_type,
            date_in: to_dbtime(date_in),
            spread_in: current_price,
            ask: current_price,
            index_out: nex_candle_index,
            price_out: current_price,
            bid: current_price,
            spread_out: current_price,
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
    index: usize,
    order_size: f64,
    instrument: &Instrument,
    entry_type: TradeType,
    stop_loss: &StopLoss,
) -> TradeResult {
    if entry_type == TradeType::EntryLong || entry_type == TradeType::EntryShort {
        let candle = instrument.data.last().unwrap();
        let current_date = candle.date();
        let close_price = candle.close();
        let quantity = round(order_size / close_price, 3);

        TradeResult::TradeIn(TradeIn {
            id: 0,
            index_in: index,
            price_in: close_price,
            spread: close_price,
            ask: close_price,
            quantity: quantity,
            stop_loss: create_bot_stop_loss(&entry_type, instrument, index, stop_loss),
            date_in: to_dbtime(current_date),
            trade_type: entry_type,
        })
    } else {
        TradeResult::None
    }
}

pub fn resolve_bot_trade_out(
    index: usize,
    instrument: &Instrument,
    trade_in: TradeIn,
    exit_type: TradeType,
) -> TradeResult {
    let data = &instrument.data;

    let quantity = trade_in.quantity;
    let index_in = trade_in.index_in;
    let price_in = trade_in.price_in;
    let spread_in = trade_in.spread;
    let ask = trade_in.ask;
    let date_in = trade_in.date_in;

    let candle = data.last().unwrap();
    let date_out = candle.date();
    let stop_loss_price = match &exit_type {
        TradeType::ExitLong => candle.low,
        TradeType::ExitShort => candle.high,
        _ => candle.high,
    };

    let stop_loss_activated = resolve_stop_loss(stop_loss_price, &trade_in);

    if exit_type == TradeType::ExitLong || exit_type == TradeType::ExitShort || stop_loss_activated
    {
        log::info!("Executing {:?}", exit_type);

        let trade_type = match stop_loss_activated {
            true => {
                log::info!("Stop loss activated");
                TradeType::StopLoss
            }
            false => exit_type,
        };

        TradeResult::TradeOut(TradeOut {
            id: 0,
            index_in,
            price_in,
            ask,
            spread_in,
            trade_type,
            date_in: date_in,
            index_out: index,
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
