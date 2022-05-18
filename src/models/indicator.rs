use crate::error::Result;
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

pub trait Indicator {
    fn new() -> Result<Self>
    where
        Self: Sized;
    fn name(&self) -> &str;
    fn next(&mut self, value: f64) -> Result<()>;
    fn get_data_a(&self) -> &Vec<f64>;
    fn get_current_a(&self) -> &f64;
    fn get_current_b(&self) -> &f64;
    fn get_data_b(&self) -> &Vec<f64>;
    fn get_current_c(&self) -> &f64;
    fn get_data_c(&self) -> &Vec<f64>;
}

pub trait CompactIndicator2 {
    fn new() -> Result<Self>
    where
        Self: Sized;
    fn name(&self) -> &str;
    fn get_current_a(&self) -> &f64;
    fn get_current_b(&self) -> &f64;
    fn get_prev_a(&self) -> &Vec<f64>;
    fn get_prev_b(&self) -> &f64;
    fn get_status(&self) -> Status;
}

pub type Indicators2 = Vec<Box<dyn Indicator>>;
pub type CompactIndicators2 = Vec<Box<dyn CompactIndicator2>>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Indicators {
    pub macd: Macd,
    pub stoch: Stoch,
    pub atr: Atr,
    pub rsi: Rsi,
    pub bb: BollingerB,
    pub ema_a: Ema,
    pub ema_b: Ema,
    pub ema_c: Ema,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CompactIndicators {
    pub macd: CompactIndicator,
    pub stoch: CompactIndicator,
    pub atr: CompactIndicator,
    pub sd: CompactIndicator,
    pub bb: CompactIndicator,
    //pub kc: CompactIndicator,
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
    pub bb: BollingerBands,
    pub data_a: Vec<f64>,
    pub data_b: Vec<f64>,
    pub data_c: Vec<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompactStoch {
    pub stoch: SlowStochastic,
    pub ema: ExponentialMovingAverage,
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
pub struct CompactEma {
    ema: ExponentialMovingAverage,
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
