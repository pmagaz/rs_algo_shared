pub mod broker_trait;
pub mod darwinex;
pub mod models;
#[cfg(feature = "xtb")]
pub mod xtb_models;
#[cfg(feature = "xtb")]
pub mod xtb;
#[cfg(feature = "xtb")]
pub mod xtb_stream;

pub use broker_trait::BrokerStream;
pub use darwinex::Darwinex;
pub use crate::ws::message::Message;
pub use models::*;

use crate::broker::broker_trait::BrokerStream as BS;
use crate::broker::models::*;
use crate::error::Result;
use crate::models::market::MarketHours;
use crate::models::order::Order;
use crate::models::swap::InstrumentSwap;
use crate::models::tick::InstrumentTick;
use crate::models::trade::*;
use crate::ws::message::{InstrumentData, ResponseBody, TradeData, TradeResponse};
use tokio::sync::mpsc::UnboundedReceiver;

/// Runtime broker selector — reads the `BROKER` env var (default: `darwinex`).
pub enum AnyBroker {
    Darwinex(darwinex::Darwinex),
    #[cfg(feature = "xtb")]
    Xtb(xtb_stream::Xtb),
}

pub async fn create_broker() -> AnyBroker {
    match std::env::var("BROKER").as_deref() {
        #[cfg(feature = "xtb")]
        Ok("xtb") => AnyBroker::Xtb(xtb_stream::Xtb::new().await),
        _ => AnyBroker::Darwinex(darwinex::Darwinex::new().await),
    }
}

#[async_trait::async_trait]
impl BS for AnyBroker {
    async fn new() -> Self
    where
        Self: Sized,
    {
        AnyBroker::Darwinex(darwinex::Darwinex::new().await)
    }

    async fn login(&mut self, username: &str, password: &str) -> Result<&mut Self>
    where
        Self: Sized,
    {
        match self {
            AnyBroker::Darwinex(b) => { b.login(username, password).await?; }
            #[cfg(feature = "xtb")]
            AnyBroker::Xtb(b) => { b.login(username, password).await?; }
        }
        Ok(self)
    }

    async fn disconnect(&mut self) -> Result<()> {
        match self {
            AnyBroker::Darwinex(b) => b.disconnect().await,
            #[cfg(feature = "xtb")]
            AnyBroker::Xtb(b) => b.disconnect().await,
        }
    }

    async fn keepalive_ping(&mut self) -> Result<()> {
        match self {
            AnyBroker::Darwinex(b) => b.keepalive_ping().await,
            #[cfg(feature = "xtb")]
            AnyBroker::Xtb(b) => b.keepalive_ping().await,
        }
    }

    async fn get_instrument_data(
        &mut self,
        symbol: &str,
        period: usize,
        start: i64,
    ) -> Result<ResponseBody<InstrumentData<VEC_DOHLC>>> {
        match self {
            AnyBroker::Darwinex(b) => b.get_instrument_data(symbol, period, start).await,
            #[cfg(feature = "xtb")]
            AnyBroker::Xtb(b) => b.get_instrument_data(symbol, period, start).await,
        }
    }

    async fn get_historic_data(
        &mut self,
        symbol: &str,
        period: usize,
        start: i64,
        end: i64,
    ) -> Result<ResponseBody<InstrumentData<VEC_DOHLC>>> {
        match self {
            AnyBroker::Darwinex(b) => b.get_historic_data(symbol, period, start, end).await,
            #[cfg(feature = "xtb")]
            AnyBroker::Xtb(b) => b.get_historic_data(symbol, period, start, end).await,
        }
    }

    async fn get_instrument_tick(
        &mut self,
        symbol: &str,
    ) -> Result<ResponseBody<InstrumentTick>> {
        match self {
            AnyBroker::Darwinex(b) => b.get_instrument_tick(symbol).await,
            #[cfg(feature = "xtb")]
            AnyBroker::Xtb(b) => b.get_instrument_tick(symbol).await,
        }
    }

    async fn get_instrument_swap(
        &mut self,
        symbol: &str,
    ) -> Result<ResponseBody<InstrumentSwap>> {
        match self {
            AnyBroker::Darwinex(b) => b.get_instrument_swap(symbol).await,
            #[cfg(feature = "xtb")]
            AnyBroker::Xtb(b) => b.get_instrument_swap(symbol).await,
        }
    }

    async fn get_ask_bid(&mut self, symbol: &str) -> Result<(f64, f64)> {
        match self {
            AnyBroker::Darwinex(b) => b.get_ask_bid(symbol).await,
            #[cfg(feature = "xtb")]
            AnyBroker::Xtb(b) => b.get_ask_bid(symbol).await,
        }
    }

    async fn get_symbols(&mut self) -> Result<ResponseBody<InstrumentData<VEC_DOHLC>>> {
        match self {
            AnyBroker::Darwinex(b) => b.get_symbols().await,
            #[cfg(feature = "xtb")]
            AnyBroker::Xtb(b) => b.get_symbols().await,
        }
    }

    async fn get_market_hours(&mut self, symbol: &str) -> Result<ResponseBody<MarketHours>> {
        match self {
            AnyBroker::Darwinex(b) => b.get_market_hours(symbol).await,
            #[cfg(feature = "xtb")]
            AnyBroker::Xtb(b) => b.get_market_hours(symbol).await,
        }
    }

    async fn is_market_open(&mut self, symbol: &str) -> Result<ResponseBody<bool>> {
        match self {
            AnyBroker::Darwinex(b) => b.is_market_open(symbol).await,
            #[cfg(feature = "xtb")]
            AnyBroker::Xtb(b) => b.is_market_open(symbol).await,
        }
    }

    async fn is_market_available(&mut self, symbol: &str) -> bool {
        match self {
            AnyBroker::Darwinex(b) => b.is_market_available(symbol).await,
            #[cfg(feature = "xtb")]
            AnyBroker::Xtb(b) => b.is_market_available(symbol).await,
        }
    }

    async fn open_trade(
        &mut self,
        trade: TradeData<TradeIn>,
        orders: Option<Vec<Order>>,
    ) -> Result<ResponseBody<TradeResponse<TradeIn>>> {
        match self {
            AnyBroker::Darwinex(b) => b.open_trade(trade, orders).await,
            #[cfg(feature = "xtb")]
            AnyBroker::Xtb(b) => b.open_trade(trade, orders).await,
        }
    }

    async fn close_trade(
        &mut self,
        trade: TradeData<TradeOut>,
    ) -> Result<ResponseBody<TradeResponse<TradeOut>>> {
        match self {
            AnyBroker::Darwinex(b) => b.close_trade(trade).await,
            #[cfg(feature = "xtb")]
            AnyBroker::Xtb(b) => b.close_trade(trade).await,
        }
    }

    async fn open_order(
        &mut self,
        trade: TradeData<TradeIn>,
        order: TradeData<Order>,
    ) -> Result<ResponseBody<TradeResponse<TradeIn>>> {
        match self {
            AnyBroker::Darwinex(b) => b.open_order(trade, order).await,
            #[cfg(feature = "xtb")]
            AnyBroker::Xtb(b) => b.open_order(trade, order).await,
        }
    }

    async fn close_order(
        &mut self,
        trade: TradeData<TradeOut>,
        order: TradeData<Order>,
    ) -> Result<ResponseBody<TradeResponse<TradeOut>>> {
        match self {
            AnyBroker::Darwinex(b) => b.close_order(trade, order).await,
            #[cfg(feature = "xtb")]
            AnyBroker::Xtb(b) => b.close_order(trade, order).await,
        }
    }

    async fn get_active_positions(
        &mut self,
        symbol: &str,
        strategy_name: &str,
    ) -> Result<ResponseBody<PositionResult>> {
        match self {
            AnyBroker::Darwinex(b) => b.get_active_positions(symbol, strategy_name).await,
            #[cfg(feature = "xtb")]
            AnyBroker::Xtb(b) => b.get_active_positions(symbol, strategy_name).await,
        }
    }

    async fn get_transaction_details(
        &mut self,
        symbol: &str,
        strategy_name: &str,
        id: Option<usize>,
    ) -> Option<TransactionDetails> {
        match self {
            AnyBroker::Darwinex(b) => b.get_transaction_details(symbol, strategy_name, id).await,
            #[cfg(feature = "xtb")]
            AnyBroker::Xtb(b) => b.get_transaction_details(symbol, strategy_name, id).await,
        }
    }

    async fn get_transactions_history(
        &mut self,
        symbol: &str,
        strategy_name: &str,
        id: Option<usize>,
    ) -> Option<TransactionDetails> {
        match self {
            AnyBroker::Darwinex(b) => b.get_transactions_history(symbol, strategy_name, id).await,
            #[cfg(feature = "xtb")]
            AnyBroker::Xtb(b) => b.get_transactions_history(symbol, strategy_name, id).await,
        }
    }

    async fn subscribe_stream(
        &mut self,
        symbol: &str,
        strategy_name: &str,
    ) -> Result<UnboundedReceiver<String>> {
        match self {
            AnyBroker::Darwinex(b) => b.subscribe_stream(symbol, strategy_name).await,
            #[cfg(feature = "xtb")]
            AnyBroker::Xtb(b) => b.subscribe_stream(symbol, strategy_name).await,
        }
    }
}
