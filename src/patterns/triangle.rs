use super::highs_lows::*;
use super::pattern::pattern_active_result;
use crate::scanner::candle::Candle;
use crate::scanner::prices::*;

use crate::scanner::pattern::{DataPoints, PatternActive, PatternType};

pub fn is_ascendant_top(data: &DataPoints) -> bool {
    upper_band_is_equal_top(data)
        && is_upperlows_bottom(data)
        && is_valid_triangle(data)
        //&& points_are_in_slope(data)
        && has_minimum_bars(data)
        // && has_minimum_target(data)
        && data[0].1 > data[1].1 && data[2].1 > data[3].1
}

pub fn is_ascendant_bottom(data: &DataPoints) -> bool {
    is_upperlows_top(data)
        && upper_band_is_equal_bottom(data)
        && is_valid_triangle(data)
        //&& points_are_in_slope(data)
        && has_minimum_bars(data)
        // && has_minimum_target(data)
        && data[0].1 < data[1].1 && data[2].1 < data[3].1
}

pub fn is_descendant_top(data: &DataPoints) -> bool {
    is_lower_highs_top(data)
        && lower_band_is_equal_bottom(data)
        && is_valid_triangle(data)
        //&& points_are_in_slope(data)
        && has_minimum_bars(data)
        // && has_minimum_target(data)
        && data[0].1 > data[1].1 && data[2].1 > data[3].1
}

pub fn is_descendant_bottom(data: &DataPoints) -> bool {
    lower_band_is_equal_top(data)
        && is_lower_highs_bottom(data)
        && is_valid_triangle(data)
        //&& points_are_in_slope(data)
        && has_minimum_bars(data)
        // && has_minimum_target(data)
        && data[0].1 < data[1].1 && data[2].1 < data[3].1
}

pub fn is_symmetrical_top(data: &DataPoints) -> bool {
    is_lower_highs_top(data)
        && is_upperlows_bottom(data)
        //&& is_valid_triangle(data)
        //&& points_are_in_slope(data)
        && bands_have_same_slope(data)
        && has_minimum_bars(data)
        // && has_minimum_target(data)
        && data[0].1 > data[1].1 && data[2].1 > data[3].1
}

pub fn is_symmetrical_bottom(data: &DataPoints) -> bool {
    is_lower_highs_bottom(data)
        && is_upperlows_top(data)
        //&& is_valid_triangle(data)
        //&& points_are_in_slope(data)
        && bands_have_same_slope(data)
        && has_minimum_bars(data)
        // && has_minimum_target(data)
        && data[0].1 < data[1].1 && data[2].1 < data[3].1
}

pub fn ascendant_top_active(
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

pub fn ascendant_bottom_active(
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

pub fn descendant_top_active(
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

pub fn descendant_bottom_active(
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

pub fn symetrical_top_active(
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

pub fn symetrical_bottom_active(
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
