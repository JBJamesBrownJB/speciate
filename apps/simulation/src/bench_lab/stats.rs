use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct TickStats {
    pub count: usize,
    pub min: f64,
    pub max: f64,
    pub mean: f64,
    pub std_dev: f64,
    pub p50: f64,
    pub p95: f64,
    pub p99: f64,
}

fn percentile(sorted: &[f64], p: f64) -> f64 {
    match sorted.len() {
        0 => 0.0,
        1 => sorted[0],
        n => {
            let rank = (p / 100.0) * (n as f64 - 1.0);
            let lo = rank.floor() as usize;
            let hi = rank.ceil() as usize;
            let frac = rank - lo as f64;
            sorted[lo] + frac * (sorted[hi] - sorted[lo])
        }
    }
}

pub fn summarize(samples: &[u64]) -> TickStats {
    if samples.is_empty() {
        return TickStats::default();
    }
    let mut sorted: Vec<f64> = samples.iter().map(|&v| v as f64).collect();
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());

    let count = sorted.len();
    let mean = sorted.iter().sum::<f64>() / count as f64;
    let variance = sorted.iter().map(|v| (v - mean).powi(2)).sum::<f64>() / count as f64;

    TickStats {
        count,
        min: sorted[0],
        max: sorted[count - 1],
        mean,
        std_dev: variance.sqrt(),
        p50: percentile(&sorted, 50.0),
        p95: percentile(&sorted, 95.0),
        p99: percentile(&sorted, 99.0),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn approx(a: f64, b: f64) {
        assert!((a - b).abs() < 1e-6, "expected {b}, got {a}");
    }

    #[test]
    fn summarize_basic_moments() {
        let stats = summarize(&[10, 20, 30, 40, 50]);
        assert_eq!(stats.count, 5);
        approx(stats.min, 10.0);
        approx(stats.max, 50.0);
        approx(stats.mean, 30.0);
        approx(stats.std_dev, 14.142135623730951);
        approx(stats.p50, 30.0);
    }

    #[test]
    fn summarize_percentiles_interpolate() {
        let samples: Vec<u64> = (1..=20).collect();
        let stats = summarize(&samples);
        approx(stats.p50, 10.5);
        approx(stats.p95, 19.05);
    }

    #[test]
    fn summarize_is_order_independent() {
        let a = summarize(&[50, 10, 40, 20, 30]);
        let b = summarize(&[10, 20, 30, 40, 50]);
        assert_eq!(a, b);
    }

    #[test]
    fn summarize_empty_is_default() {
        assert_eq!(summarize(&[]), TickStats::default());
    }
}
