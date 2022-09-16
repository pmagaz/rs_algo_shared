use crate::error::Result;
use std::time::Duration;
use std::net::TcpStream;
use tungstenite::stream::MaybeTlsStream;
pub use tungstenite::Message;
use tungstenite::{connect, WebSocket as Ws};

#[derive(Debug)]
pub enum MessageType {
    Login,
    GetSymbols,
    GetInstrumentPrice,
    Other,
}

#[derive(Debug)]
pub struct WebSocket {
    socket: Ws<MaybeTlsStream<TcpStream>>,
}

impl WebSocket {
    pub async fn connect(url: &str) -> Self {
        let (mut socket, response) = connect(url).expect("Can't connect");
    // match socket.get_mut() {
    //     MaybeTlsStream::NativeTls(t) => {

    //         println!("22222222 {:?}", t);
    //         // -- use either one or another
    //         //t.get_mut().set_nonblocking(true);
    //         t.get_mut().set_read_timeout(Some(Duration::from_millis(100))).expect("Error: cannot set read-timeout to underlying stream");
    //     },
    //     // handle more cases as necessary, this one only focuses on native-tls
    //     _ => panic!("Error: it is not TlsStream")
    // }
        println!("[SOCKET] Connected to the server");
        println!("[SOCKET] Response HTTP code: {}", response.status());
        println!("[SOCKET] Response contains the following headers:");
          for (ref header, header_value) in response.headers() {
            println!("* {}: {:?}", header, header_value);
        }

        Self { socket }
    }

    pub async fn send(&mut self, msg: &str) -> Result<()> {
        self.socket.write_message(Message::text(msg)).unwrap();
        Ok(())
    }

    pub async fn read(&mut self) -> Result<Message> {
        let msg = self
            .socket
            .read_message()
            .expect("SOCKET] Error reading message");
        Ok(msg)
    }
}
