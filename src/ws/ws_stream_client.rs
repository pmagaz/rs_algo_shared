use crate::error::Result;

use futures_util::{
    stream::{SplitSink, SplitStream},
    Future, SinkExt, StreamExt,
};
use tokio_tungstenite::{connect_async, MaybeTlsStream, WebSocketStream};

use std::env;
use tokio::net::TcpStream;
use tungstenite::Message;

#[derive(Debug)]
pub struct WebSocket {
    pub write: SplitSink<WebSocketStream<MaybeTlsStream<TcpStream>>, Message>,
    pub read: SplitStream<WebSocketStream<MaybeTlsStream<TcpStream>>>,
}

impl WebSocket {
    pub async fn connect(url: &str) -> Self {
        let (socket, response) = connect_async(url).await.expect("Can't connect");

        log::info!("Connected to the stream server");
        //log::info!("[STREAM] Response HTTP code: {}", response.status());

        let (write, read) = socket.split();
        Self { write, read }
    }

    pub async fn send(&mut self, msg: &str) -> Result<()> {
        self.write.send(Message::text(msg)).await.unwrap();
        Ok(())
    }

    pub async fn ping(&mut self, msg: &[u8]) {
        self.write.send(Message::Ping(msg.to_vec())).await.unwrap();
    }

    pub async fn disconnect(&mut self) -> Result<()> {
        self.write.close().await.unwrap();
        Ok(())
    }
}
