use crate::helpers::date::{DateTime, Local};
use crate::models::time_frame::*;
use crate::models::trade::TradeType;
use serde::{Deserialize, Serialize};

pub type DOHLC = (DateTime<Local>, f64, f64, f64, f64, f64);
pub type VEC_DOHLC = Vec<DOHLC>;
pub type LECHES = (f64, f64, f64, f64, f64, f64);
pub type VEC_LECHES = Vec<LECHES>;

#[derive(Debug, Serialize, Deserialize)]
pub enum TransactionCommand {
    BuyMarket,  // Execute a trade to buy immediately at current market price
    SellMarket, // Execute a trade to sell immediately at current market price
    BuyLimit,   // Set an order to buy at a specified or lower price
    SellLimit,  // Set an order to sell at a specified or higher price
    BuyStop,    // Place an order to buy when the price rises to a specified point
    SellStop,   // Place an order to sell when the price falls to a specified point
    Balance,    // Read-only: Check account balance, typically for trade history
    Credit,     // Read-only: Check credit information in the account
}

impl TransactionCommand {
    pub fn value(&self) -> isize {
        match self {
            TransactionCommand::BuyMarket => 0,
            TransactionCommand::SellMarket => 1,
            TransactionCommand::BuyLimit => 2,
            TransactionCommand::SellLimit => 3,
            TransactionCommand::BuyStop => 4,
            TransactionCommand::SellStop => 5,
            TransactionCommand::Balance => 6,
            TransactionCommand::Credit => 7,
        }
    }

    pub fn from_value(value: i64) -> Option<TransactionCommand> {
        match value {
            0 => Some(TransactionCommand::BuyMarket),
            1 => Some(TransactionCommand::SellMarket),
            2 => Some(TransactionCommand::BuyLimit),
            3 => Some(TransactionCommand::SellLimit),
            4 => Some(TransactionCommand::BuyStop),
            5 => Some(TransactionCommand::SellStop),
            6 => Some(TransactionCommand::Balance),
            7 => Some(TransactionCommand::Credit),
            _ => None, // Invalid value
        }
    }

    pub fn is_entry(&self) -> bool {
        matches!(
            self,
            TransactionCommand::BuyMarket
                | TransactionCommand::SellMarket
                | TransactionCommand::BuyLimit
                | TransactionCommand::SellLimit
                | TransactionCommand::BuyStop
                | TransactionCommand::SellStop
        )
    }

    pub fn is_exit(&self) -> bool {
        matches!(
            self,
            TransactionCommand::Balance | TransactionCommand::Credit
        )
    }

    pub fn is_stop(&self) -> bool {
        matches!(
            self,
            TransactionCommand::BuyStop | TransactionCommand::SellStop
        )
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub enum TransactionAction {
    Open,
    Pending,
    Close,
    Modify,
    Delete,
}

impl TransactionAction {
    pub fn value(&self) -> isize {
        match self {
            TransactionAction::Open => 0,
            TransactionAction::Pending => 1,
            TransactionAction::Close => 2,
            TransactionAction::Modify => 3,
            TransactionAction::Delete => 4,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub enum TransactionState {
    Error,
    Pending,
    Accepted,
    Rejected,
}

impl TransactionState {
    pub fn from_value(value: u64) -> TransactionState {
        match value {
            0 => TransactionState::Error,
            1 => TransactionState::Pending,
            3 => TransactionState::Accepted,
            4 => TransactionState::Rejected,
            _ => panic!(), // Return None for values that don't map to a TransactionStatus
        }
    }
    pub fn is_accepted(&self) -> bool {
        match self {
            TransactionState::Accepted => true,
            _ => false,
        }
    }

    pub fn is_pending(&self) -> bool {
        match self {
            TransactionState::Pending => true,
            _ => false,
        }
    }
    pub fn value(&self) -> u64 {
        match self {
            TransactionState::Error => 0,
            TransactionState::Pending => 1,
            TransactionState::Accepted => 3,
            TransactionState::Rejected => 4,
        }
    }
}

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
pub struct Ping {
    pub command: String,
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
pub struct HistoricInstrument {
    pub info: HistoricInstrumentCandles,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct InstrumentCandles {
    pub period: usize,
    pub start: i64,
    pub symbol: String,
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
pub struct TickParams {
    pub level: usize,
    pub symbols: Vec<String>,
    pub timestamp: i64,
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
pub struct TradingHoursCommand {
    pub symbols: Vec<String>,
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

#[derive(Debug, Serialize, Deserialize)]
pub struct TransactionStatusnResponse {
    pub comment: String,
    pub message: String,
    pub order: u64,
    pub ask: f64,
    pub bid: f64,
    pub status: TransactionState,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TransactionDetails {
    pub id: usize,
    pub open_price: f64,
    pub close_price: f64,
    // pub open_date: usize,
    // pub close_date: usize,
}

//TO PUT HERE index_in, ask, spread, trade_type
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
