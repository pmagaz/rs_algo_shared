use super::Indicator;
use crate::error::Result;

use serde::{Deserialize, Serialize};
use ta::indicators::AverageDirectionalIndex;
use ta::{Next, Reset};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Adx {
    #[serde(skip_deserializing)]
    adx: AverageDirectionalIndex,
    #[serde(skip_deserializing)]
    adx_tmp: AverageDirectionalIndex,
    data_a: Vec<f64>,
    data_b: Vec<f64>,
    data_c: Vec<f64>,
}

impl Indicator for Adx {
    fn new() -> Result<Self> {
        Ok(Self {
            adx: AverageDirectionalIndex::new(14).unwrap(),
            adx_tmp: AverageDirectionalIndex::new(14).unwrap(),
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
        let max = self.data_a.len() - 1;
        &self.data_b[max]
    }

    fn get_data_c(&self) -> &Vec<f64> {
        &self.data_c
    }

    fn get_current_c(&self) -> &f64 {
        &self.data_c.last().unwrap()
    }

    fn next(&mut self, value: f64) -> Result<()> {
        let a = self.adx.next(value);
        self.data_a.push(a);
        Ok(())
    }

    fn next_tmp(&mut self, value: f64) {
        self.adx_tmp.next(value);
    }

    //FIXME MONEKY PATCHING
    fn next_OHLC(&mut self, _OHLC: (f64, f64, f64, f64)) -> Result<()> {
        Ok(())
    }

    fn update(&mut self, value: f64) -> Result<()> {
        let a = self.adx.next(value);
        let last = self.data_a.last_mut().unwrap();
        *last = a;
        Ok(())
    }

    fn reset_tmp(&mut self) {
        self.adx_tmp.reset();
    }

    fn update_tmp(&mut self, value: f64) -> Result<()> {
        let a = self.adx_tmp.next(value);
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
        self.data_c.remove(index)
    }

    fn duplicate_last(&mut self) {
        if let Some(&a) = self.data_a.last() {
            self.data_a.insert(0, 0.);
        }
    }
}
