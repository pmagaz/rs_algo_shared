use std::fmt::{self, Display};
use thiserror::Error;

pub type Result<T> = ::anyhow::Result<T, RsAlgoError>;

#[derive(Copy, Clone, Eq, PartialEq, Debug, Error)]
pub enum RsAlgoErrorKind {
    #[error("Invalid Candle!")]
    InvalidCandle,
    #[error("Invalid Instrument!")]
    WrongInstrumentConf,
    #[error("Invalid Peak!")]
    InvalidPeak,
    #[error("Error on Request!")]
    RequestError,
    #[error("Connection Error!")]
    ConnectionError,
}

#[derive(Debug, Error)]
pub struct RsAlgoError {
    pub err: RsAlgoErrorKind,
}

impl RsAlgoError {
    pub fn kind(&self) -> RsAlgoErrorKind {
        self.err
    }
}

impl From<RsAlgoErrorKind> for RsAlgoError {
    fn from(kind: RsAlgoErrorKind) -> RsAlgoError {
        RsAlgoError { err: kind }
    }
}

impl From<tungstenite::Error> for RsAlgoError {
    fn from(error: tungstenite::Error) -> Self {
        log::error!("Error sending {:?}", error);
        RsAlgoError {
            err: RsAlgoErrorKind::ConnectionError,
        }
    }
}

impl Display for RsAlgoError {
    fn fmt(&self, err: &mut fmt::Formatter<'_>) -> fmt::Result {
        Display::fmt(&self.err, err)
    }
}
