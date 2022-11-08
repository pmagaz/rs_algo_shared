use crate::error::Result;
use crate::ws::message::*;

use std::net::TcpStream;
use std::time::Duration;
use tungstenite::stream::MaybeTlsStream;
use tungstenite::{connect, WebSocket as Ws};

#[derive(Debug)]
pub struct WebSocket {
    socket: Ws<MaybeTlsStream<TcpStream>>,
}

impl WebSocket {
    pub async fn connect(url: &str) -> Self {
        let (mut socket, response) = connect(url).expect("Can't connect");

        log::info!("[SOCKET] Connected to the server");
        log::info!("[SOCKET] Response HTTP code: {}", response.status());

        Self { socket }
    }

    pub async fn send(&mut self, msg: &str) -> Result<()> {
        self.socket.write_message(Message::text(msg)).unwrap();
        Ok(())
    }

    pub fn send2(&mut self, msg: &str) {
        self.socket.write_message(Message::text(msg)).unwrap();
    }

    pub async fn ping(&mut self, msg: &[u8]) {
        self.socket
            .write_message(Message::Ping(msg.to_vec()))
            .unwrap();
    }

    pub async fn pong(&mut self, msg: &[u8]) {
        self.socket
            .write_message(Message::Pong(msg.to_vec()))
            .unwrap();
    }

    pub async fn read(&mut self) -> Result<Message> {
        let msg = self.socket.read_message().unwrap();
        Ok(msg)
    }

    pub fn read2(&mut self) -> Result<Message> {
        let msg = self
            .socket
            .read_message()
            .expect("SOCKET] Error reading message");
        Ok(msg)
    }
}

// impl Default for WebSocket {
//     fn default() -> Self {
//         Self::new()
//     }
// }
