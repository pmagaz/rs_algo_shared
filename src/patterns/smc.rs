use crate::helpers::comp::percentage_change;
use crate::scanner::candle::{Candle, CandleType};

pub fn is_bullish_fair_value_gap(candles: &[Candle]) -> bool {
    let body_wick_ratio = 0.65;
    let min_gap_percentage = 2.;

    let left_candle = &candles[0];
    let middle_candle = &candles[1];
    let right_candle = &candles[2];

    let middle_candle_body_size = (middle_candle.open - middle_candle.close).abs();
    let middle_candle_wick_size = (middle_candle.high - middle_candle.low).abs();
    let middle_candle_body_to_wick_ratio = middle_candle_body_size / middle_candle_wick_size;
    //let gap_difference = (left_candle.low - right_candle.high).abs();

    let gap_size = (left_candle.high - right_candle.low).abs();
    let gap_size_percentage = (gap_size / right_candle.low) * 100.0;

    if middle_candle_body_to_wick_ratio >= body_wick_ratio
        && gap_size_percentage >= min_gap_percentage
    {
        // log::info!(
        //     "FVG detected: {} {} {}",
        //     middle_candle.date,
        //     middle_candle_body_to_wick_ratio,
        //     gap_size_percentage,
        // );
        true
    } else {
        false
    }
}
