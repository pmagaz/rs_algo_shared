use crate::helpers::date::*;
use crate::indicators::Indicator;
use crate::models::market::*;
use crate::models::strategy::*;
use crate::models::trade::*;
use crate::scanner::instrument::Instrument;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum StopLossType {
    Atr,
    Price,
    Percentage,
    Trailing,
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

pub fn create_stop_loss(
    entry_type: &TradeType,
    instrument: &Instrument,
    index: usize,
    stop_loss: &StopLoss,
) -> StopLoss {
    let current_price = &instrument.data.get(index).unwrap().open;
    let stop_loss_value = stop_loss.value;
    let stop_loss_price = stop_loss.price;
    let atr_value = instrument.indicators.atr.get_data_a().get(index).unwrap() * stop_loss_value;

    let price = match stop_loss.stop_type {
        StopLossType::Atr => {
            let price = match entry_type {
                TradeType::EntryLong => current_price - atr_value,
                TradeType::EntryShort => current_price + atr_value,
                _ => current_price - atr_value,
            };
            price
        }
        _ => stop_loss_price,
    };

    StopLoss {
        price,
        value: atr_value,
        stop_type: stop_loss.stop_type.to_owned(),
        created_at: to_dbtime(Local::now()),
        updated_at: to_dbtime(Local::now()),
        valid_until: to_dbtime(Local::now() + Duration::days(1000)),
    }
}

pub fn create_bot_stop_loss(
    entry_type: &TradeType,
    instrument: &Instrument,
    index: usize,
    stop_loss: &StopLoss,
) -> StopLoss {
    let current_price = &instrument.data.last().unwrap().open;
    let stop_loss_value = stop_loss.value;
    let stop_loss_price = stop_loss.price;
    let atr_value = instrument.indicators.atr.get_data_a().last().unwrap() * stop_loss_value;

    let price = match stop_loss.stop_type {
        StopLossType::Atr => {
            let price = match entry_type {
                TradeType::EntryLong => current_price - atr_value,
                TradeType::EntryShort => current_price + atr_value,
                _ => current_price - atr_value,
            };
            price
        }
        _ => stop_loss_price,
    };

    StopLoss {
        price,
        value: atr_value,
        stop_type: stop_loss.stop_type.to_owned(),
        created_at: to_dbtime(Local::now()),
        updated_at: to_dbtime(Local::now()),
        valid_until: to_dbtime(Local::now() + Duration::days(1000)),
    }
}

pub fn update_stop_loss_values(
    stop_loss: &StopLoss,
    stop_type: StopLossType,
    price: f64,
) -> StopLoss {
    StopLoss {
        price,
        value: stop_loss.value,
        stop_type,
        created_at: stop_loss.created_at,
        updated_at: to_dbtime(Local::now()),
        valid_until: stop_loss.valid_until,
    }
}

pub fn update_bot_stop_loss(price: f64, entry_type: &TradeType, stop_loss: &StopLoss) -> StopLoss {
    let stop_loss_price = stop_loss.price;
    let atr_value = stop_loss.value;
    let price = match stop_loss.stop_type {
        StopLossType::Atr => {
            let atr_value = stop_loss.value;
            let price = match entry_type {
                TradeType::EntryLong => price - atr_value,
                TradeType::EntryShort => price + atr_value,
                _ => price - atr_value,
            };
            price
        }
        _ => stop_loss_price,
    };

    StopLoss {
        price,
        value: atr_value,
        stop_type: stop_loss.stop_type.to_owned(),
        created_at: to_dbtime(Local::now()),
        updated_at: to_dbtime(Local::now()),
        valid_until: to_dbtime(Local::now() + Duration::days(1000)),
    }
}

pub fn resolve_stop_loss(current_price: f64, trade_in: &TradeIn) -> bool {
    let stop_loss_price = trade_in.stop_loss.price;

    match trade_in.trade_type {
        TradeType::EntryLong => current_price <= stop_loss_price,
        TradeType::EntryShort => current_price >= stop_loss_price,
        _ => current_price - current_price <= stop_loss_price,
    }
}
