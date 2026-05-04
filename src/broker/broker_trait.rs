use crate::broker::models::{TransactionDetails, VEC_DOHLC};
use crate::error::Result;
use crate::models::market::MarketHours;
use crate::models::order::Order;
use crate::models::swap::InstrumentSwap;
use crate::models::tick::InstrumentTick;
use crate::models::trade::{PositionResult, TradeIn, TradeOut};
use crate::ws::message::{InstrumentData, ResponseBody, TradeData, TradeResponse};

use tokio::sync::mpsc::UnboundedReceiver;

/// Broker-agnostic async trait for all broker operations.
///
/// Each broker (Darwinex, XTB, etc.) provides its own impl.
/// Connection model (single vs dual WS, REST vs WS) is an internal detail.
///
/// Note: native async fn in traits (Rust 1.75+) are used here.
/// Object safety for `Box<dyn BrokerStream>` is handled via `AnyBroker`
/// enum dispatch in the broker factory (see broker/mod.rs Step 5).
#[async_trait::async_trait]
pub trait BrokerStream: Send + Sync {
    async fn new() -> Self
    where
        Self: Sized;

    async fn login(&mut self, username: &str, password: &str) -> Result<&mut Self>
    where
        Self: Sized;

    async fn disconnect(&mut self) -> Result<()>;

    /// Send a keepalive heartbeat to the broker connection.
    async fn keepalive_ping(&mut self) -> Result<()>;

    // ── Market data ───────────────────────────────────────────────────────

    async fn get_instrument_data(
        &mut self,
        symbol: &str,
        period: usize,
        start: i64,
    ) -> Result<ResponseBody<InstrumentData<VEC_DOHLC>>>;

    async fn get_historic_data(
        &mut self,
        symbol: &str,
        period: usize,
        start: i64,
        end: i64,
    ) -> Result<ResponseBody<InstrumentData<VEC_DOHLC>>>;

    async fn get_instrument_tick(&mut self, symbol: &str) -> Result<ResponseBody<InstrumentTick>>;

    async fn get_instrument_swap(&mut self, symbol: &str) -> Result<ResponseBody<InstrumentSwap>>;

    async fn get_ask_bid(&mut self, symbol: &str) -> Result<(f64, f64)>;

    async fn get_symbols(&mut self) -> Result<ResponseBody<InstrumentData<VEC_DOHLC>>>;

    // ── Market status ─────────────────────────────────────────────────────

    async fn get_market_hours(&mut self, symbol: &str) -> Result<ResponseBody<MarketHours>>;

    async fn is_market_open(&mut self, symbol: &str) -> Result<ResponseBody<bool>>;

    async fn is_market_available(&mut self, symbol: &str) -> bool;

    // ── Trading ───────────────────────────────────────────────────────────

    async fn open_trade(
        &mut self,
        trade: TradeData<TradeIn>,
        orders: Option<Vec<Order>>,
    ) -> Result<ResponseBody<TradeResponse<TradeIn>>>;

    async fn close_trade(
        &mut self,
        trade: TradeData<TradeOut>,
    ) -> Result<ResponseBody<TradeResponse<TradeOut>>>;

    async fn open_order(
        &mut self,
        trade: TradeData<TradeIn>,
        order: TradeData<Order>,
    ) -> Result<ResponseBody<TradeResponse<TradeIn>>>;

    async fn close_order(
        &mut self,
        trade: TradeData<TradeOut>,
        order: TradeData<Order>,
    ) -> Result<ResponseBody<TradeResponse<TradeOut>>>;

    // ── Positions & history ───────────────────────────────────────────────

    async fn get_active_positions(
        &mut self,
        symbol: &str,
        strategy_name: &str,
    ) -> Result<ResponseBody<PositionResult>>;

    async fn get_transaction_details(
        &mut self,
        symbol: &str,
        strategy_name: &str,
        id: Option<usize>,
    ) -> Option<TransactionDetails>;

    async fn get_transactions_history(
        &mut self,
        symbol: &str,
        strategy_name: &str,
        id: Option<usize>,
    ) -> Option<TransactionDetails>;

    // ── Streaming ─────────────────────────────────────────────────────────

    /// Subscribe to the real-time price/candle stream for `symbol`.
    ///
    /// The broker internally manages its WebSocket read loop and sends
    /// pre-parsed `ResponseBody` JSON strings to the returned channel.
    /// The server reads from this receiver and forwards to the bot — no
    /// tungstenite types are exposed to callers.
    ///
    /// `strategy_name` is used by broker implementations that need to
    /// filter stream events by strategy (e.g. XTB trade stop-loss events).
    async fn subscribe_stream(
        &mut self,
        symbol: &str,
        strategy_name: &str,
    ) -> Result<UnboundedReceiver<String>>;
}
