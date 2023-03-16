use super::Indicator;
use crate::error::Result;

use serde::{Deserialize, Serialize};
use ta::indicators::ExponentialMovingAverage;
use ta::Next;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Ema {
    ema: ExponentialMovingAverage,
    data_a: Vec<f64>,
    data_b: Vec<f64>,
}

impl Ema {
    pub fn new_ema(index: usize) -> Result<Self> {
        Ok(Self {
            ema: ExponentialMovingAverage::new(index).unwrap(),
            data_a: vec![],
            data_b: vec![],
        })
    }
}

impl Indicator for Ema {
    fn new() -> Result<Self> {
        Ok(Self {
            ema: ExponentialMovingAverage::new(0).unwrap(),
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
        let a = self.ema.next(value);
        log::info!("EMA SIZE {:?}", self.data_a.len());
        self.data_a.push(a);
        log::info!("EMA SIZE {:?}", self.data_a.len());
        Ok(())
    }

    fn next_OHLC(&mut self, _OHLC: (f64, f64, f64, f64)) -> Result<()> {
        Ok(())
    }

    fn update(&mut self, value: f64) -> Result<()> {
        let a = self.ema.next(value);
        let last = self.data_a.last_mut().unwrap();
        *last = a;
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

    fn duplicate_last(&mut self) {
        let a = self.data_a.last().unwrap();
        //log::info!("7777777777 {:?}", self.data_a.len());
        self.data_a.push(*a);
    }
}
