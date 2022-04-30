use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Peaks {
    pub highs: Vec<f64>,
    pub close: Vec<f64>,
    pub lows: Vec<f64>,
    pub local_maxima: Vec<(usize, f64)>,
    pub local_minima: Vec<(usize, f64)>,
    pub smooth_highs: Vec<(usize, f64)>,
    pub smooth_lows: Vec<(usize, f64)>,
    pub smooth_close: Vec<(usize, f64)>,
    pub extrema_maxima: Vec<(usize, f64)>,
    pub extrema_minima: Vec<(usize, f64)>,
}

impl Peaks {
    pub fn new() -> Self {
        Peaks {
            highs: vec![],
            close: vec![],
            lows: vec![],
            local_maxima: vec![],
            local_minima: vec![],
            smooth_highs: vec![],
            smooth_lows: vec![],
            smooth_close: vec![],
            extrema_maxima: vec![],
            extrema_minima: vec![],
        }
    }
}

impl Default for Peaks {
    fn default() -> Self {
        Self::new()
    }
}
