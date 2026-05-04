use crate::helpers::date::{DateTime, Local};
use crate::models::time_frame::*;
use serde::{Deserialize, Serialize};

pub type DOHLC = (DateTime<Local>, f64, f64, f64, f64, f64);
pub type VEC_DOHLC = Vec<DOHLC>;
pub type LECHES = (f64, f64, f64, f64, f64, f64);
pub type VEC_LECHES = Vec<LECHES>;

// Generic order direction — broker-agnostic semantics

#[derive(Debug, Serialize, Deserialize)]
pub enum TransactionCommand {
    BuyMarket,
    SellMarket,
    BuyLimit,
    SellLimit,
    BuyStop,
    SellStop,
    Balance,
    Credit,
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

    pub fn from_value(value: i64) -> Option<Self> {
        match value {
            0 => Some(TransactionCommand::BuyMarket),
            1 => Some(TransactionCommand::SellMarket),
            2 => Some(TransactionCommand::BuyLimit),
            3 => Some(TransactionCommand::SellLimit),
            4 => Some(TransactionCommand::BuyStop),
            5 => Some(TransactionCommand::SellStop),
            6 => Some(TransactionCommand::Balance),
            7 => Some(TransactionCommand::Credit),
            _ => None,
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
    pub fn from_value(value: u64) -> Self {
        match value {
            0 => TransactionState::Error,
            1 => TransactionState::Pending,
            3 => TransactionState::Accepted,
            4 => TransactionState::Rejected,
            _ => TransactionState::Error,
        }
    }

    pub fn is_accepted(&self) -> bool {
        matches!(self, TransactionState::Accepted)
    }

    pub fn is_pending(&self) -> bool {
        matches!(self, TransactionState::Pending)
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

// Generic transaction result types

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
    pub profit: f64,
}

// Legacy types used by the old basic XTB Broker trait
#[cfg(feature = "xtb")]
#[derive(Debug, Serialize, Deserialize)]
pub enum MessageType {
    Login,
    GetSymbols,
    GetInstrumentPrice,
    Other,
}

#[cfg(feature = "xtb")]
#[derive(Debug, Serialize, Deserialize)]
pub struct Response<R> {
    pub msg_type: MessageType,
    pub symbol: String,
    pub time_frame: TimeFrameType,
    pub data: R,
    pub symbols: Vec<crate::broker::xtb_models::Symbol>,
}
