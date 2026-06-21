use crate::bench_lab::budget::{within_budget, BudgetMetric};
use crate::bench_lab::stats::TickStats;

#[derive(Clone, Debug)]
pub struct RampConfig {
    pub low: usize,
    pub high: usize,
    pub coarse_step: usize,
    pub tolerance: usize,
    pub budget_us: u64,
    pub metric: BudgetMetric,
}

#[derive(Clone, Debug)]
pub struct MaxPopResult {
    pub max_pop: usize,
    pub evaluations: Vec<(usize, TickStats)>,
}

pub fn find_max_pop(cfg: &RampConfig, mut run: impl FnMut(usize) -> TickStats) -> MaxPopResult {
    let mut evaluations: Vec<(usize, TickStats)> = Vec::new();

    let mut eval = |pop: usize, evals: &mut Vec<(usize, TickStats)>| -> bool {
        let stats = run(pop);
        let pass = within_budget(&stats, cfg.budget_us, cfg.metric);
        evals.push((pop, stats));
        pass
    };

    let mut last_pass: Option<usize> = None;
    let mut first_fail: Option<usize> = None;
    let mut pop = cfg.low;
    loop {
        let pass = eval(pop, &mut evaluations);
        if pass {
            last_pass = Some(pop);
            if pop >= cfg.high {
                break;
            }
            pop = (pop + cfg.coarse_step).min(cfg.high);
        } else {
            first_fail = Some(pop);
            break;
        }
    }

    let mut lo = match last_pass {
        Some(p) => p,
        None => {
            return MaxPopResult { max_pop: 0, evaluations };
        }
    };

    if let Some(mut hi) = first_fail {
        while hi - lo > cfg.tolerance {
            let mid = lo + (hi - lo) / 2;
            if eval(mid, &mut evaluations) {
                lo = mid;
            } else {
                hi = mid;
            }
        }
    }

    MaxPopResult { max_pop: lo, evaluations }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn synthetic(pop: usize) -> TickStats {
        let p99 = pop as f64 * 0.05;
        TickStats { count: 7, min: p99, max: p99, mean: p99, std_dev: 0.0, p50: p99, p95: p99, p99 }
    }

    fn cfg() -> RampConfig {
        RampConfig {
            low: 200_000,
            high: 2_000_000,
            coarse_step: 300_000,
            tolerance: 25_000,
            budget_us: 50_000,
            metric: BudgetMetric::P99,
        }
    }

    #[test]
    fn finds_crossover_within_tolerance() {
        let result = find_max_pop(&cfg(), synthetic);
        assert!(result.max_pop <= 1_000_000);
        assert!(result.max_pop >= 1_000_000 - 25_000);
        assert!(within_budget(&synthetic(result.max_pop), 50_000, BudgetMetric::P99));
    }

    #[test]
    fn records_every_evaluation() {
        let result = find_max_pop(&cfg(), synthetic);
        assert!(result.evaluations.len() >= 2);
        assert_eq!(result.evaluations[0].0, 200_000);
    }

    #[test]
    fn returns_zero_when_low_already_fails() {
        let mut c = cfg();
        c.low = 1_200_000;
        let result = find_max_pop(&c, synthetic);
        assert_eq!(result.max_pop, 0);
    }
}
