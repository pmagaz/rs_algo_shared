use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum StrategyType {
    OnlyLong,
    OnlyShort,
    LongShort,
    LongShortMultiTF,
    OnlyLongMultiTF,
    OnlyShortMultiTF,
}
