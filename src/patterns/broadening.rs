use super::highs_lows::*;
use super::pattern::pattern_active_result;
use crate::scanner::candle::Candle;
use crate::scanner::prices::*;

use crate::scanner::pattern::{DataPoints, PatternActive, PatternType};

pub fn is_top(data: &DataPoints) -> bool {
    is_upperhighs_top(data)
        && is_lower_lows_bottom(data)
        //&& points_are_in_slope(data)
        && bands_have_same_slope(data)
        && is_valid_broadening(data)
        && has_minimum_bars(data)
        // && has_minimum_target(data)
        && data[0].1 > data[1].1 && data[2].1 > data[3].1
}

pub fn is_bottom(data: &DataPoints) -> bool {
    is_upperhighs_bottom(data)
        && is_lower_lows_top(data)
        //&& points_are_in_slope(data)
        && bands_have_same_slope(data)
        && is_valid_broadening(data)
        && has_minimum_bars(data)
        // && has_minimum_target(data)
        && data[1].1 > data[0].1 && data[0].1 < data[3].1
}

pub fn broadening_top_active(
    data: &DataPoints,
    candles: &Vec<Candle>,
    pattern_type: PatternType,
) -> PatternActive {
    pattern_active_result(
        data,
        price_is_upperupper_band_top(data, candles, &pattern_type),
        price_is_lower_low_band_bottom(data, candles, &pattern_type),
    )
}

pub fn broadening_bottom_active(
    data: &DataPoints,
    candles: &Vec<Candle>,
    pattern_type: PatternType,
) -> PatternActive {
    pattern_active_result(
        data,
        price_is_upperupper_band_bottom(data, candles, &pattern_type),
        price_is_lower_low_band_top(data, candles, &pattern_type),
    )
}
