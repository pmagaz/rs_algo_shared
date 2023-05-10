pub mod adx;
pub mod atr;
pub mod bb;
pub mod bbw;
pub mod ema;
pub mod macd;
pub mod rsi;
//pub mod sd;
pub mod stoch;

use crate::error::Result;
use crate::indicators::atr::Atr;
use crate::indicators::bb::BollingerB;
use crate::indicators::bbw::BollingerBW;
use crate::indicators::ema::Ema;
use crate::indicators::macd::Macd;
use crate::indicators::rsi::Rsi;
use crate::models::time_frame::TimeFrameType;
use crate::scanner::candle::Candle;

use chrono::{DateTime, Local};
use serde::{Deserialize, Serialize};
use std::env;
use std::marker::Sized;

pub trait Indicator {
    fn new() -> Result<Self>
    where
        Self: Sized;
    fn next(&mut self, value: f64) -> Result<()>;
    fn next_tmp(&mut self, value: f64);
    fn next_OHLC(&mut self, OHLC: (f64, f64, f64, f64)) -> Result<()>;
    fn update(&mut self, value: f64) -> Result<()>;
    fn update_tmp(&mut self, value: f64) -> Result<()>;
    fn reset_tmp(&mut self);
    // fn get_mut_data_a(&mut self) -> &Vec<f64>;
    // fn get_mut_data_b(&mut self) -> &Vec<f64>;
    fn get_data_a(&self) -> &Vec<f64>;
    fn get_current_a(&self) -> &f64;
    fn remove_a(&mut self, index: usize) -> f64;
    //fn remove_a(&mut self, value: usize) -> &f64;
    fn get_current_b(&self) -> &f64;
    fn get_data_b(&self) -> &Vec<f64>;
    fn remove_b(&mut self, index: usize) -> f64;
    //fn remove_b(&mut self, data: &mut Vec<f64>, index: usize) -> f64;
    //fn remove_b(&mut self, value: usize) -> &f64;
    fn get_current_c(&self) -> &f64;
    fn get_data_c(&self) -> &Vec<f64>;
    fn duplicate_last(&mut self);
    fn remove_c(&mut self, index: usize) -> f64;
    //fn remove_c(&mut self, value: usize) -> &f64;
}

//FIXME ARRAY OF TRAIT INDICATORS
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Indicators {
    pub macd: Macd,
    //pub stoch: Stoch,
    pub atr: Atr,
    // //pub adx: Adx,
    pub rsi: Rsi,
    pub bb: BollingerB,
    pub bbw: BollingerBW,
    pub ema_a: Ema,
    pub ema_b: Ema,
    pub ema_c: Ema,
}

impl Indicators {
    pub fn new() -> Result<Self> {
        let ema_a = &env::var("EMA_A").unwrap().parse::<usize>().unwrap();
        let ema_b = &env::var("EMA_B").unwrap().parse::<usize>().unwrap();
        let ema_c = &env::var("EMA_C").unwrap().parse::<usize>().unwrap();

        Ok(Self {
            macd: Macd::new().unwrap(),
            rsi: Rsi::new().unwrap(),
            //stoch: Stoch::new().unwrap(),
            atr: Atr::new().unwrap(),
            //adx: Adx::new().unwrap(),
            bb: BollingerB::new().unwrap(),
            bbw: BollingerBW::new().unwrap(),
            ema_a: Ema::new_ema(*ema_a).unwrap(),
            ema_b: Ema::new_ema(*ema_b).unwrap(),
            ema_c: Ema::new_ema(*ema_c).unwrap(),
        })
    }

    pub fn atr(&self) -> &Atr {
        &self.atr
    }

    // pub fn adx(&self) -> &Adx {
    //     &self.adx
    // }

    pub fn bb(&self) -> &BollingerB {
        &self.bb
    }

    pub fn macd(&self) -> &Macd {
        &self.macd
    }

    pub fn rsi(&self) -> &Rsi {
        &self.rsi
    }

    // pub fn stoch(&self) -> &Stoch {
    //     &self.stoch
    // }

    pub fn ema_a(&self) -> &Ema {
        &self.ema_a
    }

    pub fn ema_b(&self) -> &Ema {
        &self.ema_b
    }

    pub fn ema_c(&self) -> &Ema {
        &self.ema_c
    }

    pub fn next(
        &mut self,
        OHLC: (f64, f64, f64, f64),
        delete: bool,
        time_frame: &TimeFrameType,
    ) -> Result<()> {
        let close = OHLC.3;
        let num_bars = env::var("NUM_BARS").unwrap().parse::<usize>().unwrap();
        let max_bars = num_bars / time_frame.clone().to_number() as usize;
        //log::info!("INDICATORS  SIZE {:?}", self.ema_a().get_data_a().len());

        if env::var("INDICATORS_ATR").unwrap().parse::<bool>().unwrap() {
            self.atr.next(close).unwrap();

            if delete && self.atr.get_data_a().len() > max_bars {
                self.atr.remove_a(0);
            }
        }

        if env::var("INDICATORS_MACD")
            .unwrap()
            .parse::<bool>()
            .unwrap()
        {
            self.macd.next(close).unwrap();

            if delete && self.macd.get_data_a().len() > max_bars {
                self.macd.remove_a(0);
                self.macd.remove_b(0);
            }
        }

        // if env::var("INDICATORS_STOCH")
        //     .unwrap()
        //     .parse::<bool>()
        //     .unwrap()
        // {
        //     self.stoch.next(close).unwrap();
        // }

        if env::var("INDICATORS_RSI").unwrap().parse::<bool>().unwrap() {
            self.rsi.next(close).unwrap();

            if delete && self.rsi.get_data_a().len() > max_bars {
                self.rsi.remove_a(0);
            }
        }

        if env::var("INDICATORS_BB").unwrap().parse::<bool>().unwrap() {
            self.bb.next(close).unwrap();

            if delete && self.bb.get_data_a().len() > max_bars {
                self.bb.remove_a(0);
                self.bb.remove_b(0);
                self.bb.remove_c(0);
            }
        }

        if env::var("INDICATORS_BBW").unwrap().parse::<bool>().unwrap() {
            self.bbw.next(close).unwrap();

            if delete && self.bbw.get_data_a().len() > max_bars {
                self.bbw.remove_a(0);
                self.bbw.remove_b(0);
                self.bbw.remove_c(0);
            }
        }

        if env::var("INDICATORS_EMA_A")
            .unwrap()
            .parse::<bool>()
            .unwrap()
        {
            self.ema_a.next(close).unwrap();

            if delete && !self.ema_a.get_data_a().len() > max_bars {
                self.ema_a.remove_a(0);
            }
        }

        if env::var("INDICATORS_EMA_B")
            .unwrap()
            .parse::<bool>()
            .unwrap()
        {
            self.ema_b.next(close).unwrap();

            if delete && self.ema_b.get_data_a().len() > max_bars {
                self.ema_b.remove_a(0);
            }
        }

        if env::var("INDICATORS_EMA_C")
            .unwrap()
            .parse::<bool>()
            .unwrap()
        {
            self.ema_c.next(close).unwrap();

            if delete && self.ema_c.get_data_a().len() > max_bars {
                self.ema_c.remove_a(0);
            }
        }

        Ok(())
    }

    pub fn next_update(
        &mut self,
        OHLC: (f64, f64, f64, f64),
        time_frame: &TimeFrameType,
    ) -> Result<()> {
        let close = OHLC.3;

        // if env::var("INDICATORS_ADX").unwrap().parse::<bool>().unwrap() {
        //     self.adx.remove_a(0);
        //     self.adx.next(close).unwrap();
        // }

        if env::var("INDICATORS_ATR").unwrap().parse::<bool>().unwrap() {
            self.atr.update(close).unwrap();
        }

        if env::var("INDICATORS_MACD")
            .unwrap()
            .parse::<bool>()
            .unwrap()
        {
            self.macd.update(close).unwrap();
        }

        // if env::var("INDICATORS_STOCH")
        //     .unwrap()
        //     .parse::<bool>()
        //     .unwrap()
        // {
        //     self.stoch.update(close).unwrap();
        // }

        if env::var("INDICATORS_RSI").unwrap().parse::<bool>().unwrap() {
            self.rsi.update(close).unwrap();
        }

        if env::var("INDICATORS_BB").unwrap().parse::<bool>().unwrap() {
            self.bb.update(close).unwrap();
        }

        if env::var("INDICATORS_BBW").unwrap().parse::<bool>().unwrap() {
            self.bbw.update(close).unwrap();
        }

        if env::var("INDICATORS_EMA_A")
            .unwrap()
            .parse::<bool>()
            .unwrap()
        {
            self.ema_a.update(close).unwrap();
        }

        if env::var("INDICATORS_EMA_B")
            .unwrap()
            .parse::<bool>()
            .unwrap()
        {
            self.ema_b.update(close).unwrap();
        }

        if env::var("INDICATORS_EMA_C")
            .unwrap()
            .parse::<bool>()
            .unwrap()
        {
            self.ema_c.update(close).unwrap();
        }

        Ok(())
    }

    pub fn next_close_delete(
        &mut self,
        OHLC: (f64, f64, f64, f64),
        time_frame: &TimeFrameType,
    ) -> Result<()> {
        let close = OHLC.3;
        let num_bars = env::var("NUM_BARS").unwrap().parse::<usize>().unwrap();
        let max_bars = num_bars / time_frame.clone().to_number() as usize;

        self.next_update(OHLC, time_frame).unwrap();
        // if env::var("INDICATORS_ADX").unwrap().parse::<bool>().unwrap() {
        //     self.adx.remove_a(0);
        //     self.adx.next(close).unwrap();
        // }

        if env::var("INDICATORS_ATR").unwrap().parse::<bool>().unwrap() {
            if self.atr.get_data_a().len() > max_bars {
                self.atr.remove_a(0);
            }
        }

        if env::var("INDICATORS_MACD")
            .unwrap()
            .parse::<bool>()
            .unwrap()
        {
            if self.macd.get_data_a().len() > max_bars {
                self.macd.remove_a(0);
                self.macd.remove_b(0);
            }
        }

        // if env::var("INDICATORS_STOCH")
        //     .unwrap()
        //     .parse::<bool>()
        //     .unwrap()
        // {
        //     self.stoch.update(close).unwrap();
        // }

        if env::var("INDICATORS_RSI").unwrap().parse::<bool>().unwrap() {
            if self.rsi.get_data_a().len() > max_bars {
                self.rsi.remove_a(0);
            }
        }

        if env::var("INDICATORS_BB").unwrap().parse::<bool>().unwrap() {
            if self.bb.get_data_a().len() > max_bars {
                self.bb.remove_a(0);
                self.bb.remove_b(0);
                self.bb.remove_c(0);
            }
        }

        if env::var("INDICATORS_BBW").unwrap().parse::<bool>().unwrap() {
            if self.bbw.get_data_a().len() > max_bars {
                self.bbw.remove_a(0);
                self.bbw.remove_b(0);
                self.bbw.remove_c(0);
            }
        }

        if env::var("INDICATORS_EMA_A")
            .unwrap()
            .parse::<bool>()
            .unwrap()
        {
            if self.ema_a.get_data_a().len() > max_bars {
                self.ema_a.remove_a(0);
            }
        }

        if env::var("INDICATORS_EMA_B")
            .unwrap()
            .parse::<bool>()
            .unwrap()
        {
            if self.ema_b.get_data_a().len() > max_bars {
                self.ema_b.remove_a(0);
            }
        }

        if env::var("INDICATORS_EMA_C")
            .unwrap()
            .parse::<bool>()
            .unwrap()
        {
            if self.ema_c.get_data_a().len() > max_bars {
                self.ema_c.remove_a(0);
            }
        }

        Ok(())
    }

    pub fn update(&mut self, OHLC: (f64, f64, f64, f64)) -> Result<()> {
        let close = OHLC.3;

        // if env::var("INDICATORS_ADX").unwrap().parse::<bool>().unwrap() {
        //     self.adx.remove_a(0);
        //     self.adx.next(close).unwrap();
        // }

        if env::var("INDICATORS_ATR").unwrap().parse::<bool>().unwrap() {
            self.atr.update_tmp(close).unwrap();
        }

        if env::var("INDICATORS_MACD")
            .unwrap()
            .parse::<bool>()
            .unwrap()
        {
            self.macd.update_tmp(close).unwrap();
        }

        // if env::var("INDICATORS_STOCH")
        //     .unwrap()
        //     .parse::<bool>()
        //     .unwrap()
        // {
        //     self.stoch.update(close).unwrap();
        // }

        if env::var("INDICATORS_RSI").unwrap().parse::<bool>().unwrap() {
            self.rsi.update_tmp(close).unwrap();
        }

        if env::var("INDICATORS_BB").unwrap().parse::<bool>().unwrap() {
            self.bb.update_tmp(close).unwrap();
        }

        if env::var("INDICATORS_BBW").unwrap().parse::<bool>().unwrap() {
            self.bbw.update_tmp(close).unwrap();
        }

        if env::var("INDICATORS_EMA_A")
            .unwrap()
            .parse::<bool>()
            .unwrap()
        {
            self.ema_a.update_tmp(close).unwrap();
        }

        if env::var("INDICATORS_EMA_B")
            .unwrap()
            .parse::<bool>()
            .unwrap()
        {
            self.ema_b.update_tmp(close).unwrap();
        }

        if env::var("INDICATORS_EMA_C")
            .unwrap()
            .parse::<bool>()
            .unwrap()
        {
            self.ema_c.update_tmp(close).unwrap();
        }

        Ok(())
    }

    pub fn create_tmp_indicators(
        &mut self,
        candle: &Candle,
        prev_data: &Vec<Candle>,
    ) -> Result<()> {
        let close = candle.close();

        self.atr.reset_tmp();
        self.ema_a.reset_tmp();
        self.ema_b.reset_tmp();
        self.ema_c.reset_tmp();
        self.bb.reset_tmp();
        self.bbw.reset_tmp();
        self.rsi.reset_tmp();
        self.macd.reset_tmp();
        //self.stoch.reset_tmp();

        //WARMING
        for prev_candle in prev_data
            .iter()
            .rev()
            .take(50)
            .rev()
            .filter(|x| x.is_closed == true)
        {
            if env::var("INDICATORS_ATR").unwrap().parse::<bool>().unwrap() {
                self.atr.next_tmp(prev_candle.close());
            }
            if env::var("INDICATORS_BB").unwrap().parse::<bool>().unwrap() {
                self.bb.next_tmp(prev_candle.close());
            }
            if env::var("INDICATORS_BBW").unwrap().parse::<bool>().unwrap() {
                self.bbw.next_tmp(prev_candle.close());
            }
            if env::var("INDICATORS_EMA_A")
                .unwrap()
                .parse::<bool>()
                .unwrap()
            {
                self.ema_a.next_tmp(prev_candle.close());
            }
            if env::var("INDICATORS_EMA_B")
                .unwrap()
                .parse::<bool>()
                .unwrap()
            {
                self.ema_b.next_tmp(prev_candle.close());
            }
            if env::var("INDICATORS_EMA_C")
                .unwrap()
                .parse::<bool>()
                .unwrap()
            {
                self.ema_c.next_tmp(prev_candle.close());
            }
            if env::var("INDICATORS_MACD")
                .unwrap()
                .parse::<bool>()
                .unwrap()
            {
                self.macd.next_tmp(prev_candle.close());
            }
            if env::var("INDICATORS_RSI").unwrap().parse::<bool>().unwrap() {
                self.rsi.next_tmp(prev_candle.close());
            }
        }

        //UPDATING LAST VALUE

        if env::var("INDICATORS_ATR").unwrap().parse::<bool>().unwrap() {
            self.atr.update_tmp(close).unwrap();
        }
        if env::var("INDICATORS_BB").unwrap().parse::<bool>().unwrap() {
            self.bb.update_tmp(close).unwrap();
        }
        if env::var("INDICATORS_BBW").unwrap().parse::<bool>().unwrap() {
            self.bbw.update_tmp(close).unwrap();
        }
        if env::var("INDICATORS_EMA_A")
            .unwrap()
            .parse::<bool>()
            .unwrap()
        {
            self.ema_a.update_tmp(close).unwrap();
        }
        if env::var("INDICATORS_EMA_B")
            .unwrap()
            .parse::<bool>()
            .unwrap()
        {
            self.ema_b.update_tmp(close).unwrap();
        }
        if env::var("INDICATORS_EMA_C")
            .unwrap()
            .parse::<bool>()
            .unwrap()
        {
            self.ema_c.update_tmp(close).unwrap();
        }
        if env::var("INDICATORS_MACD")
            .unwrap()
            .parse::<bool>()
            .unwrap()
        {
            self.macd.update_tmp(close).unwrap();
        }
        if env::var("INDICATORS_RSI").unwrap().parse::<bool>().unwrap() {
            self.rsi.update_tmp(close).unwrap();
        }

        Ok(())
    }

    pub fn duplicate_last(&mut self) -> Result<()> {
        // if env::var("INDICATORS_ADX").unwrap().parse::<bool>().unwrap() {
        //     self.adx.remove_a(0);
        //     self.adx.next(close).unwrap();
        // }
        if env::var("INDICATORS_ATR").unwrap().parse::<bool>().unwrap() {
            self.atr.duplicate_last();
        }

        if env::var("INDICATORS_MACD")
            .unwrap()
            .parse::<bool>()
            .unwrap()
        {
            self.macd.duplicate_last();
        }

        // if env::var("INDICATORS_STOCH")
        //     .unwrap()
        //     .parse::<bool>()
        //     .unwrap()
        // {
        //     self.stoch.update(close).unwrap();
        // }

        if env::var("INDICATORS_RSI").unwrap().parse::<bool>().unwrap() {
            self.rsi.duplicate_last();
        }

        if env::var("INDICATORS_BB").unwrap().parse::<bool>().unwrap() {
            self.bb.duplicate_last();
        }

        if env::var("INDICATORS_BBW").unwrap().parse::<bool>().unwrap() {
            self.bbw.duplicate_last();
        }

        if env::var("INDICATORS_EMA_A")
            .unwrap()
            .parse::<bool>()
            .unwrap()
        {
            self.ema_a.duplicate_last();
        }

        if env::var("INDICATORS_EMA_B")
            .unwrap()
            .parse::<bool>()
            .unwrap()
        {
            self.ema_b.duplicate_last();
        }

        if env::var("INDICATORS_EMA_C")
            .unwrap()
            .parse::<bool>()
            .unwrap()
        {
            self.ema_c.duplicate_last();
        }

        Ok(())
    }
}
