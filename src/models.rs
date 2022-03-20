use std::collections::HashMap;

use chrono::DateTime;
use chrono::Local;
use serde::{Deserialize, Serialize};
use ta::indicators::ExponentialMovingAverage;
use ta::indicators::RelativeStrengthIndex;
use ta::indicators::SlowStochastic;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HttpMethod {
    Post,
    Put,
    Get,
    Patch,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompactInstrument2 {
    pub symbol: String,
    pub time_frame: String,
    pub current_price: f64,
    pub current_candle: String,
    pub updated: String,
    pub patterns: Patterns,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CompactInstrument {
    pub symbol: String,
    pub time_frame: TimeFrameType,
    pub current_price: f64,
    pub current_candle: CandleType,
    pub updated: String,
    pub patterns: Patterns,
    pub indicators: CompactIndicators,
    pub divergences: CompactDivergences,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Instrument {
    pub symbol: String,
    pub time_frame: TimeFrameType,
    pub current_price: f64,
    pub min_price: f64,
    pub max_price: f64,
    pub current_candle: CandleType,
    #[serde(skip_deserializing)]
    pub updated: String,
    pub data: Vec<Candle>,
    pub peaks: Peaks,
    pub patterns: Patterns,
    pub indicators: Indicators,
    pub divergences: Divergences,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum IndicatorStatus {
    Bearish,
    BearishBellowZero,
    Bullish,
    BullishOverZero,
    Oversold,
    Overbought,
    Default,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum IndicatorType {
    Macd,
    Stoch,
    Rsi,
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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CompactIndicators {
    pub macd: CompactIndicator,
    pub stoch: CompactIndicator,
    pub rsi: CompactIndicator,
    pub ema_a: CompactIndicator,
    pub ema_b: CompactIndicator,
    pub ema_c: CompactIndicator,
    pub ema_d: CompactIndicator,
    pub ema_e: CompactIndicator,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Stoch {
    pub stoch: SlowStochastic,
    pub ema: ExponentialMovingAverage,
    pub data_a: Vec<f64>,
    pub data_b: Vec<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompactStoch {
    stoch: SlowStochastic,
    ema: ExponentialMovingAverage,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Ema {
    pub ema: ExponentialMovingAverage,
    pub data_a: Vec<f64>,
    #[serde(skip_deserializing)]
    pub data_b: Vec<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompactEma {
    ema: ExponentialMovingAverage,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompactIndicator2 {
    pub data_a: Vec<f64>,
    pub data_b: Vec<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CompactIndicator {
    pub current_a: f64,
    pub current_b: f64,
    pub prev_a: f64,
    pub prev_b: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Rsi {
    pub rsi: RelativeStrengthIndex,
    pub data_a: Vec<f64>,
    #[serde(skip_deserializing)]
    pub data_b: Vec<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompactRsi {
    rsi: RelativeStrengthIndex,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Macd {
    pub ema26: ExponentialMovingAverage,
    pub ema12: ExponentialMovingAverage,
    ema9: ExponentialMovingAverage,
    pub data_a: Vec<f64>,
    pub data_b: Vec<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompactMacd {
    ema26: ExponentialMovingAverage,
    ema12: ExponentialMovingAverage,
    ema9: ExponentialMovingAverage,
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

impl Peaks {
    pub fn new() -> Self {
        Peaks {
            highs: vec![],
            close: vec![],
            lows: vec![],
            local_maxima: vec![],
            local_minima: vec![],
            smooth_highs: vec![],
            smooth_lows: vec![],
            smooth_close: vec![],
            extrema_maxima: vec![],
            extrema_minima: vec![],
        }
    }
}

impl Default for Peaks {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
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
    pub candle_type: CandleType,
    pub date: DateTime<Local>,
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
    pub volume: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
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
    pub date: DateTime<Local>,
    pub timestamp: i64,
    pub price: f64,
    pub target: f64,
    pub change: f64,
    pub break_direction: PatternDirection,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Pattern {
    pub pattern_type: PatternType,
    pub pattern_size: PatternSize,
    pub data_points: DataPoints,
    pub direction: PatternDirection,
    pub active: PatternActive,
    pub change: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompactPattern {
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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum DivergenceType {
    Bullish,
    Bearish,
    None,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Divergence {
    pub data: DataPoints,
    pub indicator: IndicatorType,
    pub divergence_type: DivergenceType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Divergences {
    pub divergences: Vec<Divergence>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CompactDivergences {
    pub divergences: Vec<CompactDivergence>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CompactDivergence {
    pub indicator: IndicatorType,
    pub divergence_type: DivergenceType,
}
