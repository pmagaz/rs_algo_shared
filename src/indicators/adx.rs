use super::Indicator;
use crate::error::Result;
use crate::indicators::dmi::Dmi;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Adx {
    #[serde(skip_deserializing)]
    adx: Dmi,
    #[serde(skip_deserializing)]
    adx_tmp: Dmi,
    data_a: Vec<f64>,
    data_b: Vec<f64>,
    data_c: Vec<f64>,
}

impl Indicator for Adx {
    fn new() -> Result<Self> {
        Ok(Self {
            adx: Dmi::new(14),
            adx_tmp: Dmi::new(14),
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
        Ok(())
    }

    fn next_tmp(&mut self, value: f64) {
        //self.adx_tmp.next(value);
    }

    //FIXME MONEKY PATCHING
    fn next_OHLC(&mut self, OHLC: (f64, f64, f64, f64)) -> Result<()> {
        let (adx, pos, neg) = self.adx.next(OHLC).unwrap();
        self.data_a.push(pos);
        self.data_b.push(neg);
        Ok(())
    }

    fn update(&mut self, value: f64) -> Result<()> {
        // let (adx, pos, neg) = self.adx.next(value).unwrap();
        // let last_a = self.data_a.last_mut().unwrap();
        // let last_b = self.data_b.last_mut().unwrap();
        // *last_a = pos;
        // *last_b = neg;
        Ok(())
    }

    fn reset_tmp(&mut self) {
        self.adx_tmp.reset();
    }

    fn update_tmp(&mut self, value: f64) -> Result<()> {
        // let (pos, neg) = self.adx_tmp.next(value).unwrap();
        // let last_a = self.data_a.last_mut().unwrap();
        // let last_b = self.data_b.last_mut().unwrap();
        // *last_a = pos;
        // *last_b = neg;
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
