use super::Indicator;

use crate::error::Result;

use ta::indicators::SlowStochastic;

use serde::{Deserialize, Serialize};
use ta::indicators::ExponentialMovingAverage;
use ta::{Next, Reset};

#[derive(Debug, Clone, Serialize, Deserialize)]

pub struct Stoch {
    #[serde(skip_deserializing)]
    stoch: SlowStochastic,
    #[serde(skip_deserializing)]
    ema: ExponentialMovingAverage,
    #[serde(skip_deserializing)]
    stoch_tmp: SlowStochastic,
    #[serde(skip_deserializing)]
    ema_tmp: ExponentialMovingAverage,
    data_a: Vec<f64>,
    data_b: Vec<f64>,
    data_c: Vec<f64>,
}

impl Indicator for Stoch {
    fn new() -> Result<Self> {
        Ok(Self {
            stoch: SlowStochastic::new(10, 3).unwrap(),
            ema: ExponentialMovingAverage::new(3).unwrap(),
            stoch_tmp: SlowStochastic::new(10, 3).unwrap(),
            ema_tmp: ExponentialMovingAverage::new(3).unwrap(),
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
        let a = self.stoch.next(value);
        let b = self.ema.next(a);
        self.data_a.push(a);
        self.data_b.push(b);
        Ok(())
    }

    fn next_tmp(&mut self, value: f64) {
        self.stoch_tmp.next(value);
    }

    fn next_update_last(&mut self, value: f64) -> Result<()> {
        let a = self.stoch.next(value);
        let b = self.ema.next(a);
        let last_a = self.data_a.last_mut().unwrap();
        let last_b = self.data_b.last_mut().unwrap();
        *last_a = a;
        *last_b = b;
        Ok(())
    }

    fn next_update_last_tmp(&mut self, value: f64) -> Result<()> {
        let a = self.stoch.next(value);
        let b = self.ema.next(a);
        let last_a = self.data_a.last_mut().unwrap();
        let last_b = self.data_b.last_mut().unwrap();
        *last_a = a;
        *last_b = b;
        Ok(())
    }

    fn reset_tmp(&mut self) {
        self.stoch_tmp.reset();
        self.ema_tmp.reset();
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
        self.data_c.remove(index)
    }

    fn duplicate_last(&mut self) {
        if let Some(&a) = self.data_a.last() {
            self.data_a.push(a);
        }
        if let Some(&b) = self.data_b.last() {
            self.data_b.push(b);
        }
    }
}
