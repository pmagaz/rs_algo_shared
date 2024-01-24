use crate::broker::models::*;
use crate::error::{Result, RsAlgoError, RsAlgoErrorKind};
use crate::helpers::calc::*;
use crate::helpers::date::{self, parse_time_seconds, DateTime, Local, Timelike};
use crate::helpers::http::request;
use crate::helpers::http::HttpMethod;
use crate::helpers::uuid;
use crate::helpers::{calc, date::*};
use crate::models::environment;
use crate::models::market::*;
use crate::models::mode;
use crate::models::order::*;
use crate::models::stop_loss::StopLossType;
use crate::models::tick::InstrumentTick;
use crate::models::time_frame::*;
use crate::models::trade::*;
use crate::ws::message::{
    InstrumentData, Message, ResponseBody, ResponseType, TradeData, TradeResponse,
};
use crate::ws::ws_client::WebSocket;
use crate::ws::ws_stream_client::WebSocket as WebSocketClientStream;

use futures_util::{stream::SplitStream, Future};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::env;
use std::fmt::Debug;
use std::time::Duration;
use tokio::net::TcpStream;
use tokio::time::sleep;
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
    async fn get_historic_data(
        &mut self,
        symbol: &str,
        period: usize,
        start: i64,
        end: i64,
    ) -> Result<ResponseBody<InstrumentData<VEC_DOHLC>>>;
    async fn open_trade(
        &mut self,
        trade_in: TradeData<TradeIn>,
        orders: Option<Vec<Order>>,
    ) -> Result<ResponseBody<TradeResponse<TradeIn>>>;
    async fn open_trade_real(
        &mut self,
        trade_in: TradeData<TradeIn>,
        orders: Option<Vec<Order>>,
    ) -> Result<ResponseBody<TradeResponse<TradeIn>>>;
    async fn open_trade_test(
        &mut self,
        trade_in: TradeData<TradeIn>,
        orders: Option<Vec<Order>>,
    ) -> Result<ResponseBody<TradeResponse<TradeIn>>>;
    async fn get_transaction_status(&mut self, order_id: u64)
        -> Result<TransactionStatusnResponse>;
    async fn close_trade(
        &mut self,
        trade_out: TradeData<TradeOut>,
    ) -> Result<ResponseBody<TradeResponse<TradeOut>>>;
    async fn close_trade_real(
        &mut self,
        trade_out: TradeData<TradeOut>,
    ) -> Result<ResponseBody<TradeResponse<TradeOut>>>;
    async fn close_trade_test(
        &mut self,
        trade_out: TradeData<TradeOut>,
    ) -> Result<ResponseBody<TradeResponse<TradeOut>>>;
    async fn open_order(
        &mut self,
        trade: TradeData<TradeIn>,
        order: TradeData<Order>,
    ) -> Result<ResponseBody<TradeResponse<TradeIn>>>;
    async fn open_order_test(
        &mut self,
        trade: TradeData<TradeIn>,
        order: TradeData<Order>,
    ) -> Result<ResponseBody<TradeResponse<TradeIn>>>;
    async fn close_order(
        &mut self,
        trade: TradeData<TradeOut>,
        order: TradeData<Order>,
    ) -> Result<ResponseBody<TradeResponse<TradeOut>>>;
    async fn close_order_test(
        &mut self,
        trade: TradeData<TradeOut>,
        order: TradeData<Order>,
    ) -> Result<ResponseBody<TradeResponse<TradeOut>>>;
    async fn get_active_positions(
        &mut self,
        symbol: &str,
        strategy_name: &str,
    ) -> Result<ResponseBody<PositionResult>>;
    async fn get_market_hours(&mut self, symbol: &str) -> Result<ResponseBody<MarketHours>>;
    async fn is_market_open(&mut self, symbol: &str) -> Result<ResponseBody<bool>>;
    async fn is_market_available(&mut self, symbol: &str) -> bool;
    async fn get_instrument_tick(&mut self, symbol: &str) -> Result<ResponseBody<InstrumentTick>>;
    async fn get_instrument_tick_test(
        &mut self,
        symbol: &str,
        price: f64,
        trade_type: &TradeType,
    ) -> Result<ResponseBody<InstrumentTick>>;
    async fn get_ask_bid(&mut self, symbol: &str) -> Result<(f64, f64)>;
    async fn get_transaction_details(
        &mut self,
        symbol: &str,
        strategy_name: &str,
        id: Option<usize>,
    ) -> Option<TransactionDetails>;
    async fn get_transactions_history(
        &mut self,
        symbol: &str,
        strategy_name: &str,
        id: Option<usize>,
    ) -> Option<TransactionDetails>;
    async fn get_stream(&mut self) -> &mut SplitStream<WebSocketStream<MaybeTlsStream<TcpStream>>>;
    async fn subscribe_stream(&mut self, symbol: &str) -> Result<()>;
    async fn subscribe_tick_prices(&mut self, symbol: &str) -> Result<()>;
    async fn subscribe_trades(&mut self, symbol: &str) -> Result<()>;
    async fn parse_stream_data(msg: Message, symbol: &str, strategy_name: &str) -> Option<String>;
    async fn keepalive_ping(&mut self) -> Result<String>;
    async fn disconnect(&mut self) -> Result<()>;
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
            socket = WebSocket::connect(socket_url);
            stream = WebSocketClientStream::connect(stream_url).await;
        } else {
            socket = WebSocket::connect(socket_url);
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
        let res = match msg {
            Message::Text(txt) => txt,
            _ => panic!(),
        };
        let response = self.handle_response::<VEC_DOHLC>(&res).await.unwrap();
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
            "Requesting data {} since {:?}",
            time_frame,
            date::parse_time_seconds(from_date)
        );

        self.send(&instrument_command).await.unwrap();

        let res = self.get_response().await?;
        Ok(res)
    }

    async fn get_instrument_tick(&mut self, symbol: &str) -> Result<ResponseBody<InstrumentTick>> {
        let tick_command = Command {
            command: "getSymbol".to_owned(),
            arguments: SymbolArg {
                symbol: symbol.to_owned(),
            },
        };
        self.send(&tick_command).await.unwrap();
        let msg = self.socket.read().await.unwrap();
        let res = match msg {
            Message::Text(txt) => {
                let tick = self.parse_tick_data(symbol.to_owned(), txt).unwrap();

                ResponseBody {
                    response: ResponseType::GetInstrumentTick,
                    payload: Some(tick),
                }
            }
            _ => panic!(),
        };

        Ok(res)
    }

    async fn get_ask_bid(&mut self, symbol: &str) -> Result<(f64, f64)> {
        let tick_command = Command {
            command: "getSymbol".to_owned(),
            arguments: SymbolArg {
                symbol: symbol.to_owned(),
            },
        };
        self.send(&tick_command).await?;
        let msg = self.socket.read().await.unwrap();

        if let Message::Text(txt) = msg {
            let data = self.parse_message(&txt)?;
            let return_data = data
                .get("returnData")
                .ok_or_else(|| RsAlgoErrorKind::ParseError)
                .unwrap();
            let ask = return_data["ask"].as_f64().unwrap();
            let bid = return_data["bid"].as_f64().unwrap();
            Ok((ask, bid))
        } else {
            panic!();
        }
    }

    async fn get_transaction_details(
        &mut self,
        symbol: &str,
        strategy_name: &str,
        position_id: Option<usize>,
    ) -> Option<TransactionDetails> {
        let opened_only = if position_id.is_some() { false } else { true };

        let command = Command {
            command: "getTrades".to_owned(),
            arguments: GetTrades {
                openedOnly: opened_only,
            },
        };

        self.send(&command).await.unwrap();
        let msg = self.socket.read().await.unwrap();

        if let Message::Text(txt) = msg {
            let data = self.parse_message(&txt).unwrap();
            let data = data["returnData"].as_array().unwrap();
            for obj in data {
                let order_symbol = obj["symbol"].as_str().unwrap();
                let id = obj["position"].as_i64().unwrap() as usize;
                let comments = obj["customComment"].as_str().unwrap();
                let comments: TransactionComments = serde_json::from_str(comments).unwrap();

                if order_symbol == symbol
                    && strategy_name == &comments.strategy_name
                    && position_id.map_or(true, |pid| pid == id)
                {
                    let open_price = obj["open_price"].as_f64().unwrap();
                    let close_price = obj["close_price"].as_f64().unwrap();

                    return Some(TransactionDetails {
                        id,
                        open_price,
                        close_price,
                        profit: 0.,
                    });
                }
            }
            None
        } else {
            None
        }
    }

    async fn get_transactions_history(
        &mut self,
        symbol: &str,
        strategy_name: &str,
        position_id: Option<usize>,
    ) -> Option<TransactionDetails> {
        let start = (Local::now() - date::Duration::seconds(3)).timestamp_millis();
        let command = Command {
            command: "getTradesHistory".to_owned(),
            arguments: GetTradesHistory {
                start: start,
                end: 0,
            },
        };

        self.send(&command).await.unwrap();
        let msg = self.socket.read().await.unwrap();

        if let Message::Text(txt) = msg {
            let data = self.parse_message(&txt).unwrap();
            let data = data["returnData"].as_array().unwrap();

            for obj in data {
                let order_symbol = obj["symbol"].as_str().unwrap();
                let id = obj["position"].as_i64().unwrap() as usize;
                let comments = obj["customComment"].as_str().unwrap();
                let comments: TransactionComments = serde_json::from_str(comments).unwrap();

                if order_symbol == symbol
                    && strategy_name == &comments.strategy_name
                    && position_id.map_or(true, |pid| pid == id)
                {
                    let open_price = obj["open_price"].as_f64().unwrap();
                    let close_price = obj["close_price"].as_f64().unwrap();

                    let gross_profit = obj["profit"].as_f64().unwrap();
                    let swap = obj["storage"].as_f64().unwrap_or(0.0);
                    let commission = obj["commission"].as_f64().unwrap_or(0.0);

                    let profit = gross_profit + swap + commission;

                    log::info!(
                        "PROFIT DATA Gross Profit: {} Swap: {} Comission: {} Net Profit: {}",
                        gross_profit,
                        swap,
                        commission,
                        profit
                    );

                    return Some(TransactionDetails {
                        id,
                        open_price,
                        close_price,
                        profit,
                    });
                }
            }
            None
        } else {
            None
        }
    }

    async fn get_historic_data(
        &mut self,
        symbol: &str,
        time_frame: usize,
        from: i64,
        to: i64,
    ) -> Result<ResponseBody<InstrumentData<VEC_DOHLC>>> {
        self.symbol = symbol.to_owned();
        self.time_frame = time_frame;

        let instrument_command = Command {
            command: "getChartRangeRequest".to_owned(),
            arguments: HistoricInstrument {
                info: HistoricInstrumentCandles {
                    symbol: symbol.to_owned(),
                    period: time_frame,
                    start: from * 1000,
                    end: to * 1000,
                    ticks: 0,
                },
            },
        };

        log::info!(
            "Requesting historic data {} from {} to {}",
            time_frame,
            date::parse_time_seconds(from),
            date::parse_time_seconds(to)
        );

        self.send(&instrument_command).await.unwrap();

        let res = self.get_response().await?;

        Ok(res)
    }

    async fn get_instrument_tick_test(
        &mut self,
        symbol: &str,
        price: f64,
        trade_type: &TradeType,
    ) -> Result<ResponseBody<InstrumentTick>> {
        let tick_url = &format!(
            "{}{}",
            env::var("BACKEND_BACKTEST_PRICING_ENDPOINT").unwrap(),
            symbol
        );
        let tick: InstrumentTick = request(&tick_url, &String::from("all"), HttpMethod::Get)
            .await
            .unwrap()
            .json()
            .await
            .unwrap();

        let slippage_pips = env::var("SLIPPAGE_PIPS").unwrap().parse::<f64>().unwrap();

        let price_with_slippage = match trade_type.is_long() == trade_type.is_entry() {
            true => price + calc::to_pips(slippage_pips, &tick),
            false => price - calc::to_pips(slippage_pips, &tick),
        };

        let final_price = match trade_type.is_stop() {
            true => price,
            false => price_with_slippage,
        };

        let ask = format_symbol_price(final_price + tick.spread(), &symbol);
        let bid = format_symbol_price(final_price, &symbol);

        let tick = InstrumentTick::new()
            .symbol(symbol.to_string())
            .ask(ask)
            .bid(bid)
            .high(0.0)
            .low(0.0)
            .spread(tick.spread())
            .pip_size(tick.pip_size())
            .time(0)
            .build()
            .unwrap();

        Ok(ResponseBody {
            response: ResponseType::GetInstrumentTick,
            payload: Some(tick),
        })
    }

    async fn get_market_hours(&mut self, symbol: &str) -> Result<ResponseBody<MarketHours>> {
        let command = Command {
            command: "getTradingHours".to_owned(),
            arguments: TradingHoursCommand {
                symbols: vec![symbol.to_string()],
            },
        };

        self.send(&command).await.unwrap();
        let msg = self.socket.read().await.unwrap();

        let res = match msg {
            Message::Text(txt) => {
                let data = self.parse_message(&txt).unwrap();

                let mut market_hours: Vec<MarketHour> = vec![];

                for obj in data["returnData"][0]["trading"].as_array().unwrap() {
                    let day = obj["day"].as_i64().unwrap() as u32;
                    let from = obj["fromT"].as_i64().unwrap() as u32 / 3600 / 1000;
                    let to = obj["toT"].as_i64().unwrap() as u32 / 3600 / 1000;

                    let market_hour = MarketHour { day, from, to };

                    market_hours.push(market_hour);
                }

                market_hours.sort_by(|a, b| a.day.cmp(&b.day));

                ResponseBody {
                    response: ResponseType::GetMarketHours,
                    payload: Some(MarketHours::new(symbol.to_owned(), market_hours)),
                }
            }
            _ => panic!(),
        };

        Ok(res)
    }

    async fn is_market_open(&mut self, symbol: &str) -> Result<ResponseBody<bool>> {
        match self.is_market_available(symbol).await {
            true => Ok(ResponseBody {
                response: ResponseType::IsMarketOpen,
                payload: Some(true),
            }),
            false => Ok(ResponseBody {
                response: ResponseType::IsMarketOpen,
                payload: Some(false),
            }),
        }
    }

    async fn is_market_available(&mut self, symbol: &str) -> bool {
        let minutes = 5;
        let from = (Local::now() - date::Duration::minutes(minutes)).timestamp();
        let res = self
            .get_instrument_data(&symbol, minutes as usize, from)
            .await
            .unwrap();

        match res.payload {
            Some(inst) => {
                if inst.data.len() > 0 {
                    true
                } else {
                    log::warn!(
                        "No {} data found in last {}. Market not open",
                        symbol,
                        minutes
                    );
                    false
                }
            }
            None => false,
        }
    }

    async fn open_trade(
        &mut self,
        trade: TradeData<TradeIn>,
        orders: Option<Vec<Order>>,
    ) -> Result<ResponseBody<TradeResponse<TradeIn>>> {
        let is_prod = environment::from_str(&env::var("ENV").unwrap()).is_prod();

        match is_prod {
            true => self.open_trade_real(trade, orders).await,
            false => self.open_trade_test(trade, orders).await,
        }
    }

    async fn open_trade_real(
        &mut self,
        trade: TradeData<TradeIn>,
        orders: Option<Vec<Order>>,
    ) -> Result<ResponseBody<TradeResponse<TradeIn>>> {
        const MAX_RETRIES: usize = 3;
        const RETRY_AFTER: u64 = 500;
        let mut attempts = 0;
        let mut accepted = false;
        let mut status = TradeStatus::Rejected;
        let symbol = trade.symbol;
        let strategy_name = trade.strategy_name;
        let mut trade_in = trade.data;
        let mut sell_order_price = None;
        let mut stop_loss_order_price = None;
        let valid_until = (Local::now() + date::Duration::minutes(3)).timestamp_millis();
        let is_long = trade_in.trade_type.is_long();

        let order_size_limit = env::var("ORDER_SIZE_LIMIT")
            .unwrap()
            .parse::<f64>()
            .unwrap();

        let size = trade_in.size;
        let trade_size = if size > order_size_limit {
            log::error!("Trade size bigger than {:?} !!", order_size_limit);
            order_size_limit
        } else {
            size
        };

        if let Some(orders) = orders {
            for order in orders.iter() {
                match order.order_type {
                    OrderType::BuyOrderLong(_, _)
                    | OrderType::BuyOrderShort(_, _)
                    | OrderType::TakeProfitLong(_, _)
                    | OrderType::TakeProfitShort(_, _) => {
                        //FIXME WORKAROUND
                        if let Some(meta) = &order.meta {
                            stop_loss_order_price = Some(meta.sl);
                        }
                    }
                    OrderType::SellOrderLong(_, _) | OrderType::SellOrderShort(_, _) => {
                        sell_order_price = Some(order.target_price);
                    }
                    OrderType::StopLossLong(_, _) | OrderType::StopLossShort(_, _) => {
                        stop_loss_order_price = Some(order.target_price);
                    }
                    _ => {}
                }
            }
        }

        let command = match is_long {
            true => TransactionCommand::BuyMarket.value(),
            false => TransactionCommand::SellMarket.value(),
        };

        //while !accepted && attempts < MAX_RETRIES {
        let trade_type = trade_in.trade_type.clone();
        let start = Local::now();
        let (ask, bid) = self.get_ask_bid(&symbol).await.unwrap();
        let spread = ask - bid;

        let comment = serde_json::to_string(&TransactionComments {
            strategy_name: strategy_name.clone(),
            index_in: trade_in.index_in.clone(),
            sell_order_price,
            stop_loss_order_price,
            trade_type,
            spread: spread,
            bid: bid,
        })
        .unwrap();

        let opening_price = 1.;

        let trade_command: Command<TransactionInfo> = Command {
            command: "tradeTransaction".to_owned(),
            arguments: TransactionInfo {
                tradeTransInfo: TradeTransactionInfo {
                    cmd: command,
                    symbol: symbol.to_owned(),
                    trans_type: TransactionAction::Open.value(),
                    customComment: comment,
                    expiration: valid_until,
                    order: 0,
                    price: opening_price,
                    offset: 0,
                    sl: stop_loss_order_price.unwrap(),
                    tp: 0.,
                    volume: trade_size,
                },
            },
        };

        self.send(&trade_command).await.unwrap();
        let msg = self.socket.read().await.unwrap();

        log::info!("Real Opening {} {:?} trade", &symbol, &trade_in.trade_type,);

        match msg {
            Message::Text(txt) => {
                let end = Local::now();

                let (executed, order_id) = self.get_order_id_executed(&txt).unwrap();

                match executed {
                    true => {
                        let trans_status = self.get_transaction_status(order_id).await?;
                        match trans_status.status.is_accepted() {
                            true => {
                                let transaction_details = self
                                    .get_transaction_details(&symbol, &strategy_name, None)
                                    .await
                                    .unwrap();

                                log::info!(
                                    "Real Opened {} {:?} trade {}. Openinig price: {}",
                                    &symbol,
                                    &trade_in.trade_type,
                                    &transaction_details.id,
                                    &transaction_details.open_price
                                );

                                trade_in.id = transaction_details.id;
                                trade_in.price_in = transaction_details.open_price;
                                trade_in.ask = trans_status.ask;
                                trade_in.spread = spread;
                                accepted = true;
                                status = TradeStatus::Fulfilled;

                                log::info!(
                                    "Operation total time: {:?}",
                                    (end - start).num_milliseconds()
                                );
                            }
                            false => {
                                log::error!("Position {} not accepted by Broker:", order_id);
                                attempts += 1;
                            }
                        }
                    }
                    false => {
                        attempts += 1;
                    }
                }
            }
            _ => todo!(),
        }

        trade_in.status = status.clone();

        //sleep(Duration::from_millis(RETRY_AFTER));

        if !accepted {
            log::error!(
                "{:?} {:?} in Broker. Retrying...",
                &trade_in.trade_type,
                &trade_in.status
            );
        }
        // }

        Ok(ResponseBody {
            response: ResponseType::TradeInFulfilled,
            payload: Some(TradeResponse {
                symbol,
                accepted,
                data: trade_in,
            }),
        })
    }

    async fn close_trade_real(
        &mut self,
        trade: TradeData<TradeOut>,
    ) -> Result<ResponseBody<TradeResponse<TradeOut>>> {
        const MAX_RETRIES: usize = 3;
        const RETRY_AFTER: u64 = 250;
        let mut attempts = 0;
        let mut accepted = false;
        let mut status = TradeStatus::Rejected;
        let mut trade_out = trade.data;
        let symbol = trade.symbol;
        let strategy_name = trade.strategy_name;
        let valid_until = (Local::now() + date::Duration::minutes(3)).timestamp();
        let is_long = trade_out.trade_type.is_long();
        let command = match is_long {
            true => TransactionCommand::SellMarket.value(),
            false => TransactionCommand::BuyMarket.value(),
        };

        // while !accepted && attempts < MAX_RETRIES {
        let start = Local::now();
        let closing_price = 1.;
        let custom_comment = format!("Closing order {}", trade_out.id);

        let trade_command: Command<TransactionInfo> = Command {
            command: "tradeTransaction".to_owned(),
            arguments: TransactionInfo {
                tradeTransInfo: TradeTransactionInfo {
                    cmd: command,
                    trans_type: TransactionAction::Close.value(),
                    symbol: symbol.to_owned(),
                    customComment: custom_comment,
                    expiration: valid_until,
                    order: trade_out.id as isize,
                    price: closing_price,
                    offset: 0,
                    sl: 0.,
                    tp: 0.,
                    volume: trade_out.size,
                },
            },
        };

        self.send(&trade_command).await.unwrap();
        let msg = self.socket.read().await.unwrap();

        let (ask, bid) = self.get_ask_bid(&symbol).await?;
        let spread = ask - bid;

        log::info!(
            "Real Closing {} {:?} at {}.",
            &symbol,
            &trade_out.trade_type,
            closing_price,
        );

        match msg {
            Message::Text(txt) => {
                let end = Local::now();

                let (executed, order_id) = self.get_order_id_executed(&txt).unwrap();

                match executed {
                    true => {
                        let trans_status = self.get_transaction_status(order_id).await?;
                        match trans_status.status.is_accepted() {
                            true => {
                                let transaction_details = self
                                    .get_transactions_history(
                                        &symbol,
                                        &strategy_name,
                                        Some(trade_out.id),
                                    )
                                    .await
                                    .unwrap();

                                log::info!(
                                    "Real Closed {}_{} {:?} trade {}. Closing price: {} Profit: {}",
                                    &symbol,
                                    &strategy_name,
                                    &trade_out.trade_type,
                                    &transaction_details.id,
                                    &transaction_details.close_price,
                                    &transaction_details.profit,
                                );

                                trade_out.price_out = transaction_details.close_price;
                                trade_out.profit = transaction_details.profit;
                                trade_out.date_out = to_dbtime(Local::now());
                                trade_out.bid = trans_status.bid;
                                trade_out.ask = trans_status.ask;
                                trade_out.spread_out = trans_status.ask - trans_status.bid;
                                accepted = true;
                                status = TradeStatus::Fulfilled;

                                log::info!(
                                    "Operation total time: {:?}",
                                    (end - start).num_milliseconds()
                                );
                            }
                            false => {
                                attempts += 1;
                            }
                        }
                    }
                    false => {
                        attempts += 1;
                    }
                }
            }
            _ => todo!(),
        }

        trade_out.status = status.clone();

        //sleep(Duration::from_millis(RETRY_AFTER));

        if !accepted {
            log::error!(
                "{:?} {:?} in Broker. Retrying...",
                &trade_out.trade_type,
                &trade_out.status
            );
        }
        //}

        Ok(ResponseBody {
            response: ResponseType::TradeOutFulfilled,
            payload: Some(TradeResponse {
                symbol,
                accepted,
                data: trade_out,
            }),
        })
    }

    async fn get_transaction_status(
        &mut self,
        order_id: u64,
    ) -> Result<TransactionStatusnResponse> {
        const MAX_RETRIES: usize = 5;
        let retry_after = Duration::from_millis(1000);
        let mut attempts = 0;

        let status_command = Command {
            command: "tradeTransactionStatus".to_owned(),
            arguments: TransactionStatus { order: order_id },
        };

        while attempts < MAX_RETRIES {
            self.send(&status_command).await.unwrap();
            let msg = self.socket.read().await.unwrap();

            match msg {
                Message::Text(txt) => {
                    let transaction_status = self.parse_trade_status_data(txt)?;
                    let status = &transaction_status.status;

                    if !transaction_status.status.is_pending() {
                        log::info!("Transaction {} status {:?}", order_id, status);
                        return Ok(transaction_status);
                    } else {
                        log::info!(
                            "Transaction {:?} status {:?}. Retry after {} ms",
                            order_id,
                            status,
                            retry_after.as_millis()
                        );

                        sleep(retry_after).await;
                    }
                }
                _ => todo!(),
            };

            attempts += 1;
        }

        Err(RsAlgoError {
            err: RsAlgoErrorKind::NoResponse,
        })
    }

    async fn open_trade_test(
        &mut self,
        trade: TradeData<TradeIn>,
        _orders: Option<Vec<Order>>,
    ) -> Result<ResponseBody<TradeResponse<TradeIn>>> {
        let execution_mode = mode::from_str(&env::var("EXECUTION_MODE").unwrap());

        let symbol = &trade.symbol;
        let mut data = trade.data;

        let mut date_in = to_dbtime(Local::now());
        let trade_type = data.trade_type.clone();

        let tick = match execution_mode {
            mode::ExecutionMode::Bot => self
                .get_instrument_tick(&symbol)
                .await
                .unwrap()
                .payload
                .unwrap(),

            _ => {
                date_in = data.date_in;
                self.get_instrument_tick_test(&symbol, data.price_in, &trade_type)
                    .await
                    .unwrap()
                    .payload
                    .unwrap()
            }
        };

        let ask = tick.ask();
        let bid = tick.bid();
        let spread = tick.spread();

        let price_in = match trade_type.is_long() {
            true => ask,
            false => bid,
        };

        let trade_result_log = format!(
            "{} Test TradeIn {} accepted at ask: {} bid: {} tick",
            symbol, data.id, ask, bid
        );

        log::info!("{}", trade_result_log);

        data.id = uuid::generate_ts_id(from_dbtime(&date_in));
        data.price_in = price_in;
        data.ask = ask;
        data.date_in = date_in;
        data.spread = spread;
        data.status = TradeStatus::Fulfilled;

        let res = ResponseBody {
            response: ResponseType::TradeInFulfilled,
            payload: Some(TradeResponse {
                symbol: trade.symbol,
                accepted: true,
                data: data,
            }),
        };

        Ok(res)
    }

    async fn close_trade(
        &mut self,
        trade: TradeData<TradeOut>,
    ) -> Result<ResponseBody<TradeResponse<TradeOut>>> {
        let is_prod = environment::from_str(&env::var("ENV").unwrap()).is_prod();

        match is_prod {
            true => self.close_trade_real(trade).await,
            false => self.close_trade_test(trade).await,
        }
    }

    async fn close_trade_test(
        &mut self,
        trade: TradeData<TradeOut>,
    ) -> Result<ResponseBody<TradeResponse<TradeOut>>> {
        let execution_mode = mode::from_str(&env::var("EXECUTION_MODE").unwrap());

        let symbol = &trade.symbol;
        let mut data = trade.data;
        let trade_type = data.trade_type.clone();

        let mut date_out = to_dbtime(Local::now());

        let tick = match execution_mode {
            mode::ExecutionMode::Bot => self
                .get_instrument_tick(&symbol)
                .await
                .unwrap()
                .payload
                .unwrap(),
            _ => {
                date_out = data.date_out;
                self.get_instrument_tick_test(&symbol, data.price_out, &trade_type)
                    .await
                    .unwrap()
                    .payload
                    .unwrap()
            }
        };

        let ask = tick.ask();
        let bid = tick.bid();
        let spread = tick.spread();

        let non_profitable_outs = trade.options.non_profitable_out;

        let price_in = data.price_in;

        let price_out = match trade_type.is_long() {
            true => bid,
            false => ask,
        };

        let size = data.size;
        let leverage = 30.;
        let profit =
            calculate_trade_profit(size, price_in, price_out, leverage, &trade_type, symbol);

        let is_profitable = match profit {
            _ if profit > 0. => true,
            _ => false,
        };

        let accepted = match non_profitable_outs {
            true => true,
            false => is_profitable,
        };

        let str_accepted = match accepted {
            true => "accepted",
            false => "NOT accepted",
        };

        let status = match accepted {
            true => TradeStatus::Fulfilled,
            false => TradeStatus::Rejected,
        };

        let trade_result_log = format!(
            "Test {:?} {} {} {} with profit {}",
            trade_type, data.id, trade.symbol, str_accepted, profit
        );

        if accepted {
            log::info!("{}", trade_result_log);
        } else {
            log::error!("{}", trade_result_log);
        }

        data.price_out = price_out;
        data.date_out = date_out;
        data.bid = bid;
        data.profit = profit;
        data.ask = ask;
        data.spread_out = spread;
        data.status = status;

        let res = ResponseBody {
            response: ResponseType::TradeOutFulfilled,
            payload: Some(TradeResponse {
                symbol: trade.symbol,
                accepted,
                data,
            }),
        };
        Ok(res)
    }

    async fn get_active_positions(
        &mut self,
        symbol: &str,
        strategy_name: &str,
    ) -> Result<ResponseBody<PositionResult>> {
        let active_positions_command = Command {
            command: "getTrades".to_owned(),
            arguments: GetTrades { openedOnly: true },
        };

        self.send(&active_positions_command).await.unwrap();
        let msg = self.socket.read().await.unwrap();

        let res = match msg {
            Message::Text(txt) => {
                let position_result = self
                    .parse_active_positions_data(txt, symbol, strategy_name)
                    .unwrap();
                ResponseBody {
                    response: ResponseType::GetActivePositions,
                    payload: Some(position_result),
                }
            }
            _ => panic!(),
        };

        Ok(res)
    }

    async fn open_order(
        &mut self,
        trade: TradeData<TradeIn>,
        order: TradeData<Order>,
    ) -> Result<ResponseBody<TradeResponse<TradeIn>>> {
        let is_prod = environment::from_str(&env::var("ENV").unwrap()).is_prod();

        match is_prod {
            true => self.open_trade_real(trade, Some(vec![order.data])).await,
            false => self.open_order_test(trade, order).await,
        }
    }

    async fn open_order_test(
        &mut self,
        trade: TradeData<TradeIn>,
        order: TradeData<Order>,
    ) -> Result<ResponseBody<TradeResponse<TradeIn>>> {
        let execution_mode = mode::from_str(&env::var("EXECUTION_MODE").unwrap());

        let mut date_in = to_dbtime(Local::now());

        let symbol = &order.symbol;
        let order = order.data;
        let trade = trade.data;
        let trade_type = match order.order_type.is_long() {
            true => TradeType::OrderInLong,
            false => TradeType::OrderInShort,
        };
        let size = order.size;

        let tick = match execution_mode {
            mode::ExecutionMode::Bot => self
                .get_instrument_tick(&symbol)
                .await
                .unwrap()
                .payload
                .unwrap(),
            _ => {
                date_in = trade.date_in;
                self.get_instrument_tick_test(&symbol, order.target_price, &trade_type)
                    .await
                    .unwrap()
                    .payload
                    .unwrap()
            }
        };

        let ask = tick.ask();
        let bid = tick.bid();
        let spread = tick.spread();

        let price_in = match trade_type.is_long() {
            true => ask,
            false => bid,
        };

        let accepted = true;

        let str_accepted = match accepted {
            true => "accepted",
            false => "NOT accepted",
        };

        let status = match accepted {
            true => TradeStatus::Fulfilled,
            false => TradeStatus::Rejected,
        };

        let trade_result_log = format!(
            "{} Test MarketOrderIn {} Order {} at ask: {} bid: {} tick",
            symbol, trade.id, str_accepted, ask, bid
        );

        if accepted {
            log::info!("{}", trade_result_log);
        } else {
            log::error!("{}", trade_result_log);
        }

        let trade_in = TradeIn {
            id: uuid::generate_ts_id(from_dbtime(&date_in)),
            index_in: trade.index_in,
            size,
            origin_price: order.origin_price,
            price_in,
            ask: ask,
            spread,
            trade_type,
            date_in: date_in,
            status,
        };

        let res = ResponseBody {
            response: ResponseType::TradeInFulfilled,
            payload: Some(TradeResponse {
                symbol: symbol.clone(),
                accepted: true,
                data: trade_in,
            }),
        };

        Ok(res)
    }
    async fn close_order(
        &mut self,
        trade: TradeData<TradeOut>,
        order: TradeData<Order>,
    ) -> Result<ResponseBody<TradeResponse<TradeOut>>> {
        let is_prod = environment::from_str(&env::var("ENV").unwrap()).is_prod();

        match is_prod {
            true => self.close_trade_real(trade).await,
            false => self.close_order_test(trade, order).await,
        }
    }
    async fn close_order_test(
        &mut self,
        trade: TradeData<TradeOut>,
        order: TradeData<Order>,
    ) -> Result<ResponseBody<TradeResponse<TradeOut>>> {
        let execution_mode = mode::from_str(&env::var("EXECUTION_MODE").unwrap());

        let symbol = &order.symbol;
        let order_data = order.data;
        let mut trade_data = trade.data;
        let mut date_out = to_dbtime(Local::now());
        let trade_type = trade_data.trade_type.clone();
        let order_type = order_data.order_type;

        let tick = match execution_mode {
            mode::ExecutionMode::Bot => self
                .get_instrument_tick(&symbol)
                .await
                .unwrap()
                .payload
                .unwrap(),
            _ => {
                date_out = trade_data.date_out;
                self.get_instrument_tick_test(&symbol, order_data.target_price, &trade_type)
                    .await
                    .unwrap()
                    .payload
                    .unwrap()
            }
        };

        let ask = tick.ask();
        let bid = tick.bid();
        let spread = tick.spread();

        let non_profitable_outs = trade.options.non_profitable_out;
        let price_in = trade_data.price_in;

        let price_out = match trade_type.is_stop() {
            true => order_data.target_price,
            false => match trade_type.is_long() {
                true => bid,
                false => ask,
            },
        };

        //let price_out = price_out, &symbol;

        let size = trade_data.size;
        let leverage = 30.;
        let profit =
            calculate_trade_profit(size, price_in, price_out, leverage, &trade_type, symbol);

        let is_profitable = match profit {
            _ if profit > 0. => true,
            _ => false,
        };

        let accepted = match trade_type.is_stop() {
            true => true,
            false => match non_profitable_outs {
                true => true,
                false => is_profitable,
            },
        };

        let str_accepted = match accepted {
            true => "accepted",
            false => "NOT accepted",
        };

        let mut status = match accepted {
            true => TradeStatus::Fulfilled,
            false => TradeStatus::Rejected,
        };

        let trade_result_log = format!(
            "{:?} {} {} {} with profit {}",
            order_type, trade_data.id, trade.symbol, str_accepted, profit
        );

        if accepted {
            log::info!("{}", trade_result_log);
        } else {
            log::error!("{}", trade_result_log);
        }

        if trade_type.is_stop() && is_profitable {
            log::error!("Profitable {} StopLoss!!", trade.symbol);
            status = TradeStatus::Rejected;
        }

        trade_data.id = uuid::generate_ts_id(from_dbtime(&date_out));
        trade_data.price_out = price_out;
        trade_data.date_out = date_out;
        trade_data.bid = bid;
        trade_data.ask = ask;
        trade_data.spread_out = spread;
        trade_data.status = status;

        let res = ResponseBody {
            response: ResponseType::TradeOutFulfilled,
            payload: Some(TradeResponse {
                symbol: "".to_owned(),
                accepted,
                data: trade_data,
            }),
        };
        Ok(res)
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

    async fn subscribe_tick_prices(&mut self, symbol: &str) -> Result<()> {
        let is_prod = environment::from_str(&env::var("ENV").unwrap()).is_prod();

        let arrival_time = match is_prod {
            true => 1,
            false => 1000,
        };

        self.symbol = symbol.to_owned();
        let command = CommandTickStreamParams {
            command: "getTickPrices".to_owned(),
            streamSessionId: self.streamSessionId.clone(),
            symbol: symbol.to_string(),
            minArrivalTime: arrival_time,
            maxLevel: 0,
        };

        self.send_stream(&command).await.unwrap();

        Ok(())
    }

    async fn subscribe_trades(&mut self, symbol: &str) -> Result<()> {
        let command = CommandTradeStatusParams {
            command: "getTrades".to_owned(),
            streamSessionId: self.streamSessionId.clone(),
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

    async fn parse_stream_data(msg: Message, symbol: &str, strategy_name: &str) -> Option<String> {
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
                    let date = parse_time_seconds(data["ctm"].as_i64().unwrap() / 1000);
                    let open = data["open"].as_f64().unwrap();
                    let high = data["high"].as_f64().unwrap();
                    let low = data["low"].as_f64().unwrap();
                    let close = data["close"].as_f64().unwrap();
                    let size = data["vol"].as_f64().unwrap() * 1000.;

                    let ohlc = (date, open, high, low, close, size);

                    let msg: ResponseBody<(DateTime<Local>, f64, f64, f64, f64, f64)> =
                        ResponseBody {
                            response: ResponseType::SubscribeStream,
                            payload: Some(ohlc),
                        };

                    Some(serde_json::to_string(&msg).unwrap())
                } else if command == "tickPrices" {
                    let symbol = data["symbol"].as_str().unwrap().to_owned();
                    let ask = data["ask"].as_f64().unwrap();
                    let bid = data["bid"].as_f64().unwrap();
                    let high = data["high"].as_f64().unwrap();
                    let low = data["low"].as_f64().unwrap();
                    let timestamp = data["timestamp"].as_i64().unwrap();
                    let spread = data["spreadRaw"].as_f64().unwrap();

                    let tick = InstrumentTick::new()
                        .symbol(symbol)
                        .ask(ask)
                        .bid(bid)
                        .high(high)
                        .low(low)
                        .spread(spread)
                        .time(timestamp)
                        .build()
                        .unwrap();

                    let msg: ResponseBody<InstrumentTick> = ResponseBody {
                        response: ResponseType::SubscribeTickPrices,
                        payload: Some(tick),
                    };
                    Some(serde_json::to_string(&msg).unwrap())
                } else if command == "trade" {
                    match data["closed"].as_bool() {
                        Some(is_closed) => {
                            // let cmd = TransactionCommand::from_value(
                            //     data["cmd"].as_u64().unwrap() as i64
                            // )
                            // .unwrap();

                            let comment = data["comment"].as_str().unwrap_or_default();
                            let is_stop = comment == "[S/L]";

                            //FILTER ONLY STOPS FOR NOW
                            if is_closed && is_stop {
                                let stream_symbol = data["symbol"].as_str().unwrap();
                                let comments = data["customComment"].as_str().unwrap();
                                let trans_comments: TransactionComments =
                                    serde_json::from_str(&comments).unwrap();

                                // TAKING ONLY THE OWN SYMBOL & STRATEGY COMBINATION
                                if symbol == stream_symbol
                                    && strategy_name == trans_comments.strategy_name
                                {
                                    log::info!("DATA {:?}", data);
                                    let id = data["position"].as_u64().unwrap() as usize;
                                    let size = data["volume"].as_f64().unwrap();
                                    let price_in = data["open_price"].as_f64().unwrap();
                                    let price_out = data["close_price"].as_f64().unwrap();

                                    let index_in = trans_comments.index_in;
                                    let spread_in = trans_comments.spread;
                                    let strategy_name = trans_comments.strategy_name;

                                    let gross_profit = obj["profit"].as_f64().unwrap_or(0.0);
                                    let swap = obj["storage"].as_f64().unwrap_or(0.0);
                                    let commission = obj["commission"].as_f64().unwrap_or(0.0);

                                    let profit = gross_profit + swap + commission;

                                    log::info!("PROFIT DATA Gross Profit: {} Swap: {} Comission: {} Net Profit: {}", gross_profit, swap, commission, profit);

                                    let spread_out = 0.;
                                    let close_time = Local::now();
                                    let date_in = to_dbtime(parse_time_seconds(
                                        data["open_time"].as_i64().unwrap() / 1000,
                                    ));

                                    let date_out = to_dbtime(close_time);
                                    let index_out = uuid::generate_ts_id(close_time);

                                    let trade_type = match is_stop {
                                        true => match trans_comments.trade_type.is_long() {
                                            true => TradeType::StopLossLong,
                                            false => TradeType::StopLossShort,
                                        },
                                        false => match trans_comments.trade_type.is_long() {
                                            true => TradeType::MarketOutLong,
                                            false => TradeType::MarketOutShort,
                                        },
                                    };

                                    let bid = match trade_type.is_long() {
                                        true => price_out,
                                        false => price_out + spread_out,
                                    };

                                    let profit_per = 0.;
                                    let run_up = 0.;
                                    let run_up_per = 0.;
                                    let draw_down = 0.;
                                    let draw_down_per = 0.;
                                    let status = TradeStatus::Fulfilled;

                                    log::info!(
                                    "Real Closed {}_{} {:?} trade {}. Closing price: {} Profit: {}",
                                    &symbol,
                                    &strategy_name,
                                    &trade_type,
                                    &id,
                                    &price_out,
                                    &profit,
                                );

                                    let trade_out = TradeOut {
                                        id,
                                        index_in,
                                        price_in,
                                        status,
                                        size,
                                        trade_type: trade_type.clone(),
                                        date_in,
                                        spread_in,
                                        ask: price_in,
                                        index_out,
                                        price_origin: price_in,
                                        price_out,
                                        bid,
                                        spread_out,
                                        date_out,
                                        profit,
                                        profit_per,
                                        run_up,
                                        run_up_per,
                                        draw_down,
                                        draw_down_per,
                                    };

                                    let msg = ResponseBody {
                                        response: ResponseType::TradeOutFulfilled,
                                        payload: Some(TradeResponse {
                                            symbol: symbol.to_owned(),
                                            accepted: true,
                                            data: trade_out,
                                        }),
                                    };

                                    Some(serde_json::to_string(&msg).unwrap())
                                } else {
                                    None
                                }
                            } else {
                                None
                            }
                        }
                        None => None,
                    }
                } else {
                    None
                }
            }
            _ => None,
        };

        msg
    }

    async fn keepalive_ping(&mut self) -> Result<String> {
        let ping_command = Ping {
            command: "ping".to_owned(),
        };

        self.send(&ping_command).await.unwrap();
        let msg = self.socket.read().await.unwrap();
        let res = match msg {
            Message::Text(txt) => txt,
            _ => panic!(),
        };

        Ok(res)
    }

    async fn disconnect(&mut self) -> Result<()> {
        log::info!("Disconnecting from broker");
        self.socket.disconnect().await.unwrap();
        self.stream.disconnect().await.unwrap();
        Ok(())
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

    async fn send_str(&mut self, command: &str) -> Result<()> {
        self.socket.send(&command).await?;

        Ok(())
    }

    // async fn send_and_read<T>(&mut self, command: &T) -> Result<()>
    // where
    //     for<'de> T: Serialize + Deserialize<'de> + Debug,
    // {
    //     self.socket
    //         .send_and_read(&serde_json::to_string(&command).unwrap())
    //         .await?;

    //     Ok(())
    // }

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
        let res = match msg {
            Message::Text(txt) => txt,
            _ => panic!(),
        };
        let res = self.handle_response::<VEC_DOHLC>(&res).await.unwrap();

        Ok(res)
    }

    pub fn parse_message(&mut self, msg: &str) -> Result<Value> {
        let parsed: Value = serde_json::from_str(&msg).expect("Can't parse to JSON");
        Ok(parsed)
    }

    pub async fn handle_response<'a, T>(
        &mut self,
        msg: &str,
    ) -> Result<ResponseBody<InstrumentData<VEC_DOHLC>>> {
        let data = self.parse_message(&msg).unwrap();
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
            // InstrumentTick Data
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
        let return_data = data
            .get("returnData")
            .ok_or_else(|| RsAlgoErrorKind::ParseError)
            .unwrap();
        let digits = return_data["digits"].as_f64().unwrap();
        let x = 10.0_f64;
        let pow = x.powf(digits);
        for obj in return_data["rateInfos"].as_array().unwrap() {
            //FIXME!!
            let date = parse_time_seconds(obj["ctm"].as_i64().unwrap() / 1000);
            let open = obj["open"].as_f64().unwrap() / pow;
            let high = open + obj["high"].as_f64().unwrap() / pow;
            let low = open + obj["low"].as_f64().unwrap() / pow;
            let close = open + obj["close"].as_f64().unwrap() / pow;
            let size = obj["vol"].as_f64().unwrap() * 1000.;

            result.push((date, open, high, low, close, size));
        }

        Ok(result)
    }

    pub fn parse_tick_data(&mut self, symbol: String, txt: String) -> Result<InstrumentTick> {
        let data = self.parse_message(&txt).unwrap();
        let return_data = data
            .get("returnData")
            .ok_or_else(|| RsAlgoErrorKind::ParseError)
            .unwrap();

        let ask = return_data["ask"].as_f64().unwrap();
        let bid = return_data["bid"].as_f64().unwrap();
        let high = return_data["high"].as_f64().unwrap();
        let low = return_data["low"].as_f64().unwrap();
        //OJO
        let pip_size = return_data["tickSize"].as_f64().unwrap();
        let spread = return_data["spreadRaw"].as_f64().unwrap();

        let tick = InstrumentTick::new()
            .symbol(symbol)
            .ask(ask)
            .bid(bid)
            .high(high)
            .low(low)
            .spread(spread)
            .pip_size(pip_size)
            .time(Local::now().timestamp())
            .build()
            .unwrap();

        Ok(tick)
    }

    pub fn get_order_id_executed(&mut self, txt: &str) -> Result<(bool, u64)> {
        let data = self.parse_message(txt).unwrap();

        let mut status = data["status"]
            .as_bool()
            .ok_or("Missing or invalid 'status' in response data")
            .unwrap();

        let order_num_result = data["returnData"]["order"]
            .as_u64()
            .ok_or("Missing or invalid 'order' in 'returnData'");

        let order_num = match order_num_result {
            Ok(num) => num,
            Err(_) => {
                log::error!("Error parsing trade data: {}", txt);
                status = false;
                0
            }
        };

        Ok((status, order_num))
    }

    pub fn parse_trade_status_data(&mut self, txt: String) -> Result<TransactionStatusnResponse> {
        let data = self.parse_message(&txt).unwrap();
        let return_data = data
            .get("returnData")
            .ok_or_else(|| RsAlgoErrorKind::ParseError)
            .unwrap();
        let order = return_data["order"].as_u64().unwrap();
        let ask = return_data["ask"].as_f64().unwrap();
        let bid = return_data["bid"].as_f64().unwrap();
        let message = "".to_owned(); //data["returnData"]["message"].as_str().unwrap().to_owned();
        let status = TransactionState::from_value(return_data["requestStatus"].as_u64().unwrap());
        let comment = return_data["customComment"].as_str().unwrap().to_owned();

        Ok(TransactionStatusnResponse {
            comment,
            message,
            order,
            ask,
            bid,
            status,
        })
    }

    pub fn parse_market_hours(&mut self, data: &Value) -> Result<Vec<MarketHour>> {
        let mut result: Vec<MarketHour> = vec![];
        let current_date = Local::now();
        let base = current_date.date().and_hms(0, 0, 0);
        let return_data = data
            .get("returnData")
            .ok_or_else(|| RsAlgoErrorKind::ParseError)
            .unwrap();

        for obj in return_data["trading"].as_array().unwrap() {
            let day = obj["day"].as_i64().unwrap().try_into().unwrap();
            let from = obj["from"].as_i64().unwrap();
            let to = obj["to"].as_i64().unwrap();
            let date_from = (base + date::Duration::milliseconds(from)).hour();
            let date_to = (base + date::Duration::milliseconds(to)).hour();
            let market_hour = MarketHour {
                day,
                from: date_from,
                to: date_to,
            };
            result.push(market_hour);
        }
        Ok(result)
    }

    pub fn parse_active_positions_data(
        &mut self,
        txt: String,
        symbol: &str,
        strategy_name: &str,
    ) -> Result<PositionResult> {
        let data = self.parse_message(&txt).unwrap();
        let current_date = Local::now();
        let mut trade_in = TradeIn::default();
        let data = data["returnData"].as_array().unwrap();
        let mut found = false;

        let mut orders: Vec<Order> = vec![];
        for obj in data {
            let order_symbol = obj["symbol"].as_str().unwrap();
            let comment = obj["customComment"].as_str().unwrap().to_owned();
            let trans_comments: TransactionComments = serde_json::from_str(&comment).unwrap();
            let symbol_strategy_name = trans_comments.strategy_name;

            if order_symbol == symbol && strategy_name == symbol_strategy_name {
                found = true;
                let id = obj["position"].as_i64().unwrap().try_into().unwrap();
                let size = obj["volume"].as_f64().unwrap().try_into().unwrap();
                let price_in = obj["open_price"].as_f64().unwrap().try_into().unwrap();
                let origin_price = price_in;
                let stop_loss = obj["sl"].as_f64().unwrap().try_into().unwrap();
                let date_in = to_dbtime(date::parse_time_milliseconds(
                    obj["open_time"].as_i64().unwrap().try_into().unwrap(),
                ));

                let index_in = trans_comments.index_in;
                let trade_type = trans_comments.trade_type;
                let spread = trans_comments.spread;
                let bid = trans_comments.bid;
                let ask = bid + spread;
                let sell_order_target = match trans_comments.sell_order_price {
                    Some(price) => price,
                    None => 0.0,
                };

                let stop_order_type = match trade_type.is_long() {
                    true => OrderType::StopLossLong(StopLossType::Price(stop_loss), price_in),
                    false => OrderType::StopLossShort(StopLossType::Price(stop_loss), price_in),
                };

                let sell_order_type = match trade_type.is_long() {
                    true => OrderType::SellOrderLong(size, sell_order_target),
                    false => OrderType::SellOrderShort(size, sell_order_target),
                };

                trade_in = TradeIn {
                    id,
                    index_in,
                    size,
                    origin_price,
                    price_in,
                    ask,
                    spread,
                    date_in,
                    trade_type,
                    status: TradeStatus::Fulfilled,
                };

                let stop_loss_order = Order {
                    id,
                    trade_id: id,
                    index_created: id,
                    size,
                    order_type: stop_order_type,
                    index_fulfilled: 0,
                    status: OrderStatus::Pending,
                    origin_price,
                    target_price: stop_loss,
                    created_at: date_in,
                    updated_at: None,
                    full_filled_at: None,
                    valid_until: Some(to_dbtime(current_date + date::Duration::days(365 * 10))),
                    meta: None,
                };

                orders.push(stop_loss_order);

                match trans_comments.sell_order_price {
                    Some(sell_price) => {
                        let sell_order = Order {
                            id,
                            trade_id: id,
                            index_created: id,
                            size,
                            order_type: sell_order_type,
                            index_fulfilled: 0,
                            status: OrderStatus::Pending,
                            origin_price,
                            target_price: sell_price,
                            created_at: date_in,
                            updated_at: None,
                            full_filled_at: None,
                            valid_until: Some(to_dbtime(
                                current_date + date::Duration::days(365 * 10),
                            )),
                            meta: None,
                        };

                        orders.push(sell_order);
                    }
                    None => (),
                };
            }
        }

        let result = match found {
            true => PositionResult::MarketIn(TradeResult::TradeIn(trade_in), Some(orders)),
            false => PositionResult::None,
        };

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
