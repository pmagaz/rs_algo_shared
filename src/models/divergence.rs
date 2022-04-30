use crate::helpers::date::*;
use crate::models::indicator::IndicatorType;
use crate::models::pattern::DataPoints;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum DivergenceType {
    Bullish,
    Bearish,
    None,
}

impl std::fmt::Display for DivergenceType {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Divergence {
    pub data: DataPoints,
    pub date: DbDateTime,
    pub indicator: IndicatorType,
    pub divergence_type: DivergenceType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Divergences {
    pub data: Vec<Divergence>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CompactDivergences {
    pub data: Vec<CompactDivergence>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CompactDivergence {
    pub indicator: IndicatorType,
    pub date: DbDateTime,
    pub divergence_type: DivergenceType,
}
