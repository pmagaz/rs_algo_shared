use std::collections::HashMap;

use chrono::DateTime;
use chrono::Local;
use serde::{Deserialize, Serialize};
use ta::indicators::ExponentialMovingAverage;
use ta::indicators::RelativeStrengthIndex;
use ta::indicators::SlowStochastic;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Instrument {
    pub symbol: String,
    time_frame: TimeFrameType,
    current_price: f64,
    min_price: f64,
    max_price: f64,
    current_candle: CandleType,
    #[serde(skip_deserializing)]
    pub updated: String,
    data: Vec<Candle>,
    peaks: Peaks,
    //horizontal_levels: HorizontalLevels,
    patterns: Patterns,
    indicators: Indicators,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Indicators {
    pub macd: Macd,
    pub stoch: Stoch,
    pub rsi: Rsi,
    pub ema_a: Ema,
    pub ema_b: Ema,
    pub ema_c: Ema,
    pub ema_d: Ema,
    pub ema_e: Ema,
}

#[derive(Debug, Clone, Serialize, Deserialize)]

pub struct Stoch {
    stoch: SlowStochastic,
    ema: ExponentialMovingAverage,
    data_a: Vec<f64>,
    data_b: Vec<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Ema {
    ema: ExponentialMovingAverage,
    data_a: Vec<f64>,
    data_b: Vec<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Rsi {
    rsi: RelativeStrengthIndex,
    data_a: Vec<f64>,
    data_b: Vec<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Macd {
    ema26: ExponentialMovingAverage,
    ema12: ExponentialMovingAverage,
    ema9: ExponentialMovingAverage,
    data_a: Vec<f64>,
    data_b: Vec<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HorizontalLevelType {
    Resistance,
    Support,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HorizontalLevel {
    price: f64,
    min_value: f64,
    max_value: f64,
    level_type: HorizontalLevelType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HorizontalLevels {
    horizontal_levels: HashMap<usize, HorizontalLevel>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Peaks {
    pub highs: Vec<f64>,
    pub close: Vec<f64>,
    pub lows: Vec<f64>,
    pub local_maxima: Vec<(usize, f64)>,
    pub local_minima: Vec<(usize, f64)>,
    pub smooth_highs: Vec<(usize, f64)>,
    pub smooth_lows: Vec<(usize, f64)>,
    pub smooth_close: Vec<(usize, f64)>,
    pub extrema_maxima: Vec<(usize, f64)>,
    pub extrema_minima: Vec<(usize, f64)>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CandleType {
    Default,
    Doji,
    Karakasa,
    BearishKarakasa,
    Marubozu,
    BearishMarubozu,
    Harami,
    BearishHarami,
    Engulfing,
    BearishEngulfing,
    HangingMan,
    BullishCrows,
    BearishCrows,
    BullishGap,
    BearishGap,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Candle {
    candle_type: CandleType,
    date: DateTime<Local>,
    open: f64,
    high: f64,
    low: f64,
    close: f64,
    volume: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PatternDirection {
    Top,
    Bottom,
    None,
}

type Point = (usize, f64);
pub type DataPoints = Vec<Point>;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum PatternType {
    Triangle,
    TriangleSymmetrical,
    TriangleDescendant,
    TriangleSymmetricalTop,
    TriangleAscendant,
    Rectangle,
    ChannelUp,
    ChannelDown,
    Broadening,
    DoubleTop,
    DoubleBottom,
    None,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PatternSize {
    Local,
    Extrema,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatternActive {
    pub active: bool,
    pub completed: bool,
    pub index: usize,
    pub target: f64,
    pub change: f64,
    pub break_direction: PatternDirection,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Pattern {
    pub pattern_type: PatternType,
    pub pattern_size: PatternSize,
    pub data_points: DataPoints,
    pub direction: PatternDirection,
    pub active: PatternActive,
    pub change: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Patterns {
    pub local_patterns: Vec<Pattern>,
    pub extrema_patterns: Vec<Pattern>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TimeFrameType {
    H1,
    H4,
    D,
    W,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeFrame {}

impl TimeFrame {
    pub fn new(time_frame: &str) -> TimeFrameType {
        match time_frame {
            "H1" => TimeFrameType::H1,
            "H4" => TimeFrameType::H4,
            "D" => TimeFrameType::D,
            "W" => TimeFrameType::W,
            &_ => TimeFrameType::H1,
        }
    }
}

impl TimeFrameType {
    pub fn value(&self) -> usize {
        match *self {
            TimeFrameType::H1 => 60,
            TimeFrameType::H4 => 240,
            TimeFrameType::D => 1440,
            TimeFrameType::W => 10080,
        }
    }
}
