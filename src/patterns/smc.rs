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
