pub mod budget;
pub mod ramp;
pub mod report;
pub mod sampler;
pub mod stats;
pub mod world;

pub use budget::{within_budget, BudgetMetric, TICK_BUDGET_US};
pub use ramp::{find_max_pop, MaxPopResult, RampConfig};
pub use report::{diff_reports, LabReport, PhaseDelta};
pub use sampler::{sample_ticks, PhaseSamples};
pub use stats::{summarize, TickStats};
pub use world::{build_world, Distribution, WorldSpec};

#[derive(Clone, Debug)]
pub struct LabConfig {
    pub label: String,
    pub spec: WorldSpec,
    pub warmup: usize,
    pub samples: usize,
    pub dt: f32,
    pub budget_us: u64,
    pub metric: BudgetMetric,
    pub find_max: Option<RampConfig>,
}

pub fn run_lab(cfg: &LabConfig) -> LabReport {
    let mut sim = build_world(&cfg.spec);
    let samples = sample_ticks(&mut sim, cfg.warmup, cfg.samples, cfg.dt);
    let within = within_budget(&samples.total_tick, cfg.budget_us, cfg.metric)
        || within_budget(&samples.wall_total, cfg.budget_us, cfg.metric);

    let max_pop = cfg.find_max.as_ref().map(|ramp| {
        let base = cfg.spec.clone();
        let warmup = cfg.warmup;
        let n = cfg.samples;
        let dt = cfg.dt;
        let result = find_max_pop(ramp, |pop| {
            let mut spec = base.clone();
            spec.population = pop;
            let mut sim = build_world(&spec);
            sample_ticks(&mut sim, warmup, n, dt).total_tick
        });
        result.max_pop
    });

    LabReport {
        label: cfg.label.clone(),
        spec: cfg.spec.clone(),
        budget_us: cfg.budget_us,
        within_budget: within,
        max_pop,
        samples,
        build_type: if cfg!(debug_assertions) { "debug" } else { "release" }.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bench_lab::budget::TICK_BUDGET_US;
    use crate::bench_lab::world::{Distribution, WorldSpec};

    fn small_spec(pop: usize) -> WorldSpec {
        WorldSpec {
            population: pop,
            seed: 11,
            half_extent_x: 500.0,
            half_extent_y: 500.0,
            distribution: Distribution::Uniform,
        }
    }

    #[test]
    fn run_lab_produces_report_for_fixed_population() {
        let cfg = LabConfig {
            label: "unit".to_string(),
            spec: small_spec(1000),
            warmup: 2,
            samples: 5,
            dt: 0.05,
            budget_us: TICK_BUDGET_US,
            metric: BudgetMetric::P99,
            find_max: None,
        };
        let report = run_lab(&cfg);
        assert_eq!(report.spec.population, 1000);
        assert_eq!(report.samples.wall_total.count, 5);
        assert!(report.max_pop.is_none());
    }

    #[test]
    fn run_lab_is_reproducible() {
        let cfg = LabConfig {
            label: "repro".to_string(),
            spec: small_spec(1000),
            warmup: 2,
            samples: 5,
            dt: 0.05,
            budget_us: TICK_BUDGET_US,
            metric: BudgetMetric::P99,
            find_max: None,
        };
        let a = run_lab(&cfg);
        let b = run_lab(&cfg);
        assert_eq!(a.samples.wall_total.count, b.samples.wall_total.count);
        assert_eq!(a.spec.population, b.spec.population);
    }
}
