pub mod adx;
pub mod atr;
pub mod bb;
pub mod bbw;
pub mod ema;
pub mod macd;
pub mod rsi;
//pub mod sd;
//pub mod stoch;

use crate::error::Result;
use crate::indicators::atr::Atr;
use crate::indicators::bb::BollingerB;
use crate::indicators::bbw::BollingerBW;
use crate::indicators::ema::Ema;
use crate::indicators::macd::Macd;
use crate::indicators::rsi::Rsi;
use crate::models::time_frame::TimeFrameType;
use crate::scanner::candle::Candle;

use serde::{Deserialize, Serialize};
use std::env;
use std::marker::Sized;

pub trait Indicator {
    fn new() -> Result<Self>
    where
        Self: Sized;
    fn next(&mut self, value: f64) -> Result<()>;
    fn next_tmp(&mut self, value: f64);
    fn next_ohlc(&mut self, ohlc: (f64, f64, f64, f64)) -> Result<()>;
    fn next_update_last(&mut self, value: f64) -> Result<()>;
    fn next_update_last_tmp(&mut self, value: f64) -> Result<()>;
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
    fn init_indicator(&mut self);
    fn remove_c(&mut self, index: usize) -> f64;
    //fn remove_c(&mut self, value: usize) -> &f64;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct IndicatorsEnabled {
    atr: bool,
    rsi: bool,
    macd: bool,
    bb: bool,
    bbw: bool,
    ema_a: bool,
    ema_b: bool,
    ema_c: bool,
}

//FIXME DOING IT AT INIT USING OPTION
fn is_enabled() -> Result<IndicatorsEnabled> {
    Ok(IndicatorsEnabled {
        atr: env::var("INDICATORS_ATR").unwrap().parse::<bool>().unwrap(),
        rsi: env::var("INDICATORS_RSI").unwrap().parse::<bool>().unwrap(),
        macd: env::var("INDICATORS_MACD")
            .unwrap()
            .parse::<bool>()
            .unwrap(),
        bb: env::var("INDICATORS_BB").unwrap().parse::<bool>().unwrap(),
        bbw: env::var("INDICATORS_BBW").unwrap().parse::<bool>().unwrap(),
        ema_a: env::var("INDICATORS_EMA_A")
            .unwrap()
            .parse::<bool>()
            .unwrap(),
        ema_b: env::var("INDICATORS_EMA_B")
            .unwrap()
            .parse::<bool>()
            .unwrap(),
        ema_c: env::var("INDICATORS_EMA_C")
            .unwrap()
            .parse::<bool>()
            .unwrap(),
    })
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
        ohlc: (f64, f64, f64, f64),
        remove_first: bool,
        time_frame: &TimeFrameType,
    ) -> Result<()> {
        let close = ohlc.3;
        let num_bars = env::var("NUM_BARS").unwrap().parse::<usize>().unwrap();
        let max_bars = num_bars / time_frame.clone().to_number() as usize;
        let indicators_enabled = is_enabled()?;

        if indicators_enabled.atr {
            self.atr.next(close).unwrap();
            if remove_first && self.atr.get_data_a().len() > max_bars {
                self.atr.remove_a(0);
            }
        }

        if indicators_enabled.macd {
            self.macd.next(close).unwrap();
            if remove_first && self.macd.get_data_a().len() > max_bars {
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

        if indicators_enabled.rsi {
            self.rsi.next(close).unwrap();
            if remove_first && self.rsi.get_data_a().len() > max_bars {
                self.rsi.remove_a(0);
            }
        }

        if indicators_enabled.bb {
            self.bb.next(close).unwrap();
            if remove_first && self.bb.get_data_a().len() > max_bars {
                self.bb.remove_a(0);
                self.bb.remove_b(0);
                self.bb.remove_c(0);
            }
        }

        if indicators_enabled.bbw {
            self.bbw.next(close).unwrap();
            if remove_first && self.bbw.get_data_a().len() > max_bars {
                self.bbw.remove_a(0);
                self.bbw.remove_b(0);
                self.bbw.remove_c(0);
            }
        }

        if indicators_enabled.ema_a {
            self.ema_a.next(close).unwrap();

            if remove_first && !self.ema_a.get_data_a().len() > max_bars {
                self.ema_a.remove_a(0);
            }
        }

        if indicators_enabled.ema_b {
            self.ema_b.next(close).unwrap();
            if remove_first && self.ema_b.get_data_a().len() > max_bars {
                self.ema_b.remove_a(0);
            }
        }

        if indicators_enabled.ema_c {
            self.ema_c.next(close).unwrap();
            if remove_first && self.ema_c.get_data_a().len() > max_bars {
                self.ema_c.remove_a(0);
            }
        }

        Ok(())
    }

    pub fn next_close_indicators(
        &mut self,
        ohlc: (f64, f64, f64, f64),
        _time_frame: &TimeFrameType,
    ) -> Result<()> {
        let close = ohlc.3;
        let indicators_enabled = is_enabled()?;

        if indicators_enabled.atr {
            self.atr.next_update_last(close).unwrap();
        }

        if indicators_enabled.macd {
            self.macd.next_update_last(close).unwrap();
        }

        // if env::var("INDICATORS_STOCH")
        //     .unwrap()
        //     .parse::<bool>()
        //     .unwrap()
        // {
        //     self.stoch.next_update_last(close).unwrap();
        // }

        if indicators_enabled.rsi {
            self.rsi.next_update_last(close).unwrap();
        }

        if indicators_enabled.bb {
            self.bb.next_update_last(close).unwrap();
        }

        if indicators_enabled.bbw {
            self.bbw.next_update_last(close).unwrap();
        }

        if indicators_enabled.ema_a {
            self.ema_a.next_update_last(close).unwrap();
        }

        if indicators_enabled.ema_b {
            self.ema_b.next_update_last(close).unwrap();
        }

        if indicators_enabled.ema_c {
            self.ema_c.next_update_last(close).unwrap();
        }

        Ok(())
    }

    pub fn next_tmp_indicators(
        &mut self,
        current_candle: &Candle,
        data: &Vec<Candle>,
    ) -> Result<()> {
        let len = data.len();
        let num_warming_items = 40; //13 3x
        let close = current_candle.close();
        let num_items = num_warming_items.min(len);
        let indicators_enabled = is_enabled()?;

        //TAKE LATEST 50 ELEMENTS OF THE ARRAY FOR WARMING AND EXCLUDING THE LASTEST ONE
        for prev_candle in &data[len - num_items..len - 1] {
            if indicators_enabled.atr {
                self.atr.next_tmp(prev_candle.close());
            }
            if indicators_enabled.bb {
                self.bb.next_tmp(prev_candle.close());
            }
            if indicators_enabled.bbw {
                self.bbw.next_tmp(prev_candle.close());
            }
            if indicators_enabled.ema_a {
                self.ema_a.next_tmp(prev_candle.close());
            }
            if indicators_enabled.ema_b {
                self.ema_b.next_tmp(prev_candle.close());
            }
            if indicators_enabled.ema_c {
                self.ema_c.next_tmp(prev_candle.close());
            }
            if indicators_enabled.macd {
                self.macd.next_tmp(prev_candle.close());
            }
            if indicators_enabled.rsi {
                self.rsi.next_tmp(prev_candle.close());
            }
        }

        // UPDATING LAST VALUE & RESET
        if indicators_enabled.atr {
            self.atr.next_update_last_tmp(close).unwrap();
        }
        if indicators_enabled.bb {
            self.bb.next_update_last_tmp(close).unwrap();
        }
        if indicators_enabled.bbw {
            self.bbw.next_update_last_tmp(close).unwrap();
        }
        if indicators_enabled.ema_a {
            self.ema_a.next_update_last_tmp(close).unwrap();
        }
        if indicators_enabled.ema_b {
            self.ema_b.next_update_last_tmp(close).unwrap();
        }
        if indicators_enabled.ema_c {
            self.ema_c.next_update_last_tmp(close).unwrap();
        }
        if indicators_enabled.macd {
            self.macd.next_update_last_tmp(close).unwrap();
        }
        if indicators_enabled.rsi {
            self.rsi.next_update_last_tmp(close).unwrap();
        }

        Ok(())
    }

    pub fn init_indicators(
        &mut self,
        time_frame: &TimeFrameType,
        remove_first: bool,
    ) -> Result<()> {
        let num_bars = env::var("NUM_BARS").unwrap().parse::<usize>().unwrap();
        let max_bars = num_bars / time_frame.clone().to_number() as usize;
        let indicators_enabled = is_enabled()?;
        //let atx = env::var("INDICATORS_ATX").unwrap().parse::<bool>().unwrap();
        //let stoch= env::var("INDICATORS_STOCH").unwrap().parse::<bool>().unwrap();

        if indicators_enabled.rsi && self.rsi.get_data_a().len() > max_bars && remove_first {
            self.rsi.remove_a(0);
        }

        if indicators_enabled.bb && self.bb.get_data_a().len() > max_bars && remove_first {
            self.bb.remove_a(0);
            self.bb.remove_b(0);
            self.bb.remove_c(0);
        }

        if indicators_enabled.bbw && self.bbw.get_data_a().len() > max_bars && remove_first {
            self.bbw.remove_a(0);
            self.bbw.remove_b(0);
            self.bbw.remove_c(0);
        }

        if indicators_enabled.ema_a && self.ema_a.get_data_a().len() > max_bars && remove_first {
            self.ema_a.remove_a(0);
        }

        if indicators_enabled.ema_b && self.ema_b.get_data_a().len() > max_bars && remove_first {
            self.ema_b.remove_a(0);
        }

        if indicators_enabled.ema_c && self.ema_c.get_data_a().len() > max_bars && remove_first {
            self.ema_c.remove_a(0);
        }

        if indicators_enabled.atr && self.atr.get_data_a().len() > max_bars && remove_first {
            self.atr.remove_a(0);
        }

        if indicators_enabled.macd && self.macd.get_data_a().len() > max_bars && remove_first {
            self.macd.remove_a(0);
            self.macd.remove_b(0);
            self.macd.remove_c(0);
        }

        //INIT

        if indicators_enabled.rsi {
            self.rsi.init_indicator();
        }

        if indicators_enabled.bb {
            self.bb.init_indicator();
        }

        if indicators_enabled.bbw {
            self.bbw.init_indicator();
        }

        if indicators_enabled.ema_a {
            self.ema_a.init_indicator();
        }

        if indicators_enabled.ema_b {
            self.ema_b.init_indicator();
        }

        if indicators_enabled.ema_c {
            self.ema_c.init_indicator();
        }

        if indicators_enabled.atr {
            self.atr.init_indicator();
        }

        if indicators_enabled.macd {
            self.macd.init_indicator();
        }

        Ok(())
    }
}
