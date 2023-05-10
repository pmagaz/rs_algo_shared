use super::Indicator;
use crate::error::Result;

use serde::{Deserialize, Serialize};
use ta::indicators::StandardDeviation as SD;
use ta::{Next, Reset};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StandardD {
    #[serde(skip_deserializing)]
    sd: SD,
    data_a: Vec<f64>,
    data_b: Vec<f64>,
    data_c: Vec<f64>,
}

impl Indicator for StandardD {
    fn new() -> Result<Self> {
        Ok(Self {
            sd: SD::new(5).unwrap(),
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
        let a = self.sd.next(value);
        self.data_a.push(a);
        Ok(())
    }

    fn update(&mut self, value: f64) -> Result<()> {
        let a = self.sd.next(value);
        let last_index = self.data_a.len() - 1;
        let last = self.data_a.last_mut().unwrap();
        *last = a;
        Ok(())
    }

    fn update_tmp(&mut self, value: f64) -> Result<()> {
        let a = self.sd.next(value);
        let last_index = self.data_a.len() - 1;
        let last = self.data_a.last_mut().unwrap();
        *last = a;
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
        self.data_c.remove(index)
    }

    fn duplicate_last(&mut self) {
        let a = self.data_a.last().unwrap();
        let b = self.data_b.last().unwrap();
        self.data_a.push(*a);
        self.data_b.push(*b);
    }
}
