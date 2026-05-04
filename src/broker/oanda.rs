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
    InstrumentData, ResponseBody, ResponseType, TradeData, TradeResponse,
};

use chrono::DateTime;
use futures_util::StreamExt;
use serde_json::Value;
use std::env;
use std::time::Duration;
use tokio::sync::mpsc;

pub struct Oanda {
    http: reqwest::Client,
    access_token: String,
    account_id: String,
    api_base: String,
    stream_base: String,
}

#[async_trait::async_trait]
impl BrokerStream for Oanda {
    async fn new() -> Self {
        let env = env::var("OANDA_ENVIRONMENT").unwrap_or_else(|_| "practice".to_string());
        let (api_base, stream_base) = if env == "live" {
            (
                "https://api-fxtrade.oanda.com".to_string(),
                "https://stream-fxtrade.oanda.com".to_string(),
            )
        } else {
            (
                "https://api-fxpractice.oanda.com".to_string(),
                "https://stream-fxpractice.oanda.com".to_string(),
            )
        };

        let access_token = env::var("OANDA_API_TOKEN").unwrap_or_default();
        let account_id = env::var("OANDA_ACCOUNT_ID").unwrap_or_default();

        let http = reqwest::Client::builder()
            .connect_timeout(Duration::from_secs(15))
            .timeout(Duration::from_secs(30))
            .build()
            .expect("Failed to build reqwest client");

        Self {
            http,
            access_token,
            account_id,
            api_base,
            stream_base,
        }
    }

    async fn login(&mut self, _username: &str, _password: &str) -> Result<&mut Self> {
        // Oanda uses a Personal Access Token — no OAuth flow.
        // Token is read from OANDA_API_TOKEN env var in new().
        // Validate connectivity by fetching account summary.
        log::info!("Oanda: validating API token against {}", self.api_base);

        let url = format!("{}/v3/accounts/{}/summary", self.api_base, self.account_id);
        let resp = self
            .http
            .get(&url)
            .bearer_auth(&self.access_token)
            .send()
            .await
            .map_err(|e| {
                log::error!("Oanda: connection failed: {}", e);
                RsAlgoErrorKind::ConnectionError
            })?;

        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            log::error!("Oanda: login check failed {}: {}", status, body);
            return Err(RsAlgoError::from(RsAlgoErrorKind::ConnectionError).into());
        }

        log::info!("Oanda: token valid, account {} ready", self.account_id);
        Ok(self)
    }

    async fn disconnect(&mut self) -> Result<()> {
        log::info!("Oanda: disconnected (HTTP streaming, no explicit close needed)");
        Ok(())
    }

    // Oanda streaming is HTTP chunked — no WebSocket ping required.
    async fn keepalive_ping(&mut self) -> Result<()> {
        Ok(())
    }

    // ── Market data ───────────────────────────────────────────────────────

    async fn get_instrument_data(
        &mut self,
        symbol: &str,
        period: usize,
        _start: i64,
    ) -> Result<ResponseBody<InstrumentData<VEC_DOHLC>>> {
        let granularity = period_to_granularity(period);
        let count = env::var("NUM_BARS")
            .unwrap_or_else(|_| "500".to_string())
            .parse::<usize>()
            .unwrap_or(500)
            .min(5000);

        let url = format!(
            "{}/v3/instruments/{}/candles?granularity={}&count={}&price=M",
            self.api_base,
            to_oanda_symbol(symbol),
            granularity,
            count
        );

        let data = self.fetch_json(&url).await?;
        let candles = parse_candles(&data, symbol);

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
        end: i64,
    ) -> Result<ResponseBody<InstrumentData<VEC_DOHLC>>> {
        let granularity = period_to_granularity(period);
        let url = format!(
            "{}/v3/instruments/{}/candles?granularity={}&from={}&to={}&price=M",
            self.api_base,
            to_oanda_symbol(symbol),
            granularity,
            start,
            end
        );

        let data = self.fetch_json(&url).await?;
        let candles = parse_candles(&data, symbol);

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
        let url = format!(
            "{}/v3/accounts/{}/pricing?instruments={}",
            self.api_base,
            self.account_id,
            to_oanda_symbol(symbol)
        );

        let data = self.fetch_json(&url).await?;
        let prices = &data["prices"];
        let price = &prices[0];

        let ask: f64 = price["asks"][0]["price"]
            .as_str()
            .and_then(|s| s.parse().ok())
            .unwrap_or(0.0);
        let bid: f64 = price["bids"][0]["price"]
            .as_str()
            .and_then(|s| s.parse().ok())
            .unwrap_or(0.0);

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
        let url = format!(
            "{}/v3/accounts/{}/pricing?instruments={}",
            self.api_base,
            self.account_id,
            to_oanda_symbol(symbol)
        );
        let data = self.fetch_json(&url).await?;
        let price = &data["prices"][0];
        let ask: f64 = price["asks"][0]["price"]
            .as_str()
            .and_then(|s| s.parse().ok())
            .unwrap_or(0.0);
        let bid: f64 = price["bids"][0]["price"]
            .as_str()
            .and_then(|s| s.parse().ok())
            .unwrap_or(0.0);
        Ok((ask, bid))
    }

    async fn get_symbols(&mut self) -> Result<ResponseBody<InstrumentData<VEC_DOHLC>>> {
        Ok(ResponseBody {
            response: ResponseType::GetInstrumentData,
            payload: None,
        })
    }

    // ── Market status ─────────────────────────────────────────────────────

    async fn get_market_hours(
        &mut self,
        _symbol: &str,
    ) -> Result<ResponseBody<MarketHours>> {
        Ok(ResponseBody {
            response: ResponseType::GetMarketHours,
            payload: Some(MarketHours::default()),
        })
    }

    async fn is_market_open(&mut self, symbol: &str) -> Result<ResponseBody<bool>> {
        let url = format!(
            "{}/v3/accounts/{}/pricing?instruments={}",
            self.api_base,
            self.account_id,
            to_oanda_symbol(symbol)
        );
        let data = self.fetch_json(&url).await?;
        let tradeable = data["prices"][0]["tradeable"].as_bool().unwrap_or(false);
        Ok(ResponseBody {
            response: ResponseType::IsMarketOpen,
            payload: Some(tradeable),
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
        let url = format!("{}/v3/accounts/{}/orders", self.api_base, self.account_id);
        let oanda_symbol = to_oanda_symbol(&trade.symbol);

        // Positive units = long, negative = short
        let units = if trade.data.trade_type.is_long() {
            format!("{}", trade.data.size)
        } else {
            format!("-{}", trade.data.size)
        };

        let body = serde_json::json!({
            "order": {
                "type": "MARKET",
                "instrument": oanda_symbol,
                "units": units,
                "timeInForce": "FOK",
                "positionFill": "DEFAULT"
            }
        });

        let resp = self
            .http
            .post(&url)
            .bearer_auth(&self.access_token)
            .json(&body)
            .send()
            .await
            .map_err(|_| RsAlgoErrorKind::RequestError)?;

        let accepted = resp.status().is_success();
        log::info!(
            "Oanda: open_trade {} units={} accepted={}",
            oanda_symbol,
            units,
            accepted
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
        let url = format!(
            "{}/v3/accounts/{}/trades/{}/close",
            self.api_base, self.account_id, trade.data.id
        );

        let resp = self
            .http
            .put(&url)
            .bearer_auth(&self.access_token)
            .json(&serde_json::json!({ "units": "ALL" }))
            .send()
            .await
            .map_err(|_| RsAlgoErrorKind::RequestError)?;

        let accepted = resp.status().is_success();
        log::info!(
            "Oanda: close_trade {} id={} accepted={}",
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
        trade: TradeData<TradeIn>,
        order: TradeData<Order>,
    ) -> Result<ResponseBody<TradeResponse<TradeIn>>> {
        let url = format!("{}/v3/accounts/{}/orders", self.api_base, self.account_id);
        let oanda_symbol = to_oanda_symbol(&trade.symbol);

        let units = if trade.data.trade_type.is_long() {
            format!("{}", trade.data.size)
        } else {
            format!("-{}", trade.data.size)
        };

        let order_type = if order.data.order_type.is_stop() {
            "STOP"
        } else {
            "LIMIT"
        };

        let body = serde_json::json!({
            "order": {
                "type": order_type,
                "instrument": oanda_symbol,
                "units": units,
                "price": format!("{:.5}", order.data.target_price),
                "timeInForce": "GTC",
                "positionFill": "DEFAULT"
            }
        });

        let resp = self
            .http
            .post(&url)
            .bearer_auth(&self.access_token)
            .json(&body)
            .send()
            .await
            .map_err(|_| RsAlgoErrorKind::RequestError)?;

        let accepted = resp.status().is_success();
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
            "{}/v3/accounts/{}/orders/{}/cancel",
            self.api_base, self.account_id, order.data.id
        );

        let resp = self
            .http
            .put(&url)
            .bearer_auth(&self.access_token)
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
        let url = format!(
            "{}/v3/accounts/{}/openPositions",
            self.api_base, self.account_id
        );

        let data = self.fetch_json(&url).await?;
        let oanda_symbol = to_oanda_symbol(symbol);

        let has_position = data["positions"]
            .as_array()
            .map(|positions| {
                positions
                    .iter()
                    .any(|p| p["instrument"].as_str().unwrap_or("") == oanda_symbol)
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
        _id: Option<usize>,
    ) -> Option<TransactionDetails> {
        None
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
        strategy_name: &str,
    ) -> Result<mpsc::UnboundedReceiver<String>> {
        let url = format!(
            "{}/v3/accounts/{}/pricing/stream?instruments={}",
            self.stream_base,
            self.account_id,
            to_oanda_symbol(symbol)
        );

        log::info!("Oanda: subscribing to price stream for {}", symbol);

        // Build a dedicated client without the 30s timeout — streaming is long-lived
        let stream_client = reqwest::Client::builder()
            .connect_timeout(Duration::from_secs(15))
            .build()
            .expect("Failed to build stream client");

        let response = stream_client
            .get(&url)
            .bearer_auth(&self.access_token)
            .send()
            .await
            .map_err(|e| {
                log::error!("Oanda: failed to open price stream: {}", e);
                RsAlgoErrorKind::ConnectionError
            })?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            log::error!("Oanda: stream rejected {}: {}", status, body);
            return Err(RsAlgoError::from(RsAlgoErrorKind::ConnectionError).into());
        }

        let (tx, rx) = mpsc::unbounded_channel();
        let symbol_owned = symbol.to_owned();
        let strategy_name_owned = strategy_name.to_owned();

        tokio::spawn(async move {
            let mut byte_stream = response.bytes_stream();
            let mut buffer = String::new();

            while let Some(chunk) = byte_stream.next().await {
                match chunk {
                    Ok(bytes) => {
                        buffer.push_str(&String::from_utf8_lossy(&bytes));
                        // Drain all complete newline-terminated JSON lines from the buffer
                        while let Some(pos) = buffer.find('\n') {
                            let line = buffer[..pos].trim().to_string();
                            buffer.drain(..=pos);
                            if line.is_empty() {
                                continue;
                            }
                            if let Some(msg) = Oanda::parse_stream_data(
                                &line,
                                &symbol_owned,
                                &strategy_name_owned,
                            ) {
                                if tx.send(msg).is_err() {
                                    return;
                                }
                            }
                        }
                    }
                    Err(e) => {
                        log::error!("Oanda: stream chunk error for {}: {}", symbol_owned, e);
                        break;
                    }
                }
            }
            log::error!("Oanda: price stream ended for {}", symbol_owned);
        });

        Ok(rx)
    }
}

impl Oanda {
    // Shared REST fetch helper
    async fn fetch_json(&self, url: &str) -> Result<Value> {
        let resp = self
            .http
            .get(url)
            .bearer_auth(&self.access_token)
            .send()
            .await
            .map_err(|e| {
                log::error!("Oanda: request failed {}: {}", url, e);
                RsAlgoError::from(RsAlgoErrorKind::RequestError)
            })?;

        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            log::error!("Oanda: {} returned {}: {}", url, status, body);
            return Err(RsAlgoError::from(RsAlgoErrorKind::RequestError).into());
        }

        resp.json::<Value>().await.map_err(|e| {
            log::error!("Oanda: failed to parse JSON: {}", e);
            RsAlgoError::from(RsAlgoErrorKind::ParseError).into()
        })
    }

    // Parse a single line from the Oanda price stream into a ResponseBody JSON string
    fn parse_stream_data(line: &str, symbol: &str, _strategy_name: &str) -> Option<String> {
        let obj: Value = serde_json::from_str(line).ok()?;
        let msg_type = obj["type"].as_str()?;

        if msg_type == "HEARTBEAT" {
            return None;
        }
        if msg_type != "PRICE" {
            return None;
        }

        let instrument = obj["instrument"].as_str()?;
        if instrument != to_oanda_symbol(symbol) {
            return None;
        }

        let ask: f64 = obj["asks"][0]["price"].as_str()?.parse().ok()?;
        let bid: f64 = obj["bids"][0]["price"].as_str()?.parse().ok()?;
        let time_str = obj["time"].as_str()?;

        let ts = DateTime::parse_from_rfc3339(time_str)
            .map(|t| t.timestamp())
            .unwrap_or_else(|_| Local::now().timestamp());

        let pip_size = number_pips(symbol);

        let tick = InstrumentTick::new()
            .symbol(symbol.to_string())
            .ask(ask)
            .bid(bid)
            .high(ask)
            .low(bid)
            .spread(ask - bid)
            .pip_size(pip_size)
            .time(ts)
            .build()
            .ok()?;

        serde_json::to_string(&ResponseBody {
            response: ResponseType::SubscribeTickPrices,
            payload: Some(tick),
        })
        .ok()
    }
}

// ── Helpers ───────────────────────────────────────────────────────────────────

// Convert EURUSD → EUR_USD (Oanda instrument format)
fn to_oanda_symbol(symbol: &str) -> String {
    if symbol.contains('_') || symbol.len() < 6 {
        return symbol.to_uppercase();
    }
    format!("{}_{}", &symbol[..3], &symbol[3..]).to_uppercase()
}

// Map internal period (minutes) to Oanda CandlestickGranularity
fn period_to_granularity(period: usize) -> &'static str {
    match period {
        1 => "M1",
        5 => "M5",
        15 => "M15",
        30 => "M30",
        60 => "H1",
        120 => "H2",
        240 => "H4",
        480 => "H8",
        720 => "H12",
        1440 => "D",
        10080 => "W",
        43200 => "M",
        _ => "H1",
    }
}

// Parse Oanda candle array into VEC_DOHLC
fn parse_candles(data: &Value, symbol: &str) -> VEC_DOHLC {
    data["candles"]
        .as_array()
        .map(|arr| {
            arr.iter()
                .filter_map(|c| {
                    let mid = &c["mid"];
                    let time_str = c["time"].as_str()?;
                    let time = DateTime::parse_from_rfc3339(time_str)
                        .ok()?
                        .with_timezone(&chrono::Local);
                    let o: f64 = mid["o"].as_str()?.parse().ok()?;
                    let h: f64 = mid["h"].as_str()?.parse().ok()?;
                    let l: f64 = mid["l"].as_str()?.parse().ok()?;
                    let cl: f64 = mid["c"].as_str()?.parse().ok()?;
                    let v: f64 = c["volume"].as_f64().unwrap_or(0.0);
                    Some((time, o, h, l, cl, v))
                })
                .collect()
        })
        .unwrap_or_default()
}
