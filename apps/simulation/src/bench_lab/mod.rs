pub mod budget;
pub mod ramp;
pub mod report;
pub mod sampler;
pub mod stats;
pub mod sweep;
pub mod world;

pub use budget::{within_budget, BudgetMetric, TICK_BUDGET_US};
pub use ramp::{find_max_pop, MaxPopResult, RampConfig};
pub use report::{diff_reports, LabReport, PhaseDelta};
pub use sweep::{sweep_populations, SweepPoint};
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

#[derive(Debug, Clone)]
pub struct MultiSeedReport {
    pub label: String,
    pub population: usize,
    pub seeds: Vec<u64>,
    pub per_seed: Vec<LabReport>,
    pub wall_p99_across_seeds: TickStats,
    pub wall_mean_across_seeds: TickStats,
}

pub fn run_lab_multi_seed(cfg: &LabConfig, seeds: &[u64]) -> MultiSeedReport {
    let per_seed: Vec<LabReport> = seeds
        .iter()
        .map(|&s| {
            let mut c = cfg.clone();
            c.spec.seed = s;
            c.label = format!("{}_seed{}", cfg.label, s);
            run_lab(&c)
        })
        .collect();

    let p99s: Vec<u64> = per_seed.iter().map(|r| r.samples.wall_total.p99.round() as u64).collect();
    let means: Vec<u64> = per_seed.iter().map(|r| r.samples.wall_total.mean.round() as u64).collect();

    MultiSeedReport {
        label: cfg.label.clone(),
        population: cfg.spec.population,
        seeds: seeds.to_vec(),
        wall_p99_across_seeds: summarize(&p99s),
        wall_mean_across_seeds: summarize(&means),
        per_seed,
    }
}

pub fn run_lab(cfg: &LabConfig) -> LabReport {
    let mut sim = build_world(&cfg.spec);
    let samples = sample_ticks(&mut sim, cfg.warmup, cfg.samples, cfg.dt);
    let within = within_budget(&samples.wall_total, cfg.budget_us, cfg.metric);

    let max_pop = cfg.find_max.as_ref().map(|ramp| {
        let base = cfg.spec.clone();
        let warmup = cfg.warmup;
        let n = cfg.samples;
        let dt = cfg.dt;
        let result = find_max_pop(ramp, |pop| {
            let mut spec = base.clone();
            spec.population = pop;
            let mut sim = build_world(&spec);
            sample_ticks(&mut sim, warmup, n, dt).wall_total
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
mod crossing_measurement;

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

    #[test]
    fn budget_keys_on_wall_clock_not_zeroed_total_tick() {
        let cfg = LabConfig {
            label: "tiny-budget".to_string(),
            spec: small_spec(2000),
            warmup: 1,
            samples: 5,
            dt: 0.05,
            budget_us: 1,
            metric: BudgetMetric::P99,
            find_max: None,
        };
        let report = run_lab(&cfg);
        assert!(report.samples.wall_total.p99 > 1.0, "wall clock must be real/nonzero");
        assert!(!report.within_budget, "1us budget must fail on real wall-clock time");
    }

    #[test]
    fn multi_seed_aggregates_across_seeds() {
        let cfg = LabConfig {
            label: "noise".to_string(),
            spec: small_spec(1000),
            warmup: 1,
            samples: 5,
            dt: 0.05,
            budget_us: TICK_BUDGET_US,
            metric: BudgetMetric::P99,
            find_max: None,
        };
        let report = run_lab_multi_seed(&cfg, &[11, 42, 99]);
        assert_eq!(report.seeds.len(), 3);
        assert_eq!(report.per_seed.len(), 3);
        assert_eq!(report.wall_p99_across_seeds.count, 3);
        assert!(report.wall_p99_across_seeds.std_dev.is_finite(), "noise floor must be a finite number");
        assert!(report.wall_p99_across_seeds.mean > 0.0);
    }

    #[test]
    fn run_lab_find_max_exercises_search_and_returns_some() {
        use crate::bench_lab::ramp::RampConfig;

        let cfg = LabConfig {
            label: "findmax".to_string(),
            spec: small_spec(100),
            warmup: 1,
            samples: 2,
            dt: 0.05,
            budget_us: TICK_BUDGET_US,
            metric: BudgetMetric::P99,
            find_max: Some(RampConfig {
                low: 100,
                high: 400,
                coarse_step: 100,
                tolerance: 50,
                budget_us: TICK_BUDGET_US,
                metric: BudgetMetric::P99,
            }),
        };
        let report = run_lab(&cfg);
        assert!(report.max_pop.is_some(), "find_max branch must populate max_pop");
        let max = report.max_pop.unwrap();
        assert!((100..=400).contains(&max), "max_pop {max} must lie within the ramp range");
    }
}
