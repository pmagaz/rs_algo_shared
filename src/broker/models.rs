use crate::helpers::date::{DateTime, Local};
use crate::models::time_frame::*;
use serde::{Deserialize, Serialize};

pub type DOHLC = (DateTime<Local>, f64, f64, f64, f64, f64);
pub type VEC_DOHLC = Vec<DOHLC>;
pub type LECHES = (DateTime<Local>, f64, f64, f64, f64, f64, f64);
pub type VEC_LECHES = Vec<LECHES>;

#[derive(Debug, Serialize, Deserialize)]
pub enum MessageType {
    Login,
    GetSymbols,
    GetInstrumentPrice,
    Other,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Symbol {
    pub symbol: String,
    pub category: String,
    pub currency: String,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SymbolDetail {
    pub symbol: String,
    pub time: f64,
    pub ask: isize,
    pub bid: isize,
    pub contractSize: isize,
    pub leverage: f64,
    pub high: f64,
    pub low: f64,
    pub longOnly: bool,
    pub shortSelling: bool,
    pub swapLong: usize,
    pub swapShort: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SymbolDetailResponse {
    pub status: bool,
    pub returnData: SymbolDetail,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Response<R> {
    pub msg_type: MessageType,
    pub symbol: String,
    pub time_frame: TimeFrameType,
    pub data: R,
    pub symbols: Vec<Symbol>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Command<T> {
    pub command: String,
    pub arguments: T,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SymbolArg {
    pub symbol: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CommandAllSymbols {
    pub command: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CommandStreaming {
    pub command: String,
    pub streamSessionId: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CommandGetCandles {
    pub command: String,
    pub streamSessionId: String,
    pub symbol: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LoginParams {
    pub userId: String,
    pub password: String,
    pub appName: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LoginResponse {
    pub status: bool,
    pub streamSessionId: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TickerPriceParams {
    pub command: String,
    pub streamSessionId: String,
    pub symbol: String,
    pub minArrivalTime: usize,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Instrument {
    pub info: InstrumentCandles,
}
#[derive(Debug, Serialize, Deserialize)]
pub struct InstrumentCandles {
    pub period: usize,
    pub start: i64,
    pub symbol: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TickParams {
    pub level: usize,
    pub symbols: Vec<String>,
    pub timestamp: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CommandGetTickPrices {
    pub command: String,
    pub streamSessionId: String,
    pub symbol: String,
    pub minArrivalTime: usize,
    pub maxLevel: usize,
}
