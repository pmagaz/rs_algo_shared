use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstrumentTick {
    symbol: String,
    ask: f64,
    bid: f64,
    high: f64,
    low: f64,
    spread: f64,
    pip_size: f64,
    time: i64,
}

impl InstrumentTick {
    pub fn new() -> InstrumentTickBuilder {
        InstrumentTickBuilder::new()
    }
    pub fn symbol(&self) -> String {
        self.symbol.to_string()
    }
    pub fn ask(&self) -> f64 {
        self.ask
    }
    pub fn bid(&self) -> f64 {
        self.bid
    }
    pub fn high(&self) -> f64 {
        self.high
    }
    pub fn low(&self) -> f64 {
        self.low
    }
    pub fn spread(&self) -> f64 {
        self.spread
    }
    pub fn pip_size(&self) -> f64 {
        self.pip_size
    }
    pub fn time(&self) -> i64 {
        self.time
    }

    // pub fn calculate_spread(&mut self, price: f64) -> &Self {
    //     if self.time > 0. {
    //         let spread = (self.time * price) / 100.;
    //         self.spread = spread;
    //     }
    //     self
    // }
}

impl Default for InstrumentTick {
    fn default() -> Self {
        InstrumentTick {
            symbol: "".to_string(),
            ask: 0.,
            bid: 0.,
            high: 0.,
            low: 0.,
            spread: 0.,
            pip_size: 0.,
            time: 0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstrumentTickBuilder {
    symbol: Option<String>,
    ask: Option<f64>,
    bid: Option<f64>,
    high: Option<f64>,
    low: Option<f64>,
    spread: Option<f64>,
    pip_size: Option<f64>,
    time: Option<i64>,
}

impl InstrumentTickBuilder {
    pub fn new() -> Self {
        Self {
            symbol: None,
            ask: None,
            bid: None,
            high: None,
            low: None,
            spread: None,
            pip_size: None,
            time: None,
        }
    }

    pub fn symbol(mut self, symbol: String) -> Self {
        self.symbol = Some(symbol);
        self
    }

    pub fn ask(mut self, ask: f64) -> Self {
        self.ask = Some(ask);
        self
    }

    pub fn bid(mut self, bid: f64) -> Self {
        self.bid = Some(bid);
        self
    }

    pub fn high(mut self, high: f64) -> Self {
        self.high = Some(high);
        self
    }

    pub fn low(mut self, low: f64) -> Self {
        self.low = Some(low);
        self
    }

    pub fn spread(mut self, spread: f64) -> Self {
        self.spread = Some(spread);
        self
    }

    pub fn pip_size(mut self, pip_size: f64) -> Self {
        self.pip_size = Some(pip_size);
        self
    }

    pub fn time(mut self, time: i64) -> Self {
        self.time = Some(time);
        self
    }

    pub fn build(self) -> Result<InstrumentTick, &'static str> {
        let symbol = self.symbol.ok_or("Symbol is required")?;
        let ask = self.ask.ok_or("Ask price is required")?;
        let bid = self.bid.ok_or("Bid price is required")?;
        let high = self.high.ok_or("High price is required")?;
        let low = self.low.ok_or("Low price is required")?;
        let spread = self.spread.ok_or("Spread is required")?;
        let pip_size = self.pip_size.unwrap_or_default();
        let time = self.time.ok_or("time is required")?;

        Ok(InstrumentTick {
            symbol,
            ask,
            bid,
            high,
            low,
            spread,
            pip_size,
            time,
        })
    }
}

impl Default for InstrumentTickBuilder {
    fn default() -> Self {
        InstrumentTickBuilder::new()
    }
}
