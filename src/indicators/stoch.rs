use super::Indicator;

use crate::error::Result;

use ta::indicators::SlowStochastic;

use serde::{Deserialize, Serialize};
use ta::indicators::ExponentialMovingAverage;
use ta::Next;

#[derive(Debug, Clone, Serialize, Deserialize)]

pub struct Stoch {
    stoch: SlowStochastic,
    ema: ExponentialMovingAverage,
    data_a: Vec<f64>,
    data_b: Vec<f64>,
}

impl Indicator for Stoch {
    fn new() -> Result<Self> {
        Ok(Self {
            stoch: SlowStochastic::new(10, 3).unwrap(),
            ema: ExponentialMovingAverage::new(3).unwrap(),
            data_a: vec![],
            data_b: vec![],
        })
    }

    fn get_data_a(&self) -> &Vec<f64> {
        &self.data_a
    }

    fn get_current_a(&self) -> &f64 {
        let max = self.data_a.len() - 1;
        &self.data_a[max]
    }

    fn get_data_b(&self) -> &Vec<f64> {
        &self.data_b
    }

    fn get_current_b(&self) -> &f64 {
        let max = self.data_b.len() - 1;
        &self.data_b[max]
    }

    fn get_data_c(&self) -> &Vec<f64> {
        &self.data_a
    }

    fn get_current_c(&self) -> &f64 {
        let max = self.data_a.len() - 1;
        &self.data_a[max]
    }

    fn next(&mut self, value: f64) -> Result<()> {
        let a = self.stoch.next(value);
        let b = self.ema.next(a);
        self.data_a.push(a);
        self.data_b.push(b);
        Ok(())
    }

    fn update(&mut self, value: f64) -> Result<()> {
        let a = self.stoch.next(value);
        let b = self.ema.next(a);
        let last_a_index = self.data_a.len() - 1;
        let last_b_index = self.data_b.len() - 1;
        let last_a = self.data_a.get_mut(last_a_index).unwrap();
        let last_b = self.data_b.get_mut(last_b_index).unwrap();
        *last_a = a;
        *last_b = b;
        Ok(())
    }

    fn next_OHLC(&mut self, _OHLC: (f64, f64, f64, f64)) -> Result<()> {
        Ok(())
    }

    fn remove_a(&mut self, index: usize) -> f64 {
        self.data_a.remove(index)
    }

    fn remove_b(&mut self, index: usize) -> f64 {
        self.data_b.remove(index)
    }

    fn remove_c(&mut self, index: usize) -> f64 {
        self.data_b.remove(index)
    }

    fn init(&mut self) {
        let a = self.data_a.first().unwrap();
        let b = self.data_b.first().unwrap();
        self.data_a.insert(0, *a);
        self.data_b.insert(0, *b);
    }
}
