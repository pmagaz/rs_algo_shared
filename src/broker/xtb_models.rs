use crate::models::time_frame::TimeFrameType;
use crate::models::trade::TradeType;
use serde::{Deserialize, Serialize};

// XTB login protocol structs

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

// XTB command wrappers

#[derive(Debug, Serialize, Deserialize)]
pub struct Command<T> {
    pub command: String,
    pub arguments: T,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Ping {
    pub command: String,
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
pub struct CommandTickStreamParams {
    pub command: String,
    pub streamSessionId: String,
    pub symbol: String,
    pub minArrivalTime: usize,
    pub maxLevel: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CommandTradeStatusParams {
    pub command: String,
    pub streamSessionId: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CommandGetTickPrices {
    pub command: String,
    pub streamSessionId: String,
    pub symbol: String,
    pub minArrivalTime: usize,
    pub maxLevel: usize,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SymbolArg {
    pub symbol: String,
}

// XTB instrument request structs

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
pub struct HistoricInstrument {
    pub info: HistoricInstrumentCandles,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct HistoricInstrumentCandles {
    pub period: usize,
    pub start: i64,
    pub end: i64,
    pub ticks: i64,
    pub symbol: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TradingHoursCommand {
    pub symbols: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TickParams {
    pub level: usize,
    pub symbols: Vec<String>,
    pub timestamp: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TickerPriceParams {
    pub command: String,
    pub streamSessionId: String,
    pub symbol: String,
    pub minArrivalTime: usize,
}

// XTB trade execution structs

#[derive(Debug, Serialize, Deserialize)]
pub struct TradeTransactionInfo {
    pub cmd: isize,
    pub customComment: String,
    pub symbol: String,
    pub expiration: i64,
    pub order: isize,
    pub offset: i64,
    pub price: f64,
    pub sl: f64,
    pub tp: f64,
    pub volume: f64,
    #[serde(rename = "type")]
    pub trans_type: isize,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TransactionInfo {
    pub tradeTransInfo: TradeTransactionInfo,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TransactionStatus {
    pub order: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GetTrades {
    pub openedOnly: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GetTradesHistory {
    pub start: i64,
    pub end: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GetTrade {
    pub orders: Vec<usize>,
}

// XTB instrument tick (camelCase = XTB protocol)

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SymbolInstrumentTick {
    pub symbol: String,
    pub time: f64,
    pub ask: f64,
    pub bid: f64,
    pub contractSize: isize,
    pub leverage: f64,
    pub high: f64,
    pub low: f64,
    pub spreadRaw: f64,
    pub spreadTable: f64,
    pub longOnly: bool,
    pub shortSelling: bool,
    pub swapLong: f64,
    pub swapShort: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SymbolInstrumentTickResponse {
    pub status: bool,
    pub returnData: SymbolInstrumentTick,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Symbol {
    pub symbol: String,
    pub category: String,
    pub currency: String,
    pub description: String,
}

// XTB comments embedded in trade customComment field

#[derive(Debug, Serialize, Deserialize)]
pub struct TransactionComments {
    pub strategy_name: String,
    pub index_in: usize,
    pub sell_order_price: Option<f64>,
    pub stop_loss_order_price: Option<f64>,
    pub bid: f64,
    pub spread: f64,
    pub trade_type: TradeType,
}
