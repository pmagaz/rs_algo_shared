use crate::models::status::Status;

use serde::{Deserialize, Serialize};
use ta::indicators::AverageTrueRange;
use ta::indicators::BollingerBands;
use ta::indicators::ExponentialMovingAverage;
use ta::indicators::KeltnerChannel;
use ta::indicators::RelativeStrengthIndex;
use ta::indicators::SlowStochastic;
use ta::indicators::StandardDeviation;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum IndicatorType {
    Macd,
    Stoch,
    Rsi,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Indicators {
    pub macd: Macd,
    pub stoch: Stoch,
    pub atr: Atr,
    pub sd: StandardD,
    pub rsi: Rsi,
    //pub kc: KeltnerC,
    pub bb: BollingerB,
    pub ema_a: Ema,
    pub ema_b: Ema,
    pub ema_c: Ema,
    pub tema_a: Tema,
    pub tema_b: Tema,
    pub tema_c: Tema,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CompactIndicators {
    pub macd: CompactIndicator,
    pub stoch: CompactIndicator,
    pub atr: CompactIndicator,
    pub kc: CompactIndicator,
    pub rsi: CompactIndicator,
    pub ema_a: CompactIndicator,
    pub ema_b: CompactIndicator,
    pub ema_c: CompactIndicator,
    pub tema_a: CompactIndicator,
    pub tema_b: CompactIndicator,
    pub tema_c: CompactIndicator,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Stoch {
    #[serde(skip_deserializing)]
    pub stoch: SlowStochastic,
    #[serde(skip_deserializing)]
    pub ema: ExponentialMovingAverage,
    pub data_a: Vec<f64>,
    pub data_b: Vec<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeltnerC {
    #[serde(skip_deserializing)]
    pub kc: KeltnerChannel,
    pub data_a: Vec<f64>,
    pub data_b: Vec<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BollingerB {
    bb: BollingerBands,
    data_a: Vec<f64>,
    data_b: Vec<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompactStoch {
    stoch: SlowStochastic,
    ema: ExponentialMovingAverage,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Ema {
    #[serde(skip_deserializing)]
    pub ema: ExponentialMovingAverage,
    pub data_a: Vec<f64>,
    #[serde(skip_deserializing)]
    pub data_b: Vec<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Atr {
    #[serde(skip_deserializing)]
    pub atr: AverageTrueRange,
    pub data_a: Vec<f64>,
    #[serde(skip_deserializing)]
    pub data_b: Vec<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StandardD {
    #[serde(skip_deserializing)]
    pub sd: StandardDeviation,
    pub data_a: Vec<f64>,
    #[serde(skip_deserializing)]
    pub data_b: Vec<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompactEma {
    ema: ExponentialMovingAverage,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tema {
    #[serde(skip_deserializing)]
    pub ema1: ExponentialMovingAverage,
    #[serde(skip_deserializing)]
    pub ema2: ExponentialMovingAverage,
    #[serde(skip_deserializing)]
    pub ema3: ExponentialMovingAverage,
    pub data_a: Vec<f64>,
    #[serde(skip_deserializing)]
    pub data_b: Vec<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompactTema {
    tema: ExponentialMovingAverage,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CompactIndicator {
    pub current_a: f64,
    pub current_b: f64,
    pub prev_a: f64,
    pub prev_b: f64,
    pub status: Status,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Rsi {
    #[serde(skip_deserializing)]
    pub rsi: RelativeStrengthIndex,
    pub data_a: Vec<f64>,
    #[serde(skip_deserializing)]
    pub data_b: Vec<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompactRsi {
    rsi: RelativeStrengthIndex,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Macd {
    pub ema26: ExponentialMovingAverage,
    pub ema12: ExponentialMovingAverage,
    ema9: ExponentialMovingAverage,
    pub data_a: Vec<f64>,
    pub data_b: Vec<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompactMacd {
    ema26: ExponentialMovingAverage,
    ema12: ExponentialMovingAverage,
    ema9: ExponentialMovingAverage,
}
