pub mod crypto;
pub mod forex;
pub mod sp500;

pub mod xtb;
use crate::error::Result;
use crate::helpers::websocket::MessageType;

use crate::helpers::date::{DateTime, Local};
use std::future::Future;

pub type DOHLC = (DateTime<Local>, f64, f64, f64, f64, f64);
pub type VEC_DOHLC = Vec<DOHLC>;

#[derive(Debug)]
pub struct Symbol {
    pub symbol: String,
    pub category: String,
    pub currency: String,
    pub description: String,
}

#[derive(Debug)]
pub struct Response<R> {
    pub msg_type: MessageType,
    // TODO move to Symbol
    pub symbol: String,
    pub data: R,
    // TODO move to Symbol
    pub symbols: Vec<Symbol>,
}

#[async_trait::async_trait]
pub trait Broker {
    async fn new() -> Self;
    async fn listen<F, T>(&mut self, mut callback: F)
    where
        F: Send + FnMut(Response<VEC_DOHLC>) -> T,
        T: Future<Output = Result<()>> + Send + 'static;
    async fn get_instrument_data(
        &mut self,
        symbol: &str,
        period: usize,
        start: i64,
    ) -> Result<Response<VEC_DOHLC>>;
    async fn get_symbols(&mut self) -> Result<Response<VEC_DOHLC>>;
    async fn login(&mut self, username: &str, password: &str) -> Result<()>
    where
        Self: Sized;
}
