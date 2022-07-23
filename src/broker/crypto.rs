use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
struct CryptoInstruments {
    symbols: Vec<Instrument>,
}

#[derive(Debug, Serialize, Deserialize)]
struct Instrument {
    symbol: String,
    name: String,
    sector: String,
}

pub fn get_symbols() -> Vec<String> {
    let content = r#"{ "symbols": [
    {
        "symbol": "BITCOIN",
        "name": "BITCOIN",
        "sector": "Crypto"
    },
    {
        "symbol": "ETHEREUM",
        "name": "ETHEREUM",
        "sector": "Crypto"
    },
    {
        "symbol": "LITECOIN",
        "name": "LITECOIN",
        "sector": "Crypto"
    },
    {
        "symbol": "RIPPLE",
        "name": "RIPPLE",
        "sector": "Crypto"
    },
    {
        "symbol": "DOGECOIN",
        "name": "DOGECOIN",
        "sector": "Crypto"
    },
    {
        "symbol": "BINANCECOIN",
        "name": "BINANCECOIN",
        "sector": "Crypto"
    },
    {
        "symbol": "BITCOINCASH",
        "name": "BITCOINCASH",
        "sector": "Crypto"
    },
    {
        "symbol": "CARDANO",
        "name": "CARDANO",
        "sector": "Crypto"
    },
    {
        "symbol": "CHAINLINK",
        "name": "CHAINLINK",
        "sector": "Crypto"
    },
    {
        "symbol": "CHILIZ",
        "name": "CHILIZ",
        "sector": "Crypto"
    },
    {
        "symbol": "EOS",
        "name": "EOS",
        "sector": "Crypto"
    },
    {
        "symbol": "POLKADOT",
        "name": "POLKADOT",
        "sector": "Crypto"
    },
    {
        "symbol": "POLYGON",
        "name": "POLYGON",
        "sector": "Crypto"
    },
    {
        "symbol": "STELLAR",
        "name": "STELLAR",
        "sector": "Crypto"
    },
    {
        "symbol": "SOLANA",
        "name": "SOLANA",
        "sector": "Crypto"
    }
]}"#;

    let json: CryptoInstruments =
        serde_json::from_str(&content).expect("JSON was not well-formatted");
    let mut arr = vec![];
    for symbol in json.symbols {
        arr.push(symbol.symbol);
    }

    arr
}
