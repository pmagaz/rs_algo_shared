#[cfg(feature = "websocket")]
pub use tungstenite::Message;

use crate::models::strategy::StrategyType;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub enum ResponseType {
    Connected,
    GetSymbolData,
    SubscribeStream,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum CommandType {
    GetSymbolData,
    SubscribeStream,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Data<'a> {
    pub strategy: &'a str,
    pub strategy_type: StrategyType,
    pub symbol: &'a str,
    pub time_frame: &'a str,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Command<T> {
    pub command: CommandType,
    pub data: Option<T>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Response<T> {
    pub command: ResponseType,
    pub data: Option<T>,
}
