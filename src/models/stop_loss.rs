use std::env;

use super::mode;
use super::order::{self, Order, OrderDirection, OrderType};
use super::tick::InstrumentTick;

use crate::helpers::{calc, date::*};
use crate::indicators::Indicator;
use crate::scanner::instrument::Instrument;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum StopLossType {
    Atr(f64),
    Price(f64),
    Pips(f64),
    None,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct StopLoss {
    pub stop_type: StopLossType,
    pub price: f64,
    pub value: f64,
    pub created_at: DbDateTime,
    pub updated_at: DbDateTime,
    pub valid_until: DbDateTime,
}

pub fn init_stop_loss(stop_type: StopLossType, value: f64) -> StopLoss {
    StopLoss {
        price: 0.,
        value,
        stop_type,
        created_at: to_dbtime(Local::now()),
        updated_at: to_dbtime(Local::now()),
        valid_until: to_dbtime(Local::now() + Duration::days(1000)),
    }
}

pub fn create_stop_loss_order(
    index: usize,
    trade_id: usize,
    buy_price: f64,
    instrument: &Instrument,
    order_direction: &OrderDirection,
    stop_loss_type: &StopLossType,
    tick: &InstrumentTick,
) -> Order {
    let order_size = std::env::var("ORDER_SIZE").unwrap().parse::<f64>().unwrap();

    let stop_loss_spread = std::env::var("STOP_LOSS_SPREAD")
        .unwrap()
        .parse::<bool>()
        .unwrap();

    let spread = tick.spread();

    let spread_value = match stop_loss_spread {
        true => spread,
        false => 0.,
    };

    let current_atr_value = instrument.indicators.atr.get_data_a().get(index).unwrap();

    let target_price = match stop_loss_type {
        StopLossType::Atr(atr_stop_value) => match order_direction {
            OrderDirection::Up => (buy_price + spread_value) + (atr_stop_value * current_atr_value),
            OrderDirection::Down => {
                (buy_price - spread_value) - (atr_stop_value * current_atr_value)
            }
        },
        StopLossType::Price(target_price) => match order_direction {
            OrderDirection::Up => *target_price,
            OrderDirection::Down => *target_price,
        },
        StopLossType::Pips(pips) => match order_direction {
            OrderDirection::Up => (buy_price + spread_value) + calc::to_pips(*pips, tick),
            OrderDirection::Down => (buy_price - spread_value) - calc::to_pips(*pips, tick),
        },
        StopLossType::None => todo!(),
    };

    let stop_loss = match order_direction {
        OrderDirection::Up => {
            OrderType::StopLossShort(order_direction.clone(), buy_price, stop_loss_type.clone())
        }
        OrderDirection::Down => {
            OrderType::StopLossLong(order_direction.clone(), buy_price, stop_loss_type.clone())
        }
    };

    order::create_order(
        index,
        trade_id,
        instrument,
        &stop_loss,
        &target_price,
        &order_size,
    )
}
