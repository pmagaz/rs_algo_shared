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
use crate::ws::message::{InstrumentData, ResponseBody, ResponseType, TradeData, TradeResponse};

use futures_util::{SinkExt, StreamExt};
use native_tls::TlsConnector as NativeTlsConnector;
use serde_json::Value;
use std::collections::HashMap;
use std::env;
use std::time::Duration;
use tokio::sync::mpsc;
use tokio_tungstenite::{connect_async_tls_with_config, Connector};
use tokio_tungstenite::tungstenite::Message as WsMessage;

pub struct Ibkr {
    http: reqwest::Client,
    account_id: String,
    gateway_url: String,
    host_header: String,
    accept_invalid_cert: bool,
    conid_cache: HashMap<String, i64>,
}

#[async_trait::async_trait]
impl BrokerStream for Ibkr {
    async fn new() -> Self {
        let gateway_url = env::var("IBKR_GATEWAY_URL")
            .unwrap_or_else(|_| "https://localhost:5000".to_string());
        let account_id = env::var("IBKR_ACCOUNT_ID").unwrap_or_default();
        let accept_invalid_cert = env::var("IBKR_ACCEPT_INVALID_CERT")
            .map(|v| v == "true" || v == "1")
            .unwrap_or(true);

        // Strip port from gateway_url so the Host header matches what the gateway expects,
        // even when requests go through the proxy on a different port (e.g. 5100 → 5000).
        let host_header = gateway_url
            .trim_start_matches("https://")
            .trim_start_matches("http://")
            .split(':')
            .next()
            .unwrap_or("localhost")
            .to_string();

        let http = reqwest::Client::builder()
            .connect_timeout(Duration::from_secs(15))
            .timeout(Duration::from_secs(30))
            .danger_accept_invalid_certs(accept_invalid_cert)
            .build()
            .expect("Failed to build reqwest client");

        Self {
            http,
            account_id,
            gateway_url,
            host_header,
            accept_invalid_cert,
            conid_cache: HashMap::new(),
        }
    }

    async fn login(&mut self, _username: &str, _password: &str) -> Result<&mut Self> {
        tracing::info!("IBKR: initialising brokerage session at {}", self.gateway_url);

        // Poll ibeam's health endpoint (port 5001, plain HTTP, accessible from anywhere)
        // instead of the gateway's auth/status which requires localhost inside the container.
        let health_url = env::var("IBKR_HEALTH_URL")
            .unwrap_or_else(|_| ibeam_health_url(&self.gateway_url));
        let health_client = reqwest::Client::builder()
            .connect_timeout(Duration::from_secs(5))
            .timeout(Duration::from_secs(10))
            .build()
            .unwrap_or_else(|_| reqwest::Client::new());

        let ready_url = format!("{}/readyz", health_url);
        let mut authenticated = false;
        const MAX_ATTEMPTS: u32 = 12;
        for attempt in 1u32..=MAX_ATTEMPTS {
            match health_client.get(&ready_url).send().await {
                Err(e) => {
                    tracing::warn!(
                        "IBKR: ibeam not reachable (attempt {}/{}) : {}",
                        attempt,
                        MAX_ATTEMPTS,
                        e
                    );
                }
                Ok(resp) if resp.status().is_success() => {
                    tracing::info!("IBKR: ibeam ready");
                    authenticated = true;
                    break;
                }
                Ok(resp) => {
                    tracing::warn!(
                        "IBKR: ibeam not ready (attempt {}/{}) — {}",
                        attempt,
                        MAX_ATTEMPTS,
                        resp.status()
                    );
                }
            }
            tokio::time::sleep(Duration::from_secs(10)).await;
        }

        if !authenticated {
            tracing::error!(
                "IBKR: failed to authenticate after {}s — check ibeam sidecar",
                MAX_ATTEMPTS * 10
            );
            return Err(RsAlgoError::from(RsAlgoErrorKind::ConnectionError).into());
        }

        // Prime the brokerage session — ibeam's browser login gives an SSO session;
        // GET /iserver/accounts is the lightweight call that activates trading/market-data access.
        let accounts_url = format!("{}/v1/api/iserver/accounts", self.gateway_url);
        match self.http.get(&accounts_url).header("Host", &self.host_header).send().await {
            Ok(resp) if resp.status().is_success() => {
                tracing::info!("IBKR: brokerage session ready");
            }
            Ok(resp) => {
                let status = resp.status();
                let body = resp.text().await.unwrap_or_default();
                tracing::warn!("IBKR: iserver/accounts returned {} — {}", status, body);
            }
            Err(e) => {
                tracing::warn!("IBKR: iserver/accounts failed: {}", e);
            }
        }

        tracing::info!("IBKR: authenticated, account={}", self.account_id);
        Ok(self)
    }

    async fn disconnect(&mut self) -> Result<()> {
        let url = format!("{}/v1/api/logout", self.gateway_url);
        let _ = self.post_empty(&url).await;
        tracing::info!("IBKR: session ended");
        Ok(())
    }

    async fn keepalive_ping(&mut self) -> Result<()> {
        let url = format!("{}/v1/api/tickle", self.gateway_url);
        let _ = self.post_empty(&url).await;
        Ok(())
    }

    // ── Market data ───────────────────────────────────────────────────────

    async fn get_instrument_data(
        &mut self,
        symbol: &str,
        period: usize,
        _start: i64,
    ) -> Result<ResponseBody<InstrumentData<VEC_DOHLC>>> {
        let conid = self.get_conid(symbol).await?;
        let (bar, hist_period) = period_to_ibkr(period);
        let exchange = symbol_to_exchange(symbol);

        let url = format!(
            "{}/v1/api/iserver/marketdata/history?conid={}&exchange={}&period={}&bar={}&outsideRth=true",
            self.gateway_url, conid, exchange, hist_period, bar
        );
        let data = self.fetch_json(&url).await?;
        let candles = parse_ibkr_candles(&data);

        Ok(ResponseBody {
            response: ResponseType::GetInstrumentData,
            payload: Some(InstrumentData {
                symbol: symbol.to_owned(),
                time_frame: TimeFrameType::from_number(period),
                data: candles,
            }),
        })
    }

    async fn get_historic_data(
        &mut self,
        symbol: &str,
        period: usize,
        start: i64,
        _end: i64,
    ) -> Result<ResponseBody<InstrumentData<VEC_DOHLC>>> {
        let conid = self.get_conid(symbol).await?;
        let (bar, hist_period) = period_to_ibkr(period);
        let exchange = symbol_to_exchange(symbol);

        let start_time = chrono::DateTime::from_timestamp(start, 0)
            .unwrap_or_else(|| chrono::DateTime::from_timestamp(0, 0).unwrap())
            .format("%Y%m%d-%H:%M:%S")
            .to_string();

        let url = format!(
            "{}/v1/api/iserver/marketdata/history?conid={}&exchange={}&period={}&bar={}&outsideRth=true&startTime={}",
            self.gateway_url, conid, exchange, hist_period, bar, start_time
        );
        let data = self.fetch_json(&url).await?;
        let candles = parse_ibkr_candles(&data);

        Ok(ResponseBody {
            response: ResponseType::GetInstrumentData,
            payload: Some(InstrumentData {
                symbol: symbol.to_owned(),
                time_frame: TimeFrameType::from_number(period),
                data: candles,
            }),
        })
    }

    async fn get_instrument_tick(&mut self, symbol: &str) -> Result<ResponseBody<InstrumentTick>> {
        let conid = self.get_conid(symbol).await?;
        let url = format!(
            "{}/v1/api/iserver/marketdata/snapshot?conids={}&fields=31,84,85,86,87,88",
            self.gateway_url, conid
        );
        // First call subscribes internally; second returns live data
        let _ = self.fetch_json(&url).await;
        tokio::time::sleep(Duration::from_millis(500)).await;
        let data = self.fetch_json(&url).await?;
        let entry = &data[0];

        let ask: f64 = entry["85"].as_str().and_then(|s| s.parse().ok()).unwrap_or(0.0);
        let bid: f64 = entry["84"].as_str().and_then(|s| s.parse().ok()).unwrap_or(0.0);

        let tick = InstrumentTick::new()
            .symbol(symbol.to_string())
            .ask(ask)
            .bid(bid)
            .high(ask)
            .low(bid)
            .spread(ask - bid)
            .pip_size(number_pips(symbol))
            .time(Local::now().timestamp())
            .build()
            .map_err(|_| RsAlgoError::from(RsAlgoErrorKind::ParseError))?;

        Ok(ResponseBody {
            response: ResponseType::GetInstrumentTick,
            payload: Some(tick),
        })
    }

    async fn get_instrument_swap(&mut self, symbol: &str) -> Result<ResponseBody<InstrumentSwap>> {
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
        let conid = self.get_conid(symbol).await?;
        let url = format!(
            "{}/v1/api/iserver/marketdata/snapshot?conids={}&fields=84,85",
            self.gateway_url, conid
        );
        let _ = self.fetch_json(&url).await;
        tokio::time::sleep(Duration::from_millis(500)).await;
        let data = self.fetch_json(&url).await?;
        let entry = &data[0];
        let ask: f64 = entry["85"].as_str().and_then(|s| s.parse().ok()).unwrap_or(0.0);
        let bid: f64 = entry["84"].as_str().and_then(|s| s.parse().ok()).unwrap_or(0.0);
        Ok((ask, bid))
    }

    async fn get_symbols(&mut self) -> Result<ResponseBody<InstrumentData<VEC_DOHLC>>> {
        Ok(ResponseBody {
            response: ResponseType::GetInstrumentData,
            payload: None,
        })
    }

    // ── Market status ─────────────────────────────────────────────────────

    async fn get_market_hours(&mut self, symbol: &str) -> Result<ResponseBody<MarketHours>> {
        let market = Market::from_str(&env::var("MARKET").unwrap_or_default());
        Ok(ResponseBody {
            response: ResponseType::GetMarketHours,
            payload: Some(MarketHours::for_market(&market, symbol)),
        })
    }

    async fn is_market_open(&mut self, _symbol: &str) -> Result<ResponseBody<bool>> {
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
        let conid = self.get_conid(&trade.symbol).await?;
        let sec_type = format!("{}@{}", conid, symbol_to_sec_type(&trade.symbol));
        let side = if trade.data.trade_type.is_long() { "BUY" } else { "SELL" };

        let order = serde_json::json!({
            "conid": conid,
            "secType": sec_type,
            "orderType": "MKT",
            "side": side,
            "quantity": trade.data.size,
            "tif": "GTC",
            "outsideRth": false
        });

        let accepted = self.place_order(&order).await.unwrap_or(false);
        tracing::info!(
            "IBKR: open_trade {} {} size={} accepted={}",
            side, trade.symbol, trade.data.size, accepted
        );

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
        let conid = self.get_conid(&trade.symbol).await?;
        let sec_type = format!("{}@{}", conid, symbol_to_sec_type(&trade.symbol));
        // Closing side is opposite of the position's trade type
        let side = if trade.data.trade_type.is_long() { "SELL" } else { "BUY" };

        let order = serde_json::json!({
            "conid": conid,
            "secType": sec_type,
            "orderType": "MKT",
            "side": side,
            "quantity": trade.data.size,
            "tif": "GTC",
            "outsideRth": false
        });

        let accepted = self.place_order(&order).await.unwrap_or(false);
        tracing::info!(
            "IBKR: close_trade {} {} size={} accepted={}",
            side, trade.symbol, trade.data.size, accepted
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
        trade: TradeData<TradeIn>,
        order: TradeData<Order>,
    ) -> Result<ResponseBody<TradeResponse<TradeIn>>> {
        let conid = self.get_conid(&trade.symbol).await?;
        let sec_type = format!("{}@{}", conid, symbol_to_sec_type(&trade.symbol));
        let side = if trade.data.trade_type.is_long() { "BUY" } else { "SELL" };
        let order_type = if order.data.order_type.is_stop() { "STP" } else { "LMT" };

        let order_body = serde_json::json!({
            "conid": conid,
            "secType": sec_type,
            "orderType": order_type,
            "side": side,
            "quantity": trade.data.size,
            "price": order.data.target_price,
            "tif": "GTC",
            "outsideRth": false
        });

        let accepted = self.place_order(&order_body).await.unwrap_or(false);

        Ok(ResponseBody {
            response: ResponseType::TradeInFulfilled,
            payload: Some(TradeResponse {
                symbol: trade.symbol.clone(),
                accepted,
                data: trade.data,
            }),
        })
    }

    async fn close_order(
        &mut self,
        trade: TradeData<TradeOut>,
        order: TradeData<Order>,
    ) -> Result<ResponseBody<TradeResponse<TradeOut>>> {
        let url = format!(
            "{}/v1/api/iserver/account/{}/order/{}",
            self.gateway_url, self.account_id, order.data.id
        );
        let resp = self
            .http
            .delete(&url)
            .send()
            .await
            .map_err(|_| RsAlgoErrorKind::RequestError)?;
        let accepted = resp.status().is_success();

        Ok(ResponseBody {
            response: ResponseType::TradeOutFulfilled,
            payload: Some(TradeResponse {
                symbol: trade.symbol.clone(),
                accepted,
                data: trade.data,
            }),
        })
    }

    // ── Positions & history ───────────────────────────────────────────────

    async fn get_active_positions(
        &mut self,
        symbol: &str,
        _strategy_name: &str,
    ) -> Result<ResponseBody<PositionResult>> {
        let conid = self.get_conid(symbol).await?;
        let url = format!(
            "{}/v1/api/portfolio/{}/positions/0",
            self.gateway_url, self.account_id
        );
        let data = self.fetch_json(&url).await?;

        let has_position = data
            .as_array()
            .map(|positions| {
                positions.iter().any(|p| {
                    p["conid"].as_i64().map(|id| id == conid).unwrap_or(false)
                        && p["position"].as_f64().map(|pos| pos.abs() > 0.0).unwrap_or(false)
                })
            })
            .unwrap_or(false);

        let result = if has_position {
            PositionResult::MarketIn(TradeResult::None, None)
        } else {
            PositionResult::None
        };

        Ok(ResponseBody {
            response: ResponseType::GetActivePositions,
            payload: Some(result),
        })
    }

    async fn get_transaction_details(
        &mut self,
        _symbol: &str,
        _strategy_name: &str,
        id: Option<usize>,
    ) -> Option<TransactionDetails> {
        let url = format!("{}/v1/api/iserver/account/trades", self.gateway_url);
        let data = self.fetch_json(&url).await.ok()?;
        let trades = data.as_array()?;
        let target_id = id.unwrap_or(0);

        trades.iter().find_map(|t| {
            let order_id = t["orderId"].as_u64()? as usize;
            if target_id != 0 && order_id != target_id {
                return None;
            }
            let avg_price: f64 = t["avgPrice"].as_str()?.parse().ok()?;
            Some(TransactionDetails {
                id: order_id,
                open_price: avg_price,
                close_price: avg_price,
                profit: 0.0,
            })
        })
    }

    async fn get_transactions_history(
        &mut self,
        _symbol: &str,
        _strategy_name: &str,
        _id: Option<usize>,
    ) -> Option<TransactionDetails> {
        None
    }

    // ── Streaming ─────────────────────────────────────────────────────────

    async fn subscribe_stream(
        &mut self,
        symbol: &str,
        _strategy_name: &str,
    ) -> Result<mpsc::UnboundedReceiver<String>> {
        let conid = self.get_conid(symbol).await?;
        let ws_url = self
            .gateway_url
            .replace("https://", "wss://")
            .replace("http://", "ws://")
            + "/v1/api/ws";
        let accept_invalid = self.accept_invalid_cert;
        let symbol_owned = symbol.to_owned();

        tracing::info!("IBKR: subscribing stream for {} (conid={})", symbol, conid);

        let (tx, rx) = mpsc::unbounded_channel();

        tokio::spawn(async move {
            let ws = match connect_ibkr_ws(&ws_url, accept_invalid).await {
                Ok(ws) => ws,
                Err(e) => {
                    tracing::error!("IBKR: WS connect failed for {}: {}", symbol_owned, e);
                    return;
                }
            };

            let (mut write, mut read) = ws.split();

            let sub_msg = format!(
                r#"smd+{}+{{"fields":["31","84","85","86","87","88"]}}"#,
                conid
            );
            if write.send(WsMessage::Text(sub_msg.clone())).await.is_err() {
                return;
            }

            let mut heartbeat = tokio::time::interval(Duration::from_secs(10));
            let mut resub = tokio::time::interval(Duration::from_secs(540)); // 9 min, before 10-min expiry
            heartbeat.tick().await;
            resub.tick().await;

            loop {
                tokio::select! {
                    msg = read.next() => {
                        match msg {
                            Some(Ok(WsMessage::Text(text))) => {
                                if let Some(s) = parse_smd_message(&text, &symbol_owned) {
                                    if tx.send(s).is_err() { return; }
                                }
                            }
                            Some(Ok(WsMessage::Binary(bytes))) => {
                                if let Ok(text) = std::str::from_utf8(&bytes) {
                                    if let Some(s) = parse_smd_message(text, &symbol_owned) {
                                        if tx.send(s).is_err() { return; }
                                    }
                                }
                            }
                            Some(Err(e)) => {
                                tracing::error!("IBKR: WS error for {}: {}", symbol_owned, e);
                                break;
                            }
                            None => break,
                            _ => {}
                        }
                    }
                    _ = heartbeat.tick() => {
                        if write.send(WsMessage::Text("ech+hb".to_string())).await.is_err() {
                            break;
                        }
                    }
                    _ = resub.tick() => {
                        if write.send(WsMessage::Text(sub_msg.clone())).await.is_err() {
                            break;
                        }
                        tracing::debug!("IBKR: resubscribed smd for {}", symbol_owned);
                    }
                }
            }
            tracing::error!("IBKR: WS stream ended for {}", symbol_owned);
        });

        Ok(rx)
    }
}

impl Ibkr {
    async fn fetch_json(&self, url: &str) -> Result<Value> {
        let resp = self.http.get(url).header("Host", &self.host_header).send().await.map_err(|e| {
            tracing::error!("IBKR: GET {} failed: {}", url, e);
            RsAlgoError::from(RsAlgoErrorKind::RequestError)
        })?;
        let status = resp.status();
        let body = resp.text().await.unwrap_or_default();
        if !status.is_success() {
            tracing::error!("IBKR: GET {} returned {}: {}", url, status, body);
            return Err(RsAlgoError::from(RsAlgoErrorKind::RequestError).into());
        }
        serde_json::from_str(&body).map_err(|e| {
            tracing::error!("IBKR: GET {} parse error: {} — body: {}", url, e, body);
            RsAlgoError::from(RsAlgoErrorKind::ParseError).into()
        })
    }

    async fn post_json(&self, url: &str, body: &Value) -> Result<Value> {
        let resp = self.http.post(url).header("Host", &self.host_header).json(body).send().await.map_err(|e| {
            tracing::error!("IBKR: POST {} failed: {}", url, e);
            RsAlgoError::from(RsAlgoErrorKind::RequestError)
        })?;
        let status = resp.status();
        let body_text = resp.text().await.unwrap_or_default();
        if !status.is_success() {
            tracing::error!("IBKR: POST {} returned {}: {}", url, status, body_text);
            return Err(RsAlgoError::from(RsAlgoErrorKind::RequestError).into());
        }
        serde_json::from_str(&body_text).map_err(|e| {
            tracing::error!("IBKR: POST {} parse error: {} — body: {}", url, e, body_text);
            RsAlgoError::from(RsAlgoErrorKind::ParseError).into()
        })
    }

    async fn post_empty(&self, url: &str) -> Result<()> {
        self.http
            .post(url)
            .header("Host", &self.host_header)
            .send()
            .await
            .map_err(|_| RsAlgoError::from(RsAlgoErrorKind::RequestError))?;
        Ok(())
    }

    async fn post_empty_json(&self, url: &str) -> Result<Value> {
        let resp = self.http.post(url).header("Host", &self.host_header).send().await.map_err(|e| {
            tracing::error!("IBKR: POST {} failed: {}", url, e);
            RsAlgoError::from(RsAlgoErrorKind::RequestError)
        })?;
        let status = resp.status();
        let body = resp.text().await.unwrap_or_default();
        if !status.is_success() {
            tracing::error!("IBKR: POST {} returned {}: {}", url, status, body);
            return Err(RsAlgoError::from(RsAlgoErrorKind::RequestError).into());
        }
        serde_json::from_str(&body).map_err(|e| {
            tracing::error!("IBKR: POST {} parse error: {} — body: {}", url, e, body);
            RsAlgoError::from(RsAlgoErrorKind::ParseError).into()
        })
    }

    async fn get_conid(&mut self, symbol: &str) -> Result<i64> {
        if let Some(&id) = self.conid_cache.get(symbol) {
            return Ok(id);
        }
        let (search_symbol, sec_type) = if is_forex(symbol) {
            (symbol[..3].to_string(), "CASH")
        } else {
            (symbol.to_string(), "STK")
        };

        let url = format!("{}/v1/api/iserver/secdef/search", self.gateway_url);
        let body = serde_json::json!({
            "symbol": search_symbol,
            "name": false,
            "secType": sec_type
        });
        let resp = self.post_json(&url, &body).await?;
        let conid = resp[0]["conid"]
            .as_i64()
            .ok_or_else(|| RsAlgoError::from(RsAlgoErrorKind::ParseError))?;

        self.conid_cache.insert(symbol.to_string(), conid);
        tracing::debug!("IBKR: resolved conid {} = {}", symbol, conid);
        Ok(conid)
    }

    /// Submit an order to IBKR, handling the optional confirmation reply flow.
    async fn place_order(&self, order: &Value) -> Result<bool> {
        let url = format!(
            "{}/v1/api/iserver/account/{}/orders",
            self.gateway_url, self.account_id
        );
        let body = serde_json::json!({ "orders": [order] });
        let resp = self.post_json(&url, &body).await?;
        let first = &resp[0];

        if first["order_id"].as_str().is_some() || first["order_id"].as_u64().is_some() {
            return Ok(true);
        }

        // IBKR requires explicit confirmation for some order types
        if let Some(reply_id) = first["id"].as_str() {
            let reply_url = format!("{}/v1/api/iserver/reply/{}", self.gateway_url, reply_id);
            let confirmed =
                self.post_json(&reply_url, &serde_json::json!({ "confirmed": true })).await?;
            let accepted = confirmed[0]["order_id"].as_str().is_some()
                || confirmed[0]["order_id"].as_u64().is_some();
            return Ok(accepted);
        }

        Ok(false)
    }
}

// ── Helpers ── (internal) ─────────────────────────────────────────────────────

fn ibeam_health_url(gateway_url: &str) -> String {
    let host = gateway_url
        .trim_start_matches("https://")
        .trim_start_matches("http://");
    let host = host.rfind(':').map(|i| &host[..i]).unwrap_or(host);
    format!("http://{}:5001", host)
}

// ── WebSocket connector ───────────────────────────────────────────────────────

async fn connect_ibkr_ws(
    url: &str,
    accept_invalid: bool,
) -> std::result::Result<
    tokio_tungstenite::WebSocketStream<tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>>,
    tokio_tungstenite::tungstenite::Error,
> {
    if accept_invalid {
        let native_connector = NativeTlsConnector::builder()
            .danger_accept_invalid_certs(true)
            .build()
            .map_err(|e| {
                tokio_tungstenite::tungstenite::Error::Io(std::io::Error::other(e.to_string()))
            })?;
        let connector = Connector::NativeTls(native_connector);
        let (ws, _) = connect_async_tls_with_config(url, None, false, Some(connector)).await?;
        Ok(ws)
    } else {
        let (ws, _) = tokio_tungstenite::connect_async(url).await?;
        Ok(ws)
    }
}

// ── Message parsing ───────────────────────────────────────────────────────────

fn parse_smd_message(msg: &str, symbol: &str) -> Option<String> {
    let obj: Value = serde_json::from_str(msg).ok()?;
    if obj["topic"].as_str()? != "smd" {
        return None;
    }

    let ask: f64 = obj["85"].as_str().and_then(|s| s.parse().ok())?;
    let bid: f64 = obj["84"].as_str().and_then(|s| s.parse().ok())?;
    let time_ms = obj["_updated"]
        .as_i64()
        .unwrap_or_else(|| Local::now().timestamp_millis());
    let time = time_ms / 1000;

    let tick = InstrumentTick::new()
        .symbol(symbol.to_string())
        .ask(ask)
        .bid(bid)
        .high(ask)
        .low(bid)
        .spread(ask - bid)
        .pip_size(number_pips(symbol))
        .time(time)
        .build()
        .ok()?;

    serde_json::to_string(&ResponseBody {
        response: ResponseType::SubscribeTickPrices,
        payload: Some(tick),
    })
    .ok()
}

// ── Helpers ───────────────────────────────────────────────────────────────────

fn is_forex(symbol: &str) -> bool {
    symbol.len() == 6 && symbol.chars().all(|c| c.is_alphabetic())
}

fn symbol_to_exchange(symbol: &str) -> &'static str {
    if is_forex(symbol) { "IDEALPRO" } else { "SMART" }
}

fn symbol_to_sec_type(symbol: &str) -> &'static str {
    if is_forex(symbol) { "CASH" } else { "STK" }
}

// Returns (bar_size, history_period) tuned for ~500 bars per request
fn period_to_ibkr(period: usize) -> (&'static str, &'static str) {
    match period {
        1 => ("1min", "1d"),
        5 => ("5min", "3d"),
        15 => ("15min", "1w"),
        30 => ("30min", "2w"),
        60 => ("1h", "1m"),
        240 => ("4h", "3m"),
        1440 => ("1d", "2y"),
        10080 => ("1w", "5y"),
        _ => ("1h", "1m"),
    }
}

fn parse_ibkr_candles(data: &Value) -> VEC_DOHLC {
    data["data"]
        .as_array()
        .map(|arr| {
            arr.iter()
                .filter_map(|c| {
                    let t_ms: i64 = c["t"].as_i64()?;
                    let dt = chrono::DateTime::from_timestamp_millis(t_ms)?
                        .with_timezone(&chrono::Local);
                    let o: f64 = c["o"].as_f64()?;
                    let h: f64 = c["h"].as_f64()?;
                    let l: f64 = c["l"].as_f64()?;
                    let cl: f64 = c["c"].as_f64()?;
                    let v: f64 = c["v"].as_f64().unwrap_or(0.0);
                    Some((dt, o, h, l, cl, v))
                })
                .collect()
        })
        .unwrap_or_default()
}
