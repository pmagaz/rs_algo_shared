use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Pricing {
    symbol: String,
    ask: f64,
    bid: f64,
    spread: f64,
    pip_size: f64,
    percentage: f64,
}

impl Pricing {
    pub fn new(
        symbol: String,
        ask: f64,
        bid: f64,
        spread: f64,
        pip_size: f64,
        percentage: f64,
    ) -> Self {
        Pricing {
            symbol,
            ask,
            bid,
            spread,
            pip_size,
            percentage,
        }
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
    pub fn spread(&self) -> f64 {
        self.spread
    }
    pub fn pip_size(&self) -> f64 {
        self.pip_size
    }
    pub fn percentage(&self) -> f64 {
        self.percentage
    }

    pub fn calculate_spread(&mut self, price: f64) -> &Self {
        if self.percentage > 0. {
            let spread = (self.percentage * price) / 100.;
            self.spread = spread;
        }
        self
    }
}

impl Default for Pricing {
    fn default() -> Self {
        Pricing {
            symbol: "".to_string(),
            ask: 0.,
            bid: 0.,
            spread: 0.,
            pip_size: 0.,
            percentage: 0.,
        }
    }
}
