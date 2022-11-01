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
    //GetHigherTMInstrumentData,
    SubscribeStream,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum CommandType {
    GetInstrumentData,
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
pub struct SymbolData<T> {
    pub symbol: String,
    pub time_frame: TimeFrameType,
    pub data: T,
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
    pub data: Option<T>,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum Response {
    StreamResponse(ResponseBody<SymbolData<LECHES>>),
    InstrumentData(ResponseBody<SymbolData<VEC_DOHLC>>),
    //HigherTMInstrumentData(ResponseBody<SymbolData<VEC_DOHLC>>),
    Connected(ResponseBody<bool>),
    Error(ResponseBody<bool>),
}
