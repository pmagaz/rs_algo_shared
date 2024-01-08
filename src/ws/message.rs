use bson::Uuid;
#[cfg(feature = "websocket")]
pub use tungstenite::Message;

use crate::broker::{DOHLC, VEC_DOHLC};
use crate::models::bot::BotData;
use crate::models::market::MarketHours;
use crate::models::order::Order;
use crate::models::strategy::StrategyType;
use crate::models::tick::InstrumentTick;
use crate::models::time_frame::TimeFrameType;
use crate::models::trade::{PositionResult, TradeIn, TradeOut, TradeResult};

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub enum CommandType {
    InitSession,
    GetCurrentState,
    GetInstrumentData,
    GetHistoricData,
    GetInstrumentTick,
    GetActivePositions,
    GetMarketHours,
    IsMarketOpen,
    UpdateBotData,
    ExecuteTrade,
    ExecutePosition,
    SubscribeStream,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Command<T> {
    pub command: CommandType,
    pub data: Option<T>,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum ResponseType {
    Connected,
    Error,
    Reconnect,
    GetInstrumentData,
    GetInstrumentTick,
    GetMarketHours,
    GetActivePositions,
    IsMarketOpen,
    TradeInFulfilled,
    TradeOutFulfilled,
    InitSession,
    SubscribeStream,
    SubscribeTickPrices,
    SubscribeTrades,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ResponseBody<T> {
    pub response: ResponseType,
    pub payload: Option<T>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Payload<'a> {
    pub symbol: &'a str,
    pub strategy: &'a str,
    pub strategy_type: StrategyType,
    pub time_frame: TimeFrameType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstrumentDataPayload<'a> {
    pub symbol: &'a str,
    pub strategy: &'a str,
    pub num_bars: i64,
    pub strategy_type: StrategyType,
    pub time_frame: TimeFrameType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoricDataPayload<'a> {
    pub symbol: &'a str,
    pub strategy: &'a str,
    pub limit: i64,
    pub strategy_type: StrategyType,
    pub time_frame: TimeFrameType,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct InstrumentData<T> {
    pub symbol: String,
    pub time_frame: TimeFrameType,
    pub data: T,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Symbol {
    pub symbol: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TradeOptions {
    pub non_profitable_out: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TradeData<T> {
    pub symbol: String,
    pub strategy_name: String,
    pub data: T,
    pub options: TradeOptions,
}

impl<T> TradeData<T> {
    pub fn new(symbol: &str, strategy_name: &str, data: T, options: TradeOptions) -> Self
    where
        for<'de> T: Serialize + Deserialize<'de>,
    {
        Self {
            symbol: symbol.to_string(),
            strategy_name: strategy_name.to_string(),
            data,
            options: options,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TradeResponse<T> {
    pub symbol: String,
    pub accepted: bool,
    pub data: T,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ConnectedData {
    pub session_id: Uuid,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct StreamResponse {
    pub symbol: String,
    pub ask: f64,
    pub bid: f64,
    pub high: f64,
    pub low: f64,
    pub size: f64,
    pub timestamp: f64,
    pub spread: f64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ReconnectOptions {
    pub clean_data: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum MessageType {
    StreamResponse(ResponseBody<InstrumentData<DOHLC>>),
    StreamTickResponse(ResponseBody<InstrumentTick>),
    StreamTradesResponse(ResponseBody<TradeResult>),
    InstrumentData(ResponseBody<InstrumentData<VEC_DOHLC>>),
    InstrumentTick(ResponseBody<InstrumentTick>),
    ActivePositions(ResponseBody<PositionResult>),
    MarketHours(ResponseBody<MarketHours>),
    IsMarketOpen(ResponseBody<bool>),
    InitSession(ResponseBody<BotData>),
    TradeInFulfilled(ResponseBody<TradeResponse<TradeIn>>),
    TradeOutFulfilled(ResponseBody<TradeResponse<TradeOut>>),
    ExecuteOrder(ResponseBody<TradeResponse<Order>>),
    Connected(ResponseBody<Uuid>),
    Reconnect(ResponseBody<ReconnectOptions>),
    Error(ResponseBody<bool>),
}
