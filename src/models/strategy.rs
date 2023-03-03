use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum StrategyType {
    OnlyLong,
    OnlyShort,
    LongShort,
    LongShortMTF,
    OnlyLongMTF,
    OnlyShortMTF,
}

impl StrategyType {
    pub fn is_long_only(&self) -> bool {
        match *self {
            StrategyType::OnlyLong => true,
            StrategyType::OnlyLongMTF => true,
            _ => false,
        }
    }

    pub fn is_multi_timeframe(&self) -> bool {
        match *self {
            StrategyType::OnlyLongMTF => true,
            StrategyType::LongShortMTF => true,
            StrategyType::OnlyShortMTF => true,
            _ => false,
        }
    }
}

pub fn from_str(strategy: &str) -> StrategyType {
    match strategy {
        "OnlyLong" => StrategyType::OnlyLong,
        "OnlyShort" => StrategyType::OnlyShort,
        "LongShort" => StrategyType::LongShort,
        "LongShortMTF" => StrategyType::LongShortMTF,
        "OnlyLongMTF" => StrategyType::OnlyLongMTF,
        "OnlyShortMTF" => StrategyType::OnlyShortMTF,
        _ => StrategyType::OnlyLong,
    }
}

pub fn is_mtf_strategy(strategy_type: &StrategyType) -> bool {
    match strategy_type {
        StrategyType::OnlyLongMTF => true,
        StrategyType::LongShortMTF => true,
        StrategyType::OnlyShortMTF => true,
        _ => false,
    }
}

pub fn is_long_only(strategy_type: &StrategyType) -> bool {
    match strategy_type {
        StrategyType::OnlyLong => true,
        StrategyType::OnlyLongMTF => true,
        _ => false,
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct StrategyStats {
    pub trades: usize,
    pub wining_trades: usize,
    pub losing_trades: usize,
    pub won_per_trade_per: f64,
    pub lost_per_trade_per: f64,
    pub stop_losses: usize,
    pub gross_profit: f64,
    pub commissions: f64,
    pub net_profit: f64,
    pub net_profit_per: f64,
    pub profitable_trades: f64,
    pub profit_factor: f64,
    pub max_runup: f64,
    pub max_drawdown: f64,
    pub buy_hold: f64,
    pub annual_return: f64,
}

impl StrategyStats {
    pub fn new() -> StrategyStats {
        StrategyStats {
            trades: 0,
            wining_trades: 0,
            losing_trades: 0,
            won_per_trade_per: 0.,
            lost_per_trade_per: 0.,
            stop_losses: 0,
            gross_profit: 0.,
            commissions: 0.,
            net_profit: 0.,
            net_profit_per: 0.,
            profitable_trades: 0.,
            profit_factor: 0.,
            max_runup: 0.,
            max_drawdown: 0.,
            buy_hold: 0.,
            annual_return: 0.,
        }
    }
}

fn resolve_entry_position(
    &mut self,
    index: usize,
    instrument: &Instrument,
    htf_instrument: &HTFInstrument,
    pricing: &Pricing,
    orders: &Vec<Order>,
    trade_size: f64,
) -> PositionResult {
    let mut long_entry: bool = false;
    let mut short_entry: bool = false;
    let pending_orders = order::get_pending(orders);
    let overwrite_orders = env::var("OVERWRITE_ORDERS")
        .unwrap()
        .parse::<bool>()
        .unwrap();

    let entry_long = match self.strategy_type() {
        StrategyType::OnlyLong
        | StrategyType::LongShort
        | StrategyType::OnlyLongMTF
        | StrategyType::LongShortMTF => {
            match self.entry_long(index, instrument, htf_instrument, pricing) {
                Position::MarketIn(order_types) => {
                    let trade_type = TradeType::MarketInLong;
                    let trade_in_result =
                        resolve_trade_in(index, trade_size, instrument, pricing, &trade_type, None);

                    let prepared_orders = match order_types {
                        Some(orders) => {
                            long_entry = true;
                            short_entry = false;
                            Some(prepare_orders(
                                index,
                                instrument,
                                pricing,
                                &trade_type,
                                &orders,
                            ))
                        }
                        None => None,
                    };

                    let new_orders = match overwrite_orders {
                        true => prepared_orders,
                        false => match pending_orders.len().cmp(&0) {
                            std::cmp::Ordering::Equal => prepared_orders,
                            _ => None,
                        },
                    };

                    PositionResult::MarketIn(trade_in_result, new_orders)
                }
                Position::Order(order_types) => {
                    let trade_type = TradeType::OrderInLong;

                    let prepared_orders =
                        prepare_orders(index, instrument, pricing, &trade_type, &order_types);

                    let new_orders = match overwrite_orders {
                        true => prepared_orders,
                        false => match pending_orders.len().cmp(&0) {
                            std::cmp::Ordering::Equal => prepared_orders,
                            _ => vec![],
                        },
                    };

                    if new_orders.len() > 0 {
                        long_entry = true;
                        short_entry = false;
                    }
                    //log::info!("111111111 {:?}", (orders.len(), new_orders.len()));

                    PositionResult::PendingOrder(new_orders)
                }
                _ => PositionResult::None,
            }
        }
        _ => PositionResult::None,
    };
    let entry_short = match self.strategy_type() {
        StrategyType::OnlyShort
        | StrategyType::LongShort
        | StrategyType::OnlyShortMTF
        | StrategyType::LongShortMTF => {
            match self.entry_short(index, instrument, htf_instrument, pricing) {
                Position::MarketIn(order_types) => {
                    let trade_type = TradeType::MarketInShort;

                    let trade_in_result =
                        resolve_trade_in(index, trade_size, instrument, pricing, &trade_type, None);

                    let prepared_orders = match order_types {
                        Some(orders) => {
                            short_entry = true;
                            long_entry = false;
                            Some(prepare_orders(
                                index,
                                instrument,
                                pricing,
                                &trade_type,
                                &orders,
                            ))
                        }
                        None => None,
                    };

                    let new_orders = match overwrite_orders {
                        true => prepared_orders,
                        false => match pending_orders.len().cmp(&0) {
                            std::cmp::Ordering::Equal => prepared_orders,
                            _ => None,
                        },
                    };

                    PositionResult::MarketIn(trade_in_result, new_orders)
                }
                Position::Order(order_types) => {
                    let trade_type = TradeType::OrderInShort;

                    let prepared_orders =
                        prepare_orders(index, instrument, pricing, &trade_type, &order_types);

                    let new_orders = match overwrite_orders {
                        true => prepared_orders,
                        false => match pending_orders.len().cmp(&0) {
                            std::cmp::Ordering::Equal => prepared_orders,
                            _ => vec![],
                        },
                    };

                    if new_orders.len() > 0 {
                        short_entry = true;
                        long_entry = false;
                    }

                    PositionResult::PendingOrder(new_orders)
                }
                _ => PositionResult::None,
            }
        }
        _ => PositionResult::None,
    };

    if long_entry && !short_entry {
        entry_long
    } else if !long_entry && short_entry {
        entry_short
    } else {
        PositionResult::None
    }
}
