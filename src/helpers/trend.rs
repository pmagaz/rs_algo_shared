use std::env;

use crate::{models::trade::TradeDirection, scanner::candle::Candle};

// pub fn trading_direction(data: &[f64]) -> TradeDirection {
//     let window_size = 3;
//     let threshold = 4;

//     let mut trends = Vec::new();
//     let mut count = 0;
//     let mut trend = None;
//     for window in data.windows(window_size) {
//         let start = window[0];
//         let end = window[window_size - 1];

//         if end > start {
//             trend = Some(TradeDirection::Long);
//         } else if end < start {
//             trend = Some(TradeDirection::Short);
//         }

//         count += 1;
//         if count == window_size {
//             trends.push(trend.unwrap());
//             count = 0;
//             trend = None;
//         }
//     }

//     let inc_count = trends
//         .iter()
//         .filter(|t| **t == TradeDirection::Long)
//         .count();

//     let dec_count = trends
//         .iter()
//         .filter(|t| **t == TradeDirection::Short)
//         .count();

//     if inc_count > dec_count && dec_count <= threshold {
//         TradeDirection::Long
//     } else if dec_count > inc_count && inc_count <= threshold {
//         TradeDirection::Short
//     } else {
//         TradeDirection::None
//     }
// }

pub fn trading_direction(data: &[f64]) -> TradeDirection {
    let n = data.len();
    let slope = (data[n - 1] - data[0]) / (n as f64 - 1.0);

    if slope >= -5. {
        TradeDirection::Long
    } else {
        TradeDirection::Short
    }
}

pub fn determine_trend(candles: &[Candle]) -> TradeDirection {
    let back_candles = match env::var("TREND_BACK_CANDLES").unwrap().parse::<usize>() {
        Ok(value) => value,
        Err(_) => return TradeDirection::None, // Invalid TREND_BACK_CANDLES value
    };

    if candles.len() < back_candles {
        return TradeDirection::None; // Not enough data to determine the trend
    }

    let start_index = candles.len().saturating_sub(back_candles);
    let last_back_candles = &candles[start_index..];

    let highest_high = last_back_candles
        .iter()
        .map(|candle| candle.high)
        .max_by(|a, b| a.partial_cmp(b).unwrap());

    let lowest_low = last_back_candles
        .iter()
        .map(|candle| candle.low)
        .min_by(|a, b| a.partial_cmp(b).unwrap());

    match (highest_high, lowest_low) {
        (Some(highest), Some(lowest)) => {
            if highest > lowest {
                TradeDirection::Long
            } else {
                TradeDirection::Short
            }
        }
        _ => TradeDirection::None, // Unable to determine the trend
    }
}
