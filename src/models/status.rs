use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Status {
    Bullish,
    Bearish,
    Neutral,
    ChangeUp,
    ChangeDown,
    Default,
}

impl Status {
    pub fn new() -> Self {
        Status::Neutral
    }
}

impl Default for Status {
    fn default() -> Self {
        Self::new()
    }
}
