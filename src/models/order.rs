use std::env;

use super::mode;
use super::tick::InstrumentTick;
use super::trade::{PositionResult, Trade, TradeResult, TradeType};

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

    pub fn get_direction(&self) -> &OrderDirection {
        match self {
            OrderType::BuyOrderLong(d, _, _) => &d,
            OrderType::BuyOrderShort(d, _, _) => &d,
            OrderType::SellOrderLong(d, _, _) => &d,
            OrderType::SellOrderShort(d, _, _) => &d,
            OrderType::TakeProfitLong(d, _, _) => &d,
            OrderType::TakeProfitShort(d, _, _) => &d,
            OrderType::StopLossLong(d, _) => &d,
            OrderType::StopLossShort(d, _) => &d,
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

    pub fn set_valid_until(&mut self, val: DbDateTime) {
        self.valid_until = Some(val)
    }

    pub fn size(&self) -> f64 {
        self.size
    }

    pub fn update_tick(&mut self, origin_price: f64, target_price: f64) {
        self.origin_price = origin_price;
        self.target_price = target_price;
    }

    pub fn fulfill_order(&mut self, index: usize, date: DateTime<Local>) {
        self.set_full_filled_index(index);
        self.set_status(OrderStatus::Fulfilled);
        self.set_updated_at(to_dbtime(date));
        self.set_full_filled_at(to_dbtime(date));
    }

    pub fn unfulfill_order(&mut self, index: usize, date: DateTime<Local>) {
        self.set_full_filled_index(0);
        self.set_status(OrderStatus::Pending);
        self.set_updated_at(to_dbtime(date));
        self.set_full_filled_at(to_dbtime(date));
    }

    pub fn cancel_order(&mut self, date: DbDateTime) {
        self.set_status(OrderStatus::Canceled);
        self.set_updated_at(date);
    }

    pub fn is_long(&self) -> bool {
        match self.order_type {
            OrderType::BuyOrderLong(_, _, _)
            | OrderType::SellOrderLong(_, _, _)
            | OrderType::TakeProfitLong(_, _, _)
            | OrderType::StopLossLong(_, _) => true,
            _ => false,
        }
    }
    pub fn is_short(&self) -> bool {
        match self.order_type {
            OrderType::BuyOrderShort(_, _, _)
            | OrderType::SellOrderShort(_, _, _)
            | OrderType::TakeProfitShort(_, _, _)
            | OrderType::StopLossShort(_, _) => true,
            _ => false,
        }
    }

    pub fn is_full_filled(&self) -> bool {
        match self.full_filled_at {
            Some(_x) => true,
            None => false,
        }
    }

    pub fn is_pending(&self) -> bool {
        match self.status {
            OrderStatus::Pending => true,
            _ => false,
        }
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

    pub fn is_still_valid(&self, date_compare: DateTime<Local>) -> bool {
        let valid_until = from_dbtime(&self.valid_until.unwrap());
        date_compare < valid_until && self.status == OrderStatus::Pending
    }
}

pub fn prepare_orders(
    index: usize,
    instrument: &Instrument,
    trade_type: &TradeType,
    order_types: &Vec<OrderType>,
    tick: &InstrumentTick,
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
        true => instrument.data().get(index).unwrap(),
        false => instrument.data.last().unwrap(),
    };

    let trade_id = uuid::generate_ts_id(next_candle.date());
    let order_with_spread = env::var("ORDER_WITH_SPREAD")
        .unwrap()
        .parse::<bool>()
        .unwrap();

    for order_type in order_types {
        match order_type {
            OrderType::BuyOrderLong(direction, order_size, target_price)
            | OrderType::BuyOrderShort(direction, order_size, target_price)
            | OrderType::SellOrderLong(direction, order_size, target_price)
            | OrderType::SellOrderShort(direction, order_size, target_price)
            | OrderType::TakeProfitLong(direction, order_size, target_price)
            | OrderType::TakeProfitShort(direction, order_size, target_price) => {
                if validate_target_price(order_type, direction, &close_price, target_price) {
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
                                true => match order_with_spread {
                                    true => order.target_price,
                                    false => order.target_price + tick.spread(),
                                },
                                false => order.target_price,
                            }
                        }
                        false => {
                            buy_order_target = order.origin_price;
                            sell_order_target = match order_type.is_long() {
                                true => order.target_price,
                                false => match order_with_spread {
                                    true => order.target_price,
                                    false => order.target_price + tick.spread(),
                                },
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

                if is_valid_buy_sell_order {
                    let stop_loss = create_stop_loss_order(
                        index,
                        trade_id,
                        instrument,
                        direction,
                        stop_loss_type,
                        tick,
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
                if stop_order_target >= buy_order_target && buy_order_target > 0. {
                    log::error!(
                        "Stop loss can't be placed higher than buy level {:?}",
                        (buy_order_target, stop_order_target)
                    );
                }
            }
            false => {
                if stop_order_target <= buy_order_target && buy_order_target > 0. {
                    log::error!(
                        "Stop loss can't be placed lower than buy level {:?}",
                        (buy_order_target, stop_order_target)
                    );
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
                );
                //panic!();
            }
        }
        false => {
            if sell_order_target >= buy_order_target && sell_order_target > 0. {
                orders = vec![];
                log::error!(
                    "Sell Order can't be placed higher than buy level {:?}",
                    (buy_order_target, sell_order_target)
                );
                //panic!();
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
        true => instrument.data().get(index + 1).unwrap(),
        false => instrument.data().last().unwrap(),
    };

    let origin_price = match execution_mode.is_back_test() {
        true => current_candle.open(),
        false => current_candle.close(),
    };

    let current_date = &current_candle.date();
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
        index_created: index,
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
    tick: &InstrumentTick,
    use_tick_price: bool,
) -> Position {
    let mut order_position: Position = Position::None;

    for (_id, order) in orders
        .iter()
        .enumerate()
        .filter(|(_id, order)| order.status == OrderStatus::Pending)
    {
        match is_activated_order(index, order, instrument, tick, use_tick_price) {
            true => {
                match order.order_type {
                    OrderType::BuyOrderLong(_, _, _) | OrderType::BuyOrderShort(_, _, _) => {
                        order_position = Position::MarketInOrder(order.clone());
                    }
                    OrderType::SellOrderLong(_, _, _)
                    | OrderType::SellOrderShort(_, _, _)
                    | OrderType::TakeProfitLong(_, _, _)
                    | OrderType::TakeProfitShort(_, _, _) => {
                        order_position = Position::MarketOutOrder(order.clone());
                    }
                    OrderType::StopLossLong(_, _) | OrderType::StopLossShort(_, _) => {
                        order_position = Position::MarketOutOrder(order.clone());
                    }
                    _ => todo!(),
                };
            }
            false => (),
        }
    }

    match has_executed_buy_order(orders, &order_position) {
        true => order_position,
        false => Position::None,
    }
}

fn is_activated_order(
    index: usize,
    order: &Order,
    instrument: &Instrument,
    tick: &InstrumentTick,
    use_tick_price: bool,
) -> bool {
    let activation_source = &env::var("ORDER_ACTIVATION_SOURCE").unwrap();
    let execution_mode = mode::from_str(&env::var("EXECUTION_MODE").unwrap());

    let current_candle = match execution_mode.is_back_test() {
        true => instrument.data().get(index).unwrap(),
        false => instrument.data().last().unwrap(),
    };

    let candle_ts = uuid::generate_ts_id(current_candle.date());
    let source = if use_tick_price { "Tick" } else { "Candle" };
    let is_bot = execution_mode.is_bot();
    let is_stop = order.order_type.is_stop();
    let is_next_bar = candle_ts > order.id;
    let direction = order.order_type.get_direction();
    let target_price = order.target_price;
    let is_closed = match activation_source.as_ref() {
        "close" => current_candle.is_closed(),
        _ => true,
    };

    let (price_over, price_below) = match use_tick_price {
        true => {
            let tick_price = if order.is_long() {
                tick.bid()
            } else {
                tick.ask()
            };

            (tick_price, tick_price)
        }
        false => {
            let (price_over, price_below) = get_order_activation_price(current_candle, order, tick);
            (price_over, price_below)
        }
    };

    match direction {
        OrderDirection::Up => {
            let cross_over = if is_bot {
                price_over >= target_price
            } else {
                if is_stop {
                    is_closed && current_candle.high() >= target_price
                } else {
                    price_over >= target_price && is_next_bar && is_closed
                }
            };

            if cross_over {
                log::info!(
                    "{} Over {:?} activated. Price {:?} > {} target",
                    source,
                    order.order_type,
                    price_over,
                    target_price
                );
            }

            cross_over
        }
        OrderDirection::Down => {
            let cross_below = if is_bot {
                price_below <= target_price
            } else {
                if is_stop {
                    is_closed && current_candle.low() <= target_price
                } else {
                    price_below <= target_price && is_next_bar && is_closed
                }
            };

            if cross_below {
                log::info!(
                    "{} Below {:?} activated. Price {:?} < {} target",
                    source,
                    order.order_type,
                    price_below,
                    target_price
                );
            }

            cross_below
        }
    }
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

    let _overwrite_orders = env::var("OVERWRITE_ORDERS")
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
        .cloned()
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

    let len = orders.len();

    let pending_orders = match len > 0 {
        true => {
            let pending_orders: Vec<Order> = orders
                .iter()
                .skip(len.saturating_sub(max_pending_orders))
                .filter(|x| x.status == OrderStatus::Pending)
                .take(max_pending_orders)
                .cloned()
                .collect();
            pending_orders
        }
        false => vec![],
    };

    pending_orders
}

pub fn has_executed_buy_order(orders: &Vec<Order>, operation: &Position) -> bool {
    let max_buy_orders = env::var("MAX_BUY_ORDERS")
        .unwrap()
        .parse::<usize>()
        .unwrap();

    let (pending_buy_orders, _sell_orders, _stop_losses) = get_num_pending_orders(orders);

    match operation {
        Position::MarketOutOrder(_) => match pending_buy_orders.cmp(&max_buy_orders) {
            //No Active buy
            std::cmp::Ordering::Equal => false,
            //Aactive buy
            _ => true,
        },
        _ => true,
    }
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

pub fn cancel_pending_expired_orders(
    index: usize,
    instrument: &Instrument,
    orders: &mut Vec<Order>,
) -> Vec<Order> {
    let execution_mode = mode::from_str(&env::var("EXECUTION_MODE").unwrap());
    match execution_mode.is_bot() || execution_mode.is_back_test() {
        true => {
            let current_date = instrument.data.get(index).unwrap().date();
            let mut i = 0;
            while i < orders.len() {
                let order = &mut orders[i];
                if order.status == OrderStatus::Pending && !order.is_still_valid(current_date) {
                    orders.remove(i);
                } else {
                    i += 1;
                }
            }
            orders.clone()
        }
        false => {
            let current_date = Local::now();
            orders
                .iter_mut()
                .map(|x| {
                    if x.status == OrderStatus::Pending && !x.is_still_valid(current_date) {
                        x.cancel_order(to_dbtime(Local::now()));
                    }
                    x.clone()
                })
                .collect()
        }
    }
}

pub fn extend_all_pending_orders(orders: &mut Vec<Order>) {
    for order in orders {
        if order.status == OrderStatus::Pending {
            let current_valid = from_dbtime(&order.valid_until.unwrap());
            let new_valid_date = current_valid + date::Duration::days(365 * 10);
            order.set_valid_until(to_dbtime(new_valid_date));
        }
    }
}

pub fn cancel_trade_pending_orders<T: Trade>(trade: &T, orders: &mut Vec<Order>) {
    let execution_mode = mode::from_str(&env::var("EXECUTION_MODE").unwrap());
    match execution_mode.is_back_test() {
        true => {
            let mut i = 0;
            while i < orders.len() {
                let order = &mut orders[i];
                if order.status == OrderStatus::Pending {
                    orders.remove(i);
                } else {
                    i += 1;
                }
            }
        }
        false => {
            for order in orders {
                if order.status == OrderStatus::Pending {
                    log::info!("Canceling Pending order to {:?}", order.id);
                    order.cancel_order(*trade.get_date());
                }
            }
        }
    }
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
            orders.get_mut(x).unwrap().fulfill_order(index, date);
        }
        None => {}
    }
}

pub fn fulfill_bot_order<T: Trade>(
    trade: &T,
    order: &Order,
    orders: &mut Vec<Order>,
    instrument: &Instrument,
) {
    let index = instrument.data().len() - 1;
    fulfill_trade_order(index, trade, order, orders)
}

fn get_order_activation_price(candle: &Candle, order: &Order, tick: &InstrumentTick) -> (f64, f64) {
    let order_engine = &env::var("EXECUTION_MODE").unwrap();
    let activation_source = &env::var("ORDER_ACTIVATION_SOURCE").unwrap();
    let execution_mode = mode::from_str(&env::var("EXECUTION_MODE").unwrap());
    let spread = tick.spread();

    let (price_over, price_below) = match execution_mode.is_bot() {
        true => (candle.close(), candle.close()),
        false => match order_engine.as_ref() {
            "broker" => (candle.high(), candle.low()),
            "bot" => match activation_source.to_lowercase().as_ref() {
                "highs_lows" => (candle.high(), candle.low()),
                _ => (candle.close(), candle.close()),
            },
            _ => panic!("ORDER_ENGINE not found!"),
        },
    };

    if order.is_long() {
        (price_over, price_below)
    } else {
        (price_over + spread, price_below + spread)
    }
}
