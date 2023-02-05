use super::*;
use crate::error::Result;
use crate::ws::ws_client::WebSocket;

use crate::helpers::date::parse_time;
use crate::models::time_frame::*;

use futures_util::Future;

use crate::ws::message::{Message, ResponseBody, ResponseType};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::env;
use std::fmt::Debug;

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
    async fn get_instrument_pricing(&mut self, symbol: &str)
        -> Result<ResponseBody<SymbolPricing>>;
    async fn get_tick_prices(
        &mut self,
        symbol: &str,
        level: usize,
        timestamp: i64,
    ) -> Result<String>;
    async fn parse_stream_data(msg: Message) -> Option<String>;
    async fn keepalive_ping(&mut self) -> Result<String>;
}

#[derive(Debug)]
pub struct Xtb {
    websocket: WebSocket,
    symbol: String,
    streamSessionId: String,
    time_frame: usize,
    from_date: i64,
}

#[async_trait::async_trait]
impl Broker for Xtb {
    async fn new() -> Self {
        let url = &env::var("BROKER_URL").unwrap();

        Self {
            websocket: WebSocket::connect(url).await,
            streamSessionId: "".to_owned(),
            symbol: "".to_owned(),
            time_frame: 0,
            from_date: 0,
        }
    }

    async fn login(&mut self, username: &str, password: &str) -> Result<&mut Self> {
        self.send(&Command {
            command: String::from("login"),
            arguments: LoginParams {
                userId: String::from(username),
                password: String::from(password),
                appName: String::from("rs-algo-scanner"),
            },
        })
        .await?;

        let res = self.get_response().await?;

        Ok(self)
    }

    fn get_session_id(&mut self) -> &String {
        &self.streamSessionId
    }

    async fn read(&mut self) -> Result<Response<VEC_DOHLC>> {
        let msg = self.websocket.read().await.unwrap();
        let txt_msg = match msg {
            Message::Text(txt) => txt,
            _ => panic!(),
        };
        let response = self.handle_response::<VEC_DOHLC>(&txt_msg).await.unwrap();
        Ok(response)
    }

    async fn get_symbols(&mut self) -> Result<Response<VEC_DOHLC>> {
        self.send(&CommandAllSymbols {
            command: "getAllSymbols".to_owned(),
        })
        .await?;
        let res = self.get_response().await?;

        Ok(res)
    }

    async fn get_instrument_data(
        &mut self,
        symbol: &str,
        time_frame: usize,
        from_date: i64,
    ) -> Result<Response<VEC_DOHLC>> {
        self.symbol = symbol.to_owned();
        self.time_frame = time_frame;
        self.send(&Command {
            command: "getChartLastRequest".to_owned(),
            arguments: Instrument {
                info: InstrumentCandles {
                    symbol: symbol.to_owned(),
                    period: time_frame,
                    start: from_date * 1000,
                },
            },
        })
        .await?;

        let res = self.get_response().await?;

        Ok(res)
    }

    async fn get_instrument_pricing(
        &mut self,
        symbol: &str,
    ) -> Result<ResponseBody<SymbolPricing>> {
        let symbol_command = Command {
            command: "getSymbol".to_owned(),
            arguments: SymbolArg {
                symbol: symbol.to_owned(),
            },
        };

        self.send(&symbol_command).await.unwrap();
        let msg = self.websocket.read().await.unwrap();
        let txt_msg = match msg {
            Message::Text(txt) => {
                let parsed: SymbolPricingResponse = serde_json::from_str(&txt).unwrap();
                let symbol_detail: SymbolPricing = parsed.returnData;
                ResponseBody {
                    response: ResponseType::ExecuteTradeIn,
                    payload: Some(symbol_detail),
                }
            }
            _ => panic!(),
        };

        Ok(txt_msg)
    }

    async fn get_tick_prices(
        &mut self,
        symbol: &str,
        level: usize,
        timestamp: i64,
    ) -> Result<String> {
        let tick_command = &Command {
            command: "getTickPrices".to_owned(),
            arguments: TickParams {
                timestamp,
                symbols: vec![symbol.to_string()],
                level,
            },
        };

        self.websocket
            .send(&serde_json::to_string(&tick_command).unwrap())
            .await
            .unwrap();
        let msg = self.websocket.read().await.unwrap();
        let txt_msg = match msg {
            Message::Text(txt) => txt,
            _ => panic!(),
        };

        Ok(txt_msg)
    }

    async fn listen<F, T>(&mut self, symbol: &str, session_id: String, mut callback: F)
    where
        F: Send + FnMut(Message) -> T,
        T: Future<Output = Result<()>> + Send + 'static,
    {
        self.send(&CommandAllSymbols {
            command: "getAllSymbols".to_owned(),
        })
        .await
        .unwrap();

        loop {
            let msg = self.websocket.read().await.unwrap();
            let txt_msg = match msg {
                Message::Text(txt) => txt,
                _ => panic!(),
            };
            let response = self.handle_response::<VEC_DOHLC>(&txt_msg).await.unwrap();
            //tokio::spawn(callback(response));
        }
    }

    async fn parse_stream_data(msg: Message) -> Option<String> {
        let txt = match msg {
            Message::Text(txt) => txt,
            _ => "".to_owned(),
        };

        let obj: Value = serde_json::from_str(&txt).unwrap();
        // let leches = (
        //     data["ask"].as_f64().unwrap(),
        //     data["bid"].as_f64().unwrap(),
        //     data["high"].as_f64().unwrap(),
        //     data["low"].as_f64().unwrap(),
        //     data["bidVolume"].as_f64().unwrap(),
        //     data["timestamp"].as_f64().unwrap(),
        //     data["spreadTable"].as_f64().unwrap(),
        // );

        let msg = match &obj {
            Value::Object(obj) => {
                let command = &obj["command"];
                let data = &obj["data"];
                if command == "candle" {
                    log::info!("Broker Stream data received");

                    let date = data["ctm"].as_f64().unwrap() / 1000.;
                    let open = data["open"].as_f64().unwrap();
                    let high = open + data["high"].as_f64().unwrap();
                    let low = open + data["low"].as_f64().unwrap();
                    let close = open + data["close"].as_f64().unwrap();
                    let volume = data["vol"].as_f64().unwrap() * 1000.;

                    let leches = (date, open, high, low, close, volume, 0.);

                    let msg: ResponseBody<(f64, f64, f64, f64, f64, f64, f64)> = ResponseBody {
                        response: ResponseType::SubscribeStream,
                        payload: Some(leches),
                    };

                    Some(serde_json::to_string(&msg).unwrap())
                } else {
                    None
                }
            }
            _ => None,
        };

        msg
    }

    async fn keepalive_ping(&mut self) -> Result<String> {
        //log::info!("Server sending keepalive ping");
        let ping_command = Ping {
            command: "ping".to_owned(),
        };

        self.send(&ping_command).await.unwrap();
        let msg = self.websocket.read().await.unwrap();
        let txt_msg = match msg {
            Message::Text(txt) => txt,
            _ => panic!(),
        };

        Ok(txt_msg)
    }
}

impl Xtb {
    async fn send<T>(&mut self, command: &T) -> Result<()>
    where
        for<'de> T: Serialize + Deserialize<'de> + Debug,
    {
        self.websocket
            .send(&serde_json::to_string(&command).unwrap())
            .await?;

        Ok(())
    }

    async fn get_response(&mut self) -> Result<Response<VEC_DOHLC>> {
        let msg = self.websocket.read().await.unwrap();
        let txt_msg = match msg {
            Message::Text(txt) => txt,
            _ => panic!(),
        };
        let res = self.handle_response::<VEC_DOHLC>(&txt_msg).await.unwrap();

        Ok(res)
    }

    pub async fn parse_message(&mut self, msg: &str) -> Result<Value> {
        let parsed: Value = serde_json::from_str(&msg).expect("Can't parse to JSON");
        Ok(parsed)
    }

    pub async fn handle_response<'a, T>(&mut self, msg: &str) -> Result<Response<VEC_DOHLC>> {
        let data = self.parse_message(&msg).await.unwrap();

        let response: Response<VEC_DOHLC> = match &data {
            // Login
            _x if matches!(&data["streamSessionId"], Value::String(_x)) => {
                self.streamSessionId = data["streamSessionId"].as_str().unwrap().to_owned();
                Response {
                    msg_type: MessageType::Login,
                    symbol: "".to_owned(),
                    time_frame: TimeFrameType::from_number(self.time_frame),
                    data: vec![],
                    symbols: vec![],
                }
            }
            // GetInstrumentPrice
            _x if matches!(&data["returnData"]["digits"], Value::Number(_x)) => {
                Response::<VEC_DOHLC> {
                    msg_type: MessageType::GetInstrumentPrice,
                    symbol: self.symbol.to_owned(),
                    time_frame: TimeFrameType::from_number(self.time_frame),
                    symbols: vec![],
                    data: self.parse_price_data(&data).await.unwrap(),
                }
            }
            // GetSymbols
            _x if matches!(&data["returnData"], Value::Array(_x)) => Response::<VEC_DOHLC> {
                msg_type: MessageType::GetInstrumentPrice,
                symbol: self.symbol.to_owned(),
                time_frame: TimeFrameType::from_number(self.time_frame),
                data: vec![],
                symbols: self.parse_symbols_data(&data).await.unwrap(),
            },

            _ => {
                println!("[Error] {:?}", msg);
                Response {
                    msg_type: MessageType::Other,
                    symbol: "".to_owned(),
                    time_frame: TimeFrameType::M1,
                    data: vec![],
                    symbols: vec![],
                }
            }
        };
        Ok(response)
    }

    async fn parse_price_data(&mut self, data: &Value) -> Result<VEC_DOHLC> {
        let mut result: VEC_DOHLC = vec![];
        let digits = data["returnData"]["digits"].as_f64().unwrap();
        let x = 10.0_f64;
        let pow = x.powf(digits);
        for obj in data["returnData"]["rateInfos"].as_array().unwrap() {
            //FIXME!!
            let date = parse_time(obj["ctm"].as_i64().unwrap() / 1000);
            let open = obj["open"].as_f64().unwrap() / pow;
            let high = open + obj["high"].as_f64().unwrap() / pow;
            let low = open + obj["low"].as_f64().unwrap() / pow;
            let close = open + obj["close"].as_f64().unwrap() / pow;
            let volume = obj["vol"].as_f64().unwrap() * 1000.;
            result.push((date, open, high, low, close, volume));
        }

        Ok(result)
    }

    async fn parse_symbols_data(&mut self, data: &Value) -> Result<Vec<Symbol>> {
        let mut result: Vec<Symbol> = vec![];
        let symbols = data["returnData"].as_array().unwrap();
        for s in symbols {
            let symbol = match &s["symbol"] {
                Value::String(s) => s.to_string(),
                _ => panic!("Symbol parse error"),
            };
            let currency = match &s["currency"] {
                Value::String(s) => s.to_string(),
                _ => panic!("Currency parse error"),
            };
            let category = match &s["symbol"] {
                Value::String(s) => s.to_string(),
                _ => panic!("Category parse error"),
            };

            let description = match &s["description"] {
                Value::String(s) => s.to_string(),
                _ => panic!("Description parse error"),
            };

            result.push(Symbol {
                symbol,
                currency,
                category,
                description,
            });
        }
        Ok(result)
    }
}
pub fn parse_symbol(symbol: &String) -> Result<String> {
    if symbol.contains('_') {
        let symbol_str: Vec<&str> = symbol.split('_').collect();
        Ok(symbol_str[0].to_owned())
    } else {
        log::error!("Change fucking xtb");
        Ok(symbol.to_owned())
    }
}
