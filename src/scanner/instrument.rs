use crate::error::{Result, RsAlgoError, RsAlgoErrorKind};
use crate::helpers::comp::*;
use crate::helpers::date::*;
use crate::indicators::{Indicator, Indicators};
use crate::models::indicator::CompactIndicators;
use crate::models::mode::ExecutionMode;

use crate::models::time_frame::*;
use crate::models::{market::*, mode};
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
pub enum HTFInstrument {
    HTFInstrument(Instrument),
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

    pub fn get_scale_ohlc(
        &self,
        x: (DateTime<Local>, f64, f64, f64, f64, f64, bool),
        logarithmic_scanner: bool,
    ) -> (f64, f64, f64, f64) {
        let open: f64;
        let high: f64;
        let low: f64;
        let close: f64;
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

        (open, high, low, close)
    }

    pub fn get_scale_ohlc_indicators(
        &mut self,
        candle: &Candle,
        logarithmic_scanner: bool,
    ) -> (f64, f64, f64, f64) {
        match logarithmic_scanner {
            true => (
                candle.open().exp(),
                candle.high().exp(),
                candle.low().exp(),
                candle.close().exp(),
            ),
            false => (candle.open(), candle.high(), candle.low(), candle.close()),
        }
    }

    pub fn process_candle(
        &mut self,
        id: usize,
        data: &Vec<(DateTime<Local>, f64, f64, f64, f64, f64)>,
        adapted: (DateTime<Local>, f64, f64, f64, f64, f64, bool),
        logarithmic_scanner: bool,
    ) -> Candle {
        let execution_mode = mode::from_str(&env::var("EXECUTION_MODE").unwrap());
        let date = adapted.0;
        let volume = adapted.5;
        let is_closed = match execution_mode {
            ExecutionMode::ScannerBackTest => true,
            _ => adapted.6,
        };

        let (open, high, low, close) = self.get_scale_ohlc(adapted, logarithmic_scanner);

        let prev_0 = id.saturating_sub(1);
        let prev_1 = prev_0.saturating_sub(1);
        let prev_2 = prev_1.saturating_sub(1);
        let prev_3 = prev_2.saturating_sub(1);

        Candle::new()
            .date(date)
            .open(open)
            .high(high)
            .low(low)
            .close(close)
            .volume(volume)
            .is_closed(is_closed)
            .previous_candles(vec![data[prev_0], data[prev_1], data[prev_2], data[prev_3]])
            .logarithmic(logarithmic_scanner)
            .build()
            .unwrap()
    }

    pub fn generate_candle(
        &self,
        id: usize,
        data: (DateTime<Local>, f64, f64, f64, f64, f64, bool),
        candles: &Vec<Candle>,
        logarithmic_scanner: bool,
    ) -> Candle {
        let date = data.0;
        let volume = data.5;
        let is_closed = data.6;
        let (open, high, low, close) = self.get_scale_ohlc(data, logarithmic_scanner);

        let prev_0 = id.saturating_sub(2);
        let prev_1 = prev_0.saturating_sub(1);
        let prev_2 = prev_1.saturating_sub(1);
        let prev_3 = prev_2.saturating_sub(1);

        let candle_0 = candles[prev_0].clone();
        let candle_1 = candles[prev_1].clone();
        let candle_2 = candles[prev_2].clone();
        let candle_3 = candles[prev_3].clone();

        let data_0 = (
            candle_0.date(),
            candle_0.open(),
            candle_0.high(),
            candle_0.low(),
            candle_0.close(),
            candle_0.volume(),
        );

        let data_1 = (
            candle_1.date(),
            candle_1.open(),
            candle_1.high(),
            candle_1.low(),
            candle_1.close(),
            candle_1.volume(),
        );

        let data_2 = (
            candle_2.date(),
            candle_2.open(),
            candle_2.high(),
            candle_2.low(),
            candle_2.close(),
            candle_2.volume(),
        );

        let data_3 = (
            candle_3.date(),
            candle_3.open(),
            candle_3.high(),
            candle_3.low(),
            candle_3.close(),
            candle_3.volume(),
        );

        Candle::new()
            .date(date)
            .open(open)
            .high(high)
            .low(low)
            .close(close)
            .volume(volume)
            .is_closed(is_closed)
            .previous_candles(vec![data_0, data_1, data_2, data_3])
            .logarithmic(logarithmic_scanner)
            .build()
            .unwrap()
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

        let process_indicators = env::var("INDICATORS").unwrap().parse::<bool>().unwrap();
        let process_patterns = env::var("PATTERNS").unwrap().parse::<bool>().unwrap();
        let process_divergences = env::var("DIVERGENCES").unwrap().parse::<bool>().unwrap();
        let process_horizontal_levels = env::var("HORIZONTAL_LEVELS")
            .unwrap()
            .parse::<bool>()
            .unwrap();

        let avg_volume_days = env::var("AVG_VOLUME_DAYS")
            .unwrap()
            .parse::<usize>()
            .unwrap();

        //FIXME Instrument should be Optional
        self.init();

        let candles: Vec<Candle> = data
            .iter()
            .enumerate()
            .map(|(id, x)| {
                let formated_dohlcc = format_open_until(*x, &self.time_frame, false);
                let candle = self.process_candle(id, &data, formated_dohlcc, logarithmic_scanner);
                let _num_bars = env::var("NUM_BARS").unwrap().parse::<usize>().unwrap();

                let low = candle.low();
                let high = candle.high();
                let _open = candle.open();
                let _close = candle.close();
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

                if process_patterns && candle.is_closed() {
                    self.peaks.next(&candle);
                }

                if process_indicators {
                    if candle.is_closed() {
                        let ohlc_indicators =
                            self.get_scale_ohlc_indicators(&candle, logarithmic_scanner);
                        let delete_previous = false;

                        self.indicators
                            .next(ohlc_indicators, delete_previous, &self.time_frame().clone())
                            .unwrap();
                    } else {
                        self.indicators
                            .init_indicators(&self.time_frame().clone(), false)
                            .unwrap();
                    }
                }

                candle
            })
            .collect();

        if !candles.is_empty() {
            if process_patterns {
                self.peaks
                    .calculate_peaks(&self.max_price, &self.min_price, &0)
                    .unwrap();

                let local_maxima = self.peaks.local_maxima();
                let local_minima = self.peaks.local_minima();
                // let extrema_maxima = self.peaks.extrema_maxima();
                // let extrema_minima = self.peaks.extrema_minima();

                self.patterns.detect_pattern(
                    PatternSize::Local,
                    local_maxima,
                    local_minima,
                    &candles,
                );

                // self.patterns.process_pattern(
                //     PatternSize::Extrema,
                //     extrema_maxima,
                //     extrema_minima,
                //     &candles,
                // );
            }

            if process_horizontal_levels {
                self.horizontal_levels
                    .calculate_horizontal_highs(&self.current_price, &self.peaks)
                    .unwrap();

                self.horizontal_levels
                    .calculate_horizontal_lows(&self.current_price, &self.peaks)
                    .unwrap();
            }

            if process_divergences {
                // self.divergences.process_divergences(
                //     &self.indicators,
                //     &self.patterns.local_patterns,
                //     &candles,
                //     &local_maxima,
                // );
            }

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
                        self.max_price = data.high();
                    }
                    if data.high() > self.max_price {
                        self.max_price = data.high();
                    }

                    data
                })
                .collect();

            //self.data = candles;

            self.set_current_price(self.data.last().unwrap().close());

            self.current_candle = self.current_candle().candle_type().clone();
            self.avg_volume =
                average_f64(&avg_volume.into_iter().rev().take(avg_volume_days).collect());
        }

        Ok(())
    }

    pub fn next(&mut self, data: (DateTime<Local>, f64, f64, f64, f64, f64)) -> Result<Candle> {
        let logarithmic_scanner = env::var("LOGARITHMIC_SCANNER")
            .unwrap()
            .parse::<bool>()
            .unwrap();

        let next_id = self.data.len();
        let last_candle = &self.data().last().unwrap().clone();

        let time_frame = &self.time_frame.clone();
        let formated_dohlcc = format_open_until(data, &self.time_frame, true);
        let formated_candle =
            self.generate_candle(next_id, formated_dohlcc, &self.data, logarithmic_scanner);

        if formated_candle.is_closed() {
            self.close_last_candle();
            self.close_indicators(&last_candle);
        } else {
            self.adapt_last_candle_tf(formated_candle.clone(), last_candle, time_frame);
            let updated_candle = &self.data.last().unwrap().clone();
            self.update_tmp_indicators(updated_candle);
        }

        Ok(formated_candle)
    }

    pub fn close_indicators(&mut self, candle: &Candle) {
        let logarithmic_scanner = env::var("LOGARITHMIC_SCANNER")
            .unwrap()
            .parse::<bool>()
            .unwrap();

        let process_indicators = env::var("INDICATORS").unwrap().parse::<bool>().unwrap();

        if process_indicators {
            let ohlc_indicators = self.get_scale_ohlc_indicators(candle, logarithmic_scanner);
            self.indicators
                .next_close_indicators(ohlc_indicators, &self.time_frame().clone())
                .unwrap();
        }
    }

    pub fn update_tmp_indicators(&mut self, candle: &Candle) {
        let process_indicators = env::var("INDICATORS").unwrap().parse::<bool>().unwrap();
        if process_indicators {
            self.indicators
                .next_tmp_indicators(candle, &self.data)
                .unwrap();
        }
    }

    pub fn next_peaks(&mut self, candle: &Candle) {
        let _logarithmic_scanner = env::var("LOGARITHMIC_SCANNER")
            .unwrap()
            .parse::<bool>()
            .unwrap();
        let process_patterns = env::var("PATTERNS").unwrap().parse::<bool>().unwrap();
        if process_patterns {
            //FIXME peaks next detection iterates the whole list
            self.peaks.update(candle);
            self.peaks
                .calculate_peaks(&self.max_price, &self.min_price, &0)
                .unwrap();
            let local_maxima = self.peaks.local_maxima();
            let local_minima = self.peaks.local_minima();
            //Fixme CALCULATE ONLY LAST CHANGES clean first pattern
            self.patterns
                .update(PatternSize::Local, local_maxima, local_minima, &self.data);
        }
    }

    pub fn close_last_candle(&mut self) {
        let last_candle = self.data.last_mut().unwrap();
        last_candle.set_is_closed(true);
    }

    pub fn adapt_last_candle_tf(
        &mut self,
        mut candle: Candle,
        last_candle: &Candle,
        time_frame: &TimeFrameType,
    ) {
        let current_high = candle.high();
        let previous_open = last_candle.open();
        let previous_high = last_candle.high();
        let _previous_close = last_candle.close();

        let current_low = candle.low();
        let previous_low = last_candle.low();

        let higher_value = match current_high {
            _ if current_high > previous_high => current_high,
            _ if current_high <= previous_high => previous_high,
            _ => previous_high,
        };

        let lower_value = match current_low {
            _ if current_low < previous_low => current_low,
            _ => previous_low,
        };

        if !time_frame.is_base_time_frame() {
            candle.set_open(previous_open);
            candle.set_high(higher_value);
            candle.set_low(lower_value);
        }

        *self.data.last_mut().unwrap() = candle.clone();
    }

    pub fn init_candle(
        &mut self,
        data: (DateTime<Local>, f64, f64, f64, f64, f64),
        time_frame: &Option<TimeFrameType>,
    ) {
        let logarithmic_scanner = env::var("LOGARITHMIC_SCANNER")
            .unwrap()
            .parse::<bool>()
            .unwrap();

        let num_bars = env::var("NUM_BARS").unwrap().parse::<usize>().unwrap();

        let adapted = format_open_until(data, &self.time_frame, true);
        let open_from = get_open_from(data, &self.time_frame, true);

        let len = self.data.len();

        let mut candle = self.generate_candle(len, adapted, &self.data, logarithmic_scanner);

        candle.set_is_closed(false);
        candle.set_date(open_from);

        let max_bars = num_bars / time_frame.clone().unwrap().to_number() as usize;

        if len > max_bars {
            self.data.remove(0);
        }

        self.data.push(candle);
    }

    pub fn init(&mut self) {
        self.data = vec![];
        self.peaks = Peaks::new();
        self.horizontal_levels = HorizontalLevels::new();
        self.patterns = Patterns::new();
        self.indicators = Indicators::new().unwrap();
        self.divergences = Divergences::new().unwrap();
        //self.set_data(data).unwrap();
    }
}

pub struct InstrumentBuilder {
    symbol: Option<String>,
    market: Option<Market>,
    time_frame: Option<TimeFrameType>,
    //indicators: Option<Indicators>,
}

impl Default for InstrumentBuilder {
    fn default() -> Self {
        Self::new()
    }
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
                time_frame,
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
