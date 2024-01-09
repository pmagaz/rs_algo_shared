use std::fmt::{self, Display};
use thiserror::Error;

pub type Result<T> = ::anyhow::Result<T, RsAlgoError>;

#[derive(Copy, Clone, Eq, PartialEq, Debug, Error)]
pub enum RsAlgoErrorKind {
    #[error("Error while Parsing!")]
    ParseError,
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
    #[error("Can't send after disconect!")]
    SendingAfter,
    #[error("Can't read socket!")]
    CantRead,
    #[error("No response!")]
    NoResponse,
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

impl Display for RsAlgoError {
    fn fmt(&self, err: &mut fmt::Formatter<'_>) -> fmt::Result {
        Display::fmt(&self.err, err)
    }
}
