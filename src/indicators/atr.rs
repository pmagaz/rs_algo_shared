use super::Indicator;
use crate::error::Result;

use serde::{Deserialize, Serialize};
use ta::indicators::AverageTrueRange;
use ta::test_helper::Bar;
use ta::{Next, Reset};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Atr {
    #[serde(skip_deserializing)]
    atr: AverageTrueRange,
    #[serde(skip_deserializing)]
    atr_tmp: AverageTrueRange,
    data_a: Vec<f64>,
    data_b: Vec<f64>,
    data_c: Vec<f64>,
}

impl Indicator for Atr {
    fn new() -> Result<Self> {
        Ok(Self {
            atr: AverageTrueRange::new(14).unwrap(),
            atr_tmp: AverageTrueRange::new(14).unwrap(),
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
        let a = self.atr.next(value);
        self.data_a.push(a);
        Ok(())
    }

    fn next_tmp(&mut self, value: f64) {
        self.atr_tmp.next(value);
    }

    //FIXME MONEKY PATCHING
    fn next_OHLC(&mut self, OHLC: (f64, f64, f64, f64)) -> Result<()> {
        panic!("1111");
        let bar = Bar::new().high(OHLC.1).low(OHLC.2).close(OHLC.3);
        let a = self.atr.next(&bar);
        self.data_a.push(a);
        Ok(())
    }

    fn update(&mut self, value: f64) -> Result<()> {
        let a = self.atr.next(value);
        let last = self.data_a.last_mut().unwrap();
        *last = a;
        Ok(())
    }

    fn update_tmp(&mut self, value: f64) -> Result<()> {
        let a = self.atr_tmp.next(value);
        let last = self.data_a.last_mut().unwrap();
        *last = a;
        Ok(())
    }

    fn reset_tmp(&mut self) {
        self.atr_tmp.reset();
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
        self.data_a.push(*a);
    }
}
