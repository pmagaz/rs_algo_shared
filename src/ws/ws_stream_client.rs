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

        log::info!("[STREAM] Connected to the server");
        log::info!("[STREAM] Response HTTP code: {}", response.status());

        let (write, read) = socket.split();
        Self { write, read }
    }

    // async fn listen<F, T>(username: &str, password: &str, mut callback: F)
    // where
    //     F: Send + FnMut(Message) -> T,
    //     T: Future<Output = Result<()>> + Send + 'static,
    // {
    //     let url = &env::var("BROKER_STREAM_URL").unwrap();
    //     let (mut ws_stream, _) = connect_async(url).await.expect("Can't connect");
    //     while let Some(msg) = ws_stream.next().await {
    //         let msg = msg.unwrap();
    //         tokio::spawn(callback(msg));
    //         // if msg.is_text() || msg.is_binary() {
    //         //     ws_stream.send(msg).await.unwrap();
    //         // }
    //     }
    // }

    pub async fn send(&mut self, msg: &str) -> Result<()> {
        self.write.send(Message::text(msg)).await.unwrap();
        Ok(())
    }

    pub async fn ping(&mut self, msg: &[u8]) {
        self.write.send(Message::Ping(msg.to_vec())).await.unwrap();
    }
}
