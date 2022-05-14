use std::iter::Sum;

pub fn percentage_change(x: f64, y: f64) -> f64 {
    let max = x.max(y);
    let min = y.min(x);
    let increase = max - min;
    let percentage_increase = (increase / x) * 100.;
    percentage_increase
}

pub fn price_change(prev: f64, curr: f64) -> f64 {
    (curr - prev) / prev * 100.
}

pub fn is_equal(x: f64, y: f64, threshold: f64) -> bool {
    let percentage_change = percentage_change(x, y);
    if percentage_change <= 0. || percentage_change < threshold {
        true
    } else {
        false
    }
}

pub fn is_same_band(x: f64, y: f64, threshold: f64) -> bool {
    let percentage_change = percentage_change(x, y);
    if percentage_change > 0. && percentage_change < threshold {
        true
    } else {
        false
    }
}

pub fn is_equal_distance(a: (f64, f64), b: (f64, f64), threshold: f64) -> bool {
    let move_a = (a.0 - a.1).abs();
    let percentage_move_a = (move_a / b.1) * 100.;

    let move_b = (b.0 - b.1).abs();
    let percentage_move_b = (move_b / b.1) * 100.;

    if (percentage_move_a - percentage_move_b).abs() < threshold {
        true
    } else {
        false
    }
}

pub fn increase_equally(a: (f64, f64), b: (f64, f64), threshold: f64) -> bool {
    let increase_a = (a.0 - a.1).abs();
    let percentage_increase_a = (increase_a / b.1) * 100.;

    let increase_b = (b.0 - b.1).abs();
    let percentage_increase_b = (increase_b / b.1) * 100.;

    if (percentage_increase_a - percentage_increase_b).abs() < threshold {
        true
    } else {
        false
    }
}

pub fn max_number(data: &Vec<f64>) -> f64 {
    let min = data
        .iter()
        .max_by(|a, b| a.partial_cmp(&b).unwrap())
        .unwrap();
    *min
}

pub fn min_number(data: &Vec<f64>) -> f64 {
    let min = data
        .iter()
        .min_by(|a, b| a.partial_cmp(&b).unwrap())
        .unwrap();
    *min
}

pub fn average_iter<T, I: Iterator<Item = T>>(iter: I) -> Option<f64>
where
    T: Into<f64> + Sum<T>,
{
    let mut len = 0;
    let sum = iter
        .map(|t| {
            len += 1;
            t
        })
        .sum::<T>();

    match len {
        0 => None,
        _ => Some(sum.into() / len as f64),
    }
}
