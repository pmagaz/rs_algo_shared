use bson::Uuid;
#[cfg(feature = "websocket")]
pub use tungstenite::Message;

use crate::broker::{DOHLC, VEC_DOHLC};
use crate::models::bot::BotData;
use crate::models::market::MarketHours;
use crate::models::order::Order;
use crate::models::pricing::Pricing;
use crate::models::strategy::StrategyType;
use crate::models::time_frame::TimeFrameType;
use crate::models::trade::{TradeIn, TradeOut};

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub enum CommandType {
    InitSession,
    GetCurrentState,
    GetInstrumentData,
    GetInstrumentPricing,
    GetMarketHours,
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
    GetInstrumentPricing,
    GetMarketHours,
    ExecuteTradeIn,
    ExecuteTradeOut,
    InitSession,
    SubscribeStream,
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

#[derive(Debug, Serialize, Deserialize)]
pub struct TradeData<T> {
    pub symbol: String,
    pub data: T,
}

impl<T> TradeData<T> {
    pub fn new(symbol: &str, data: T) -> Self
    where
        for<'de> T: Serialize + Deserialize<'de>,
    {
        Self {
            symbol: symbol.to_string(),
            data,
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
pub struct PricingData {
    pub symbol: String,
    pub data: Pricing,
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
    pub volume: f64,
    pub timestamp: f64,
    pub spread: f64,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum MessageType {
    StreamResponse(ResponseBody<InstrumentData<DOHLC>>),
    InstrumentData(ResponseBody<InstrumentData<VEC_DOHLC>>),
    PricingData(ResponseBody<Pricing>),
    MarketHours(ResponseBody<MarketHours>),
    InitSession(ResponseBody<BotData>),
    ExecuteTradeIn(ResponseBody<TradeResponse<TradeIn>>),
    ExecuteTradeOut(ResponseBody<TradeResponse<TradeOut>>),
    ExecuteOrder(ResponseBody<TradeResponse<Order>>),
    Connected(ResponseBody<Uuid>),
    Reconnect(ResponseBody<bool>),
    Error(ResponseBody<bool>),
}
