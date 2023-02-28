use std::env;

use super::mode;
use super::pricing::Pricing;
use super::trade::{Trade, TradeType};

use crate::helpers::calc::*;
use crate::helpers::uuid;
use crate::helpers::{date, date::*};
use crate::models::stop_loss::*;
use crate::models::trade::Position;
use crate::scanner::candle::Candle;
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
    StopLossLong(OrderDirection, StopLossType),
    StopLossShort(OrderDirection, StopLossType),
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

    pub fn is_entry(&self) -> bool {
        match *self {
            OrderType::BuyOrderLong(_, _, _) | OrderType::BuyOrderShort(_, _, _) => true,
            _ => false,
        }
    }

    pub fn is_exit(&self) -> bool {
        match *self {
            OrderType::SellOrderLong(_, _, _)
            | OrderType::SellOrderShort(_, _, _)
            | OrderType::TakeProfitLong(_, _, _)
            | OrderType::TakeProfitShort(_, _, _) => true,
            _ => false,
        }
    }

    pub fn is_stop(&self) -> bool {
        match self {
            OrderType::StopLossLong(_, _) | OrderType::StopLossShort(_, _) => true,
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
    pub size: f64,
    pub order_type: OrderType,
    pub status: OrderStatus,
    pub origin_price: f64,
    pub target_price: f64,
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

    pub fn size(&self) -> f64 {
        self.size
    }

    pub fn update_pricing(&mut self, origin_price: f64, target_price: f64) {
        self.origin_price = origin_price;
        self.target_price = target_price;
    }

    pub fn fulfill_order(&mut self, index: usize, date: DateTime<Local>) {
        self.set_full_filled_index(index);
        self.set_status(OrderStatus::Fulfilled);
        self.set_updated_at(to_dbtime(date));
        self.set_full_filled_at(to_dbtime(date));
    }

    pub fn fulfill_bot2_order(&mut self, date: DateTime<Local>) {
        self.set_status(OrderStatus::Fulfilled);
        self.set_updated_at(to_dbtime(date));
        self.set_full_filled_at(to_dbtime(date));
    }

    pub fn cancel_order(&mut self, date: DbDateTime) {
        self.set_status(OrderStatus::Canceled);
        self.set_updated_at(date);
    }

    pub fn to_trade_type(&self) -> TradeType {
        match self.order_type {
            OrderType::BuyOrderLong(_, _, _) => TradeType::MarketInLong,
            OrderType::SellOrderLong(_, _, _) | OrderType::TakeProfitLong(_, _, _) => {
                TradeType::MarketOutLong
            }
            OrderType::StopLossLong(_, _) => TradeType::StopLossLong,
            OrderType::BuyOrderShort(_, _, _) => TradeType::MarketInShort,
            OrderType::SellOrderShort(_, _, _) | OrderType::TakeProfitShort(_, _, _) => {
                TradeType::MarketOutShort
            }
            OrderType::StopLossShort(_, _) => TradeType::StopLossShort,
        }
    }
}

pub fn prepare_orders(
    index: usize,
    instrument: &Instrument,
    pricing: &Pricing,
    trade_type: &TradeType,
    order_types: &Vec<OrderType>,
) -> Vec<Order> {
    let execution_mode = mode::from_str(&env::var("EXECUTION_MODE").unwrap());
    let mut buy_order_target = 0.;
    let mut sell_order_target = 0.;
    let mut stop_order_target = 0.;
    let mut is_stop_loss = false;
    let mut is_valid_buy_sell_order = true;
    let mut stop_loss_direction = OrderDirection::Up;
    let mut orders: Vec<Order> = vec![];

    let current_candle = instrument.data().get(index).unwrap();
    let close_price = current_candle.close();

    let next_candle = match execution_mode.is_back_test() {
        true => instrument.data().get(index + 1).unwrap(),
        false => instrument.data.last().unwrap(),
    };

    let trade_id = uuid::generate_ts_id(next_candle.date());

    for order_type in order_types {
        match order_type {
            OrderType::BuyOrderLong(direction, order_size, target_price)
            | OrderType::BuyOrderShort(direction, order_size, target_price)
            | OrderType::SellOrderLong(direction, order_size, target_price)
            | OrderType::SellOrderShort(direction, order_size, target_price)
            | OrderType::TakeProfitLong(direction, order_size, target_price)
            | OrderType::TakeProfitShort(direction, order_size, target_price) => {
                if validate_target_price(order_type, direction, &close_price, target_price) {
                    //log::info!("{:?} validated", &order_type,);
                    let order = create_order(
                        index,
                        trade_id,
                        instrument,
                        order_type,
                        target_price,
                        order_size,
                    );

                    match order_type.is_entry() {
                        true => {
                            buy_order_target = match order_type.is_long() {
                                true => order.target_price + pricing.spread(),
                                false => order.target_price,
                            }
                        }
                        false => {
                            sell_order_target = match order_type.is_long() {
                                true => order.target_price,
                                false => order.target_price + pricing.spread(),
                            }
                        }
                    };

                    orders.push(order);
                } else {
                    is_valid_buy_sell_order = false;
                    log::error!("{:?} not valid", &order_type,);
                }
            }
            OrderType::StopLossLong(direction, stop_loss_type)
            | OrderType::StopLossShort(direction, stop_loss_type) => {
                is_stop_loss = true;

                let target_price = match orders.first() {
                    Some(order) => order.target_price,
                    None => next_candle.open(),
                };

                let order_size = match orders.first() {
                    Some(order) => order.size,
                    None => std::env::var("ORDER_SIZE").unwrap().parse::<f64>().unwrap(),
                };

                if is_valid_buy_sell_order {
                    let stop_loss = create_stop_loss_order(
                        index,
                        trade_id,
                        instrument,
                        pricing,
                        direction,
                        stop_loss_type,
                        target_price,
                        order_size,
                    );
                    stop_order_target = stop_loss.target_price;
                    stop_loss_direction = direction.clone();
                    orders.push(stop_loss);
                }
            }
        }
    }

    //CHECK STOP LOSS
    if is_stop_loss {
        match stop_loss_direction == OrderDirection::Down {
            true => {
                if stop_order_target >= buy_order_target + pricing.spread() && buy_order_target > 0.
                {
                    log::error!(
                        "Stop loss can't be placed higher than buy level {:?}",
                        (buy_order_target, stop_order_target)
                    );
                    panic!();
                }
            }
            false => {
                if stop_order_target <= buy_order_target && buy_order_target > 0. {
                    log::error!(
                        "Stop loss can't be placed lower than buy level {:?}",
                        (buy_order_target, stop_order_target)
                    );
                    panic!();
                }
            }
        }
    };

    //CHECK STOP LOSS
    if is_stop_loss {
        match stop_loss_direction == OrderDirection::Down {
            true => {
                if stop_order_target >= buy_order_target && buy_order_target > 0. {
                    log::error!(
                        "Stop loss can't be placed higher than buy level {:?}",
                        (buy_order_target, stop_order_target)
                    );
                    //panic!();
                }
            }
            false => {
                if stop_order_target <= buy_order_target && buy_order_target > 0. {
                    log::error!(
                        "Stop loss can't be placed lower than buy level {:?}",
                        (buy_order_target, stop_order_target)
                    );
                    //panic!();
                }
            }
        }
    };

    //CHECK SELL ORDER VALUE
    match trade_type.is_long() {
        true => {
            if sell_order_target <= buy_order_target && sell_order_target > 0. {
                orders = vec![];
                log::error!(
                    "Sell Order can't be placed lower than buy level {:?}",
                    (buy_order_target, sell_order_target)
                )
            }
        }
        false => {
            if sell_order_target >= buy_order_target && sell_order_target > 0. {
                orders = vec![];
                log::error!(
                    "Sell Order can't be placed higher than buy level {:?}",
                    (buy_order_target, sell_order_target)
                )
            }
        }
    };

    orders
}

pub fn validate_target_price(
    order_type: &OrderType,
    direction: &OrderDirection,
    close_price: &f64,
    target_price: &f64,
) -> bool {
    match direction {
        OrderDirection::Up => {
            if close_price >= target_price {
                log::error!(
                    "{:?} not valid. Target price {} is higher than {}",
                    order_type,
                    target_price,
                    close_price,
                );
                panic!();
                false
            } else {
                true
            }
        }
        OrderDirection::Down => {
            if close_price <= target_price {
                log::error!(
                    "{:?} not valid. Target price {} is lower than {}",
                    order_type,
                    target_price,
                    close_price,
                );
                panic!();
                false
            } else {
                true
            }
        }
    }
}

pub fn create_order(
    index: usize,
    trade_id: usize,
    instrument: &Instrument,
    order_type: &OrderType,
    target_price: &f64,
    order_size: &f64,
) -> Order {
    let execution_mode = mode::from_str(&env::var("EXECUTION_MODE").unwrap());

    let current_candle = match execution_mode.is_back_test() {
        true => instrument.data().get(index).unwrap(),
        false => instrument.data().last().unwrap(),
    };

    let next_index = index + 1;
    let current_date = &current_candle.date();
    let origin_price = current_candle.close();
    let time_frame = instrument.time_frame();
    let valid_until_bars = &env::var("VALID_UNTIL_BARS")
        .unwrap()
        .parse::<i64>()
        .unwrap();

    let valid_until = match instrument.time_frame().is_minutely_time_frame() {
        true => *current_date + date::Duration::minutes(valid_until_bars * time_frame.to_minutes()),
        false => *current_date + date::Duration::hours(valid_until_bars * time_frame.to_hours()),
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
        size: *order_size,
        created_at: to_dbtime(*current_date),
        updated_at: None,
        full_filled_at: None,
        valid_until: Some(to_dbtime(valid_until)),
    }
}

pub fn resolve_active_orders(
    index: usize,
    instrument: &Instrument,
    orders: &Vec<Order>,
    pricing: &Pricing,
) -> Position {
    let mut order_position: Position = Position::None;
    let mut orders_activated = vec![];
    for (_id, order) in orders
        .iter()
        .enumerate()
        .filter(|(_id, order)| order.status == OrderStatus::Pending)
    {
        match order_activated(index, order, instrument, pricing) {
            true => {
                match order.order_type {
                    OrderType::BuyOrderLong(_, _, _) | OrderType::BuyOrderShort(_, _, _) => {
                        order_position = Position::MarketInOrder(order.clone());
                        orders_activated.push(order_position.clone());
                    }
                    OrderType::SellOrderLong(_, _, _)
                    | OrderType::SellOrderShort(_, _, _)
                    | OrderType::TakeProfitLong(_, _, _)
                    | OrderType::TakeProfitShort(_, _, _) => {
                        order_position = Position::MarketOutOrder(order.clone());
                        orders_activated.push(order_position.clone());
                    }
                    OrderType::StopLossLong(_, _) | OrderType::StopLossShort(_, _) => {
                        order_position = Position::MarketOutOrder(order.clone());
                        orders_activated.push(order_position.clone());
                    }
                    _ => todo!(),
                };
            }
            false => (),
        }
    }

    let resolved = match has_executed_buy_order(&orders, &order_position) {
        true => order_position,
        false => Position::None,
    };

    resolved
}

fn order_activated(
    index: usize,
    order: &Order,
    instrument: &Instrument,
    pricing: &Pricing,
) -> bool {
    let data = &instrument.data;
    let prev_index = get_prev_index(index);
    let current_candle = data.get(index).unwrap();
    let prev_candle = data.get(prev_index).unwrap();
    let is_next_order = index > order.index_fulfilled;
    let (current_price_over, current_price_bellow, previous_price_over, previous_price_bellow) =
        get_order_activation_price(current_candle, prev_candle);

    let cross_over = current_price_over >= order.target_price
        && previous_price_over < current_price_over
        && is_next_order;

    let cross_bellow = current_price_bellow <= order.target_price
        && previous_price_bellow > current_price_bellow
        && is_next_order;

    let stop_cross_over = current_candle.high() >= order.target_price
        && prev_candle.high() < current_candle.high()
        && is_next_order;

    let stop_cross_bellow = current_candle.low() <= order.target_price
        && prev_candle.low() > current_candle.low()
        && is_next_order;

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
        OrderType::StopLossLong(direction, _) => stop_cross_bellow,
        OrderType::StopLossShort(direction, _) => stop_cross_over,
        _ => todo!(),
    };
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

    let overwrite_orders = env::var("OVERWRITE_ORDERS")
        .unwrap()
        .parse::<bool>()
        .unwrap();

    let (buy_orders, sell_orders, stop_losses) = get_num_pending_orders(&orders);
    let result: Vec<Order> = new_orders
        .iter()
        .filter(|order| order.status == OrderStatus::Pending)
        .filter(|order| match order.order_type {
            OrderType::BuyOrderLong(_, _, _) | OrderType::BuyOrderShort(_, _, _) => {
                buy_orders < max_buy_orders && stop_losses < max_stop_losses
            }
            OrderType::SellOrderLong(_, _, _)
            | OrderType::SellOrderShort(_, _, _)
            | OrderType::TakeProfitLong(_, _, _)
            | OrderType::TakeProfitShort(_, _, _) => sell_orders < max_sell_orders,
            OrderType::StopLossLong(_, _) | OrderType::StopLossShort(_, _) => {
                stop_losses < max_stop_losses
            }
        })
        .map(|order| order.clone())
        .collect();

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
            OrderType::StopLossLong(_, _) | OrderType::StopLossShort(_, _) => stop_losses += 1,
        };
    }
    (buy_orders, sell_orders, stop_losses)
}

pub fn cancel_all_pending_orders(
    index: usize,
    instrument: &Instrument,
    mut orders: Vec<Order>,
) -> Vec<Order> {
    let execution_mode = mode::from_str(&env::var("EXECUTION_MODE").unwrap());
    let current_date = match execution_mode.is_back_test() {
        true => instrument.data.get(index).unwrap().date().clone(),
        false => Local::now(),
    };

    orders
        .iter_mut()
        .map(|x| {
            let valid_until = fom_dbtime(&x.valid_until.unwrap());
            if current_date >= valid_until && x.status == OrderStatus::Pending {
                //log::warn!("Order canceled {:?} ", (current_date, valid_until));
                x.cancel_order(to_dbtime(Local::now()));
            }
            x.clone()
        })
        .map(|x| x.clone())
        .collect()
}

pub fn cancel_trade_pending_orders<T: Trade>(trade: &T, mut orders: Vec<Order>) -> Vec<Order> {
    orders
        .iter_mut()
        .map(|x| {
            if x.status == OrderStatus::Pending {
                x.cancel_order(*trade.get_date());
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
    let order_position = orders
        .iter()
        .position(|x| x.status == OrderStatus::Pending && x.order_type == order.order_type);

    match order_position {
        Some(x) => {
            //log::info!("FULFILLING {} @ {:?}", index, (order.order_type));
            orders.get_mut(x).unwrap().fulfill_order(index, date);
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

pub fn fulfill_bot_order<T: Trade>(
    trade: &T,
    order: &Order,
    orders: &mut Vec<Order>,
    instrument: &Instrument,
) {
    let index = instrument.data().len() - 1;
    fulfill_trade_order(index, trade, &order, orders)
}

fn get_order_activation_price(candle: &Candle, prev_candle: &Candle) -> (f64, f64, f64, f64) {
    let activation_source = &env::var("ORDER_ACTIVATION_SOURCE").unwrap();
    match activation_source.as_ref() {
        "highs_lows" => (
            candle.high(),
            candle.low(),
            prev_candle.high(),
            prev_candle.low(),
        ),
        "close" => (
            candle.close(),
            candle.close(),
            prev_candle.close(),
            prev_candle.close(),
        ),
        _ => (
            candle.close(),
            candle.close(),
            prev_candle.close(),
            prev_candle.close(),
        ),
    }
}
