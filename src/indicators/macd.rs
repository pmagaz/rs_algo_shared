use super::Indicator;
use crate::error::Result;

use serde::{Deserialize, Serialize};
use std::env;
use ta::indicators::ExponentialMovingAverage;
use ta::{Next, Reset};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Macd {
    #[serde(skip_deserializing)]
    ema_a: ExponentialMovingAverage,
    #[serde(skip_deserializing)]
    ema_b: ExponentialMovingAverage,
    #[serde(skip_deserializing)]
    ema_c: ExponentialMovingAverage,
    #[serde(skip_deserializing)]
    ema_a_tmp: ExponentialMovingAverage,
    #[serde(skip_deserializing)]
    ema_b_tmp: ExponentialMovingAverage,
    #[serde(skip_deserializing)]
    ema_c_tmp: ExponentialMovingAverage,
    data_a: Vec<f64>,
    data_b: Vec<f64>,
    data_c: Vec<f64>,
}

impl Indicator for Macd {
    fn new() -> Result<Self> {
        let macd_a = env::var("MACD_A").unwrap().parse::<usize>().unwrap();
        let macd_b = env::var("MACD_B").unwrap().parse::<usize>().unwrap();
        let macd_c = env::var("MACD_C").unwrap().parse::<usize>().unwrap();

        Ok(Self {
            ema_a: ExponentialMovingAverage::new(macd_a).unwrap(),
            ema_b: ExponentialMovingAverage::new(macd_b).unwrap(),
            ema_c: ExponentialMovingAverage::new(macd_c).unwrap(),
            ema_a_tmp: ExponentialMovingAverage::new(macd_a).unwrap(),
            ema_b_tmp: ExponentialMovingAverage::new(macd_b).unwrap(),
            ema_c_tmp: ExponentialMovingAverage::new(macd_c).unwrap(),
            data_a: vec![],
            data_b: vec![],
            data_c: vec![],
        })
    }
    fn get_data_a(&self) -> &Vec<f64> {
        &self.data_a
    }

    fn get_current_a(&self) -> &f64 {
        self.data_a.last().unwrap()
    }

    fn get_data_b(&self) -> &Vec<f64> {
        &self.data_b
    }

    fn get_current_b(&self) -> &f64 {
        self.data_b.last().unwrap()
    }

    fn get_data_c(&self) -> &Vec<f64> {
        &self.data_c
    }

    fn get_current_c(&self) -> &f64 {
        self.data_c.last().unwrap()
    }

    fn next(&mut self, value: f64) -> Result<()> {
        let a = self.ema_a.next(value) - self.ema_b.next(value);
        let b = self.ema_c.next(a);
        self.data_a.push(a);
        self.data_b.push(b);
        Ok(())
    }

    fn next_tmp(&mut self, value: f64) {
        let a = self.ema_a_tmp.next(value) - self.ema_b_tmp.next(value);
        self.ema_c_tmp.next(a);
    }

    fn next_OHLC(&mut self, _OHLC: (f64, f64, f64, f64)) -> Result<()> {
        Ok(())
    }

    fn update(&mut self, value: f64) -> Result<()> {
        let a = self.ema_a.next(value) - self.ema_b.next(value);
        let b = self.ema_c.next(a);
        let last_a = self.data_a.last_mut().unwrap();
        let last_b = self.data_b.last_mut().unwrap();
        *last_a = a;
        *last_b = b;
        Ok(())
    }

    fn update_tmp(&mut self, value: f64) -> Result<()> {
        let a = self.ema_a_tmp.next(value) - self.ema_b_tmp.next(value);
        let b = self.ema_c_tmp.next(a);
        let last_a = self.data_a.last_mut().unwrap();
        let last_b = self.data_b.last_mut().unwrap();
        *last_a = a;
        *last_b = b;
        Ok(())
    }

    fn reset_tmp(&mut self) {
        self.ema_a_tmp.reset();
        self.ema_b_tmp.reset();
        self.ema_c_tmp.reset();
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
