pub mod models;
pub mod xtb;
pub mod xtb_stream;

use crate::error::Result;
use crate::helpers::date::{DateTime, Local};
use crate::ws::message::Message;
use models::*;

use futures_util::{
    stream::{SplitSink, SplitStream},
    Future,
};

use serde::{Deserialize, Serialize};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio_tungstenite::MaybeTlsStream;
use tokio_tungstenite::WebSocketStream;

#[async_trait::async_trait]
pub trait Broker {
    async fn new() -> Self;
    async fn login(&mut self, username: &str, password: &str) -> Result<&mut Self>
    where
        Self: Sized;
    async fn get_symbols(&mut self) -> Result<Response<VEC_DOHLC>>;
    async fn read(&mut self) -> Result<Response<VEC_DOHLC>>;
    fn get_session_id(&mut self) -> &String;
    async fn listen<F, T>(&mut self, symbol: &str, session_id: String, mut callback: F)
    where
        F: Send + FnMut(Message) -> T,
        T: Future<Output = Result<()>> + Send + 'static;
    async fn get_instrument_data(
        &mut self,
        symbol: &str,
        period: usize,
        start: i64,
    ) -> Result<Response<VEC_DOHLC>>;
    async fn get_tick_prices(
        &mut self,
        symbol: &str,
        level: usize,
        timestamp: i64,
    ) -> Result<String>;
}

#[async_trait::async_trait]
pub trait BrokerStream {
    async fn new() -> Self;
    async fn login(&mut self, username: &str, password: &str) -> Result<&mut Self>
    where
        Self: Sized;
    async fn get_symbols(&mut self) -> Result<Response<VEC_DOHLC>>;
    async fn read(&mut self) -> Result<Response<VEC_DOHLC>>;
    fn get_session_id(&mut self) -> &String;
    async fn listen<F, T>(&mut self, symbol: &str, session_id: String, mut callback: F)
    where
        F: Send + FnMut(Message) -> T,
        T: Future<Output = Result<()>> + Send + 'static;
    async fn get_instrument_data(
        &mut self,
        symbol: &str,
        period: usize,
        start: i64,
    ) -> Result<Response<VEC_DOHLC>>;
    async fn get_stream(&mut self) -> &mut SplitStream<WebSocketStream<MaybeTlsStream<TcpStream>>>;
    async fn get_instrument_streaming(
        &mut self,
        symbol: &str,
        minArrivalTime: usize,
        maxLevel: usize,
    ) -> Result<()>;
    async fn get_tick_prices(
        &mut self,
        symbol: &str,
        level: usize,
        timestamp: i64,
    ) -> Result<String>;
}
