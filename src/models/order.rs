use std::env;

use super::pricing::Pricing;
use super::strategy::StrategyType;
use super::trade::{Trade, TradeDirection, TradeType};
use super::trade::{TradeIn, TradeOut};
use crate::helpers::calc::*;
use crate::helpers::date::*;
use crate::helpers::uuid;
use crate::models::stop_loss::*;
use crate::models::trade::Position;
use crate::scanner::instrument::*;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum OrderType {
    BuyOrderLong(OrderDirection, f64, f64),
    BuyOrderShort(OrderDirection, f64, f64),
    SellOrderLong(OrderDirection, f64, f64),
    SellOrderShort(OrderDirection, f64, f64),
    TakeProfitLong(OrderDirection, f64, f64),
    TakeProfitShort(OrderDirection, f64, f64),
    StopLoss(OrderDirection, StopLossType),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum OrderDirection {
    Up,
    Down,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum OrderCondition {
    Greater,
    Equal,
    Lower,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum OrderStatus {
    Pending,
    Fulfilled,
    Canceled,
}

impl OrderType {
    pub fn is_long(&self) -> bool {
        match self {
            OrderType::BuyOrderLong(_, _, _)
            | OrderType::SellOrderLong(_, _, _)
            | OrderType::TakeProfitLong(_, _, _) => true,
            _ => false,
        }
    }

    pub fn is_stop(&self) -> bool {
        match self {
            OrderType::StopLoss(_, _) => true,
            _ => false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Order {
    pub id: usize,
    pub trade_id: usize,
    pub index_created: usize,
    pub index_fulfilled: usize,
    pub order_type: OrderType,
    pub status: OrderStatus,
    pub origin_price: f64,
    pub target_price: f64,
    pub quantity: f64,
    pub created_at: DbDateTime,
    pub updated_at: Option<DbDateTime>,
    pub full_filled_at: Option<DbDateTime>,
    pub valid_until: Option<DbDateTime>,
}

impl Order {
    pub fn set_status(&mut self, val: OrderStatus) {
        self.status = val
    }

    pub fn set_updated_at(&mut self, val: DbDateTime) {
        self.updated_at = Some(val)
    }

    pub fn set_full_filled_index(&mut self, val: usize) {
        self.index_fulfilled = val
    }

    pub fn set_full_filled_at(&mut self, val: DbDateTime) {
        self.full_filled_at = Some(val)
    }

    pub fn set_trade_id(&mut self, val: usize) {
        self.trade_id = val
    }

    pub fn update_pricing(&mut self, origin_price: f64, target_price: f64) {
        self.origin_price = origin_price;
        self.target_price = target_price;
    }

    pub fn fulfill_order(&mut self, index: usize, date: DateTime<Local>) {
        self.set_status(OrderStatus::Fulfilled);
        self.set_updated_at(to_dbtime(date));
        self.set_full_filled_index(index);
        self.set_full_filled_at(to_dbtime(date));
    }

    pub fn cancel_order(&mut self, date: DbDateTime) {
        self.set_status(OrderStatus::Canceled);
        self.set_updated_at(date);
    }
}

pub fn prepare_orders(
    index: usize,
    instrument: &Instrument,
    pricing: &Pricing,
    trade_type: &TradeType,
    order_types: &Vec<OrderType>,
) -> Vec<Order> {
    let mut buy_order_target = 0.;
    let mut sell_order_target = 0.;
    let mut stop_order_target = 0.;

    let mut orders: Vec<Order> = vec![];
    let trade_id = uuid::generate_ts_id(instrument.data.get(index + 1).unwrap().date());
    for order_type in order_types {
        match order_type {
            OrderType::BuyOrderLong(_, quantity, target_price)
            | OrderType::BuyOrderShort(_, quantity, target_price)
            | OrderType::SellOrderLong(_, quantity, target_price)
            | OrderType::SellOrderShort(_, quantity, target_price)
            | OrderType::TakeProfitLong(_, quantity, target_price)
            | OrderType::TakeProfitShort(_, quantity, target_price) => {
                let order = create_order(
                    index,
                    trade_id,
                    instrument,
                    trade_type,
                    order_type,
                    target_price,
                    quantity,
                );

                match trade_type.is_entry() {
                    true => buy_order_target = order.target_price,
                    false => sell_order_target = order.target_price,
                }

                orders.push(order);
            }
            OrderType::StopLoss(order_direction, stop_loss_stype) => {
                let target_price = match orders.first() {
                    Some(order) => order.target_price,
                    None => instrument.data.get(index + 1).unwrap().open(),
                };

                let stop_loss = create_stop_loss_order(
                    index,
                    trade_id,
                    instrument,
                    pricing,
                    trade_type,
                    order_direction,
                    stop_loss_stype,
                    target_price,
                );
                stop_order_target = stop_loss.target_price;
                orders.push(stop_loss);
            }
        }
    }

    log::warn!(
        "44444444444444 {:?}",
        (
            trade_type,
            buy_order_target,
            sell_order_target,
            stop_order_target
        )
    );

    //CHECK SELL ORDER VALUE
    match trade_type.is_exit() && trade_type.is_long() {
        true => {
            if sell_order_target <= buy_order_target {
                panic!(
                    "[PANIC] Sell Order can't be placed lower than buy level {:?}",
                    (buy_order_target, sell_order_target)
                )
            }
        }
        false => {
            if sell_order_target >= buy_order_target {
                panic!(
                    "[PANIC] Sell Order can't be placed higher than buy level {:?}",
                    (buy_order_target, sell_order_target)
                )
            }
        }
    };

    match trade_type.is_long() {
        true => {
            if stop_order_target >= buy_order_target {
                panic!(
                    "[PANIC] Stop loss can't be placed higher than buy level {:?}",
                    (buy_order_target, stop_order_target)
                )
            }
        }
        false => {
            if stop_order_target <= buy_order_target {
                panic!(
                    "[PANIC] Stop loss can't be placed lower than buy level {:?}",
                    (buy_order_target, stop_order_target)
                )
            }
        }
    };

    orders
}

pub fn create_order(
    index: usize,
    trade_id: usize,
    instrument: &Instrument,
    trade_type: &TradeType,
    order_type: &OrderType,
    target_price: &f64,
    quantity: &f64,
) -> Order {
    let next_index = index + 1;
    let current_price = &instrument.data.get(next_index).unwrap().open();
    let current_date = &instrument.data.get(next_index).unwrap().date();
    let origin_price = instrument.data().get(index).unwrap().close();

    log::info!(
        "CREATING NEW ORDER{} @ {:?} origin {} target {} id {}",
        index,
        order_type,
        origin_price,
        target_price,
        uuid::generate_ts_id(*current_date)
    );
    //let trade_id = uuid::generate_ts_id(*current_date);
    let condition = match trade_type {
        TradeType::MarketInLong => {
            if current_price < target_price {
                OrderCondition::Greater
            } else {
                OrderCondition::Lower
            }
        }
        TradeType::MarketInShort => {
            if current_price < target_price {
                OrderCondition::Lower
            } else {
                OrderCondition::Greater
            }
        }
        _ => OrderCondition::Equal,
    };

    Order {
        id: uuid::generate_ts_id(*current_date),
        index_created: next_index,
        index_fulfilled: 0,
        trade_id,
        order_type: order_type.clone(),
        status: OrderStatus::Pending,
        origin_price,
        target_price: *target_price,
        quantity: *quantity,
        created_at: to_dbtime(*current_date),
        updated_at: None,
        full_filled_at: None,
        valid_until: None,
    }
}

pub fn resolve_active_orders(
    index: usize,
    instrument: &Instrument,
    strategy_type: &StrategyType,
    orders: &Vec<Order>,
) -> Position {
    let mut order_operation: Position = Position::None;

    for (_id, order) in orders
        .iter()
        .enumerate()
        .filter(|(_id, order)| order.status == OrderStatus::Pending)
    {
        match order_activated(index, order, instrument, strategy_type) {
            true => {
                match order.order_type {
                    OrderType::BuyOrderLong(_, _, _) | OrderType::BuyOrderShort(_, _, _) => {
                        order_operation = Position::MarketInOrder(order.clone());
                    }
                    OrderType::SellOrderLong(_, _, _)
                    | OrderType::SellOrderShort(_, _, _)
                    | OrderType::TakeProfitLong(_, _, _)
                    | OrderType::TakeProfitShort(_, _, _) => {
                        order_operation = Position::MarketOutOrder(order.clone());
                    }
                    OrderType::StopLoss(_, _) => {
                        order_operation = Position::MarketOutOrder(order.clone());
                    }
                    _ => todo!(),
                };
            }
            false => (),
        }
    }

    let resolved = match has_executed_buy_order(&orders, &order_operation) {
        true => order_operation,
        false => Position::None,
    };

    match resolved {
        Position::None => (),
        _ => log::info!("ORDER ACTIVATED {} @@@ {:?}", index, &resolved),
    };

    resolved
}

fn order_activated(
    index: usize,
    order: &Order,
    instrument: &Instrument,
    strategy_type: &StrategyType,
) -> bool {
    let data = &instrument.data;
    let prev_index = get_prev_index(index);
    let current_candle = data.get(index).unwrap();
    let prev_candle = data.get(prev_index).unwrap();

    let cross_over = current_candle.high() >= order.target_price; // && prev_candle.high() < order.target_price;
                                                                  //||
                                                                  //Cross bellow
    let cross_bellow = current_candle.low() <= order.target_price; //&& prev_candle.low() > order.target_price;

    let activated = match &order.order_type {
        OrderType::BuyOrderLong(direction, _, _) | OrderType::BuyOrderShort(direction, _, _) => {
            match direction {
                OrderDirection::Up => cross_over,
                OrderDirection::Down => cross_bellow,
            }
        }
        OrderType::SellOrderLong(direction, _, _)
        | OrderType::SellOrderShort(direction, _, _)
        | OrderType::TakeProfitLong(direction, _, _)
        | OrderType::TakeProfitShort(direction, _, _) => match direction {
            OrderDirection::Up => cross_over,
            OrderDirection::Down => cross_bellow,
        },
        OrderType::StopLoss(direction, stop) => match direction {
            OrderDirection::Up => {
                if cross_over {
                    log::info!("STOP LOSS CROSS OVER");
                }
                cross_over
            }
            OrderDirection::Down => {
                if cross_bellow {
                    log::info!("STOP LOSS CROSS BELLOW");
                }
                cross_bellow
            }
        },
        _ => todo!(),
    };

    // if activated {
    //     log::info!(
    //         "ACTIVATING {} @@@ {:?} ",
    //         index,
    //         (&order.order_type, order.target_price, current_candle),
    //     );
    // }
    activated
}

pub fn add_pending(orders: Vec<Order>, new_orders: Vec<Order>) -> Vec<Order> {
    let max_buy_orders = env::var("MAX_BUY_ORDERS")
        .unwrap()
        .parse::<usize>()
        .unwrap();
    let max_sell_orders = env::var("MAX_SELL_ORDERS")
        .unwrap()
        .parse::<usize>()
        .unwrap();
    let max_stop_losses = env::var("MAX_STOP_LOSSES")
        .unwrap()
        .parse::<usize>()
        .unwrap();

    let max_pending_orders = env::var("MAX_PENDING_ORDERS")
        .unwrap()
        .parse::<usize>()
        .unwrap();

    let (buy_orders, sell_orders, stop_losses) = get_num_pending_orders(&orders);

    log::warn!("MAX PENDING {:?}", (buy_orders, sell_orders, stop_losses));

    let result: Vec<Order> = new_orders
        .iter()
        .filter(|order| order.status == OrderStatus::Pending)
        .filter(|order| match order.order_type {
            OrderType::BuyOrderLong(_, _, _) | OrderType::BuyOrderShort(_, _, _) => {
                buy_orders < max_buy_orders
            }
            OrderType::SellOrderLong(_, _, _)
            | OrderType::SellOrderShort(_, _, _)
            | OrderType::TakeProfitLong(_, _, _)
            | OrderType::TakeProfitShort(_, _, _) => sell_orders < max_sell_orders,
            OrderType::StopLoss(_, _) => stop_losses < max_stop_losses,
        })
        .map(|order| order.clone())
        .collect();

    // log::warn!(
    //     "11111111 {} {} {} {}",
    //     buy_orders,
    //     sell_orders,
    //     stop_losses,
    //     result.len()
    // );

    if result.len() <= max_pending_orders {
        [orders, result].concat()
    } else {
        orders
    }
}

pub fn get_pending(orders: &Vec<Order>) -> Vec<Order> {
    let max_pending_orders = env::var("MAX_PENDING_ORDERS")
        .unwrap()
        .parse::<usize>()
        .unwrap();
    let pending_orders: Vec<Order> = orders
        .iter()
        .rev()
        .take(max_pending_orders)
        .filter(|x| x.status == OrderStatus::Pending)
        .map(|x| x.clone())
        .collect();
    pending_orders
}

pub fn has_executed_buy_order(orders: &Vec<Order>, operation: &Position) -> bool {
    let max_buy_orders = env::var("MAX_BUY_ORDERS")
        .unwrap()
        .parse::<usize>()
        .unwrap();

    let (pending_buy_orders, _sell_orders, _stop_losses) = get_num_pending_orders(&orders);

    let has_active_buy_order = match operation {
        Position::MarketOutOrder(_) => match pending_buy_orders.cmp(&max_buy_orders) {
            //No Active buy
            std::cmp::Ordering::Equal => false,
            //Aactive buy
            _ => true,
        },
        _ => true,
    };

    has_active_buy_order
}

pub fn get_num_pending_orders(orders: &Vec<Order>) -> (usize, usize, usize) {
    let max_pending_orders = env::var("MAX_PENDING_ORDERS")
        .unwrap()
        .parse::<usize>()
        .unwrap();
    let mut buy_orders = 0;
    let mut sell_orders = 0;
    let mut stop_losses = 0;

    for order in orders
        .iter()
        .rev()
        .take(max_pending_orders)
        .filter(|x| x.status == OrderStatus::Pending)
    {
        match order.order_type {
            OrderType::BuyOrderLong(_, _, _) | OrderType::BuyOrderShort(_, _, _) => buy_orders += 1,
            OrderType::SellOrderLong(_, _, _)
            | OrderType::SellOrderShort(_, _, _)
            | OrderType::TakeProfitLong(_, _, _)
            | OrderType::TakeProfitShort(_, _, _) => sell_orders += 1,
            OrderType::StopLoss(_, _) => stop_losses += 1,
        };
    }
    (buy_orders, sell_orders, stop_losses)
}

pub fn cancel_trade_pending_orders<T: Trade>(trade: &T, mut orders: Vec<Order>) -> Vec<Order> {
    orders
        .iter_mut()
        .map(|x| {
            if x.status == OrderStatus::Pending {
                x.cancel_order(*trade.get_date());
                log::info!("CANCELED {:?}", x.order_type);
            }
            x.clone()
        })
        .collect()
}

pub fn update_pending_trade_orders<T: Trade>(trade: &T, orders: &mut Vec<Order>) -> Vec<Order> {
    orders
        .iter_mut()
        .map(|x| {
            if x.status == OrderStatus::Pending {
                //x.cancel_order(*trade.get_date());
                log::info!("UPDATING {:?}", x.order_type);
            }
            x.clone()
        })
        .collect()
}

pub fn fulfill_trade_order<T: Trade>(
    index: usize,
    trade: &T,
    order: &Order,
    orders: &mut Vec<Order>,
) {
    let date = trade.get_chrono_date();
    log::info!(
        "UPDATING PENDING {} @ {:?}",
        index,
        get_pending(orders).len()
    );

    let order_position = orders
        .iter()
        .position(|x| x.status == OrderStatus::Pending && x.order_type == order.order_type);

    match order_position {
        Some(x) => {
            log::info!("FULFILLING {} @ {:?}", index, (order.order_type));
            orders.get_mut(x).unwrap().fulfill_order(index, date);
            //UPDATE STOP LOSS AND SELL ORDER BASED ON PRICE_IN
        }
        None => {}
    }
}

pub fn fulfill_order_and_update_pricing<T: Trade>(
    index: usize,
    trade: &T,
    pricing: &Pricing,
    order: &Order,
    mut orders: &mut Vec<Order>,
) {
    fulfill_trade_order(index, trade, &order, &mut orders);

    let original_price_in = order.target_price;
    let final_price_in = trade.get_price_in();
    let diff = final_price_in - original_price_in;

    log::info!(
        "NEW TARGET_PRICE from {:?} {} to {} -> {}",
        order.order_type,
        original_price_in,
        final_price_in,
        diff
    );

    for order in orders
        .iter_mut()
        .filter(|x| x.status == OrderStatus::Pending)
    {
        //let final_target = (order.target_price * final_price_in) / original_price_in;
        let final_target = order.target_price + diff;

        log::info!(
            "UPDATING {:?} @@ from {} to {} -> {}",
            &order.order_type,
            order.target_price,
            final_target,
            order.target_price + diff
        );

        order.update_pricing(*final_price_in, final_target);
    }

    //update_pending_trade_orders(trade, orders);
}

/*
BUY 132.23209999999997 to 132.21899999999997 -> -0.01310000000000855
STOp 132.35350000000003 to 132.34040000000002

ACTIVATED 0.91759

FINAL IN/OUT 0.91701 / 0.91706


1671668700
1671656400
1671656400



   current_candle.low() <= order.target_price && prev_candle.low() > order.target_price;

132.34040000000002,
open: 132.412,
high: 132.41299999999995,
low: 132.296,
close: 132.32200000000006

132.296 < 132.34040000000002

*/
