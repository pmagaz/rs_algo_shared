use crate::helpers::date::*;
use crate::helpers::uuid::Uuid;
use crate::models::market::*;
use crate::models::strategy::*;
use crate::models::time_frame::*;
use crate::models::trade::*;
use crate::scanner::instrument::{HigherTMInstrument, Instrument};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
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

impl BotData {
    pub fn uuid(&self) -> &Uuid {
        &self._id
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
}
