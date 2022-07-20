use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Market {
    Stock,
    Forex,
    Crypto,
    Default,
}
