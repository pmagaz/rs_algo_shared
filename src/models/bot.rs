use crate::helpers::date::*;
use crate::helpers::uuid::Uuid;
use crate::models::market::*;
use crate::models::strategy::*;
use crate::models::time_frame::*;
use crate::models::trade::*;
use crate::scanner::instrument::{HigherTMInstrument, Instrument};
use serde::{Deserialize, Deserializer, Serialize};

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct BotData {
    _id: Uuid,
    symbol: String,
    market: Market,
    strategy_name: String,
    strategy_type: StrategyType,
    time_frame: TimeFrameType,
    higher_time_frame: TimeFrameType,
    date_start: DbDateTime,
    last_update: DbDateTime,
    instrument: Instrument,
    higher_tf_instrument: HigherTMInstrument,
    trades_in: Vec<TradeIn>,
    trades_out: Vec<TradeOut>,
    strategy_stats: StrategyStats,
}

fn null_to_default<'de, D, T>(de: D) -> Result<T, D::Error>
where
    D: Deserializer<'de>,
    T: Default + Deserialize<'de>,
{
    let key = Option::<T>::deserialize(de)?;
    Ok(key.unwrap_or_default())
}

#[derive(Clone, Serialize, Deserialize, Debug, PartialEq)]
pub struct CompactBotData {
    pub _id: Uuid,
    pub symbol: String,
    pub market: Market,
    pub strategy_name: String,
    pub strategy_type: StrategyType,
    pub time_frame: TimeFrameType,
    pub higher_time_frame: TimeFrameType,
    pub date_start: DbDateTime,
    pub last_update: DbDateTime,
    pub strategy_stats: StrategyStats,
}

impl BotData {
    pub fn uuid(&self) -> &Uuid {
        &self._id
    }
    pub fn symbol(&self) -> &str {
        &self.symbol
    }
    pub fn instrument(&self) -> &Instrument {
        &self.instrument
    }
    pub fn higher_tf_instrument(&self) -> &HigherTMInstrument {
        &self.higher_tf_instrument
    }
    pub fn date_start(&self) -> &DbDateTime {
        &self.date_start
    }
    pub fn trades_in(&self) -> &Vec<TradeIn> {
        &self.trades_in
    }
    pub fn trades_out(&self) -> &Vec<TradeOut> {
        &self.trades_out
    }
    pub fn strategy_stats(&self) -> &StrategyStats {
        &self.strategy_stats
    }
    pub fn strategy_name(&self) -> &String {
        &self.strategy_name
    }
    pub fn strategy_type(&self) -> &StrategyType {
        &self.strategy_type
    }
}
