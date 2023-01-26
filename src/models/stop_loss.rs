use super::order::{self, Order, OrderCondition, OrderDirection, OrderStatus, OrderType};
use super::pricing::Pricing;
use super::trade::*;

use crate::helpers::{calc, date::*, uuid};
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
    instrument: &Instrument,
    pricing: &Pricing,
    trade_type: &TradeType,
    order_direction: &OrderDirection,
    stop_loss_type: &StopLossType,
    target_price: f64,
) -> Order {
    let spread = pricing.spread();

    let stop_loss_spread = std::env::var("STOP_LOSS_SPREAD")
        .unwrap()
        .parse::<bool>()
        .unwrap();

    let atr_multiplier = std::env::var("ATR_STOP_LOSS")
        .unwrap()
        .parse::<f64>()
        .unwrap();

    let spread = match stop_loss_spread {
        true => spread,
        false => 0.,
    };

    let next_index = index + 1;
    let current_atr_value =
        instrument.indicators.atr.get_data_a().get(index).unwrap() * atr_multiplier;

    let target_price = match stop_loss_type {
        StopLossType::Atr(atr_value) => match order_direction {
            OrderDirection::Up => (target_price + spread) + (atr_value * current_atr_value),
            OrderDirection::Down => (target_price + spread) - (atr_value * current_atr_value),
        },
        StopLossType::Price(target_price) => match order_direction {
            OrderDirection::Up => *target_price,
            OrderDirection::Down => *target_price,
        },
        StopLossType::Pips(pips) => match order_direction {
            OrderDirection::Up => (target_price + spread) + calc::to_pips(*pips, pricing),
            OrderDirection::Down => (target_price + spread) - calc::to_pips(*pips, pricing),
        },
        StopLossType::None => todo!(),
    };

    order::create_order(
        index,
        trade_id,
        instrument,
        trade_type,
        &OrderType::StopLoss(order_direction.clone(), stop_loss_type.clone()),
        &target_price,
        &100.,
    )
}
