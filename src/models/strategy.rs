use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum StrategyType {
    LongShort,
    OnlyLong,
    OnlyShort,
    LongShortMultiTF,
    OnlyLongMultiTF,
    OnlyShortMultiTF,
}
