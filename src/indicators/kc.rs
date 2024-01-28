use super::Indicator;
use crate::error::Result;

use serde::{Deserialize, Serialize};
use ta::indicators::{KeltnerChannel, KeltnerChannelOutput};
use ta::{Next, Reset};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeltnerC {
    #[serde(skip_deserializing)]
    kc: KeltnerChannel,
    data_a: Vec<f64>,
    data_b: Vec<f64>,
    data_c: Vec<f64>,
}

impl Indicator for KeltnerC {
    fn new() -> Result<Self> {
        Ok(Self {
            kc: KeltnerChannel::new(20, 2.0).unwrap(),
            data_a: vec![],
            data_b: vec![],
            data_c: vec![],
        })
    }

    fn get_data_a(&self) -> &Vec<f64> {
        &self.data_a
    }

    fn get_current_a(&self) -> &f64 {
        &self.data_a.last().unwrap()
    }

    fn get_data_b(&self) -> &Vec<f64> {
        &self.data_b
    }

    fn get_current_b(&self) -> &f64 {
        &self.data_b.last().unwrap()
    }

    fn get_data_c(&self) -> &Vec<f64> {
        &self.data_c
    }

    fn get_current_c(&self) -> &f64 {
        &self.data_c.last().unwrap()
    }

    fn next(&mut self, value: f64) -> Result<()> {
        let a = self.kc.next(value);
        self.data_a.push(a.upper);
        self.data_b.push(a.lower);
        Ok(())
    }
    fn next_ohlc(&mut self, OHLC: (f64, f64, f64, f64)) -> Result<()> {
        Ok(())
    }
}
