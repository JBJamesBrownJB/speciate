use crate::bench_lab::stats::TickStats;
use serde::{Deserialize, Serialize};

pub const TICK_BUDGET_US: u64 = 50_000;

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum BudgetMetric {
    P99,
    Max,
    Mean,
}

impl Default for BudgetMetric {
    fn default() -> Self {
        BudgetMetric::P99
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn stats_with(p99: f64, max: f64, mean: f64) -> TickStats {
        TickStats { count: 7, min: 0.0, max, mean, std_dev: 0.0, p50: mean, p95: p99, p99 }
    }

    #[test]
    fn p99_under_budget_passes() {
        let s = stats_with(49_887.0, 49_888.0, 49_447.0);
        assert!(within_budget(&s, TICK_BUDGET_US, BudgetMetric::P99));
    }

    #[test]
    fn p99_over_budget_fails_even_when_mean_passes() {
        let s = stats_with(51_000.0, 52_000.0, 49_000.0);
        assert!(!within_budget(&s, TICK_BUDGET_US, BudgetMetric::P99));
        assert!(within_budget(&s, TICK_BUDGET_US, BudgetMetric::Mean));
    }

    #[test]
    fn default_metric_is_p99() {
        assert_eq!(BudgetMetric::default(), BudgetMetric::P99);
    }
}

pub fn within_budget(stats: &TickStats, budget_us: u64, metric: BudgetMetric) -> bool {
    let value = match metric {
        BudgetMetric::P99 => stats.p99,
        BudgetMetric::Max => stats.max,
        BudgetMetric::Mean => stats.mean,
    };
    value <= budget_us as f64
}
