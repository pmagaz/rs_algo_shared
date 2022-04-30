use crate::helpers::date::*;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum HorizontalLevelType {
    Resistance,
    Support,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct HorizontalLevel {
    pub price: f64,
    pub occurrences: usize,
    pub date: DbDateTime,
    pub level_type: HorizontalLevelType,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct HorizontalLevels {
    pub highs: Vec<HorizontalLevel>,
    pub lows: Vec<HorizontalLevel>,
}
