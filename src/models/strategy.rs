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

pub fn from_str(strategy: &str) -> StrategyType {
    match strategy {
        "OnlyLong" => StrategyType::OnlyLong,
        "OnlyShort" => StrategyType::OnlyShort,
        "LongShort" => StrategyType::LongShort,
        "LongShortMultiTF" => StrategyType::LongShortMultiTF,
        "OnlyLongMultiTF" => StrategyType::OnlyLongMultiTF,
        "OnlyShortMultiTF" => StrategyType::OnlyShortMultiTF,
        _ => StrategyType::OnlyLong,
    }
}
