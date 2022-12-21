use super::highs_lows::*;
use super::pattern::pattern_active_result;
use crate::helpers::comp::*;
use crate::scanner::candle::Candle;
use crate::scanner::prices::*;

use crate::scanner::pattern::{DataPoints, PatternActive, PatternType};

pub fn is_hs(data: &DataPoints) -> bool {
    data[0].1 > data[1].1
        && data[2].1 > data[1].1
        && data[2].1 > data[4].1
        && (data[0].1 - data[4].1).abs() <= 0.03 * average_f64(&[data[0].1, data[4].1].to_vec())
        && (data[1].1 - data[3].1).abs() <= 0.03 * average_f64(&[data[0].1, data[4].1].to_vec()) && has_minimum_bars(data)
}

pub fn is_inverse(data: &DataPoints) -> bool {
    data[0].1 < data[1].1
        && data[2].1 < data[1].1
        && data[2].1 < data[4].1
        && (data[0].1 - data[4].1).abs() <= 0.03 * average_f64(&[data[0].1, data[4].1].to_vec())
        && (data[1].1 - data[3].1).abs() <= 0.03 * average_f64(&[data[0].1, data[4].1].to_vec()) && has_minimum_bars(data)
}

pub fn hs_active(
    data: &DataPoints,
    candles: &Vec<Candle>,
    pattern_type: PatternType,
) -> PatternActive {
    pattern_active_result(
        data,
        price_is_upperpeak(data[2], candles, &pattern_type),
        price_is_lower_peak(data[2], candles, &pattern_type),
    )
}

// pub fn inverse_active(data: &DataPoints, candles: &Vec<Candle>, pattern_type: PatternType) -> PatternActive {
//     pattern_active_result(
//         &data,
//         price_is_upperpeak(&data, candles, &pattern_type),
//         price_is_lower_low_band_bottom(&data, candles, &pattern_type),
//     )
// }
