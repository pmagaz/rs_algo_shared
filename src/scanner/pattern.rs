use crate::helpers::slope_intercept::add_next_bottom_points;
use crate::helpers::{poly::poly_fit, slope_intercept::add_next_top_points};

use crate::patterns::*;
use crate::scanner::candle::Candle;
use crate::scanner::prices::calculate_price_target;

use crate::helpers::comp::percentage_change;
use crate::helpers::date::*;
use crate::models::status::Status;
use serde::{Deserialize, Serialize};

use std::env;

pub type PatternActiveResult = (bool, usize, f64, DbDateTime);

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum PatternDirection {
    Top,
    Bottom,
    None,
}

impl std::fmt::Display for PatternDirection {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

type Point = (usize, f64);
pub type DataPoints = Vec<Point>;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum PatternType {
    BullishFairValueGap,
    BearishFairValueGap,
    BullishOrderBlock,
    BearishOrderBlock,
    BullishReversal,
    Triangle,
    TriangleSym,
    TriangleDown,
    TriangleUp,
    Rectangle,
    ChannelUp,
    ChannelDown,
    Broadening,
    DoubleTop,
    DoubleBottom,
    HeadShoulders,
    HigherHighsHigherLows,
    LowerHighsLowerLows,
    None,
}

impl std::fmt::Display for PatternType {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum PatternSize {
    Local,
    Extrema,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PatternActive {
    pub active: bool,
    pub completed: bool,
    pub index: usize,
    pub date: DbDateTime,
    pub price: f64,
    pub status: Status,
    pub break_direction: PatternDirection,
    pub target: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Pattern {
    pub index: usize,
    pub date: DbDateTime,
    pub pattern_type: PatternType,
    pub pattern_size: PatternSize,
    pub data_points: DataPoints,
    pub direction: PatternDirection,
    pub active: PatternActive,
    pub target: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompactPattern {
    pub index: usize,
    pub date: DbDateTime,
    pub pattern_type: PatternType,
    pub pattern_size: PatternSize,
    pub direction: PatternDirection,
    pub active: PatternActive,
    pub change: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Patterns {
    pub local_patterns: Vec<Pattern>,
    pub extrema_patterns: Vec<Pattern>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompactPatterns {
    pub local_patterns: Vec<CompactPattern>,
    pub extrema_patterns: Vec<CompactPattern>,
}

impl Default for Patterns {
    fn default() -> Self {
        Self::new()
    }
}

impl Patterns {
    pub fn new() -> Self {
        Patterns {
            local_patterns: vec![],
            extrema_patterns: vec![],
        }
    }

    pub fn next(
        &mut self,
        pattern_size: PatternSize,
        maxima: &Vec<(usize, f64)>,
        minima: &Vec<(usize, f64)>,
        candles: &Vec<Candle>,
    ) {
        let _pattern_prev_bars = env::var("MAX_PREVIOUS_BARS")
            .unwrap()
            .parse::<usize>()
            .unwrap();
        let _len = candles.len();

        // let last_candles = match len.cmp(&0) {
        //     Ordering::Greater => {
        //         let last_candles: Vec<Candle> = candles[len - pattern_prev_bars..len].to_vec();
        //         last_candles
        //     }
        //     _ => vec![],
        // };

        self.detect_pattern(pattern_size, maxima, minima, candles);
    }

    pub fn update(
        &mut self,
        pattern_size: PatternSize,
        maxima: &Vec<(usize, f64)>,
        minima: &Vec<(usize, f64)>,
        candles: &Vec<Candle>,
    ) {
        self.detect_pattern(pattern_size, maxima, minima, candles);
    }

    pub fn detect_pattern(
        &mut self,
        pattern_size: PatternSize,
        maxima: &Vec<(usize, f64)>,
        minima: &Vec<(usize, f64)>,
        candles: &Vec<Candle>,
    ) {
        let local_max_points = env::var("PATTERNS_MAX_POINTS")
            .unwrap()
            .parse::<usize>()
            .unwrap();

        let min_points = env::var("PATTERNS_MIN_POINTS")
            .unwrap()
            .parse::<usize>()
            .unwrap();

        let _window_size = env::var("PATTERNS_WINDOW_SIZE")
            .unwrap()
            .parse::<usize>()
            .unwrap();

        let mut max_start = 0;
        let mut max_end = 0;
        let mut min_start = 0;
        let mut min_end = 0;
        let maxima_length = maxima.len();
        let minima_length = minima.len();
        if maxima_length >= min_points && minima_length >= min_points {
            if maxima_length > local_max_points {
                max_start = maxima_length - local_max_points;
                max_end = maxima_length;
            } else {
                max_start = 0;
                max_end = maxima_length;
            }

            if minima_length > local_max_points {
                min_start = minima_length - local_max_points;
                min_end = minima_length;
            } else {
                min_start = 0;
                min_end = minima_length;
            }

            let mut locals = [&maxima[max_start..max_end], &minima[min_start..min_end]].concat();

            locals.sort_by(|(id_a, _price_a), (id_b, _price_b)| id_a.cmp(id_b));
            //locals.reverse();

            //PRICE ACTIOIN BASED STRUCTURES
            // ...

            let window_size = 4;
            let mut not_found: bool = true;
            let _index = 0;

            // for index in 0..candles.len() - window_size + 1 {
            //     let window = &candles[index..index + window_size];

            // if smc::is_bullish_fair_value_gap(window) {
            //     self.set_pattern(
            //         PatternType::BullishFairValueGap,
            //         PatternDirection::Top,
            //         &pattern_size,
            //         vec![
            //             (index, window[0].high),
            //             (index + 10, window[2].low),
            //             (index + 10, window[0].high),
            //             (index + 20, window[2].low),
            //         ],
            //         window[1].date,
            //         PatternActive {
            //             active: false,
            //             completed: false,
            //             index: 0,
            //             date: to_dbtime(window[1].date),
            //             price: window[2].close,
            //             status: Status::Bullish,
            //             break_direction: PatternDirection::None,
            //             target: 0.,
            //         },
            //     );
            // }
            // } else if smc::is_bullish_reversal(window) {
            //     self.set_pattern(
            //         PatternType::BullishFairValueGap,
            //         PatternDirection::Top,
            //         &pattern_size,
            //         vec![
            //             (index, window[0].high),
            //             (index + 10, window[2].low),
            //             (index + 10, window[0].high),
            //             (index + 20, window[2].low),
            //         ],
            //         window[1].date,
            //         PatternActive {
            //             active: false,
            //             completed: false,
            //             index: 0,
            //             date: to_dbtime(window[1].date),
            //             price: window[2].close,
            //             status: Status::Bullish,
            //             break_direction: PatternDirection::None,
            //             target: 0.,
            //         },
            //     );
            // }
            //}

            //DATAPOINTS BASED PATTERNS
            let mut iter = locals.windows(window_size);
            let _not_found = true;

            while not_found {
                match iter.next() {
                    Some(window) => {
                        let mut data_points = window.to_vec();
                        let last_index = data_points.last().unwrap().0;
                        let candle_date = candles.get(last_index).unwrap().date();

                        if rectangle::is_renctangle_top(&data_points) {
                            data_points = add_next_top_points(data_points);

                            let is_pattern_active = rectangle::rectangle_top_active(
                                &data_points,
                                candles,
                                PatternType::Rectangle,
                            );

                            self.set_pattern(
                                PatternType::Rectangle,
                                PatternDirection::Top,
                                &pattern_size,
                                data_points.to_owned(),
                                candle_date,
                                is_pattern_active,
                            );
                            not_found = true;
                        } else if rectangle::is_renctangle_bottom(&data_points) {
                            data_points = add_next_bottom_points(data_points);

                            let is_pattern_active = rectangle::rectangle_bottom_active(
                                &data_points,
                                candles,
                                PatternType::Rectangle,
                            );

                            self.set_pattern(
                                PatternType::Rectangle,
                                PatternDirection::Bottom,
                                &pattern_size,
                                data_points.to_owned(),
                                candle_date,
                                is_pattern_active,
                            );
                            not_found = true;
                        } else if double::is_top(&data_points) {
                            data_points = add_next_top_points(data_points);

                            let is_pattern_active =
                                double::top_active(&data_points, candles, PatternType::DoubleTop);

                            self.set_pattern(
                                PatternType::DoubleTop,
                                PatternDirection::Top,
                                &pattern_size,
                                data_points.to_owned(),
                                candle_date,
                                is_pattern_active,
                            );
                            not_found = true;
                        } else if double::is_bottom(&data_points) {
                            data_points = add_next_bottom_points(data_points);

                            let is_pattern_active = double::top_active(
                                &data_points,
                                candles,
                                PatternType::DoubleBottom,
                            );

                            self.set_pattern(
                                PatternType::DoubleBottom,
                                PatternDirection::Bottom,
                                &pattern_size,
                                data_points.to_owned(),
                                candle_date,
                                is_pattern_active,
                            );
                            not_found = true;
                        } else if channel::is_ascendant_top(&data_points) {
                            data_points = add_next_top_points(data_points);

                            let is_pattern_active = channel::channel_ascendant_top_active(
                                &data_points,
                                candles,
                                PatternType::ChannelUp,
                            );

                            self.set_pattern(
                                PatternType::ChannelUp,
                                PatternDirection::Top,
                                &pattern_size,
                                data_points.to_owned(),
                                candle_date,
                                is_pattern_active,
                            );
                            not_found = true;
                        } else if channel::is_ascendant_bottom(&data_points) {
                            data_points = add_next_bottom_points(data_points);

                            let is_pattern_active = channel::channel_ascendant_bottom_active(
                                &data_points,
                                candles,
                                PatternType::ChannelUp,
                            );

                            self.set_pattern(
                                PatternType::ChannelUp,
                                PatternDirection::Bottom,
                                &pattern_size,
                                data_points.to_owned(),
                                candle_date,
                                is_pattern_active,
                            );
                            not_found = true;
                        } else if triangle::is_ascendant_top(&data_points) {
                            data_points = add_next_top_points(data_points);

                            let is_pattern_active = triangle::ascendant_top_active(
                                &data_points,
                                candles,
                                PatternType::TriangleUp,
                            );

                            self.set_pattern(
                                PatternType::TriangleUp,
                                PatternDirection::Top,
                                &pattern_size,
                                data_points.to_owned(),
                                candle_date,
                                is_pattern_active,
                            );
                            not_found = true;
                        } else if triangle::is_ascendant_bottom(&data_points) {
                            data_points = add_next_bottom_points(data_points);

                            let is_pattern_active = triangle::ascendant_bottom_active(
                                &data_points,
                                candles,
                                PatternType::TriangleUp,
                            );

                            self.set_pattern(
                                PatternType::TriangleUp,
                                PatternDirection::Bottom,
                                &pattern_size,
                                data_points.to_owned(),
                                candle_date,
                                is_pattern_active,
                            );
                            not_found = true;
                        } else if triangle::is_descendant_top(&data_points) {
                            data_points = add_next_top_points(data_points);

                            let is_pattern_active = triangle::descendant_top_active(
                                &data_points,
                                candles,
                                PatternType::TriangleDown,
                            );

                            self.set_pattern(
                                PatternType::TriangleDown,
                                PatternDirection::Top,
                                &pattern_size,
                                data_points.to_owned(),
                                candle_date,
                                is_pattern_active,
                            );
                            not_found = true;
                        } else if triangle::is_descendant_bottom(&data_points) {
                            data_points = add_next_bottom_points(data_points);

                            let is_pattern_active = triangle::descendant_bottom_active(
                                &data_points,
                                candles,
                                PatternType::TriangleDown,
                            );

                            self.set_pattern(
                                PatternType::TriangleDown,
                                PatternDirection::Bottom,
                                &pattern_size,
                                data_points.to_owned(),
                                candle_date,
                                is_pattern_active,
                            );
                            not_found = true;
                        } else if channel::is_descendant_top(&data_points) {
                            data_points = add_next_top_points(data_points);

                            let is_pattern_active = channel::channel_descendant_top_active(
                                &data_points,
                                candles,
                                PatternType::ChannelDown,
                            );

                            self.set_pattern(
                                PatternType::ChannelDown,
                                PatternDirection::Top,
                                &pattern_size,
                                data_points.to_owned(),
                                candle_date,
                                is_pattern_active,
                            );
                            not_found = true;
                        } else if channel::is_descendant_bottom(&data_points) {
                            data_points = add_next_bottom_points(data_points);
                            let is_pattern_active = channel::channel_descendant_bottom_active(
                                &data_points,
                                candles,
                                PatternType::ChannelDown,
                            );

                            self.set_pattern(
                                PatternType::ChannelDown,
                                PatternDirection::Top,
                                &pattern_size,
                                data_points.to_owned(),
                                candle_date,
                                is_pattern_active,
                            );
                            not_found = true;
                        } else if broadening::is_top(&data_points) {
                            data_points = add_next_top_points(data_points);

                            let is_pattern_active = broadening::broadening_top_active(
                                &data_points,
                                candles,
                                PatternType::Broadening,
                            );

                            self.set_pattern(
                                PatternType::Broadening,
                                PatternDirection::Top,
                                &pattern_size,
                                data_points.to_owned(),
                                candle_date,
                                is_pattern_active,
                            );
                            not_found = true;
                        } else if triangle::is_symmetrical_top(&data_points) {
                            data_points = add_next_top_points(data_points);

                            let is_pattern_active = triangle::symetrical_top_active(
                                &data_points,
                                candles,
                                PatternType::TriangleSym,
                            );

                            self.set_pattern(
                                PatternType::TriangleSym,
                                PatternDirection::Top,
                                &pattern_size,
                                data_points.to_owned(),
                                candle_date,
                                is_pattern_active,
                            );
                            not_found = true;
                        } else if triangle::is_symmetrical_bottom(&data_points) {
                            data_points = add_next_bottom_points(data_points);

                            let is_pattern_active = triangle::symetrical_bottom_active(
                                &data_points,
                                candles,
                                PatternType::TriangleSym,
                            );

                            self.set_pattern(
                                PatternType::TriangleSym,
                                PatternDirection::Bottom,
                                &pattern_size,
                                data_points.to_owned(),
                                candle_date,
                                is_pattern_active,
                            );
                            not_found = true;
                        } else if broadening::is_bottom(&data_points) {
                            data_points = add_next_bottom_points(data_points);

                            let is_pattern_active = broadening::broadening_top_active(
                                &data_points,
                                candles,
                                PatternType::Broadening,
                            );

                            self.set_pattern(
                                PatternType::Broadening,
                                PatternDirection::Bottom,
                                &pattern_size,
                                data_points.to_owned(),
                                candle_date,
                                is_pattern_active,
                            );
                            not_found = true;
                        } else if highs_lows::is_upperhighs_upperlows_top(&data_points) {
                            data_points = add_next_top_points(data_points);

                            let is_pattern_active = highs_lows::ascendant_top_active(
                                &data_points,
                                candles,
                                PatternType::HigherHighsHigherLows,
                            );

                            self.set_pattern(
                                PatternType::HigherHighsHigherLows,
                                PatternDirection::Top,
                                &pattern_size,
                                data_points.to_owned(),
                                candle_date,
                                is_pattern_active,
                            );
                            not_found = true;
                        } else if highs_lows::is_upperhighs_upperlows_bottom(&data_points) {
                            data_points = add_next_top_points(data_points);

                            let is_pattern_active = highs_lows::ascendant_bottom_active(
                                &data_points,
                                candles,
                                PatternType::HigherHighsHigherLows,
                            );

                            self.set_pattern(
                                PatternType::HigherHighsHigherLows,
                                PatternDirection::Top,
                                &pattern_size,
                                data_points.to_owned(),
                                candle_date,
                                is_pattern_active,
                            );
                            not_found = true;
                        } else if highs_lows::is_lower_highs_lower_lows_top(&data_points) {
                            data_points = add_next_top_points(data_points);

                            let is_pattern_active = highs_lows::descendant_top_active(
                                &data_points,
                                candles,
                                PatternType::HigherHighsHigherLows,
                            );

                            self.set_pattern(
                                PatternType::LowerHighsLowerLows,
                                PatternDirection::Top,
                                &pattern_size,
                                data_points.to_owned(),
                                candle_date,
                                is_pattern_active,
                            );
                            not_found = true;
                        } else if highs_lows::is_lower_highs_lower_lows_bottom(&data_points) {
                            data_points = add_next_top_points(data_points);

                            let is_pattern_active = highs_lows::descendant_bottom_active(
                                &data_points,
                                candles,
                                PatternType::HigherHighsHigherLows,
                            );

                            self.set_pattern(
                                PatternType::LowerHighsLowerLows,
                                PatternDirection::Top,
                                &pattern_size,
                                data_points.to_owned(),
                                candle_date,
                                is_pattern_active,
                            );
                            not_found = true;
                        }
                        // } else if head_shoulders::is_hs(&data_points) {
                        //     data_points = add_next_top_points(data_points);
                        //     self.set_pattern(
                        //         PatternType::HeadShoulders,
                        //         PatternDirection::Bottom,
                        //         &pattern_size,
                        //         data_points.to_owned(),
                        //         change,
                        //         candle_date,
                        //         head_shoulders::hs_active(
                        //             &data_points,
                        //             candles,
                        //             PatternType::HeadShoulders,
                        //         ),
                        //     );
                        //     not_found = true;
                        // } else if head_shoulders::is_inverse(&data_points) {
                        //     data_points = add_next_top_points(data_points);
                        //     self.set_pattern(
                        //         PatternType::HeadShoulders,
                        //         PatternDirection::Top,
                        //         &pattern_size,
                        //         data_points.to_owned(),
                        //         change,
                        //         candle_date,
                        //         head_shoulders::hs_active(
                        //             &data_points,
                        //             candles,
                        //             PatternType::HeadShoulders,
                        //         ),
                        //     );
                        //     not_found = true;
                        // }
                    }
                    None => {
                        let date = Local::now() - Duration::days(1000);
                        self.set_pattern(
                            PatternType::None,
                            PatternDirection::None,
                            &pattern_size,
                            vec![(0, 0.)],
                            date,
                            non_activated(),
                        );
                        not_found = false;
                    }
                }
            }
        }
    }

    fn calculate_change(&self, data_points: &DataPoints) -> f64 {
        percentage_change(data_points[0].1, data_points[1].1).abs()
    }

    //FXIME too many arguments
    //TOO complex I can't barely understand it after a while XD
    fn set_pattern(
        &mut self,
        pattern_type: PatternType,
        direction: PatternDirection,
        pattern_size: &PatternSize,
        data_points: DataPoints,
        date: DateTime<Local>,
        active: PatternActive,
    ) {
        let len = data_points.len();
        let target = calculate_price_target(&direction, &data_points);

        if len > 3 {
            let index = data_points.get(data_points.len() - 2).unwrap().0;
            if pattern_type != PatternType::None {
                let x_values_top: Vec<f64> = data_points
                    .iter()
                    .enumerate()
                    .filter(|(key, _x)| key % 2 == 0)
                    .map(|(_key, x)| x.0 as f64)
                    .collect();

                let y_values_top: Vec<f64> = data_points
                    .iter()
                    .enumerate()
                    .filter(|(key, _x)| key % 2 == 0)
                    .map(|(_key, x)| x.1)
                    .collect();

                let x_values_bottom: Vec<f64> = data_points
                    .iter()
                    .enumerate()
                    .filter(|(key, _x)| key % 2 != 0)
                    .map(|(_key, x)| x.0 as f64)
                    .collect();

                let y_values_bottom: Vec<f64> = data_points
                    .iter()
                    .enumerate()
                    .filter(|(key, _x)| key % 2 != 0)
                    .map(|(_key, x)| x.1)
                    .collect();

                let polynomial_top = poly_fit(&x_values_top, &y_values_top, 1);
                let polynomial_bottom = poly_fit(&x_values_bottom, &y_values_bottom, 1);
                let top_len = polynomial_top.len();
                let bottom_len = polynomial_bottom.len();

                let mut poly_points = match direction {
                    PatternDirection::Top => [
                        &polynomial_top[0..top_len],
                        &polynomial_bottom[0..bottom_len],
                    ]
                    .concat(),
                    PatternDirection::Bottom => [
                        &polynomial_top[0..top_len],
                        &polynomial_bottom[0..bottom_len],
                    ]
                    .concat(),
                    PatternDirection::None => [
                        &polynomial_top[0..top_len],
                        &polynomial_bottom[0..bottom_len],
                    ]
                    .concat(),
                };

                poly_points.sort_by(|(id_a, _price_a), (id_b, _price_b)| id_a.cmp(id_b));
                //data_points = poly_points;
                match &pattern_size {
                    PatternSize::Local => self.local_patterns.push(Pattern {
                        pattern_type,
                        target,
                        index,
                        date: to_dbtime(date),
                        direction,
                        active,
                        pattern_size: pattern_size.clone(),
                        data_points,
                    }),
                    PatternSize::Extrema => self.extrema_patterns.push(Pattern {
                        pattern_type,
                        target,
                        index,
                        date: to_dbtime(date),
                        direction,
                        active,
                        pattern_size: pattern_size.clone(),
                        data_points,
                    }),
                };
            }
        }
    }
}

pub fn pattern_active_result(
    data: &DataPoints,
    top: PatternActiveResult,
    bottom: PatternActiveResult,
) -> PatternActive {
    let (top_result, top_id, top_price, top_date) = top;
    let (bottom_result, bottom_id, bottom_price, bottom_date) = bottom;

    //FIXME pattern direction
    if top_result {
        let target = calculate_price_target(&PatternDirection::Top, data);

        PatternActive {
            active: true,
            completed: false,
            status: Status::Default,
            index: top_id,
            price: top_price,
            date: top_date,
            break_direction: PatternDirection::Top,
            target,
        }
    } else if bottom_result {
        let target = calculate_price_target(&PatternDirection::Bottom, data);

        PatternActive {
            active: true,
            completed: false,
            status: Status::Default,
            index: bottom_id,
            date: bottom_date,
            price: bottom_price,
            break_direction: PatternDirection::Bottom,
            target,
        }
    } else {
        non_activated()
    }
}

fn non_activated() -> PatternActive {
    PatternActive {
        active: false,
        completed: false,
        status: Status::Default,
        index: 0,
        date: to_dbtime(Local::now() - Duration::days(10000)),
        price: 0.,
        break_direction: PatternDirection::None,
        target: 0.,
    }
}
