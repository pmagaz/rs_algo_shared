use crate::scanner::candle::{Candle, CandleType};

pub fn is_bullish_fair_value_gap(candles: &[Candle]) -> bool {
    let body_wick_ratio = 0.80;
    let min_gap_percentage = 3.;

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
        true
    } else {
        false
    }
}

pub fn is_bullish_reversal(candles: &[Candle]) -> bool {
    let min_diff_size = 0.1;

    let left_candle = &candles[0];
    let middle_candle = &candles[1];
    let right_candle = &candles[2];
    // log::info!(
    //     "aaaa: {} {} {}",
    //     left_candle.date,
    //     middle_candle.date,
    //     right_candle.date,
    // );
    let diff_size = (left_candle.close - right_candle.close).abs();
    let diff_size_percentage = (diff_size / right_candle.close) * 100.0;

    if left_candle.candle_type() == &CandleType::Karakasa
        && (middle_candle.close() > left_candle.close()
            && right_candle.close > middle_candle.close()
            && diff_size_percentage > min_diff_size)
    {
        // log::info!(
        //     "Reversal detected: {} {}",
        //     left_candle.date,
        //     diff_size_percentage,
        // );
        true
    } else {
        false
    }
}
