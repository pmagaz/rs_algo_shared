use crate::error::Result;

use futures_util::{
    stream::{SplitSink, SplitStream},
    SinkExt, StreamExt,
};
use tokio::net::TcpStream;
use tokio_tungstenite::{connect_async, MaybeTlsStream, WebSocketStream};
use tungstenite::Message;

#[derive(Debug)]
pub struct WebSocket {
    pub write: SplitSink<WebSocketStream<MaybeTlsStream<TcpStream>>, Message>,
    /// `Option` so the read half can be moved into a streaming task via `take_read()`.
    pub read: Option<SplitStream<WebSocketStream<MaybeTlsStream<TcpStream>>>>,
}

impl WebSocket {
    pub async fn connect(url: &str) -> Self {
        let (socket, _response) = connect_async(url).await.expect("Can't connect");
        log::info!("Connected to stream server: {}", url);
        let (write, read) = socket.split();
        Self {
            write,
            read: Some(read),
        }
    }

    pub async fn send(&mut self, msg: &str) -> Result<()> {
        self.write.send(Message::text(msg)).await.unwrap();
        Ok(())
    }

    pub async fn ping(&mut self, msg: &[u8]) {
        self.write.send(Message::Ping(msg.to_vec())).await.unwrap();
    }

    /// Take ownership of the read half so it can be moved into a tokio task.
    /// Panics if called more than once (stream already consumed).
    pub fn take_read(&mut self) -> SplitStream<WebSocketStream<MaybeTlsStream<TcpStream>>> {
        self.read.take().expect("stream read half already consumed")
    }

    pub async fn disconnect(&mut self) -> Result<()> {
        self.write.close().await.unwrap();
        Ok(())
    }
}
