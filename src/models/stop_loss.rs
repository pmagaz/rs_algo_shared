use super::order::{Order, OrderCondition, OrderStatus, OrderType};
use super::trade::*;

use crate::helpers::{calc, date::*, uuid};
use crate::indicators::Indicator;
use crate::scanner::instrument::Instrument;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum StopLossType {
    Atr,
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
    entry_type: &TradeType,
    stop_loss_type: &StopLossType,
    spread: f64,
) -> Order {
    let atr_multiplier = std::env::var("ATR_STOP_LOSS")
        .unwrap()
        .parse::<f64>()
        .unwrap();
    let next_index = index + 1;
    let current_price = &instrument.data.get(next_index).unwrap().open();
    let current_date = &instrument.data.get(next_index).unwrap().date();
    let current_atr_value =
        instrument.indicators.atr.get_data_a().get(index).unwrap() * atr_multiplier;

    let origin_price = instrument.data().get(index).unwrap().close();

    let target_price = match stop_loss_type {
        StopLossType::Atr => match entry_type.is_long() {
            true => (current_price + spread) - current_atr_value,
            false => (current_price - spread) + current_atr_value,
        },
        StopLossType::Price(target_price) => match entry_type.is_long() {
            true => target_price + spread,
            false => target_price - spread,
        },
        StopLossType::Pips(pips) => match entry_type.is_long() {
            true => {
                // log::warn!(
                //     "STOOOOOP CALCULATION {} {}",
                //     current_price,
                //     (current_price + spread) - calc::to_pips(pips)
                // );
                (current_price + spread) - calc::to_pips(pips)
            }
            false => (current_price - spread) + calc::to_pips(pips),
        },
        StopLossType::None => todo!(),
    };

    let condition = match entry_type.is_long() {
        true => OrderCondition::Lower,
        false => OrderCondition::Greater,
    };

    Order {
        id: uuid::generate_ts_id(*current_date),
        index_created: next_index,
        index_fulfilled: 0,
        trade_id,
        order_type: OrderType::StopLoss(stop_loss_type.clone()),
        status: OrderStatus::Pending,
        condition,
        origin_price,
        target_price,
        quantity: 100.,
        created_at: to_dbtime(Local::now()),
        updated_at: None,
        full_filled_at: None,
        valid_until: None,
    }
}
