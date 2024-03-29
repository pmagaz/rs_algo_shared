use super::highs_lows::*;
use super::pattern::pattern_active_result;
use crate::helpers::date::*;
use crate::scanner::candle::Candle;
use crate::scanner::pattern::{DataPoints, PatternActive, PatternType};
use crate::scanner::prices::*;

pub fn is_ascendant_top(data: &DataPoints) -> bool {
    is_upperhighs_top(data)
        && is_upperlows_bottom(data)
        //&& points_are_in_slope(data)
        && bands_have_same_slope(data)
        && are_parallel_lines(data)
        && has_minimum_bars(data)
        //// && has_minimum_target(data)
        && data[0].1 > data[1].1 && data[2].1 > data[3].1
}

pub fn is_ascendant_bottom(data: &DataPoints) -> bool {
    is_upperhighs_bottom(data)
        && is_upperlows_top(data)
        //&& points_are_in_slope(data)
        && bands_have_same_slope(data)
        && are_parallel_lines(data)
        && has_minimum_bars(data)
        // && has_minimum_target(data)
        && data[0].1 < data[1].1 && data[2].1 < data[3].1
}

pub fn is_descendant_top(data: &DataPoints) -> bool {
    is_lower_highs_top(data)
        && is_lower_lows_bottom(data)
        //&& points_are_in_slope(data)
        && bands_have_same_slope(data)
       && are_parallel_lines(data)
        && has_minimum_bars(data)
        // && has_minimum_target(data)
        && data[0].1 > data[1].1 && data[2].1 > data[3].1
}

pub fn is_descendant_bottom(data: &DataPoints) -> bool {
    is_lower_highs_bottom(data)
        && is_lower_lows_top(data)
        //&& points_are_in_slope(data)
        && bands_have_same_slope(data)
        && are_parallel_lines(data)
        && has_minimum_bars(data)
        // && has_minimum_target(data)
        && data[0].1 < data[1].1 && data[2].1 < data[3].1
}

pub fn channel_descendant_top_active(
    data: &DataPoints,
    candles: &Vec<Candle>,
    pattern_type: PatternType,
) -> PatternActive {
    pattern_active_result(
        data,
        price_is_upperupper_band_top(data, candles, &pattern_type),
        (false, 0, 0., to_dbtime(Local::now() - Duration::days(1000))),
    )
}

pub fn channel_descendant_bottom_active(
    data: &DataPoints,
    candles: &Vec<Candle>,
    pattern_type: PatternType,
) -> PatternActive {
    pattern_active_result(
        data,
        price_is_upperupper_band_bottom(data, candles, &pattern_type),
        (false, 0, 0., to_dbtime(Local::now() - Duration::days(1000))),
    )
}

pub fn channel_ascendant_top_active(
    data: &DataPoints,
    candles: &Vec<Candle>,
    pattern_type: PatternType,
) -> PatternActive {
    pattern_active_result(
        data,
        (false, 0, 0., to_dbtime(Local::now() - Duration::days(1000))),
        price_is_lower_low_band_bottom(data, candles, &pattern_type),
    )
}

pub fn channel_ascendant_bottom_active(
    data: &DataPoints,
    candles: &Vec<Candle>,
    pattern_type: PatternType,
) -> PatternActive {
    pattern_active_result(
        data,
        (false, 0, 0., to_dbtime(Local::now() - Duration::days(1000))),
        price_is_lower_low_band_top(data, candles, &pattern_type),
    )
}
