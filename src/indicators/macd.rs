use super::Indicator;
use crate::error::Result;

use serde::{Deserialize, Serialize};
use std::env;
use ta::indicators::ExponentialMovingAverage;
use ta::Next;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Macd {
    ema_a: ExponentialMovingAverage,
    ema_b: ExponentialMovingAverage,
    ema_c: ExponentialMovingAverage,
    data_a: Vec<f64>,
    data_b: Vec<f64>,
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
        let a = self.ema_a.next(value) - self.ema_b.next(value);
        let b = self.ema_c.next(a);
        self.data_a.push(a);
        self.data_b.push(b);
        Ok(())
    }

    fn next_OHLC(&mut self, _OHLC: (f64, f64, f64, f64)) -> Result<()> {
        Ok(())
    }

    fn update(&mut self, value: f64) -> Result<()> {
        let a = self.ema_a.next(value) - self.ema_b.next(value);
        let b = self.ema_c.next(a);
        let last_a_index = self.data_a.len() - 1;
        let last_b_index = self.data_b.len() - 1;
        let last_a = self.data_a.get_mut(last_a_index).unwrap();
        let last_b = self.data_b.get_mut(last_b_index).unwrap();
        *last_a = a;
        *last_b = b;
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
}
