use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
struct ForexInstruments {
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
        "symbol": "AUDCAD",
        "name": "AUDCAD",
        "sector": "Currency"
    },
    {
        "symbol": "AUDCHF",
        "name": "AUDCHF",
        "sector": "Currency"
    },
    {
        "symbol": "AUDJPY",
        "name": "AUDJPY",
        "sector": "Currency"
    },
    {
        "symbol": "AUDNZD",
        "name": "AUDNZD",
        "sector": "Currency"
    },
    {
        "symbol": "AUDUSD",
        "name": "AUDUSD",
        "sector": "Currency"
    },
    {
        "symbol": "CADCHF",
        "name": "CADCHF",
        "sector": "Currency"
    },
    {
        "symbol": "CADJPY",
        "name": "CADJPY",
        "sector": "Currency"
    },
    {
        "symbol": "EURUSD",
        "name": "EURUSD",
        "sector": "Currency"
    },
    {
        "symbol": "GBPUSD",
        "name": "GBPUSD",
        "sector": "Currency"
    },
    {
        "symbol": "USDCHF",
        "name": "USDCHF",
        "sector": "Currency"
    },
    {
        "symbol": "USDJPY",
        "name": "USDJPY",
        "sector": "Currency"
    },
    {
        "symbol": "NZDUSD",
        "name": "NZDUSD",
        "sector": "Currency"
    },
    {
        "symbol": "USDCAD",
        "name": "USDCAD",
        "sector": "Currency"
    },
    {
        "symbol": "CHFPLN",
        "name": "CHFPLN",
        "sector": "Currency"
    },
    {
        "symbol": "CHFJPY",
        "name": "CHFJPY",
        "sector": "Currency"
    }
]}"#;

    let json: ForexInstruments =
        serde_json::from_str(&content).expect("JSON was not well-formatted");
    let mut arr = vec![];
    for symbol in json.symbols {
        arr.push(symbol.symbol);
    }

    arr
}
