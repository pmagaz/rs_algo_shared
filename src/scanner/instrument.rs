use crate::error::{Result, RsAlgoError, RsAlgoErrorKind};
use crate::helpers::comp::*;
use crate::helpers::date::*;
use crate::indicators::Indicators;
use crate::models::indicator::CompactIndicators;
use crate::models::market::*;
use crate::models::time_frame::TimeFrameType;
use crate::scanner::candle::{Candle, CandleType};
use crate::scanner::divergence::{CompactDivergences, Divergences};
use crate::scanner::horizontal_level::HorizontalLevels;
use crate::scanner::pattern::PatternSize;
use crate::scanner::pattern::Patterns;
use crate::scanner::peak::Peaks;
use serde::{Deserialize, Serialize};

use std::env;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CompactInstrument {
    pub symbol: String,
    pub time_frame: TimeFrameType,
    pub market: Market,
    pub current_price: f64,
    pub prev_price: f64,
    pub avg_volume: f64,
    pub current_candle: CandleType,
    pub prev_candle: CandleType,
    pub date: DbDateTime,
    pub patterns: Patterns,
    pub horizontal_levels: HorizontalLevels,
    pub indicators: CompactIndicators,
    pub divergences: CompactDivergences,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Instrument {
    pub symbol: String,
    pub time_frame: TimeFrameType,
    pub market: Market,
    pub current_price: f64,
    pub min_price: f64,
    pub max_price: f64,
    pub avg_volume: f64,
    pub current_candle: CandleType,
    pub date: DbDateTime,
    pub data: Vec<Candle>,
    pub peaks: Peaks,
    pub patterns: Patterns,
    pub horizontal_levels: HorizontalLevels,
    pub indicators: Indicators,
    pub divergences: Divergences,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MainInstrument {
    MainInstrument(Instrument),
    None,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HigherTMInstrument {
    HigherTMInstrument(Instrument),
    None,
}

impl Instrument {
    pub fn new() -> InstrumentBuilder {
        InstrumentBuilder::new()
    }

    pub fn symbol(&self) -> &str {
        &self.symbol
    }

    pub fn market(&self) -> &Market {
        &self.market
    }

    pub fn time_frame(&self) -> &TimeFrameType {
        &self.time_frame
    }

    pub fn indicators(&self) -> &Indicators {
        &self.indicators
    }

    pub fn data(&self) -> &Vec<Candle> {
        &self.data
    }

    pub fn set_current_price(&mut self, current_price: f64) -> f64 {
        self.current_price = current_price;
        self.current_price
    }

    pub fn date(&self) -> &DbDateTime {
        &self.date
    }

    // pub fn current_price(&self) -> f64 {
    //     self.current_price
    // }

    pub fn current_candle(&self) -> &Candle {
        let num_candles = &self.data().len() - 1;
        &self.data()[num_candles]
    }

    pub fn min_price(&self) -> f64 {
        self.min_price
    }

    pub fn max_price(&self) -> f64 {
        self.max_price
    }

    pub fn peaks(&self) -> &Peaks {
        &self.peaks
    }
    pub fn patterns(&self) -> &Patterns {
        &self.patterns
    }
    pub fn horizontal_levels(&self) -> &HorizontalLevels {
        &self.horizontal_levels
    }

    pub fn divergences(&self) -> &Divergences {
        &self.divergences
    }

    pub fn process_candle(
        &mut self,
        id: usize,
        x: (DateTime<Local>, f64, f64, f64, f64, f64),
        data: &Vec<(DateTime<Local>, f64, f64, f64, f64, f64)>,
        logarithmic_scanner: bool,
    ) -> Candle {
        let date = x.0;
        let open: f64;
        let high: f64;
        let low: f64;
        let close: f64;
        let volume = x.5;

        match logarithmic_scanner {
            true => {
                open = x.1.ln();
                high = x.2.ln();
                close = x.4.ln();
                low = match x.3 {
                    _x if x.3 > 0. => x.3.ln(),
                    _x if x.3 <= 0. => 0.01,
                    _ => x.3.ln(),
                };
            }
            false => {
                open = x.1;
                high = x.2;
                close = x.4;
                low = match x.3 {
                    _x if x.3 > 0. => x.3,
                    _x if x.3 <= 0. => 0.01,
                    _ => x.3,
                };
            }
        };

        let pre_0 = match id {
            0 => id,
            _ => id - 1,
        };

        let prev_1 = match pre_0 {
            0 => id,
            _ => id - 1,
        };

        Candle::new()
            .date(date)
            .open(open)
            .high(high)
            .low(low)
            .close(close)
            .volume(volume)
            .previous_candles(vec![data[pre_0], data[prev_1]])
            .logarithmic(logarithmic_scanner)
            .build()
            .unwrap()
    }

    pub fn process_next_candle(
        &self,
        id: usize,
        x: (DateTime<Local>, f64, f64, f64, f64, f64, f64),
        data: &Vec<Candle>,
        logarithmic_scanner: bool,
    ) -> Candle {
        let date = x.0;
        let open: f64;
        let high: f64;
        let low: f64;
        let close: f64;
        let volume = x.5;
        let _spread = x.6;

        match logarithmic_scanner {
            true => {
                open = x.1.ln();
                high = x.2.ln();
                close = x.4.ln();
                low = match x.3 {
                    _x if x.3 > 0. => x.3.ln(),
                    _x if x.3 <= 0. => 0.01,
                    _ => x.3.ln(),
                };
            }
            false => {
                open = x.1;
                high = x.2;
                close = x.4;
                low = match x.3 {
                    _x if x.3 > 0. => x.3,
                    _x if x.3 <= 0. => 0.01,
                    _ => x.3,
                };
            }
        };

        let pre_0 = match id {
            0 => id,
            _ => id - 1,
        };

        let prev_1 = match pre_0 {
            0 => id,
            _ => id - 1,
        };

        let last = data[pre_0].clone();
        let last_candle = (
            last.date(),
            last.open(),
            last.high(),
            last.low(),
            last.close(),
            last.volume(),
        );

        let second_last = data[prev_1].clone();
        let second_last_candle = (
            second_last.date(),
            second_last.open(),
            second_last.high(),
            second_last.low(),
            second_last.close(),
            second_last.volume(),
        );
        //DOHLCV

        Candle::new()
            .date(date)
            .open(open)
            .high(high)
            .low(low)
            .close(close)
            .volume(volume)
            .previous_candles(vec![last_candle, second_last_candle])
            .logarithmic(logarithmic_scanner)
            .build()
            .unwrap()
    }

    pub fn get_ohlc_indicators(
        &mut self,
        candle: &Candle,
        logarithmic_scanner: bool,
    ) -> (f64, f64, f64, f64) {
        let ohlc_indicators = match logarithmic_scanner {
            true => (
                candle.open().exp(),
                candle.high().exp(),
                candle.low().exp(),
                candle.close().exp(),
            ),
            false => (candle.open(), candle.high(), candle.low(), candle.close()),
        };

        ohlc_indicators
    }

    pub fn set_data(
        &mut self,
        data: Vec<(DateTime<Local>, f64, f64, f64, f64, f64)>,
    ) -> Result<()> {
        let mut avg_volume = vec![];
        let logarithmic_scanner = env::var("LOGARITHMIC_SCANNER")
            .unwrap()
            .parse::<bool>()
            .unwrap();

        let avg_volume_days = env::var("AVG_VOLUME_DAYS")
            .unwrap()
            .parse::<usize>()
            .unwrap();

        let candles: Vec<Candle> = data
            .iter()
            .enumerate()
            .map(|(id, x)| {
                // let date = x.0;
                // let open: f64;
                // let high: f64;
                // let low: f64;
                // let close: f64;
                // let volume = x.5;

                // match logarithmic_scanner {
                //     true => {
                //         open = x.1.ln();
                //         high = x.2.ln();
                //         close = x.4.ln();
                //         low = match x.3 {
                //             _x if x.3 > 0. => x.3.ln(),
                //             _x if x.3 <= 0. => 0.01,
                //             _ => x.3.ln(),
                //         };
                //     }
                //     false => {
                //         open = x.1;
                //         high = x.2;
                //         close = x.4;
                //         low = match x.3 {
                //             _x if x.3 > 0. => x.3,
                //             _x if x.3 <= 0. => 0.01,
                //             _ => x.3,
                //         };
                //     }
                // };

                // let pre_0 = match id {
                //     0 => id,
                //     _ => id - 1,
                // };

                // let prev_1 = match pre_0 {
                //     0 => id,
                //     _ => id - 1,
                // };

                // let ohlc_indicators = match logarithmic_scanner {
                //     true => (open.exp(), high.exp(), low.exp(), close.exp()),
                //     false => (open, high, low, close),
                // };

                let candle = self.process_candle(id, *x, &data, logarithmic_scanner);

                let low = candle.low();
                let high = candle.high();
                let open = candle.open();
                let close = candle.close();
                let volume = candle.volume();

                if self.min_price == -100. {
                    self.min_price = candle.low();
                }
                if low < self.min_price {
                    self.min_price = candle.low();
                }
                if self.max_price == -100. {
                    self.max_price = candle.high();
                }
                if high > self.max_price {
                    self.max_price = candle.high();
                }

                avg_volume.push(volume);
                self.peaks.highs.push(high);
                self.peaks.lows.push(low);
                self.peaks.close.push(close);

                let ohlc_indicators = self.get_ohlc_indicators(&candle, logarithmic_scanner);

                self.indicators
                    .calculate_indicators(ohlc_indicators)
                    .unwrap();

                candle
            })
            .collect();

        if candles.len() > 0 {
            self.peaks
                .calculate_peaks(&self.max_price, &self.min_price)
                .unwrap();

            let local_maxima = self.peaks.local_maxima();
            let local_minima = self.peaks.local_minima();
            // let extrema_maxima = self.peaks.extrema_maxima();
            // let extrema_minima = self.peaks.extrema_minima();

            self.patterns
                .detect_pattern(PatternSize::Local, local_maxima, local_minima, &candles);

            // self.patterns.detect_pattern(
            //     PatternSize::Extrema,
            //     extrema_maxima,
            //     extrema_minima,
            //     &candles,
            // );

            self.horizontal_levels
                .calculate_horizontal_highs(&self.current_price, &self.peaks)
                .unwrap();

            self.horizontal_levels
                .calculate_horizontal_lows(&self.current_price, &self.peaks)
                .unwrap();

            // self.divergences.detect_divergences(
            //     &self.indicators,
            //     &self.patterns.local_patterns,
            //     &candles,
            //     &local_maxima,
            // );

            self.data = candles
                .into_iter()
                .map(|candle| {
                    let data = match logarithmic_scanner {
                        true => candle.from_logarithmic_values(),
                        false => candle,
                    };

                    if self.min_price == -100. {
                        self.min_price = data.low();
                    }
                    if data.low() < self.min_price {
                        self.min_price = data.low();
                    }
                    if self.max_price == -100. {
                        self.max_price = data.low();
                    }
                    if data.high() > self.max_price {
                        self.max_price = data.high();
                    }
                    data
                })
                .collect();

            self.set_current_price(self.data.last().unwrap().close());

            self.current_candle = self.current_candle().candle_type().clone();
            self.avg_volume =
                average_f64(&avg_volume.into_iter().rev().take(avg_volume_days).collect());
        }
        Ok(())
    }

    pub fn next(&mut self, x: (DateTime<Local>, f64, f64, f64, f64, f64, f64)) -> Result<()> {
        let logarithmic_scanner = env::var("LOGARITHMIC_SCANNER")
            .unwrap()
            .parse::<bool>()
            .unwrap();

        let next_id = self.data.len() + 1;

        let candle = self.process_next_candle(next_id, x, &self.data, logarithmic_scanner);

        let ohlc_indicators = self.get_ohlc_indicators(&candle, logarithmic_scanner);

        self.indicators
            .calculate_indicators(ohlc_indicators)
            .unwrap();

        println!("111111 {:?}", self.indicators);
        Ok(())
    }
}

pub struct InstrumentBuilder {
    symbol: Option<String>,
    market: Option<Market>,
    time_frame: Option<TimeFrameType>,
    //indicators: Option<Indicators>,
}

impl InstrumentBuilder {
    pub fn new() -> InstrumentBuilder {
        Self {
            symbol: None,
            market: None,
            time_frame: None,
        }
    }
    pub fn symbol(mut self, val: &str) -> Self {
        self.symbol = Some(String::from(val));
        self
    }

    pub fn market(mut self, val: Market) -> Self {
        self.market = Some(val);
        self
    }

    pub fn time_frame(mut self, val: TimeFrameType) -> Self {
        self.time_frame = Some(val);
        self
    }

    pub fn build(self) -> Result<Instrument> {
        if let (Some(symbol), Some(market), Some(time_frame)) =
            (self.symbol, self.market, self.time_frame)
        {
            Ok(Instrument {
                symbol,
                market,
                time_frame: time_frame,
                current_price: 0.,
                date: to_dbtime(Local::now()), //FIXME
                current_candle: CandleType::Default,
                min_price: env::var("MIN_PRICE").unwrap().parse::<f64>().unwrap(),
                max_price: env::var("MIN_PRICE").unwrap().parse::<f64>().unwrap(),
                avg_volume: 0.,
                data: vec![],
                peaks: Peaks::new(),
                horizontal_levels: HorizontalLevels::new(),
                patterns: Patterns::new(),
                indicators: Indicators::new().unwrap(),
                divergences: Divergences::new().unwrap(),
            })
        } else {
            Err(RsAlgoError {
                err: RsAlgoErrorKind::WrongInstrumentConf,
            })
        }
    }
}
