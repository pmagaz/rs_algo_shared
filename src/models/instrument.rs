use crate::helpers::date::*;
use crate::models::candle::{Candle, CandleType};
use crate::models::divergence::{CompactDivergences, Divergences};
use crate::models::horizontal_level::HorizontalLevels;
use crate::models::indicator::{CompactIndicators, Indicators};
use crate::models::pattern::Patterns;
use crate::models::peak::Peaks;
use crate::models::time_frame::TimeFrameType;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CompactInstrument {
    pub symbol: String,
    pub time_frame: TimeFrameType,
    pub current_price: f64,
    pub prev_price: f64,
    pub avg_volume: f64,
    pub current_candle: CandleType,
    pub prev_candle: CandleType,
    pub date: DbDateTime,
    pub patterns: Patterns,
    pub horizontal_levels: HorizontalLevels,
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
    pub avg_volume: f64,
    pub current_candle: CandleType,
    pub date: DbDateTime,
    pub data: Vec<Candle>,
    pub peaks: Peaks,
    pub patterns: Patterns,
    pub horizontal_levels: HorizontalLevels,
    pub indicators: Indicators,
    pub divergences: Divergences,
}
