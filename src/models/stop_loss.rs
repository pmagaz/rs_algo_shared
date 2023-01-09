use super::order::{Order, OrderCondition, OrderType};
use super::trade::*;

use crate::helpers::date::*;
use crate::indicators::Indicator;
use crate::scanner::instrument::Instrument;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum StopLossType {
    Atr,
    Price(f64),
    //Percentage(f64),
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
    instrument: &Instrument,
    entry_type: &TradeType,
    stop_loss_type: &StopLossType,
) -> Order {
    let atr_multiplier = std::env::var("BACKTEST_ATR_STOP_LOSS")
        .unwrap()
        .parse::<f64>()
        .unwrap();

    let next_index = index + 1;
    let current_price = &instrument.data.get(next_index).unwrap().open();
    let current_date = &instrument.data.get(next_index).unwrap().date();
    let current_atr_value = instrument
        .indicators
        .atr
        .get_data_a()
        .get(next_index)
        .unwrap()
        * atr_multiplier;

    let origin_price = instrument.data().get(next_index).unwrap().open();

    let target_price = match stop_loss_type {
        StopLossType::Atr => match entry_type.is_long() {
            true => current_price - current_atr_value,
            false => current_price + current_atr_value,
        },
        StopLossType::Price(target_price) => *target_price,
        StopLossType::None => todo!(),
    };

    let condition = match entry_type.is_long() {
        true => OrderCondition::Lower,
        false => OrderCondition::Greater,
    };

    Order {
        id: current_date.timestamp_millis() as usize,
        order_type: OrderType::StopLoss(stop_loss_type.clone()),
        condition,
        fulfilled: false,
        origin_price,
        target_price,
        quantity: 100.,
        created_at: to_dbtime(Local::now()),
        updated_at: None,
        full_filled_at: None,
        valid_until: to_dbtime(Local::now() + Duration::days(1000)),
    }
}

// pub fn create_bot_stop_loss(
//     entry_type: &TradeType,
//     instrument: &Instrument,
//     _index: usize,
//     stop_loss: &StopLoss,
// ) -> StopLoss {
//     let current_price = &instrument.data.last().unwrap().open;
//     let stop_loss_value = stop_loss.value;
//     let stop_loss_price = stop_loss.price;
//     let atr_value = instrument.indicators.atr.get_data_a().last().unwrap() * stop_loss_value;

//     let price = match stop_loss.stop_type {
//         StopLossType::Atr => match entry_type {
//             TradeType::EntryLong => current_price - atr_value,
//             TradeType::EntryShort => current_price + atr_value,
//             _ => current_price - atr_value,
//         },
//         _ => stop_loss_price,
//     };

//     StopLoss {
//         price,
//         value: atr_value,
//         stop_type: stop_loss.stop_type.to_owned(),
//         created_at: to_dbtime(Local::now()),
//         updated_at: to_dbtime(Local::now()),
//         valid_until: to_dbtime(Local::now() + Duration::days(1000)),
//     }
// }

// pub fn update_stop_loss_values(
//     stop_loss: &StopLoss,
//     stop_type: StopLossType,
//     price: f64,
// ) -> StopLoss {
//     StopLoss {
//         price,
//         value: stop_loss.value,
//         stop_type,
//         created_at: stop_loss.created_at,
//         updated_at: to_dbtime(Local::now()),
//         valid_until: stop_loss.valid_until,
//     }
// }

// pub fn update_bot_stop_loss(price: f64, entry_type: &TradeType, stop_loss: &StopLoss) -> StopLoss {
//     let stop_loss_price = stop_loss.price;
//     let atr_value = stop_loss.value;
//     let price = match stop_loss.stop_type {
//         StopLossType::Atr => {
//             let atr_value = stop_loss.value;

//             match entry_type {
//                 TradeType::EntryLong => price - atr_value,
//                 TradeType::EntryShort => price + atr_value,
//                 _ => price - atr_value,
//             }
//         }
//         _ => stop_loss_price,
//     };

//     StopLoss {
//         price,
//         value: atr_value,
//         stop_type: stop_loss.stop_type.to_owned(),
//         created_at: to_dbtime(Local::now()),
//         updated_at: to_dbtime(Local::now()),
//         valid_until: to_dbtime(Local::now() + Duration::days(1000)),
//     }
// }

// pub fn resolve_stop_loss(current_price: f64, trade_in: &TradeIn) -> bool {
//     let stop_loss_price = trade_in.stop_loss.price;

//     match trade_in.trade_type {
//         TradeType::EntryLong => current_price <= stop_loss_price,
//         TradeType::EntryShort => current_price >= stop_loss_price,
//         _ => current_price - current_price <= stop_loss_price,
//     }
// }
