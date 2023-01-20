use std::env;

use super::strategy::StrategyType;
use super::trade::{TradeDirection, TradeType};
use super::trade::{TradeIn, TradeOut};
use crate::helpers::calc::*;
use crate::helpers::date::*;
use crate::helpers::uuid;
use crate::models::stop_loss::*;
use crate::models::trade::Operation;
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
    trade_type: &TradeType,
    order_types: &Vec<OrderType>,
    spread: f64,
) -> Vec<Order> {
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
                orders.push(order);
            }
            OrderType::StopLoss(_, stop_loss_stype) => {
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

    log::info!("PREPARED {:?} {}", order_types, orders.len());

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
        "NEW ORDER CREATED {:?} @ {:?} {:?} {:?}",
        (next_index, current_price, target_price),
        (origin_price, current_price, target_price),
        current_date,
        order_type
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
) -> Operation {
    let mut order_operation: Operation = Operation::None;

    for (_id, order) in orders
        .iter()
        .enumerate()
        .filter(|(_id, order)| order.status == OrderStatus::Pending)
    {
        match order_activated(index, order, instrument, strategy_type) {
            true => {
                match order.order_type {
                    OrderType::BuyOrderLong(_, _, _) | OrderType::BuyOrderShort(_, _, _) => {
                        order_operation = Operation::MarketInOrder(order.clone());
                    }
                    OrderType::SellOrderLong(_, _, _)
                    | OrderType::SellOrderShort(_, _, _)
                    | OrderType::TakeProfitLong(_, _, _)
                    | OrderType::TakeProfitShort(_, _, _) => {
                        order_operation = Operation::MarketOutOrder(order.clone());
                    }
                    OrderType::StopLoss(_, _) => {
                        order_operation = Operation::MarketOutOrder(order.clone());
                    }
                    _ => todo!(),
                };
            }
            false => (),
        }
    }

    let resolved = match has_executed_buy_order(&orders, &order_operation) {
        true => order_operation,
        false => Operation::None,
    };

    match resolved {
        Operation::None => (),
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

    let cross_over =
        current_candle.high() >= order.target_price && prev_candle.high() < order.target_price;
    //||
    //Cross bellow
    let cross_bellow =
        current_candle.low() <= order.target_price && prev_candle.low() > order.target_price;

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
        OrderType::StopLoss(direction, stop) => cross_over || cross_bellow,
        _ => todo!(),
    };

    if activated {
        log::info!(
            "ACTIVATING {} @@@ {:?} ",
            index,
            (
                &order.order_type,
                current_candle.date(),
                current_candle.high(),
                current_candle.close(),
                order.target_price
            ),
        );
    }
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

    let overwrite_orders = env::var("OVERWRITE_ORDERS")
        .unwrap()
        .parse::<bool>()
        .unwrap();

    let max_pending_orders = env::var("MAX_PENDING_ORDERS")
        .unwrap()
        .parse::<usize>()
        .unwrap();

    let (buy_orders, sell_orders, stop_losses) = get_num_pending_orders(&orders);

    log::warn!("MAX PENDING {}", new_orders.len());

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

pub fn cancel_pending_trade_orders(trade_out: &TradeOut, mut orders: Vec<Order>) -> Vec<Order> {
    orders
        .iter_mut()
        .map(|x| {
            if x.status == OrderStatus::Pending {
                x.cancel_order(trade_out.date_out);
                log::info!("CANCELED {:?}", x.order_type);
            }
            x.clone()
        })
        .collect()
}

pub fn cancel_pending_trade_orders_in(
    index: usize,
    instrument: &Instrument,
    mut orders: Vec<Order>,
) -> Vec<Order> {
    let data = &instrument.data;
    let prev_index = get_prev_index(index);
    let current_candle = data.get(index).unwrap();

    // let mut long_pending_orders: Vec<Order> = orders
    //     .iter()
    //     .rev()
    //     .take(5)
    //     .filter(|x| x.status == OrderStatus::Pending)
    //     .map(|x| x.clone())
    //     .collect();

    let long_pending_orders = orders
        .iter_mut()
        .map(|x| {
            if x.status == OrderStatus::Pending {
                x.cancel_order(to_dbtime(current_candle.date()));
            }
            x.clone()
        })
        .collect();
    long_pending_orders
    //[orders, long_pending_orders].concat()
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

pub fn has_executed_buy_order(orders: &Vec<Order>, operation: &Operation) -> bool {
    let max_buy_orders = env::var("MAX_BUY_ORDERS")
        .unwrap()
        .parse::<usize>()
        .unwrap();

    let (pending_buy_orders, _sell_orders, _stop_losses) = get_num_pending_orders(&orders);

    let has_active_buy_order = match operation {
        Operation::MarketOutOrder(_) => match pending_buy_orders.cmp(&max_buy_orders) {
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

pub fn fulfill_order(index: usize, date: &DateTime<Local>, order: &Order, orders: &mut Vec<Order>) {
    // let data = &instrument.data;
    // let prev_index = get_prev_index(index);
    // let candle = data.get(index).unwrap();

    let order_position = orders
        .iter()
        .position(|x| x.status == OrderStatus::Pending && x.order_type == order.order_type);

    match order_position {
        Some(x) => {
            log::info!("FULFILLING {} @ {:?}", index, (order.order_type));
            orders.get_mut(x).unwrap().fulfill_order(index, *date);

            //UPDATE STOP LOSS AND SELL ORDER BASED ON PRICE_IN
        }
        None => {}
    }
}

pub fn fulfill_trade_in_order(
    index: usize,
    trade_in: &TradeIn,
    order: &Order,
    mut orders: &mut Vec<Order>,
) {
    // let data = &instrument.data;
    // let prev_index = get_prev_index(index);
    // let candle = data.get(index).unwrap();
    let date = trade_in.date_in;
    fulfill_order(index, &fom_dbtime(date), &order, &mut orders);

    log::info!(
        "UPDATING PENDING {} @ {:?}",
        index,
        get_pending(orders).len()
    );

    let target_price = order.target_price;
    let price_in = trade_in.price_in;
    // let order_position = orders
    //     .iter()
    //     .position(|x| x.status == OrderStatus::Pending && x.order_type == order.order_type);

    // match order_position {
    //     Some(x) => {
    //         orders
    //             .get_mut(x)
    //             .unwrap()
    //             .fulfill_order(index, candle.date());

    //         //UPDATE STOP LOSS AND SELL ORDER BASED ON PRICE_IN
    //         log::info!("FULFILLING {} @ {:?}", index, (order.order_type));
    //     }
    //     None => {}
    // }
    // log::info!(
    //     "UPDATING PENDING {} @ {:?}",
    //     index,
    //     get_pending(orders).len()
    // );
}
