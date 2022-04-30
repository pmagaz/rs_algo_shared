use crate::helpers::date::*;

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Clone, Deserialize, PartialEq)]
pub struct WatchInstrument {
    pub symbol: String,
    pub alarm: Alarm,
}

#[derive(Debug, Serialize, Clone, Deserialize, PartialEq)]
pub struct Alarm {
    pub active: bool,
    pub completed: bool,
    pub price: f64,
    pub date: DbDateTime,
    pub condition: AlarmCondition,
}

#[derive(Debug, Serialize, Clone, Deserialize, PartialEq)]
pub enum AlarmCondition {
    CrossOver,
    CrossBellow,
    None,
}

impl std::fmt::Display for WatchInstrument {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl std::fmt::Display for Alarm {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}
