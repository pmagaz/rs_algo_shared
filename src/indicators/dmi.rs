use crate::error::Result;
use serde::{Deserialize, Serialize};
use ta::Next;
#[derive(Debug, Default, Serialize, Clone)]
pub struct Dmi {
    period: usize,
    previous_high: Option<f64>,
    previous_low: Option<f64>,
    tr_values: Vec<f64>,
    plus_dm_values: Vec<f64>,
    minus_dm_values: Vec<f64>,
    plus_di_values: Vec<f64>,
    minus_di_values: Vec<f64>,
    dx_values: Vec<f64>,
    adx_values: Vec<f64>,
}

impl Dmi {
    pub fn new(period: usize) -> Self {
        Dmi {
            period,
            previous_high: None,
            previous_low: None,
            tr_values: Vec::new(),
            plus_dm_values: Vec::new(),
            minus_dm_values: Vec::new(),
            plus_di_values: Vec::new(),
            minus_di_values: Vec::new(),
            dx_values: Vec::new(),
            adx_values: Vec::new(),
        }
    }

    pub fn next(&mut self, ohlc: (f64, f64, f64, f64)) -> Result<(f64, f64, f64)> {
        let (open, high, low, close) = ohlc;

        // Calculate True Range (TR)
        let tr = match (self.previous_high, self.previous_low) {
            (Some(previous_high), Some(previous_low)) => {
                let hl = (high - low).abs();
                let hc = (high - previous_high).abs();
                let lc = (previous_low - low).abs();
                hl.max(hc).max(lc)
            }
            _ => high - low,
        };
        self.tr_values.push(tr);

        // Calculate +DM and -DM
        let plus_dm = match (self.previous_high, self.previous_low) {
            (Some(previous_high), Some(previous_low)) => {
                let up_move = high - previous_high;
                let down_move = previous_low - low;
                if up_move > down_move && up_move > 0.0 {
                    up_move
                } else {
                    0.0
                }
            }
            _ => 0.0,
        };
        self.plus_dm_values.push(plus_dm);

        let minus_dm = match (self.previous_high, self.previous_low) {
            (Some(previous_high), Some(previous_low)) => {
                let up_move = high - previous_high;
                let down_move = previous_low - low;
                if down_move > up_move && down_move > 0.0 {
                    down_move
                } else {
                    0.0
                }
            }
            _ => 0.0,
        };
        self.minus_dm_values.push(minus_dm);

        let plus_di = if self.plus_dm_values.len() >= self.period {
            let plus_dm_sum: f64 = self.plus_dm_values.iter().rev().take(self.period).sum();
            let tr_sum: f64 = self.tr_values.iter().rev().take(self.period).sum();
            (plus_dm_sum / tr_sum) * 100.0
        } else {
            0.0
        };
        self.plus_di_values.push(plus_di);

        let minus_di = if self.minus_dm_values.len() >= self.period {
            let minus_dm_sum: f64 = self.minus_dm_values.iter().rev().take(self.period).sum();
            let tr_sum: f64 = self.tr_values.iter().rev().take(self.period).sum();
            (minus_dm_sum / tr_sum) * 100.0
        } else {
            0.0
        };
        self.minus_di_values.push(minus_di);

        // Calculate Directional Movement Index (DX)
        if self.plus_di_values.len() >= self.period && self.minus_di_values.len() >= self.period {
            let plus_di = self.plus_di_values.last().copied().unwrap_or(0.0);
            let minus_di = self.minus_di_values.last().copied().unwrap_or(0.0);
            let dx = ((plus_di - minus_di).abs() / (plus_di + minus_di)) * 100.0;
            self.dx_values.push(dx);

            let adx = if self.dx_values.len() >= self.period {
                let dx_sum: f64 = self.dx_values.iter().rev().take(self.period).sum();
                dx_sum / self.period as f64
            } else {
                0.0
            };
            self.adx_values.push(adx);
        }

        // Update previous values
        self.previous_high = Some(high);
        self.previous_low = Some(low);

        // Return ADX, +DI, and -DI
        let adx = self.adx_values.last().copied().unwrap_or(0.0);
        let plus_di = self.plus_di_values.last().copied().unwrap_or(0.0);
        let minus_di = self.minus_di_values.last().copied().unwrap_or(0.0);

        Ok((adx, plus_di, minus_di))
    }

    pub fn reset(&mut self) {
        self.previous_high = None;
        self.previous_low = None;
        self.tr_values.clear();
        self.plus_dm_values.clear();
        self.minus_dm_values.clear();
        self.plus_di_values.clear();
        self.minus_di_values.clear();
        self.dx_values.clear();
        self.adx_values.clear();
    }
}
