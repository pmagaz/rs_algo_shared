use bson::Uuid;
#[cfg(feature = "websocket")]
pub use tungstenite::Message;

use crate::broker::{DOHLC, VEC_DOHLC};
use crate::models::bot::BotData;
use crate::models::strategy::StrategyType;
use crate::models::time_frame::TimeFrameType;
use crate::models::trade::{TradeIn, TradeOut};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub enum CommandType {
    InitSession,
    GetCurrentState,
    GetInstrumentData,
    UpdateBotData,
    ExecuteTrade,
    //GetHigherTMInstrumentData,
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
    GetInstrumentData,
    ExecuteTradeIn,
    ExecuteTradeOut,
    InitSession,
    //GetHigherTMInstrumentData,
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

#[derive(Debug, Serialize, Deserialize)]
pub struct InstrumentData<T> {
    pub symbol: String,
    pub time_frame: TimeFrameType,
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
    pub volume: f64,
    pub timestamp: f64,
    pub spread: f64,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum MessageType {
    StreamResponse(ResponseBody<InstrumentData<DOHLC>>),
    InstrumentData(ResponseBody<InstrumentData<VEC_DOHLC>>),
    InitSession(ResponseBody<BotData>),
    ExecuteTradeIn(ResponseBody<TradeIn>),
    ExecuteTradeOut(ResponseBody<TradeOut>),
    //HigherTMInstrumentData(ResponseBody<InstrumentData<VEC_DOHLC>>),
    Connected(ResponseBody<Uuid>),
    Error(ResponseBody<bool>),
}
