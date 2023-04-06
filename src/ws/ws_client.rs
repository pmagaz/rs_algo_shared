use crate::error::Result;
use crate::ws::message::*;

use std::net::TcpStream;
use tungstenite::stream::MaybeTlsStream;
use tungstenite::{connect, WebSocket as Ws};

#[derive(Debug)]
pub struct WebSocket {
    url: String,
    socket: Ws<MaybeTlsStream<TcpStream>>,
}

impl WebSocket {
    pub async fn connect(url: &str) -> Self {
        let (mut socket, response) = connect(url).expect("Can't connect");

        log::info!("[SOCKET] Connected to the server");
        log::info!("[SOCKET] Response HTTP code: {}", response.status());

        Self {
            url: url.to_string(),
            socket,
        }
    }

    pub async fn send(&mut self, msg: &str) -> Result<()> {
        self.socket.write_message(Message::text(msg)).unwrap();
        Ok(())
    }

    pub async fn re_connect(&mut self) {
        log::info!("[SOCKET] Reconnecting to the server");
        let url = self.url.to_owned();
        let (socket, _response) = connect(url).expect("Can't connect");
        self.socket = socket;
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

    pub async fn read_msg(
        &mut self,
    ) -> std::result::Result<tungstenite::Message, tungstenite::Error> {
        self.socket.read_message()
    }

    pub async fn disconnect(&mut self) -> Result<()> {
        log::info!("[SOCKET] Disconnecting from server!");
        self.socket.close(None).unwrap();
        Ok(())
    }
}
