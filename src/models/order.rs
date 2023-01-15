use std::env;

use super::trade::TradeOut;
use super::trade::TradeType;
use crate::helpers::calc::*;
use crate::helpers::date::*;
use crate::helpers::uuid;
use crate::models::stop_loss::*;
use crate::models::trade::Operation;
use crate::scanner::instrument::*;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum OrderType {
    BuyOrder(f64, f64),
    SellOrder(f64, f64),
    TakeProfit(f64, f64),
    StopLoss(StopLossType),
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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Order {
    pub id: usize,
    pub trade_id: usize,
    pub index_created: usize,
    pub index_fulfilled: usize,
    pub order_type: OrderType,
    pub status: OrderStatus,
    pub condition: OrderCondition,
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

    pub fn fulfill_order(&mut self, index: usize, date: DateTime<Local>) {
        self.set_status(OrderStatus::Fulfilled);
        self.set_updated_at(to_dbtime(date));
        self.set_full_filled_index(index);
        self.set_full_filled_at(to_dbtime(date));
    }

    pub fn cancel_order(&mut self, date: DateTime<Local>) {
        self.set_status(OrderStatus::Canceled);
        self.set_updated_at(to_dbtime(date));
    }
}

pub fn fulfill_orders(order: Order, mut orders: Vec<Order>) {
    let leches = orders.get_mut(1).unwrap();
}

pub fn prepare_orders(
    index: usize,
    instrument: &Instrument,
    trade_type: &TradeType,
    order_types: &Vec<OrderType>,
    spread: f64,
) -> Vec<Order> {
    let mut orders: Vec<Order> = vec![];
    let trade_id = uuid::generate_ts_id(instrument.data.get(index + 1).unwrap().date());
    for order_type in order_types {
        match order_type {
            OrderType::BuyOrder(quantity, target_price)
            | OrderType::SellOrder(quantity, target_price)
            | OrderType::TakeProfit(quantity, target_price) => {
                let order = create_order(
                    index,
                    trade_id,
                    instrument,
                    trade_type,
                    order_type,
                    target_price,
                    quantity,
                );
                orders.push(order);
            }
            OrderType::StopLoss(stop_loss_stype) => {
                let stop_loss = create_stop_loss_order(
                    index,
                    trade_id,
                    instrument,
                    trade_type,
                    stop_loss_stype,
                    spread,
                );
                orders.push(stop_loss);
            }
        }
    }

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
        condition,
        origin_price,
        target_price: *target_price,
        quantity: *quantity,
        created_at: to_dbtime(*current_date),
        updated_at: None,
        full_filled_at: None,
        valid_until: None,
    }
}

pub fn resolve(index: usize, instrument: &Instrument, mut orders: Vec<Order>) -> Operation {
    let mut operations: Vec<Operation> = vec![];

    for (_id, order) in orders
        .iter_mut()
        .enumerate()
        .filter(|(_id, order)| order.status == OrderStatus::Pending)
    {
        match order_activated(order, instrument, index) {
            true => {
                let candle = instrument.data.get(index).unwrap();
                order.fulfill_order(index, candle.date());
                match order.order_type {
                    OrderType::BuyOrder(_, _) => {
                      log::warn!(
                            "9999999 Order activated {} at {:?} {:?}",
                            index,
                            order.target_price,
                            order.full_filled_at
                        );
                        operations.push(Operation::MarketInOrder(order.clone()));
                    }
                    OrderType::SellOrder(_, _) => {
                        log::warn!(
                            "9999999 SellOrder activated {} at {:?} {:?}",
                            index,
                            order.target_price,
                            order.full_filled_at
                        );

                        operations.push(Operation::MarketOutOrder(order.clone()));
                    }
                    OrderType::StopLoss(_) => {
                        log::warn!(
                            "9999999 StopLoss activated {} at {:?}",
                            index,
                            order.full_filled_at,
                        );

                        operations.push(Operation::MarketOutOrder(order.clone()));
                    }
                    _ => todo!(),
                };
            }
            false => (),
        }
    }

    // let stop_loss_position = orders.iter().position(|x| x.status == OrderStatus::Pending);
    // let stop_loss = match stop_loss_position {
    //     Some(pos) => match orders.get(pos) {
    //         Some(order) => order.cancel_order(),
    //         None => (),
    //     },
    //     None => todo!(),
    // };

    match operations.len().cmp(&0) {
        std::cmp::Ordering::Greater => operations.last().unwrap().clone(),
        _ => Operation::None,
    }
}

fn order_activated(mut order: &Order, instrument: &Instrument, index: usize) -> bool {
    let data = &instrument.data;
    let prev_index = get_prev_index(index);
    let current_candle = data.get(index).unwrap();
    let prev_candle = data.get(prev_index).unwrap();

    //Cross over
    current_candle.high() >= order.target_price && prev_candle.high() < order.target_price
    ||
    //Cross bellow
    current_candle.low() <= order.target_price && prev_candle.low() > order.target_price
    || 
    // Price already activated
    match order.condition{
        OrderCondition::Greater => current_candle.close() >= order.target_price,
        OrderCondition::Lower => current_candle.close() <= order.target_price,
                OrderCondition::Equal => todo!(),

    }
}

pub fn add_pending(orders: Vec<Order>, mut new_orders: Vec<Order>) -> Vec<Order> {
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
    let max_pending = max_buy_orders + max_sell_orders + max_stop_losses;

    let mut buy_orders = 0;
    let mut sell_orders = 0;
    let mut stop_losses = 0;

    for order in orders
        .iter()
        .rev()
        .take(10)
        .filter(|x| x.status == OrderStatus::Pending)
    {
        match order.order_type {
            OrderType::BuyOrder(_, _) => buy_orders += 1,
            OrderType::SellOrder(_, _) | OrderType::TakeProfit(_, _) => sell_orders += 1,
            OrderType::StopLoss(_) => stop_losses += 1,
        };
    }

    let result: Vec<Order> = new_orders
        .iter()
        .filter(|order| order.status == OrderStatus::Pending)
        .filter(|order| match order.order_type {
            OrderType::BuyOrder(_, _) => buy_orders < max_buy_orders,
            OrderType::SellOrder(_, _) | OrderType::TakeProfit(_, _) => {
                sell_orders < max_sell_orders
            }
            OrderType::StopLoss(_) => stop_losses < max_stop_losses,
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

    if result.len() <= max_pending {
        [orders, result].concat()
    } else {
        log::error!("11111111 {}", result.len());
        orders
    }
}

pub fn cancel_pending(trade_out: &TradeOut, mut orders: Vec<Order>) -> Vec<Order> {
    let stop_loss_index = orders
        .iter()
        .position(|x| x.status == OrderStatus::Pending)
        .unwrap();
    orders.remove(stop_loss_index);
    orders
    // if pending_orders.len() <= 3 {
    //     [orders, new_orders].concat()
    // } else {
    //     log::error!("11111111 {}", pending_orders.len());

    //     orders
    // }
}
