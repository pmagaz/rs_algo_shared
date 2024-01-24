pub fn percentage_change(x: f64, y: f64) -> f64 {
    let max = x.max(y);
    let min = y.min(x);
    let increase = max - min;

    (increase / x) * 100.
}

pub fn price_change(prev: f64, curr: f64) -> f64 {
    (curr - prev) / prev * 100.
}

pub fn is_equal(x: f64, y: f64, threshold: f64) -> bool {
    let percentage_change = percentage_change(x, y);
    percentage_change <= 0. || percentage_change < threshold
}

pub fn is_same_band(x: f64, y: f64, threshold: f64) -> bool {
    let percentage_change = percentage_change(x, y);
    percentage_change > 0. && percentage_change < threshold
}

pub fn is_equal_distance(a: (f64, f64), b: (f64, f64), threshold: f64) -> bool {
    let move_a = (a.0 - a.1).abs();
    let percentage_move_a = (move_a / b.1) * 100.;

    let move_b = (b.0 - b.1).abs();
    let percentage_move_b = (move_b / b.1) * 100.;

    (percentage_move_a - percentage_move_b).abs() < threshold
}

pub fn increase_equally(a: (f64, f64), b: (f64, f64), threshold: f64) -> bool {
    let increase_a = (a.0 - a.1).abs();
    let percentage_increase_a = (increase_a / b.1) * 100.;

    let increase_b = (b.0 - b.1).abs();
    let percentage_increase_b = (increase_b / b.1) * 100.;

    (percentage_increase_a - percentage_increase_b).abs() < threshold
}

pub fn max_number(data: &Vec<f64>) -> f64 {
    let max = data
        .iter()
        .max_by(|a, b| a.partial_cmp(b).unwrap())
        .unwrap();
    *max
}

pub fn min_number(data: &Vec<f64>) -> f64 {
    let min = data
        .iter()
        .min_by(|a, b| a.partial_cmp(b).unwrap())
        .unwrap();
    *min
}

//FIXME GENERIC
pub fn average_f64(numbers: &Vec<f64>) -> f64 {
    if numbers.is_empty() {
        0.
    } else {
        numbers.iter().sum::<f64>() / numbers.len() as f64
    }
}

pub fn average_usize(numbers: &Vec<usize>) -> usize {
    if numbers.is_empty() {
        0
    } else {
        numbers.iter().sum::<usize>() / numbers.len()
    }
}

pub fn is_upward_trend(data: &[f64]) -> bool {
    if data.len() < 2 {
        return false; // Not enough data to determine a trend
    }

    for i in 0..data.len() - 1 {
        if data[i] > data[i + 1] {
            return false; // Not an upward trend
        }
    }

    true // It's an upward trend
}

pub fn is_downward_trend(data: &[f64]) -> bool {
    if data.len() < 2 {
        return false; // Not enough data to determine a trend
    }

    for i in 0..data.len() - 1 {
        if data[i] < data[i + 1] {
            return false; // Not a downward trend
        }
    }

    true // It's a downward trend
}

pub fn symbol_in_list(symbol: &str, sp_symbols: &Vec<String>) -> bool {
    let mut result = false;
    for sp_symbol in sp_symbols {
        let compare: &str;
        if symbol.contains('_') {
            let arr: Vec<&str> = symbol.split('_').collect();
            compare = arr[0];
        } else {
            compare = symbol;
        }

        if compare == sp_symbol {
            result = true;
            break;
        }
    }
    result
}
