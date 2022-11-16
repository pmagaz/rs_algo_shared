use bson::Uuid;
#[cfg(feature = "websocket")]
pub use tungstenite::Message;

use crate::broker::{LECHES, VEC_DOHLC};
use crate::models::strategy::StrategyType;
use crate::models::time_frame::TimeFrameType;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub enum ResponseType {
    Connected,
    Error,
    GetInstrumentData,
    ExecuteTrade,
    //GetHigherTMInstrumentData,
    SubscribeStream,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum CommandType {
    InitSession,
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Payload<'a> {
    pub symbol: &'a str,
    pub strategy: &'a str,
    pub strategy_type: StrategyType,
    pub time_frame: TimeFrameType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionData {
    #[serde(rename = "_id")]
    pub id: Uuid,
    pub strategy: String,
    pub strategy_type: StrategyType,
    pub symbol: String,
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
pub struct ResponseBody<T> {
    pub response: ResponseType,
    pub payload: Option<T>,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum Response {
    StreamResponse(ResponseBody<InstrumentData<LECHES>>),
    InstrumentData(ResponseBody<InstrumentData<VEC_DOHLC>>),
    //HigherTMInstrumentData(ResponseBody<InstrumentData<VEC_DOHLC>>),
    Connected(ResponseBody<Uuid>),
    Error(ResponseBody<bool>),
}
