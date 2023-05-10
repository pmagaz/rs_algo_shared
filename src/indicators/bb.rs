use super::Indicator;
use crate::error::Result;

use serde::{Deserialize, Serialize};
use ta::indicators::BollingerBands;
use ta::{Next, Reset};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BollingerB {
    #[serde(skip_deserializing)]
    bb: BollingerBands,
    #[serde(skip_deserializing)]
    bb_tmp: BollingerBands,
    data_a: Vec<f64>,
    data_b: Vec<f64>,
    data_c: Vec<f64>,
}

impl Indicator for BollingerB {
    fn new() -> Result<Self> {
        Ok(Self {
            bb: BollingerBands::new(20, 2.0).unwrap(),
            bb_tmp: BollingerBands::new(20, 2.0).unwrap(),
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
        let a = self.bb.next(value);
        self.data_a.push(a.upper);
        self.data_b.push(a.lower);
        self.data_c.push(a.average);
        Ok(())
    }

    fn next_tmp(&mut self, value: f64) {
        self.bb_tmp.next(value);
    }

    fn next_OHLC(&mut self, _OHLC: (f64, f64, f64, f64)) -> Result<()> {
        Ok(())
    }

    fn update(&mut self, value: f64) -> Result<()> {
        let a = self.bb.next(value);
        let last_a = self.data_a.last_mut().unwrap();
        let last_b = self.data_b.last_mut().unwrap();
        let last_c = self.data_c.last_mut().unwrap();
        *last_a = a.upper;
        *last_b = a.lower;
        *last_c = a.average;
        Ok(())
    }

    fn update_tmp(&mut self, value: f64) -> Result<()> {
        let a = self.bb_tmp.next(value);
        let last_a = self.data_a.last_mut().unwrap();
        let last_b = self.data_b.last_mut().unwrap();
        let last_c = self.data_c.last_mut().unwrap();

        *last_a = a.upper;
        *last_b = a.lower;
        *last_c = a.average;

        Ok(())
    }

    fn reset_tmp(&mut self) {
        self.bb_tmp.reset();
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
        let c = self.data_c.last().unwrap();
        self.data_a.push(*a);
        self.data_b.push(*b);
        self.data_c.push(*c);
    }
}
