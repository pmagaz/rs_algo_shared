use crate::error::{Result, RsAlgoError, RsAlgoErrorKind};
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
    pub fn connect(url: &str) -> Self {
        let (socket, _) = connect(url).expect("Can't connect");
        log::info!("Connected to {} !", &url);
        Self {
            url: url.to_string(),
            socket,
        }
    }

    pub async fn send(&mut self, msg: &str) -> Result<()> {
        match self.socket.write_message(Message::text(msg)) {
            Ok(_) => Ok(()),
            Err(err) => {
                log::error!("Error sending message: {}", err);
                Err(RsAlgoError {
                    err: RsAlgoErrorKind::SendingAfter,
                })
            }
        }
    }

    pub async fn re_connect(&mut self) {
        log::info!("Reconnecting to the server...");
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

    pub async fn read(&mut self) -> std::result::Result<tungstenite::Message, tungstenite::Error> {
        match self.socket.read_message() {
            Ok(msg) => Ok(msg),
            Err(err) => {
                log::error!("Error reading msg: {}", err);
                Err(err)
            }
        }
    }

    pub async fn send_and_read(
        &mut self,
        msg: &str,
    ) -> std::result::Result<tungstenite::Message, tungstenite::Error> {
        self.socket
            .write_message(Message::text(msg))
            .map_err(|err| {
                log::error!("Error sending message: {}", err);
                RsAlgoError {
                    err: RsAlgoErrorKind::SendingAfter,
                }
            })
            .unwrap();

        match self.socket.read_message() {
            Ok(msg) => Ok(msg),
            Err(err) => {
                log::error!("Error reading msg: {}", err);
                Err(err)
            }
        }
    }

    pub async fn disconnect(&mut self) -> Result<()> {
        self.socket.close(None).unwrap();
        Ok(())
    }
}
