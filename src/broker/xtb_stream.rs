use super::*;
use crate::error::Result;
use crate::helpers::calc;
use crate::helpers::date;
use crate::helpers::date::parse_time;
use crate::helpers::date::*;
use crate::helpers::uuid;
use crate::models::market::*;
use crate::models::order::*;
use crate::models::pricing::Pricing;
use crate::models::time_frame::*;
use crate::models::trade::*;
use crate::ws::message::{
    InstrumentData, Message, ResponseBody, ResponseType, TradeData, TradeResponse,
};
use crate::ws::ws_client::WebSocket;
use crate::ws::ws_stream_client::WebSocket as WebSocketClientStream;

use chrono::{DateTime, Local};
use futures_util::{stream::SplitStream, Future};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::env;
use std::fmt::Debug;
use tokio::net::TcpStream;
use tokio_tungstenite::MaybeTlsStream;
use tokio_tungstenite::WebSocketStream;

#[async_trait::async_trait]
pub trait BrokerStream {
    async fn new() -> Self;
    async fn login(&mut self, username: &str, password: &str) -> Result<&mut Self>
    where
        Self: Sized;
    async fn get_symbols(&mut self) -> Result<ResponseBody<InstrumentData<VEC_DOHLC>>>;
    async fn read(&mut self) -> Result<ResponseBody<InstrumentData<VEC_DOHLC>>>;
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
    ) -> Result<ResponseBody<InstrumentData<VEC_DOHLC>>>;
    async fn open_trade(
        &mut self,
        trade_in: TradeData<TradeIn>,
    ) -> Result<ResponseBody<TradeResponse<TradeIn>>>;
    async fn close_trade(
        &mut self,
        trade_out: TradeData<TradeOut>,
    ) -> Result<ResponseBody<TradeResponse<TradeOut>>>;
    async fn order_in(
        &mut self,
        order: TradeData<Order>,
    ) -> Result<ResponseBody<TradeResponse<TradeIn>>>;
    async fn order_out(
        &mut self,
        order: TradeData<Order>,
    ) -> Result<ResponseBody<TradeResponse<TradeOut>>>;
    async fn get_market_hours(&mut self, symbol: &str) -> Result<ResponseBody<MarketHours>>;
    async fn get_instrument_pricing(&mut self, symbol: &str) -> Result<ResponseBody<Pricing>>;
    async fn get_stream(&mut self) -> &mut SplitStream<WebSocketStream<MaybeTlsStream<TcpStream>>>;
    async fn subscribe_stream(&mut self, symbol: &str) -> Result<()>;
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
    socket: WebSocket,
    stream: WebSocketClientStream,
    symbol: String,
    streamSessionId: String,
    time_frame: usize,
    from_date: i64,
}

#[async_trait::async_trait]
impl BrokerStream for Xtb {
    async fn new() -> Self {
        let mut socket;
        let stream;
        let socket_url = &env::var("BROKER_URL").unwrap();
        let stream_url = &env::var("BROKER_STREAM_URL").unwrap();
        let stream_subscribe = env::var("STREAM_SUBSCRIBE")
            .unwrap()
            .parse::<bool>()
            .unwrap();

        if stream_subscribe {
            socket = WebSocket::connect(socket_url).await;
            stream = WebSocketClientStream::connect(stream_url).await;
        } else {
            socket = WebSocket::connect(socket_url).await;
            stream = WebSocketClientStream::connect(socket_url).await;
        }

        Self {
            socket: socket,
            stream: stream,
            streamSessionId: "".to_owned(),
            symbol: "".to_owned(),
            time_frame: 0,
            from_date: 0,
        }
    }

    fn get_session_id(&mut self) -> &String {
        &self.streamSessionId
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

    async fn get_stream(&mut self) -> &mut SplitStream<WebSocketStream<MaybeTlsStream<TcpStream>>> {
        &mut self.stream.read
    }

    async fn read(&mut self) -> Result<ResponseBody<InstrumentData<VEC_DOHLC>>> {
        let msg = self.socket.read().await.unwrap();
        let txt_msg = match msg {
            Message::Text(txt) => txt,
            _ => panic!(),
        };
        let response = self.handle_response::<VEC_DOHLC>(&txt_msg).await.unwrap();
        Ok(response)
    }

    async fn get_symbols(&mut self) -> Result<ResponseBody<InstrumentData<VEC_DOHLC>>> {
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
    ) -> Result<ResponseBody<InstrumentData<VEC_DOHLC>>> {
        self.symbol = symbol.to_owned();
        self.time_frame = time_frame;
        let instrument_command = Command {
            command: "getChartLastRequest".to_owned(),
            arguments: Instrument {
                info: InstrumentCandles {
                    symbol: symbol.to_owned(),
                    period: time_frame,
                    start: from_date * 1000,
                },
            },
        };

        log::info!(
            "[SOCKET] requesting {} data since {:?}",
            time_frame,
            date::parse_time(from_date)
        );

        self.send(&instrument_command).await.unwrap();

        let res = self.get_response().await?;
        Ok(res)
    }

    async fn get_instrument_pricing(&mut self, symbol: &str) -> Result<ResponseBody<Pricing>> {
        let tick_command = Command {
            command: "getSymbol".to_owned(),
            arguments: SymbolArg {
                symbol: symbol.to_owned(),
            },
        };

        self.send(&tick_command).await.unwrap();
        let msg = self.socket.read().await.unwrap();
        let txt_msg = match msg {
            Message::Text(txt) => {
                let data = self.parse_message(&txt).await.unwrap();
                let ask = data["returnData"]["ask"].as_f64().unwrap();
                let bid = data["returnData"]["bid"].as_f64().unwrap();
                let pip_size = data["returnData"]["tickSize"].as_f64().unwrap() * 10.;
                let spread = ask - bid;
                let percentage = 0.;
                let pricing =
                    Pricing::new(symbol.to_owned(), ask, bid, spread, pip_size, percentage);
                ResponseBody {
                    response: ResponseType::GetInstrumentPricing,
                    payload: Some(pricing),
                }
            }
            _ => panic!(),
        };

        Ok(txt_msg)
    }

    async fn get_market_hours(&mut self, symbol: &str) -> Result<ResponseBody<MarketHours>> {
        let trading_hours_command = Command {
            command: "getTradingHours".to_owned(),
            arguments: TradingHoursCommand {
                symbols: vec![symbol.to_string()],
            },
        };

        self.send(&trading_hours_command).await.unwrap();
        let msg = self.socket.read().await.unwrap();

        let txt_msg = match msg {
            Message::Text(txt) => {
                let data = self.parse_message(&txt).await.unwrap();
                let mut result: Vec<MarketHour> = vec![];

                let current_date = Local::now();
                let week_day = date::get_week_day(current_date) as isize;
                let mut open = false;
                for obj in data["returnData"][0]["trading"].as_array().unwrap() {
                    let day = obj["day"].as_i64().unwrap() as isize;
                    let from = obj["fromT"].as_i64().unwrap();
                    let to = obj["toT"].as_i64().unwrap();
                    let base = current_date.date().and_hms(0, 0, 0);
                    let date_from = base + Duration::milliseconds(from);
                    let date_to = base + Duration::milliseconds(to);
                    if day == week_day {
                        if current_date > date_from && current_date < date_to {
                            open = true
                        } else {
                            open = false
                        }
                    };
                    let market_hour = MarketHour {
                        day,
                        from: date_from,
                        to: date_to,
                    };
                    result.push(market_hour);
                }

                match self.get_instrument_pricing(&symbol).await {
                    Ok(_) => open = true,
                    Err(_) => open = false,
                };

                ResponseBody {
                    response: ResponseType::GetMarketHours,
                    payload: Some(MarketHours::new(open, symbol.to_owned(), result)),
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
        self.symbol = symbol.to_owned();
        let tick_command = Command {
            command: "getTickPrices".to_owned(),
            arguments: TickParams {
                timestamp,
                symbols: vec![symbol.to_string()],
                level,
            },
        };

        self.send(&tick_command).await.unwrap();
        let msg = self.socket.read().await.unwrap();
        let txt_msg = match msg {
            Message::Text(txt) => txt,
            _ => panic!(),
        };

        Ok(txt_msg)
    }

    async fn open_trade(
        &mut self,
        trade: TradeData<TradeIn>,
    ) -> Result<ResponseBody<TradeResponse<TradeIn>>> {
        let trade_command = Command {
            command: "tradeTransaction".to_owned(),
            arguments: Transaction {
                cmd: "".to_owned(),
                symbol: "".to_owned(),
                customComment: "".to_owned(),
                expiration: 0,
                order: 0,
                price: 0.,
                sl: 0.,
                tp: 0.,
                volume: 0.,
                trans_type: 0,
            },
        };

        let symbol = &trade.symbol;
        let pricing = self.get_instrument_pricing(&symbol).await.unwrap();
        let pricing = pricing.payload.unwrap();
        let ask = pricing.ask();
        let bid = pricing.bid();
        let spread = pricing.spread();
        let mut data = trade.data;
        let trade_type = data.trade_type.clone();

        let price_in = match trade_type.is_long() {
            true => ask,
            false => bid,
        };

        log::info!(
            "{} TradeIn accepted at ask: {} bid: {} pricing",
            trade.symbol,
            ask,
            bid
        );

        data.id = uuid::generate_ts_id(Local::now());
        data.price_in = price_in;
        data.ask = ask;
        data.spread = spread;

        let txt_msg = ResponseBody {
            response: ResponseType::ExecuteTradeIn,
            payload: Some(TradeResponse {
                symbol: trade.symbol,
                accepted: true,
                //time_frame: trade.time_frame,
                data: data,
            }),
        };

        Ok(txt_msg)
    }
    async fn close_trade(
        &mut self,
        trade: TradeData<TradeOut>,
    ) -> Result<ResponseBody<TradeResponse<TradeOut>>> {
        let symbol = &trade.symbol;
        //CONTINUE HERE RETURN REJECTED NOT PROFITABLE
        let pricing = self.get_instrument_pricing(&symbol).await.unwrap();
        let pricing = pricing.payload.unwrap();
        let ask = pricing.ask();
        let bid = pricing.bid();
        let spread = pricing.spread();
        let mut data = trade.data;
        let trade_type = data.trade_type.clone();
        let price_in = data.price_in;

        let price_out = match trade_type.is_stop() {
            true => match trade_type {
                TradeType::StopLossLong => bid,
                TradeType::StopLossShort => ask,
                _ => todo!(),
            },
            false => match trade_type.is_long() {
                true => bid,
                false => ask,
            },
        };

        let profit = match trade_type.is_long() {
            true => price_out - price_in,
            false => price_in - price_out,
        };

        let is_profitable = match profit {
            _ if profit > 0. => {
                log::info!(
                    "{:?} {} accepted at ask: {} bid: {} with {} profit",
                    trade_type,
                    trade.symbol,
                    ask,
                    bid,
                    profit
                );
                true
            }
            _ => {
                log::info!(
                    "{:?} {} NOT accepted at ask: {} bid: {}",
                    trade_type,
                    trade.symbol,
                    ask,
                    bid,
                );
                false
            }
        };

        data.id = uuid::generate_ts_id(Local::now());
        data.price_out = price_out;
        data.date_out = to_dbtime(Local::now());
        data.bid = bid;
        data.ask = ask;
        data.spread_out = spread;

        let txt_msg = ResponseBody {
            response: ResponseType::ExecuteTradeOut,
            payload: Some(TradeResponse {
                symbol: trade.symbol,
                accepted: is_profitable,
                data: data,
            }),
        };
        Ok(txt_msg)
    }

    async fn order_in(
        &mut self,
        order: TradeData<Order>,
    ) -> Result<ResponseBody<TradeResponse<TradeIn>>> {
        let trade_command = Command {
            command: "tradeTransaction".to_owned(),
            arguments: Transaction {
                cmd: "".to_owned(),
                symbol: "".to_owned(),
                customComment: "".to_owned(),
                expiration: 0,
                order: 0,
                price: 0.,
                sl: 0.,
                tp: 0.,
                volume: 0.,
                trans_type: 0,
            },
        };

        let symbol = &order.symbol;
        let order = order.data;
        let pricing = self.get_instrument_pricing(&symbol).await.unwrap();
        let pricing = pricing.payload.unwrap();
        let spread = pricing.spread();

        let trade_type = match order.order_type.is_long() {
            true => TradeType::OrderInLong,
            false => TradeType::OrderInShort,
        };

        let ask = pricing.ask();
        let price_in = match trade_type.is_long() {
            true => pricing.ask(),
            false => pricing.bid(),
        };

        let quantity = calc::calculate_quantity(order.size(), price_in);

        let trade_in = TradeIn {
            id: uuid::generate_ts_id(Local::now()),
            index_in: order.index_created,
            quantity,
            origin_price: order.origin_price,
            price_in,
            ask: pricing.ask(),
            spread,
            trade_type,
            date_in: to_dbtime(Local::now()),
        };

        let txt_msg = ResponseBody {
            response: ResponseType::ExecuteTradeIn,
            payload: Some(TradeResponse {
                symbol: symbol.clone(),
                accepted: true,
                data: trade_in,
            }),
        };

        Ok(txt_msg)
    }

    async fn order_out(
        &mut self,
        order: TradeData<Order>,
    ) -> Result<ResponseBody<TradeResponse<TradeOut>>> {
        let trade_command = Command {
            command: "tradeTransaction".to_owned(),
            arguments: Transaction {
                cmd: "".to_owned(),
                symbol: "".to_owned(),
                customComment: "".to_owned(),
                expiration: 0,
                order: 0,
                price: 0.,
                sl: 0.,
                tp: 0.,
                volume: 0.,
                trans_type: 0,
            },
        };

        let symbol = &order.symbol;
        let order = order.data;
        let pricing = self.get_instrument_pricing(&symbol).await.unwrap();
        let pricing = pricing.payload.unwrap();
        let spread = pricing.spread();

        let trade_type = match order.order_type.is_long() {
            true => TradeType::OrderOutLong,
            false => TradeType::OrderOutShort,
        };

        let price_out = match trade_type.is_long() {
            true => pricing.bid(),
            false => pricing.ask(),
        };

        let now = Local::now();

        log::info!("{:?} accepted at {}", order.order_type, price_out);

        let trade_out = TradeOut {
            id: uuid::generate_ts_id(now),
            trade_type,
            index_in: order.index_created,
            price_in: order.origin_price,
            ask: pricing.ask(),
            spread_in: spread,
            date_in: order.full_filled_at.unwrap(),
            index_out: order.index_fulfilled,
            price_origin: order.origin_price,
            price_out,
            bid: pricing.bid(),
            spread_out: spread,
            date_out: to_dbtime(now),
            profit: 0.,
            profit_per: 0.,
            run_up: 0.,
            run_up_per: 0.,
            draw_down: 0.,
            draw_down_per: 0.,
        };

        let txt_msg = ResponseBody {
            response: ResponseType::ExecuteTradeIn,
            payload: Some(TradeResponse {
                symbol: symbol.clone(),
                accepted: true,
                data: trade_out,
            }),
        };

        Ok(txt_msg)
    }

    async fn subscribe_stream(&mut self, symbol: &str) -> Result<()> {
        let command_alive = CommandStreaming {
            command: "getKeepAlive".to_owned(),
            streamSessionId: self.streamSessionId.clone(),
        };

        self.send_stream(&command_alive).await.unwrap();

        let command = CommandGetCandles {
            command: "getCandles".to_owned(),
            streamSessionId: self.streamSessionId.clone(),
            symbol: symbol.to_owned(),
        };

        self.send_stream(&command).await.unwrap();

        Ok(())
    }

    async fn listen<F, T>(&mut self, symbol: &str, session_id: String, mut callback: F)
    where
        F: Send + FnMut(Message) -> T,
        T: Future<Output = Result<()>> + Send + 'static,
    {
    }

    async fn parse_stream_data(msg: Message) -> Option<String> {
        let txt = match msg {
            Message::Text(txt) => txt,
            _ => "".to_owned(),
        };

        let obj: Value = serde_json::from_str(&txt).unwrap();

        let msg = match &obj {
            Value::Object(obj) => {
                let command = &obj["command"];
                let data = &obj["data"];
                if command == "candle" {
                    log::info!("Broker Stream data received");
                    let date = parse_time(data["ctm"].as_i64().unwrap() / 1000);
                    let open = data["open"].as_f64().unwrap();
                    let high = data["high"].as_f64().unwrap();
                    let low = data["low"].as_f64().unwrap();
                    let close = data["close"].as_f64().unwrap();
                    let volume = data["vol"].as_f64().unwrap() * 1000.;

                    let leches = (date, open, high, low, close, volume);

                    // println!(
                    //     "1111111 open {} high {} low {} close {} date {}",
                    //     leches.1, leches.2, leches.3, leches.4, leches.0
                    // );

                    let msg: ResponseBody<(DateTime<Local>, f64, f64, f64, f64, f64)> =
                        ResponseBody {
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
        let msg = self.socket.read().await.unwrap();
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
        self.socket
            .send(&serde_json::to_string(&command).unwrap())
            .await?;

        Ok(())
    }

    async fn send_stream<T>(&mut self, command: &T) -> Result<()>
    where
        for<'de> T: Serialize + Deserialize<'de> + Debug,
    {
        self.stream
            .send(&serde_json::to_string(&command).unwrap())
            .await
            .unwrap();

        Ok(())
    }

    async fn get_response(&mut self) -> Result<ResponseBody<InstrumentData<VEC_DOHLC>>> {
        let msg = self.socket.read().await.unwrap();
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

    pub async fn handle_response<'a, T>(
        &mut self,
        msg: &str,
    ) -> Result<ResponseBody<InstrumentData<VEC_DOHLC>>> {
        let data = self.parse_message(&msg).await.unwrap();
        let response: ResponseBody<InstrumentData<VEC_DOHLC>> = match &data {
            // Login
            _x if matches!(&data["streamSessionId"], Value::String(_x)) => {
                self.streamSessionId = data["streamSessionId"].as_str().unwrap().to_owned();
                ResponseBody {
                    response: ResponseType::GetInstrumentData,
                    payload: Some(InstrumentData {
                        symbol: "".to_owned(),
                        time_frame: TimeFrameType::from_number(self.time_frame),
                        data: vec![],
                    }),
                }
            }
            // Pricing Data
            _x if matches!(&data["returnData"]["digits"], Value::Number(_x)) => ResponseBody {
                response: ResponseType::GetInstrumentData,
                payload: Some(InstrumentData {
                    symbol: self.symbol.clone(),
                    time_frame: TimeFrameType::from_number(self.time_frame),
                    data: self.parse_price_data(&data).await.unwrap(),
                }),
            },
            _ => ResponseBody {
                response: ResponseType::GetInstrumentData,
                payload: Option::None,
            },
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

    pub fn parse_market_hours(&mut self, data: &Value) -> Result<Vec<MarketHour>> {
        let mut result: Vec<MarketHour> = vec![];
        let current_date = Local::now();
        let base = current_date.date().and_hms(0, 0, 0);

        for obj in data["returnData"]["trading"].as_array().unwrap() {
            let day = obj["day"].as_i64().unwrap().try_into().unwrap();
            let from = obj["from"].as_i64().unwrap();
            let to = obj["to"].as_i64().unwrap();
            let date_from = base + Duration::milliseconds(from);
            let date_to = base + Duration::milliseconds(to);
            let market_hour = MarketHour {
                day,
                from: date_from,
                to: date_to,
            };
            result.push(market_hour);
        }
        Ok(result)
    }

    pub fn parse_symbol(symbol: String) -> Result<String> {
        if symbol.contains('_') {
            let symbol_str: Vec<&str> = symbol.split('_').collect();
            Ok(symbol_str[0].to_owned())
        } else {
            log::error!("Change fucking xtb");
            Ok(symbol)
        }
    }
}
