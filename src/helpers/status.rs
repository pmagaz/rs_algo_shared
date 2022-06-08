use crate::helpers::date::*;
use crate::models::indicator::*;
use crate::models::instrument::*;
use crate::models::pattern::*;
use crate::models::status::*;
use round::round;

pub fn get_rsi_status(indicator: CompactIndicator) -> Status {
    match indicator {
        _x if indicator.current_a > 20. && indicator.current_a <= 40. => Status::Bullish,
        _x if indicator.current_a > 40. && indicator.current_a < 70. => Status::Neutral,
        _x if indicator.current_a >= 70. => Status::Bearish,
        _ => Status::Default,
    }
}

pub fn get_bb_status(indicator: CompactIndicator, instrument: &CompactInstrument) -> Status {
    match indicator {
        _x if instrument.current_price <= indicator.current_b
            && instrument.prev_price >= indicator.prev_b =>
        {
            Status::Bullish
        }
        _x if instrument.current_price >= indicator.current_a
            && instrument.prev_price <= indicator.prev_a =>
        {
            Status::Bearish
        }
        _x if (instrument.current_price >= indicator.current_c
            && instrument.prev_price <= indicator.prev_c)
            || (instrument.current_price <= indicator.current_c
                && instrument.prev_price >= indicator.prev_c) =>
        {
            Status::Neutral
        }
        _ => Status::Default,
    }
}

pub fn get_stoch_status(indicator: CompactIndicator) -> Status {
    match indicator {
        _x if indicator.current_a > indicator.current_b
            && indicator.current_a > 20.
            && indicator.current_a < 30. =>
        {
            Status::Bullish
        }
        _x if indicator.current_a < indicator.current_b => Status::Bearish,
        _x if indicator.current_a >= 70. => Status::Bearish,
        _x if indicator.current_a > 40. && indicator.current_a < 60. => Status::Default,
        _x if indicator.current_a > 60. && indicator.current_a < 70. => Status::Neutral,
        _ => Status::Neutral,
    }
}

pub fn get_macd_status(indicator: CompactIndicator) -> Status {
    match indicator {
        _x if round(indicator.current_a, 2) > round(indicator.current_b, 2)
            && indicator.current_a > 0. =>
        {
            Status::Bullish
        }
        _x if round(indicator.clone().current_a, 2) < round(indicator.clone().current_b, 2)
            && round(indicator.current_a, 2) < 0. =>
        {
            Status::Bearish
        }
        _x if round(indicator.current_a, 1) >= round(indicator.current_b, 1)
            && round(indicator.current_a, 1) <= 0. =>
        {
            Status::Neutral
        }
        //_x if indicator.current_a < indicator.current_b => Status::Bearish,
        _ => Status::Default,
    }
}

pub fn get_profit_factor_status(profit_factor: f64) -> Status {
    match profit_factor {
        _x if profit_factor < 1.4 => Status::Bearish,
        _x if profit_factor >= 1.4 && profit_factor < 1.75 => Status::Neutral,
        _x if profit_factor >= 1.75 => Status::Bullish,
        _ => Status::Neutral,
    }
}

pub fn get_pattern_status(
    pattern: Option<&Pattern>,
    second_last_pattern_type: &PatternType,
    max_days: i64,
) -> Status {
    let max_pattern_date = to_dbtime(Local::now() - Duration::days(max_days));

    let max_activated_date = to_dbtime(Local::now() - Duration::days(max_days));

    let super_date = to_dbtime(Local::now() - Duration::days(35));

    let fake_date = to_dbtime(Local::now() - Duration::days(1000));

    match pattern {
        Some(_pat) => {
            let _pattern_type = match pattern {
                Some(pat) => pat.pattern_type.clone(),
                None => PatternType::None,
            };
            let pattern_active = match pattern {
                Some(pat) => pat.active.active,
                None => false,
            };

            let pattern_date = match pattern {
                Some(val) => val.date,
                None => fake_date,
            };

            let pattern_active_date = match pattern {
                Some(val) => val.active.date,
                None => fake_date,
            };

            let pattern_type = match pattern {
                Some(val) => val.pattern_type.clone(),
                None => PatternType::None,
            };

            match pattern {
                _x if pattern_type == PatternType::ChannelUp
                    || pattern_type == PatternType::HigherHighsHigherLows
                    || pattern_type == PatternType::TriangleUp =>
                {
                    Status::Bullish
                }
                _x if pattern_type == PatternType::ChannelDown
                    || pattern_type == PatternType::LowerHighsLowerLows
                    || pattern_type == PatternType::TriangleDown =>
                {
                    Status::Bearish
                }
                _x if pattern_type == PatternType::Broadening
                    || pattern_type == PatternType::Rectangle
                    || pattern_type == PatternType::TriangleSym =>
                {
                    Status::Neutral
                }

                _x if pattern_active && pattern_active_date > max_activated_date => Status::Bullish,
                _x if (second_last_pattern_type == &PatternType::ChannelDown
                    || second_last_pattern_type == &PatternType::LowerHighsLowerLows)
                    && &pattern_type != second_last_pattern_type =>
                {
                    Status::Bullish
                }
                _x if (second_last_pattern_type == &PatternType::ChannelDown
                    || second_last_pattern_type == &PatternType::LowerHighsLowerLows)
                    && &pattern_type == second_last_pattern_type =>
                {
                    Status::Bearish
                }

                _x if pattern_date > max_pattern_date => Status::Neutral,
                _x if pattern_date > super_date => Status::Neutral,
                _x if pattern_type == PatternType::None => Status::Default,
                _ => Status::Default,
            }
        }
        None => Status::Default,
    }
}

pub fn get_profitable_trades_status(profitable_trades: f64) -> Status {
    match profitable_trades {
        _x if profitable_trades <= 40. => Status::Bearish,
        _x if profitable_trades > 40. && profitable_trades <= 50. => Status::Neutral,
        _x if profitable_trades > 50. => Status::Bullish,
        _ => Status::Neutral,
    }
}

pub fn get_profit_status(profit: f64, profitable_trades: f64) -> Status {
    match profit {
        _x if profit <= 10. => Status::Bearish,
        _x if profit > 10. && profitable_trades < 12. => Status::Neutral,
        _x if profit >= 15. => Status::Bullish,
        _ => Status::Neutral,
    }
}

pub fn get_max_drawdown_status(max_drawdown: f64) -> Status {
    match max_drawdown {
        _x if max_drawdown >= 20. => Status::Bearish,
        _x if max_drawdown > 15. && max_drawdown < 20. => Status::Neutral,
        _x if max_drawdown <= 15. => Status::Bullish,
        _ => Status::Neutral,
    }
}

pub fn get_avg_won_status(won_per_trade: f64) -> Status {
    match won_per_trade {
        _x if won_per_trade > 15. => Status::Bullish,
        _x if won_per_trade > 10. && won_per_trade < 15. => Status::Neutral,
        _x if won_per_trade <= 10. => Status::Bullish,
        _ => Status::Neutral,
    }
}

pub fn get_avg_lost_per_trade(lost_per_trade: f64) -> Status {
    match lost_per_trade {
        _x if lost_per_trade > -5. => Status::Bullish,
        _x if lost_per_trade < -5. && lost_per_trade > -10. => Status::Neutral,
        _x if lost_per_trade <= -10. => Status::Bearish,
        _ => Status::Neutral,
    }
}
