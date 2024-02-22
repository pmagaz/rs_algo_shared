use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstrumentSwap {
    symbol: String,
    enabled: bool,
    swap_long: f64,
    swap_short: f64,
    swap_weekend: f64,
}

impl InstrumentSwap {
    pub fn new() -> InstrumentSwapBuilder {
        InstrumentSwapBuilder::new()
    }
}

#[derive(Debug, Clone)]
pub struct InstrumentSwapBuilder {
    symbol: Option<String>,
    enabled: Option<bool>,
    swap_long: Option<f64>,
    swap_short: Option<f64>,
    swap_weekend: Option<f64>,
}

impl InstrumentSwapBuilder {
    pub fn new() -> Self {
        Self {
            symbol: None,
            enabled: None,
            swap_long: None,
            swap_short: None,
            swap_weekend: None,
        }
    }

    pub fn symbol(mut self, symbol: String) -> Self {
        self.symbol = Some(symbol);
        self
    }

    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = Some(enabled);
        self
    }

    pub fn swap_long(mut self, swap_long: f64) -> Self {
        self.swap_long = Some(swap_long);
        self
    }

    pub fn swap_short(mut self, swap_short: f64) -> Self {
        self.swap_short = Some(swap_short);
        self
    }

    pub fn swap_weekend(mut self, swap_weekend: f64) -> Self {
        self.swap_weekend = Some(swap_weekend);
        self
    }

    pub fn build(self) -> Result<InstrumentSwap, &'static str> {
        Ok(InstrumentSwap {
            symbol: self.symbol.ok_or("Symbol is required")?,
            enabled: self.enabled.ok_or("Enabled is required")?,
            swap_long: self.swap_long.ok_or("Swap long is required")?,
            swap_short: self.swap_short.ok_or("Swap short is required")?,
            swap_weekend: self.swap_weekend.ok_or("Swap weekend is required")?,
        })
    }
}
