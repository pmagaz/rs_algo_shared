#[cfg(feature = "websocket")]
pub use tungstenite::Message;

use crate::broker::{VEC_DOHLC, VEC_LECHES};
use crate::models::strategy::StrategyType;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub enum ResponseType {
    Connected,
    Error,
    GetSymbolData,
    SubscribeStream,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum CommandType {
    GetSymbolData,
    SubscribeStream,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Command<T> {
    pub command: CommandType,
    pub data: Option<T>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Data<'a> {
    pub strategy: &'a str,
    pub strategy_type: StrategyType,
    pub symbol: &'a str,
    pub time_frame: &'a str,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SymbolData<T> {
    pub symbol: String,
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
    StreamResponse(ResponseBody<SymbolData<VEC_LECHES>>),
    DataResponse(ResponseBody<SymbolData<VEC_DOHLC>>),
    Connected(ResponseBody<bool>),
    Error(ResponseBody<bool>),
}
