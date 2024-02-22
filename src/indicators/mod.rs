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
pub struct Indicators {
    pub macd: Option<Macd>,
    //pub stoch: Stoch,
    pub atr: Option<Atr>,
    // //pub adx: Adx,
    pub rsi: Option<Rsi>,
    pub bb: Option<BollingerB>,
    pub bbw: Option<BollingerBW>,
    pub ema_a: Option<Ema>,
    pub ema_b: Option<Ema>,
    pub ema_c: Option<Ema>,
}

impl Indicators {
    pub fn new() -> Result<Self> {
        let macd = env::var("INDICATORS_MACD")
            .ok()
            .and_then(|v| v.parse::<bool>().ok())
            .and_then(|enabled| if enabled { Macd::new().ok() } else { None });

        let rsi = env::var("INDICATORS_RSI")
            .ok()
            .and_then(|v| v.parse::<bool>().ok())
            .and_then(|enabled| if enabled { Rsi::new().ok() } else { None });

        // let stoch = env::var("INDICATORS_STOCH")
        //     .ok()
        //     .and_then(|v| v.parse::<bool>().ok())
        //     .and_then(|enabled| if enabled { Stoch::new().ok() } else { None });

        let atr = env::var("INDICATORS_ATR")
            .ok()
            .and_then(|v| v.parse::<bool>().ok())
            .and_then(|enabled| if enabled { Atr::new().ok() } else { None });

        // let adx = env::var("INDICATORS_ADX")
        //     .ok()
        //     .and_then(|v| v.parse::<bool>().ok())
        //     .and_then(|enabled| if enabled { Adx::new().ok() } else { None });

        let bb = env::var("INDICATORS_BB")
            .ok()
            .and_then(|v| v.parse::<bool>().ok())
            .and_then(|enabled| {
                if enabled {
                    BollingerB::new().ok()
                } else {
                    None
                }
            });

        let bbw = env::var("INDICATORS_BBW")
            .ok()
            .and_then(|v| v.parse::<bool>().ok())
            .and_then(|enabled| {
                if enabled {
                    BollingerBW::new().ok()
                } else {
                    None
                }
            });

        let ema_a = env::var("INDICATORS_EMA_A")
            .ok()
            .and_then(|v| v.parse::<bool>().ok())
            .and_then(|enabled| {
                if enabled {
                    let ema_a = env::var("EMA_A").unwrap().parse::<usize>().unwrap();
                    Some(Ema::new_ema(ema_a).unwrap())
                } else {
                    None
                }
            });

        let ema_b = env::var("INDICATORS_EMA_B")
            .ok()
            .and_then(|v| v.parse::<bool>().ok())
            .and_then(|enabled| {
                if enabled {
                    let ema_a = env::var("EMA_B").unwrap().parse::<usize>().unwrap();
                    Some(Ema::new_ema(ema_a).unwrap())
                } else {
                    None
                }
            });

        let ema_c = env::var("INDICATORS_EMA_C")
            .ok()
            .and_then(|v| v.parse::<bool>().ok())
            .and_then(|enabled| {
                if enabled {
                    let ema_a = env::var("EMA_C").unwrap().parse::<usize>().unwrap();
                    Some(Ema::new_ema(ema_a).unwrap())
                } else {
                    None
                }
            });

        Ok(Self {
            macd,
            rsi,
            //stoch,
            atr,
            //adx,
            bb,
            bbw,
            ema_a,
            ema_b,
            ema_c,
        })
    }

    pub fn atr(&self) -> Option<&Atr> {
        self.atr.as_ref()
    }

    // pub fn adx(&self) -> &Adx {
    //     &self.adx
    // }

    pub fn bb(&self) -> Option<&BollingerB> {
        self.bb.as_ref()
    }

    pub fn macd(&self) -> Option<&Macd> {
        self.macd.as_ref()
    }

    pub fn rsi(&self) -> Option<&Rsi> {
        self.rsi.as_ref()
    }

    // pub fn stoch(&self) -> &Stoch {
    //     &self.stoch
    // }

    pub fn ema_a(&self) -> Option<&Ema> {
        self.ema_a.as_ref()
    }

    pub fn ema_b(&self) -> Option<&Ema> {
        self.ema_b.as_ref()
    }

    pub fn ema_c(&self) -> Option<&Ema> {
        self.ema_c.as_ref()
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

        // STOCH
        // if let Some(stoch) = &mut self.stoch {
        //     stoch.next(close).unwrap();
        //     if remove_first && stoch.get_data_a().len() > max_bars {
        //         stoch.remove_a(0);
        //     }
        // }

        // ATR
        if let Some(atr) = &mut self.atr {
            atr.next(close).unwrap();
            if remove_first && atr.get_data_a().len() > max_bars {
                atr.remove_a(0);
            }
        }

        // MACD
        if let Some(macd) = &mut self.macd {
            macd.next(close).unwrap();
            if remove_first && macd.get_data_a().len() > max_bars {
                macd.remove_a(0);
                macd.remove_b(0);
            }
        }

        // RSI
        if let Some(rsi) = &mut self.rsi {
            rsi.next(close).unwrap();
            if remove_first && rsi.get_data_a().len() > max_bars {
                rsi.remove_a(0);
            }
        }

        // Bollinger Bands
        if let Some(bb) = &mut self.bb {
            bb.next(close).unwrap();
            if remove_first && bb.get_data_a().len() > max_bars {
                bb.remove_a(0);
                bb.remove_b(0);
                bb.remove_c(0);
            }
        }

        // Bollinger Bandwidth
        if let Some(bbw) = &mut self.bbw {
            bbw.next(close).unwrap();
            if remove_first && bbw.get_data_a().len() > max_bars {
                bbw.remove_a(0);
                bbw.remove_b(0);
                bbw.remove_c(0);
            }
        }

        // EMA A
        if let Some(ema_a) = &mut self.ema_a {
            ema_a.next(close).unwrap();
            if remove_first && ema_a.get_data_a().len() > max_bars {
                ema_a.remove_a(0);
            }
        }

        // EMA B
        if let Some(ema_b) = &mut self.ema_b {
            ema_b.next(close).unwrap();
            if remove_first && ema_b.get_data_a().len() > max_bars {
                ema_b.remove_a(0);
            }
        }

        // EMA C
        if let Some(ema_c) = &mut self.ema_c {
            ema_c.next(close).unwrap();
            if remove_first && ema_c.get_data_a().len() > max_bars {
                ema_c.remove_a(0);
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

        if let Some(atr) = &mut self.atr {
            atr.next_update_last(close).unwrap();
        }

        if let Some(macd) = &mut self.macd {
            macd.next_update_last(close).unwrap();
        }

        // if let Some(stoch) = &mut self.stoch {
        //     stoch.next_update_last(close).unwrap();
        // }

        if let Some(rsi) = &mut self.rsi {
            rsi.next_update_last(close).unwrap();
        }

        if let Some(bb) = &mut self.bb {
            bb.next_update_last(close).unwrap();
        }

        if let Some(bbw) = &mut self.bbw {
            bbw.next_update_last(close).unwrap();
        }

        if let Some(ema_a) = &mut self.ema_a {
            ema_a.next_update_last(close).unwrap();
        }

        if let Some(ema_b) = &mut self.ema_b {
            ema_b.next_update_last(close).unwrap();
        }

        if let Some(ema_c) = &mut self.ema_c {
            ema_c.next_update_last(close).unwrap();
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

        //TAKE LATEST 50 ELEMENTS OF THE ARRAY FOR WARMING AND EXCLUDING THE LASTEST ONE
        for prev_candle in &data[len - num_items..len - 1] {
            if let Some(atr) = &mut self.atr {
                atr.next_tmp(prev_candle.close());
            }
            if let Some(bb) = &mut self.bb {
                bb.next_tmp(prev_candle.close());
            }
            if let Some(bbw) = &mut self.bbw {
                bbw.next_tmp(prev_candle.close());
            }
            if let Some(ema_a) = &mut self.ema_a {
                ema_a.next_tmp(prev_candle.close());
            }
            if let Some(ema_b) = &mut self.ema_b {
                ema_b.next_tmp(prev_candle.close());
            }
            if let Some(ema_c) = &mut self.ema_c {
                ema_c.next_tmp(prev_candle.close());
            }
            if let Some(macd) = &mut self.macd {
                macd.next_tmp(prev_candle.close());
            }
            if let Some(rsi) = &mut self.rsi {
                rsi.next_tmp(prev_candle.close());
            }
        }

        // UPDATING LAST VALUE & RESET

        //DO THIS FOR ALL INDICATORS
        if let Some(atr) = &mut self.atr {
            atr.next_update_last_tmp(close).unwrap();
        }
        if let Some(bb) = &mut self.bb {
            bb.next_update_last_tmp(close).unwrap();
        }
        if let Some(bbw) = &mut self.bbw {
            bbw.next_update_last_tmp(close).unwrap();
        }
        if let Some(ema_a) = &mut self.ema_a {
            ema_a.next_update_last_tmp(close).unwrap();
        }
        if let Some(ema_b) = &mut self.ema_b {
            ema_b.next_update_last_tmp(close).unwrap();
        }
        if let Some(ema_c) = &mut self.ema_c {
            ema_c.next_update_last_tmp(close).unwrap();
        }
        if let Some(macd) = &mut self.macd {
            macd.next_update_last_tmp(close).unwrap();
        }
        if let Some(rsi) = &mut self.rsi {
            rsi.next_update_last_tmp(close).unwrap();
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

        if let Some(rsi) = &mut self.rsi {
            if rsi.get_data_a().len() > max_bars && remove_first {
                rsi.remove_a(0);
            }
            rsi.init_indicator();
        }

        if let Some(bb) = &mut self.bb {
            if bb.get_data_a().len() > max_bars && remove_first {
                bb.remove_a(0);
                bb.remove_b(0);
                bb.remove_c(0);
            }
            bb.init_indicator();
        }

        if let Some(bbw) = &mut self.bbw {
            if bbw.get_data_a().len() > max_bars && remove_first {
                bbw.remove_a(0);
                bbw.remove_b(0);
                bbw.remove_c(0);
            }
            bbw.init_indicator();
        }

        if let Some(ema_a) = &mut self.ema_a {
            if ema_a.get_data_a().len() > max_bars && remove_first {
                ema_a.remove_a(0);
            }
            ema_a.init_indicator();
        }

        if let Some(ema_b) = &mut self.ema_b {
            if ema_b.get_data_a().len() > max_bars && remove_first {
                ema_b.remove_a(0);
            }
            ema_b.init_indicator();
        }

        if let Some(ema_c) = &mut self.ema_c {
            if ema_c.get_data_a().len() > max_bars && remove_first {
                ema_c.remove_a(0);
            }
            ema_c.init_indicator();
        }

        if let Some(atr) = &mut self.atr {
            if atr.get_data_a().len() > max_bars && remove_first {
                atr.remove_a(0);
            }
            atr.init_indicator();
        }

        if let Some(macd) = &mut self.macd {
            if macd.get_data_a().len() > max_bars && remove_first {
                macd.remove_a(0);
                macd.remove_b(0);
                macd.remove_c(0);
            }
            macd.init_indicator();
        }

        Ok(())
    }
}
