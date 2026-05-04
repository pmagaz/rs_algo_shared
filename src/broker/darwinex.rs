use crate::broker::broker_trait::BrokerStream;
use crate::broker::models::*;
use crate::error::{Result, RsAlgoError, RsAlgoErrorKind};
use crate::helpers::calc::number_pips;
use crate::helpers::date::Local;
use crate::models::market::*;
use crate::models::order::*;
use crate::models::swap::InstrumentSwap;
use crate::models::tick::InstrumentTick;
use crate::models::time_frame::TimeFrameType;
use crate::models::trade::*;
use crate::ws::message::{
    InstrumentData, Message, ResponseBody, ResponseType, TradeData, TradeResponse,
};
use crate::ws::ws_stream_client::WebSocket as WebSocketStream;

use futures_util::StreamExt;
use serde::Deserialize;
use serde_json::Value;
use std::env;
use std::time::Duration;
use tokio::sync::mpsc;

#[derive(Debug)]
pub struct Darwinex {
    ws: Option<WebSocketStream>,
    http: reqwest::Client,
    access_token: String,
    refresh_token: String,
    consumer_key: String,
    consumer_secret: String,
    symbol: String,
    account_id: String,
    api_base: String,
    token_url: String,
    ws_url: String,
}

#[derive(Debug, Deserialize)]
struct TokenResponse {
    access_token: String,
    refresh_token: String,
    expires_in: u64,
}

fn base64_encode(input: &str) -> String {
    let bytes = input.as_bytes();
    let table = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = String::new();
    for chunk in bytes.chunks(3) {
        let b0 = chunk[0] as u32;
        let b1 = if chunk.len() > 1 { chunk[1] as u32 } else { 0 };
        let b2 = if chunk.len() > 2 { chunk[2] as u32 } else { 0 };
        let n = (b0 << 16) | (b1 << 8) | b2;
        out.push(table[((n >> 18) & 63) as usize] as char);
        out.push(table[((n >> 12) & 63) as usize] as char);
        out.push(if chunk.len() > 1 { table[((n >> 6) & 63) as usize] as char } else { '=' });
        out.push(if chunk.len() > 2 { table[(n & 63) as usize] as char } else { '=' });
    }
    out
}

#[async_trait::async_trait]
impl BrokerStream for Darwinex {
    async fn new() -> Self {
        let api_base = env::var("DARWINEX_API_BASE_URL")
            .unwrap_or_else(|_| "https://api.darwinex.com".to_string());
        let token_url = env::var("DARWINEX_TOKEN_URL")
            .unwrap_or_else(|_| "https://api.darwinex.com/token".to_string());
        let ws_url = env::var("DARWINEX_WS_URL")
            .unwrap_or_else(|_| "wss://api.darwinex.com/quotewebsocket/1.0.0".to_string());
        let account_id = env::var("DARWINEX_ACCOUNT_ID").unwrap_or_default();

        let consumer_key = env::var("DARWINEX_CONSUMER_KEY").unwrap_or_default();
        let consumer_secret = env::var("DARWINEX_CONSUMER_SECRET").unwrap_or_default();
        let refresh_token = env::var("DARWINEX_REFRESH_TOKEN").unwrap_or_default();

        let http = reqwest::Client::builder()
            .connect_timeout(Duration::from_secs(15))
            .timeout(Duration::from_secs(30))
            .build()
            .expect("Failed to build reqwest client");

        Self {
            ws: None,
            http,
            access_token: String::new(),
            refresh_token,
            consumer_key,
            consumer_secret,
            symbol: String::new(),
            account_id,
            api_base,
            token_url,
            ws_url,
        }
    }

    async fn login(&mut self, _username: &str, _password: &str) -> Result<&mut Self> {
        // Darwinex uses OAuth2 refresh token grant (not password grant).
        // Consumer key/secret and refresh token come from env vars set at startup.
        // Initial tokens must be obtained manually from https://www.darwinex.com/data/darwin-api
        log::info!("Darwinex: refreshing OAuth token via {}", self.token_url);

        let refresh_token = self.refresh_token.clone();
        let basic = base64_encode(&format!("{}:{}", self.consumer_key, self.consumer_secret));

        let resp = self
            .http
            .post(&self.token_url)
            .header("Authorization", format!("Basic {}", basic))
            .header("Content-Type", "application/x-www-form-urlencoded")
            .form(&[
                ("grant_type", "refresh_token"),
                ("refresh_token", &refresh_token),
            ])
            .send()
            .await
            .map_err(|e| {
                log::error!("Darwinex: token request failed: {}", e);
                RsAlgoErrorKind::ConnectionError
            })?;

        let status = resp.status();
        if !status.is_success() {
            let body = resp.text().await.unwrap_or_default();
            log::error!("Darwinex: token endpoint returned {}: {}", status, body);
            return Err(RsAlgoError::from(RsAlgoErrorKind::ConnectionError).into());
        }

        let token_data: TokenResponse = resp.json().await.map_err(|e| {
            log::error!("Darwinex: failed to parse token response: {}", e);
            RsAlgoErrorKind::ParseError
        })?;

        self.access_token = token_data.access_token.clone();
        self.refresh_token = token_data.refresh_token.clone();
        log::info!("Darwinex: OAuth token acquired (expires_in={}s)", token_data.expires_in);

        // Connect WebSocket with Bearer token in Authorization header
        self.ws = Some(
            WebSocketStream::connect_with_auth(&self.ws_url, &self.access_token).await,
        );
        log::info!("Darwinex: connected to quote WebSocket");

        Ok(self)
    }

    async fn disconnect(&mut self) -> Result<()> {
        log::info!("Darwinex: disconnecting");
        if let Some(ws) = self.ws.as_mut() {
            ws.disconnect().await?;
        }
        Ok(())
    }

    async fn keepalive_ping(&mut self) -> Result<()> {
        if let Some(ws) = self.ws.as_mut() {
            ws.ping(&[]).await;
        }
        Ok(())
    }

    // ── Market data ───────────────────────────────────────────────────────

    async fn get_instrument_data(
        &mut self,
        symbol: &str,
        period: usize,
        start: i64,
    ) -> Result<ResponseBody<InstrumentData<VEC_DOHLC>>> {
        // TODO: Darwinex Darwin Info API historical OHLC endpoint
        // Confirm endpoint path from: https://darwinex.github.io/darwin-api-tutorials/
        log::warn!("Darwinex: get_instrument_data not yet implemented, returning empty");
        Ok(ResponseBody {
            response: ResponseType::GetInstrumentData,
            payload: Some(InstrumentData {
                symbol: symbol.to_owned(),
                time_frame: TimeFrameType::from_number(period),
                data: vec![],
            }),
        })
    }

    async fn get_historic_data(
        &mut self,
        symbol: &str,
        period: usize,
        start: i64,
        end: i64,
    ) -> Result<ResponseBody<InstrumentData<VEC_DOHLC>>> {
        // TODO: Darwinex Darwin Info API historical OHLC endpoint
        log::warn!("Darwinex: get_historic_data not yet implemented, returning empty");
        Ok(ResponseBody {
            response: ResponseType::GetInstrumentData,
            payload: Some(InstrumentData {
                symbol: symbol.to_owned(),
                time_frame: TimeFrameType::from_number(period),
                data: vec![],
            }),
        })
    }

    async fn get_instrument_tick(&mut self, symbol: &str) -> Result<ResponseBody<InstrumentTick>> {
        // GET {api_base}/productquotes/1.0/{symbol}
        let url = format!("{}/productquotes/1.0/{}", self.api_base, symbol);
        let resp = self
            .http
            .get(&url)
            .bearer_auth(&self.access_token)
            .send()
            .await
            .map_err(|_| RsAlgoErrorKind::RequestError)?;

        let data: Value = resp
            .json()
            .await
            .map_err(|_| RsAlgoErrorKind::ParseError)?;

        let ask = data["ask"].as_f64().unwrap_or(0.0);
        let bid = data["bid"].as_f64().unwrap_or(0.0);
        let spread = ask - bid;
        let pip_size = number_pips(symbol);

        let tick = InstrumentTick::new()
            .symbol(symbol.to_string())
            .ask(ask)
            .bid(bid)
            .high(ask)
            .low(bid)
            .spread(spread)
            .pip_size(pip_size)
            .time(Local::now().timestamp())
            .build()
            .map_err(|_| RsAlgoError::from(RsAlgoErrorKind::ParseError))?;

        Ok(ResponseBody {
            response: ResponseType::GetInstrumentTick,
            payload: Some(tick),
        })
    }

    async fn get_instrument_swap(&mut self, symbol: &str) -> Result<ResponseBody<InstrumentSwap>> {
        // Darwinex does not expose swap rates via API — return disabled
        let swap = InstrumentSwap::new()
            .symbol(symbol.to_string())
            .enabled(false)
            .swap_long(0.0)
            .swap_short(0.0)
            .swap_weekend(0.0)
            .build()
            .map_err(|_| RsAlgoError::from(RsAlgoErrorKind::ParseError))?;

        Ok(ResponseBody {
            response: ResponseType::GetInstrumentSwap,
            payload: Some(swap),
        })
    }

    async fn get_ask_bid(&mut self, symbol: &str) -> Result<(f64, f64)> {
        // GET {api_base}/productquotes/1.0/{symbol}
        let url = format!("{}/productquotes/1.0/{}", self.api_base, symbol);
        let resp = self
            .http
            .get(&url)
            .bearer_auth(&self.access_token)
            .send()
            .await
            .map_err(|_| RsAlgoErrorKind::RequestError)?;

        let data: Value = resp
            .json()
            .await
            .map_err(|_| RsAlgoErrorKind::ParseError)?;

        let ask = data["ask"].as_f64().unwrap_or(0.0);
        let bid = data["bid"].as_f64().unwrap_or(0.0);
        Ok((ask, bid))
    }

    async fn get_symbols(&mut self) -> Result<ResponseBody<InstrumentData<VEC_DOHLC>>> {
        // TODO: Darwinex product list endpoint
        Ok(ResponseBody {
            response: ResponseType::GetInstrumentData,
            payload: None,
        })
    }

    // ── Market status ─────────────────────────────────────────────────────

    async fn get_market_hours(&mut self, symbol: &str) -> Result<ResponseBody<MarketHours>> {
        // TODO: derive from Darwinex schedule API if available
        Ok(ResponseBody {
            response: ResponseType::GetMarketHours,
            payload: Some(MarketHours::default()),
        })
    }

    async fn is_market_open(&mut self, symbol: &str) -> Result<ResponseBody<bool>> {
        // TODO: check against Darwinex trading hours
        Ok(ResponseBody {
            response: ResponseType::IsMarketOpen,
            payload: Some(true),
        })
    }

    async fn is_market_available(&mut self, _symbol: &str) -> bool {
        true
    }

    // ── Trading ───────────────────────────────────────────────────────────

    async fn open_trade(
        &mut self,
        trade: TradeData<TradeIn>,
        _orders: Option<Vec<Order>>,
    ) -> Result<ResponseBody<TradeResponse<TradeIn>>> {
        // POST {api_base}/darwintrading/1.0/{account_id}/portfolio/{darwin}
        let url = format!(
            "{}/darwintrading/1.0/{}/portfolio/{}",
            self.api_base, self.account_id, trade.symbol
        );

        let body = serde_json::json!({ "amount": trade.data.size });

        let resp = self
            .http
            .post(&url)
            .bearer_auth(&self.access_token)
            .json(&body)
            .send()
            .await
            .map_err(|_| RsAlgoErrorKind::RequestError)?;

        let accepted = resp.status().is_success();
        log::info!("Darwinex open_trade {} accepted={}", trade.symbol, accepted);

        Ok(ResponseBody {
            response: ResponseType::TradeInFulfilled,
            payload: Some(TradeResponse {
                symbol: trade.symbol.clone(),
                accepted,
                data: trade.data,
            }),
        })
    }

    async fn close_trade(
        &mut self,
        trade: TradeData<TradeOut>,
    ) -> Result<ResponseBody<TradeResponse<TradeOut>>> {
        // DELETE {api_base}/darwintrading/1.0/{account_id}/portfolio/{darwin}/{order_id}
        let url = format!(
            "{}/darwintrading/1.0/{}/portfolio/{}/{}",
            self.api_base, self.account_id, trade.symbol, trade.data.id
        );

        let resp = self
            .http
            .delete(&url)
            .bearer_auth(&self.access_token)
            .send()
            .await
            .map_err(|_| RsAlgoErrorKind::RequestError)?;

        let accepted = resp.status().is_success();
        log::info!(
            "Darwinex close_trade {} id={} accepted={}",
            trade.symbol,
            trade.data.id,
            accepted
        );

        Ok(ResponseBody {
            response: ResponseType::TradeOutFulfilled,
            payload: Some(TradeResponse {
                symbol: trade.symbol.clone(),
                accepted,
                data: trade.data,
            }),
        })
    }

    async fn open_order(
        &mut self,
        _trade: TradeData<TradeIn>,
        _order: TradeData<Order>,
    ) -> Result<ResponseBody<TradeResponse<TradeIn>>> {
        // TODO: Darwinex limit order endpoint
        Err(RsAlgoError::from(RsAlgoErrorKind::RequestError).into())
    }

    async fn close_order(
        &mut self,
        _trade: TradeData<TradeOut>,
        _order: TradeData<Order>,
    ) -> Result<ResponseBody<TradeResponse<TradeOut>>> {
        // TODO: Darwinex cancel order endpoint
        Err(RsAlgoError::from(RsAlgoErrorKind::RequestError).into())
    }

    // ── Positions & history ───────────────────────────────────────────────

    async fn get_active_positions(
        &mut self,
        _symbol: &str,
        _strategy_name: &str,
    ) -> Result<ResponseBody<PositionResult>> {
        // GET {api_base}/investoraccountinfo/1.0/{account_id}/productportfolio
        let url = format!(
            "{}/investoraccountinfo/1.0/{}/productportfolio",
            self.api_base, self.account_id
        );

        let resp = self
            .http
            .get(&url)
            .bearer_auth(&self.access_token)
            .send()
            .await
            .map_err(|_| RsAlgoErrorKind::RequestError)?;

        let _data: Value = resp
            .json()
            .await
            .map_err(|_| RsAlgoErrorKind::ParseError)?;

        // TODO: map Darwinex portfolio response → PositionResult
        Ok(ResponseBody {
            response: ResponseType::GetActivePositions,
            payload: None,
        })
    }

    async fn get_transaction_details(
        &mut self,
        _symbol: &str,
        _strategy_name: &str,
        _id: Option<usize>,
    ) -> Option<TransactionDetails> {
        // TODO: GET {api_base}/investoraccountinfo/1.0/{account_id}/orders
        None
    }

    async fn get_transactions_history(
        &mut self,
        _symbol: &str,
        _strategy_name: &str,
        _id: Option<usize>,
    ) -> Option<TransactionDetails> {
        // TODO: GET {api_base}/investoraccountinfo/1.0/{account_id}/closedpositions
        None
    }

    // ── Streaming ─────────────────────────────────────────────────────────

    async fn subscribe_stream(
        &mut self,
        symbol: &str,
        strategy_name: &str,
    ) -> Result<mpsc::UnboundedReceiver<String>> {
        let ws = self
            .ws
            .as_mut()
            .expect("Darwinex WS not connected — call login() first");

        // Send Darwinex-specific subscribe command (internal detail)
        let subscribe_msg = serde_json::json!({
            "op": "subscribe",
            "productNames": [symbol]
        });
        ws.send(&subscribe_msg.to_string()).await?;

        let mut ws_read = ws.take_read();
        let symbol = symbol.to_owned();
        let strategy_name = strategy_name.to_owned();
        let (tx, rx) = mpsc::unbounded_channel();

        tokio::spawn(async move {
            while let Some(msg_result) = ws_read.next().await {
                match msg_result {
                    Ok(Message::Text(txt)) => {
                        if let Some(parsed) =
                            Darwinex::parse_stream_data(&txt, &symbol, &strategy_name)
                        {
                            if tx.send(parsed).is_err() {
                                break;
                            }
                        }
                    }
                    Ok(Message::Close(_)) | Err(_) => {
                        log::error!("Darwinex: WS stream closed");
                        break;
                    }
                    _ => {}
                }
            }
        });

        Ok(rx)
    }
}

impl Darwinex {
    // Internal: parse a raw Darwinex WS quote message into a serialized ResponseBody.
    // Darwinex delivers: {"productName": "EURUSD", "quote": 1.08432, "timestamp": 1715000000000}
    fn parse_stream_data(txt: &str, symbol: &str, _strategy_name: &str) -> Option<String> {
        let obj: Value = serde_json::from_str(txt).ok()?;

        let product = obj["productName"].as_str()?;
        let quote = obj["quote"].as_f64()?;
        let ts = obj["timestamp"].as_i64()?;

        if product != symbol {
            return None;
        }

        let pip_size = number_pips(symbol);

        let tick = InstrumentTick::new()
            .symbol(product.to_string())
            .ask(quote)
            .bid(quote)
            .high(quote)
            .low(quote)
            .spread(0.0)
            .pip_size(pip_size)
            .time(ts / 1000)
            .build()
            .ok()?;

        let response = ResponseBody {
            response: ResponseType::SubscribeTickPrices,
            payload: Some(tick),
        };
        serde_json::to_string(&response).ok()
    }
}
