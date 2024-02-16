use std::env;

use super::environment;
use super::mode::{self, ExecutionMode};
use super::tick::InstrumentTick;
use super::trade::{Trade, TradeIn, TradeType};

use crate::helpers::{calc, uuid};
use crate::helpers::{date, date::*};
use crate::models::stop_loss::*;
use crate::models::trade::Position;
use crate::scanner::candle::Candle;
use crate::scanner::instrument::*;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum OrderType {
    BuyOrderLong(f64, f64),
    BuyOrderShort(f64, f64),
    SellOrderLong(f64, f64),
    SellOrderShort(f64, f64),
    TakeProfitLong(f64, f64),
    TakeProfitShort(f64, f64),
    StopLossLong(StopLossType, f64),
    StopLossShort(StopLossType, f64),
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
            OrderType::BuyOrderLong(_, _)
            | OrderType::SellOrderLong(_, _)
            | OrderType::TakeProfitLong(_, _) => true,
            _ => false,
        }
    }

    pub fn is_entry(&self) -> bool {
        match *self {
            OrderType::BuyOrderLong(_, _) | OrderType::BuyOrderShort(_, _) => true,
            _ => false,
        }
    }

    pub fn get_direction(&self) -> &OrderDirection {
        match self {
            OrderType::BuyOrderLong(_, _)
            | OrderType::SellOrderLong(_, _)
            | OrderType::TakeProfitLong(_, _)
            | OrderType::StopLossShort(_, _) => &OrderDirection::Up,
            OrderType::BuyOrderShort(_, _)
            | OrderType::SellOrderShort(_, _)
            | OrderType::TakeProfitShort(_, _)
            | OrderType::StopLossLong(_, _) => &OrderDirection::Down,
        }
    }

    pub fn is_exit(&self) -> bool {
        match *self {
            OrderType::SellOrderLong(_, _)
            | OrderType::SellOrderShort(_, _)
            | OrderType::TakeProfitLong(_, _)
            | OrderType::TakeProfitShort(_, _) => true,
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
pub struct MetaData {
    pub sl: f64,
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
    pub meta: Option<MetaData>,
}

impl Default for Order {
    fn default() -> Self {
        Order {
            id: 0,
            trade_id: 0,
            index_created: 0,
            index_fulfilled: 0,
            size: 0.0,
            order_type: OrderType::BuyOrderLong(0.0, 0.0),
            status: OrderStatus::Pending,
            origin_price: 0.0,
            target_price: 0.0,
            created_at: to_dbtime(Local::now()),
            updated_at: None,
            full_filled_at: None,
            valid_until: None,
            meta: None,
        }
    }
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

    pub fn trade_id(&self) -> usize {
        self.trade_id
    }

    pub fn set_origin_price(&mut self, val: f64) {
        self.origin_price = val
    }

    pub fn set_valid_until(&mut self, val: DbDateTime) {
        self.valid_until = Some(val)
    }

    pub fn size(&self) -> f64 {
        self.size
    }

    pub fn has_active_trade(&self) -> bool {
        self.trade_id() > 0
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

    pub fn unfulfill_order(&mut self, _index: usize, date: DateTime<Local>) {
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
            OrderType::BuyOrderLong(_, _)
            | OrderType::SellOrderLong(_, _)
            | OrderType::TakeProfitLong(_, _)
            | OrderType::StopLossLong(_, _) => true,
            _ => false,
        }
    }
    pub fn is_short(&self) -> bool {
        match self.order_type {
            OrderType::BuyOrderShort(_, _)
            | OrderType::SellOrderShort(_, _)
            | OrderType::TakeProfitShort(_, _)
            | OrderType::StopLossShort(_, _) => true,
            _ => false,
        }
    }

    pub fn is_stop(&self) -> bool {
        match self.order_type {
            OrderType::StopLossLong(_, _) | OrderType::StopLossShort(_, _) => true,
            _ => false,
        }
    }

    pub fn is_entry(&self) -> bool {
        self.order_type.is_entry()
    }

    pub fn is_fulfilled(&self) -> bool {
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
            OrderType::BuyOrderLong(_, _) => TradeType::MarketInLong,
            OrderType::SellOrderLong(_, _) | OrderType::TakeProfitLong(_, _) => {
                TradeType::MarketOutLong
            }
            OrderType::StopLossLong(_, _) => TradeType::StopLossLong,
            OrderType::BuyOrderShort(_, _) => TradeType::MarketInShort,
            OrderType::SellOrderShort(_, _) | OrderType::TakeProfitShort(_, _) => {
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

    let current_candle = match execution_mode.is_back_test() {
        true => instrument.data().get(index + 1).unwrap(),
        false => instrument.data().last().unwrap(),
    };

    let current_price = match execution_mode.is_back_test() {
        true => current_candle.open(),
        false => current_candle.close(),
    };

    let _trade_id = uuid::generate_ts_id(current_candle.date());
    let order_with_spread = env::var("ORDER_WITH_SPREAD")
        .unwrap()
        .parse::<bool>()
        .unwrap();

    for order_type in order_types {
        match order_type {
            OrderType::BuyOrderLong(order_size, target_price)
            | OrderType::BuyOrderShort(order_size, target_price)
            | OrderType::SellOrderLong(order_size, target_price)
            | OrderType::SellOrderShort(order_size, target_price)
            | OrderType::TakeProfitLong(order_size, target_price)
            | OrderType::TakeProfitShort(order_size, target_price) => {
                let direction = order_type.get_direction();
                if validate_target_price(order_type, direction, &current_price, target_price) {
                    let order =
                        create_order(index, instrument, order_type, target_price, order_size);

                    match order_type.is_entry() {
                        true => {
                            buy_order_target = match order_type.is_long() {
                                true => match order_with_spread {
                                    true => order.target_price,
                                    false => order.target_price - tick.spread(),
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
                                    false => order.target_price - tick.spread(),
                                },
                            }
                        }
                    };

                    match trade_type.is_long() {
                        true => {
                            if sell_order_target <= buy_order_target && sell_order_target > 0. {
                                orders = vec![];
                                log::error!(
                                    "Sell Order can't be placed lower than buy level {:?}",
                                    (buy_order_target, sell_order_target)
                                );
                                panic!();
                            }
                        }
                        false => {
                            if sell_order_target >= buy_order_target && sell_order_target > 0. {
                                orders = vec![];
                                log::error!(
                                    "Sell Order can't be placed higher than buy level {:?}",
                                    (buy_order_target, sell_order_target)
                                );
                                panic!();
                            }
                        }
                    };

                    orders.push(order);
                } else {
                    is_valid_buy_sell_order = false;
                    log::error!("{:?} not valid", &order_type,);
                }
            }
            OrderType::StopLossLong(stop_loss_type, buy_price)
            | OrderType::StopLossShort(stop_loss_type, buy_price) => {
                is_stop_loss = true;
                let direction = order_type.get_direction();

                if is_valid_buy_sell_order {
                    let stop_loss = create_stop_loss_order(
                        index,
                        *buy_price,
                        instrument,
                        direction,
                        stop_loss_type,
                        tick,
                    );

                    stop_order_target = stop_loss.target_price;
                    stop_loss_direction = direction.clone();

                    let buy_order_target = match stop_loss_direction {
                        OrderDirection::Down => *buy_price,
                        OrderDirection::Up => *buy_price + tick.spread(),
                    };

                    match stop_loss_direction == OrderDirection::Down {
                        true => {
                            if stop_order_target >= buy_order_target && buy_order_target > 0. {
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

                    //FIXME WORKARROUND
                    if execution_mode.is_bot() {
                        if let Some(order) = orders.last_mut() {
                            order.meta = Some(MetaData {
                                sl: stop_order_target,
                            });
                        }
                    }

                    orders.push(stop_loss);
                }
            }
        }
    }

    orders
}

pub fn validate_target_price(
    order_type: &OrderType,
    direction: &OrderDirection,
    current_price: &f64,
    target_price: &f64,
) -> bool {
    match direction {
        OrderDirection::Up => {
            if current_price > target_price {
                log::error!(
                    "{:?} not valid. Target price {} is higher than {}",
                    order_type,
                    target_price,
                    current_price,
                );
                panic!();
            } else {
                true
            }
        }
        OrderDirection::Down => {
            if current_price < target_price {
                log::error!(
                    "{:?} not valid. Target price {} is lower than {}",
                    order_type,
                    target_price,
                    current_price,
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

    let current_price = match execution_mode.is_back_test() {
        true => current_candle.open(),
        false => current_candle.close(),
    };

    let current_date = &current_candle.date();
    let time_frame = instrument.time_frame();
    let valid_until_bars = &env::var("ORDER_VALID_UNTIL_BARS")
        .unwrap()
        .parse::<i64>()
        .unwrap();

    let valid_until = match instrument.time_frame().is_minutely_time_frame() {
        true => *current_date + date::Duration::minutes(valid_until_bars * time_frame.to_minutes()),
        false => *current_date + date::Duration::hours(valid_until_bars * time_frame.to_hours()),
    };

    let trade_id = match execution_mode.is_back_test() {
        true => 1,
        false => 0,
    };

    let target_price = calc::format_symbol_price(*target_price, &instrument.symbol);

    Order {
        id: uuid::generate_ts_id(*current_date),
        index_created: index,
        index_fulfilled: 0,
        trade_id,
        order_type: order_type.clone(),
        status: OrderStatus::Pending,
        origin_price: current_price,
        target_price,
        size: *order_size,
        created_at: to_dbtime(*current_date),
        updated_at: None,
        full_filled_at: None,
        valid_until: Some(to_dbtime(valid_until)),
        meta: None,
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

    let filtered_orders: Vec<Order> = orders
        .iter()
        .filter(|order| {
            order.status == OrderStatus::Pending && (order.has_active_trade() || order.is_entry())
        })
        .cloned()
        .collect();

    for (_id, order) in filtered_orders.iter().enumerate() {
        match is_activated_order(index, order, instrument, tick, use_tick_price) {
            true => {
                match order.order_type {
                    OrderType::BuyOrderLong(_, _) | OrderType::BuyOrderShort(_, _) => {
                        order_position = Position::MarketInOrder(order.clone());
                    }
                    OrderType::SellOrderLong(_, _)
                    | OrderType::SellOrderShort(_, _)
                    | OrderType::TakeProfitLong(_, _)
                    | OrderType::TakeProfitShort(_, _)
                    | OrderType::StopLossLong(_, _)
                    | OrderType::StopLossShort(_, _) => {
                        order_position = Position::MarketOutOrder(order.clone());
                    }
                    _ => todo!(),
                };
            }
            false => {}
        }
    }

    let (_, _, pending_stop_losses) = get_num_pending_orders(orders);
    let has_active_trade = pending_stop_losses > 0;

    match has_active_trade {
        true => order_position,
        false => Position::None,
    }
}

fn calculate_order_price_origin(
    execution_mode: &ExecutionMode,
    candle: &Candle,
    activation_source: &str,
) -> (f64, f64) {
    let order_engine = &env::var("ORDER_ENGINE").unwrap();

    match execution_mode.is_bot() || execution_mode.is_bot_test() {
        true => (candle.close(), candle.close()),
        false => match order_engine.as_ref() {
            "broker" => (candle.high(), candle.low()),
            "bot" => match activation_source.to_lowercase().as_ref() {
                "highs_lows" => (candle.high(), candle.low()),
                _ => (candle.close(), candle.close()),
            },
            _ => panic!("ORDER_ENGINE not found!"),
        },
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
    let env = environment::from_str(&env::var("ENV").unwrap());

    let candle = match execution_mode.is_back_test() {
        true => instrument.data().get(index).unwrap(),
        false => instrument.data().last().unwrap(),
    };

    let candle_ts = uuid::generate_ts_id(candle.date());
    let source = if use_tick_price { "Tick" } else { "Candle" };
    let is_bot = execution_mode.is_bot();
    let is_bot_test = execution_mode.is_bot_test();
    let is_stop = order.order_type.is_stop();
    let is_next_bar = candle_ts > order.id;
    let direction = order.order_type.get_direction();
    let target_price = order.target_price;
    let is_closed = match activation_source.as_ref() {
        "close" => candle.is_closed(),
        _ => true,
    };

    let spread = tick.spread();

    let (price_over, price_below) =
        match execution_mode.is_back_test() || execution_mode.is_bot_test() {
            //ONLY BACKTESTING & BOT BACKTESTING, NO TICK PRICING
            true => {
                let (price_over, price_below) =
                    calculate_order_price_origin(&execution_mode, candle, activation_source);

                if order.is_stop() && order.is_short() {
                    (price_over + spread, price_below + spread)
                } else if order.is_entry() == order.is_long() {
                    (price_over, price_below)
                    //(price_over + spread, price_below + spread)
                } else {
                    (price_over, price_below)
                }
            }
            //PRODUCTION BOT
            false => {
                let (price_over, price_below) = if use_tick_price {
                    if order.is_stop() && order.is_short() {
                        (tick.ask(), tick.ask())
                    } else if order.is_entry() == order.is_long() {
                        (tick.bid(), tick.bid())
                    } else {
                        (tick.bid(), tick.bid())
                    }
                } else {
                    let (price_over, price_below) =
                        calculate_order_price_origin(&execution_mode, candle, activation_source);

                    (price_over, price_below)
                };

                (price_over, price_below)
            }
        };

    // log::info!(
    //     "Source: {} Target: {} Over: {} Below: {} Ask/Bid: {:?}",
    //     source,
    //     order.target_price,
    //     price_over,
    //     price_below,
    //     (tick.ask(), tick.bid())
    // );
    match direction {
        OrderDirection::Up => {
            let mut cross_over = false;

            if is_bot || is_bot_test {
                cross_over = price_over >= target_price;
            } else {
                cross_over = price_over >= target_price && is_next_bar && is_closed;
            }

            if cross_over {
                log::info!(
                    "{} Over {:?} activated. Price {:?} > {} target {:?}",
                    source,
                    order.order_type,
                    price_over,
                    target_price,
                    order.target_price
                );
            }

            if env.is_prod() && execution_mode.is_bot() && is_stop {
                false
            } else {
                cross_over
            }
        }
        OrderDirection::Down => {
            let mut cross_below = false;

            if is_bot || is_bot_test {
                cross_below = price_below <= target_price;
            } else {
                cross_below = price_below <= target_price && is_next_bar && is_closed;
            }

            if cross_below {
                log::info!(
                    "{} Below {:?} activated. Price {:?} < {} target",
                    source,
                    order.order_type,
                    price_below,
                    target_price
                );
            }

            if env.is_prod() && execution_mode.is_bot() && is_stop {
                false
            } else {
                cross_below
            }
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

    let _overwrite_orders = env::var("ORDERS_OVERWRITE")
        .unwrap()
        .parse::<bool>()
        .unwrap();

    let (buy_orders, sell_orders, stop_losses) = get_num_pending_orders(&orders);
    let result: Vec<Order> = new_orders
        .iter()
        .filter(|order| order.status == OrderStatus::Pending)
        .filter(|order| match order.order_type {
            OrderType::BuyOrderLong(_, _) | OrderType::BuyOrderShort(_, _) => {
                buy_orders < max_buy_orders && stop_losses < max_stop_losses
            }
            OrderType::SellOrderLong(_, _)
            | OrderType::SellOrderShort(_, _)
            | OrderType::TakeProfitLong(_, _)
            | OrderType::TakeProfitShort(_, _) => sell_orders < max_sell_orders,
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
            OrderType::BuyOrderLong(_, _) | OrderType::BuyOrderShort(_, _) => buy_orders += 1,
            OrderType::SellOrderLong(_, _)
            | OrderType::SellOrderShort(_, _)
            | OrderType::TakeProfitLong(_, _)
            | OrderType::TakeProfitShort(_, _) => sell_orders += 1,
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
    match execution_mode.is_bot_test() || execution_mode.is_back_test() {
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

pub fn update_trade_pending_orders(orders: &mut Vec<Order>, trade_in: &TradeIn) {
    for order in orders {
        if order.status == OrderStatus::Pending {
            let current_valid = from_dbtime(&order.valid_until.unwrap());
            let new_valid_date = current_valid + date::Duration::days(365 * 10);

            order.set_trade_id(trade_in.id);
            order.set_origin_price(trade_in.price_in);
            order.set_valid_until(to_dbtime(new_valid_date));
        }
    }
}

pub fn update_state_pending_orders<T: Trade>(trade: &T, orders: &mut Vec<Order>) {
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
                    if trade.get_type().is_stop() {
                        order.fulfill_order(*trade.get_index_out(), from_dbtime(trade.get_date()));
                    } else {
                        order.cancel_order(*trade.get_date());
                    }
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

pub fn order_exists(orders: &[Order], search_id: usize) -> bool {
    orders.iter().any(|order| order.id == search_id)
}

pub fn update_orders(orders: &mut Vec<Order>, new_orders: &[Order]) -> bool {
    let mut updated = false;

    for new_order in new_orders {
        if let Some(order) = orders.iter_mut().find(|o| o.id == new_order.id) {
            *order = new_order.clone();
            updated = true;
        }
    }

    updated
}
