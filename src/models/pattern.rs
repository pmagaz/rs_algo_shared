use crate::helpers::date::*;
use crate::models::status::Status;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum PatternDirection {
    Top,
    Bottom,
    None,
}

impl std::fmt::Display for PatternDirection {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

type Point = (usize, f64);
pub type DataPoints = Vec<Point>;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum PatternType {
    Triangle,
    TriangleSym,
    TriangleDown,
    TriangleUp,
    Rectangle,
    ChannelUp,
    ChannelDown,
    Broadening,
    DoubleTop,
    DoubleBottom,
    HeadShoulders,
    HigherHighsHigherLows,
    LowerHighsLowerLows,
    None,
}

impl std::fmt::Display for PatternType {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
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
    pub date: DbDateTime,
    pub price: f64,
    pub status: Status,
    pub break_direction: PatternDirection,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Pattern {
    pub index: usize,
    pub date: DbDateTime,
    pub pattern_type: PatternType,
    pub pattern_size: PatternSize,
    pub data_points: DataPoints,
    pub direction: PatternDirection,
    pub active: PatternActive,
    pub target: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompactPattern {
    pub index: usize,
    pub date: DbDateTime,
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
