# rs_algo_shared

Shared library for the rs-algo-bot algorithmic trading system. Provides the broker-agnostic layer, data models, WebSocket utilities, and helpers used by the server and bot crates.

## Architecture

```
rs_algo_shared/src/
├── broker/
│   ├── broker_trait.rs   BrokerStream trait — all brokers implement this
│   ├── darwinex.rs       Darwinex broker (single WS + REST)
│   ├── xtb_stream.rs     XTB broker (legacy, dual WS)
│   ├── xtb_models.rs     XTB-specific protocol structs
│   ├── models.rs         Broker-agnostic types (DOHLC, TransactionCommand, …)
│   └── mod.rs            AnyBroker enum dispatch + create_broker() factory
├── ws/
│   ├── ws_stream_client.rs  Async WebSocket (tokio-tungstenite, with auth support)
│   ├── ws_client.rs         Sync WebSocket (tungstenite)
│   └── message.rs           Shared message types (ResponseBody, CommandType, …)
├── models/                  Domain models: trade, tick, swap, market, bot, …
└── helpers/                 calc, date, uuid, http, …
```

## Broker-Agnostic Design

The server never imports or calls broker-specific code. All protocol details are encapsulated inside each broker struct. The server interacts only via the `BrokerStream` trait.

```rust
// Server selects broker at runtime via BROKER env var
let mut broker = create_broker().await;    // returns AnyBroker (Darwinex by default)
broker.login(&username, &password).await?;

// Streaming: broker manages its own WS loop internally
let mut stream_rx = broker.subscribe_stream(&symbol, &strategy).await?;
while let Some(msg) = stream_rx.recv().await {
    // msg is a pre-serialized ResponseBody JSON string — no broker types exposed
}
```

## Brokers

### Darwinex (default)

| Concern | Implementation |
|---|---|
| Auth | OAuth2 password grant → Bearer token (`POST /token`) |
| Streaming | Single WebSocket (`wss://api.darwinex.com/quotewebsocket/1.0.0`) with Bearer token in header |
| Subscribe | `{"op": "subscribe", "productNames": ["EURUSD"]}` |
| Data / Trading | REST (`https://api.darwinex.com`) |
| Connections | **One** (WS for quotes + HTTP for everything else) |

### XTB (legacy)

| Concern | Implementation |
|---|---|
| Auth | WebSocket command (`login`) |
| Streaming | Dual WebSocket (command socket + stream socket) |
| Connections | **Two** |

## Broker Selection

Set the `BROKER` environment variable:

```env
BROKER=darwinex   # default
BROKER=xtb        # legacy
```

## Environment Variables

| Variable | Description |
|---|---|
| `BROKER` | `darwinex` (default) or `xtb` |
| `BROKER_USERNAME` | Broker account username |
| `BROKER_PASSWORD` | Broker account password |
| `DARWINEX_WS_URL` | `wss://api.darwinex.com/quotewebsocket/1.0.0` |
| `DARWINEX_API_BASE_URL` | `https://api.darwinex.com` |
| `DARWINEX_TOKEN_URL` | `https://api.darwinex.com/token` |
| `DARWINEX_ACCOUNT_ID` | Investor account ID for REST calls |

## Edition & Dependencies

- Rust 2024 edition
- `tokio` 1.44, `tokio-tungstenite` 0.24 (rustls), `reqwest` 0.12 (rustls)
- No OpenSSL dependency
