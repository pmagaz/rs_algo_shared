use serde::Serialize;
use thiserror::Error;

pub type Result<T> = ::anyhow::Result<T, RsAlgoError>;

#[derive(Serialize)]
struct ErrorResponse {
    code: u16,
    error: String,
    message: String,
}

#[derive(Error, Debug)]
pub enum RsAlgoError {
    #[error("Requested user was not found")]
    NotFound,
    #[error("You ared forbidden to access this resource.")]
    Forbidden,
    #[error("Unknown Internal Error")]
    Unknown,
    #[error("No user found")]
    NoUserFound,
    #[error("Can't connect to database")]
    NoDbConnection,
    #[error("Invalid Token")]
    InvalidToken,
}

impl RsAlgoError {
    pub fn name(&self) -> String {
        match self {
            Self::NotFound => "NotFound".to_string(),
            Self::Forbidden => "Forbidden".to_string(),
            Self::Unknown => "Unknown".to_string(),
            Self::NoUserFound => "NoUserFound".to_string(),
            Self::NoDbConnection => "NoDbConnection".to_string(),
            Self::InvalidToken => "InvalidToken".to_string(),
        }
    }
}

pub fn map_io_error(e: std::io::Error) -> RsAlgoError {
    match e.kind() {
        std::io::ErrorKind::NotFound => RsAlgoError::NotFound,
        std::io::ErrorKind::PermissionDenied => RsAlgoError::Forbidden,
        _ => RsAlgoError::Unknown,
    }
}
