use crate::bench_lab::sampler::sample_ticks;
use crate::bench_lab::world::{build_world, WorldSpec};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SweepPoint {
    pub population: usize,
    pub wall_mean_us: f64,
    pub wall_p99_us: f64,
    pub within_budget: bool,
}

pub fn sweep_populations(
    base: &WorldSpec,
    populations: &[usize],
    warmup: usize,
    samples: usize,
    dt: f32,
    budget_us: u64,
) -> Vec<SweepPoint> {
    populations
        .iter()
        .map(|&pop| {
            let mut spec = base.clone();
            spec.population = pop;
            let mut sim = build_world(&spec);
            let wall = sample_ticks(&mut sim, warmup, samples, dt).wall_total;
            SweepPoint {
                population: pop,
                wall_mean_us: wall.mean,
                wall_p99_us: wall.p99,
                within_budget: wall.p99 <= budget_us as f64,
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bench_lab::budget::TICK_BUDGET_US;
    use crate::bench_lab::world::{Distribution, WorldSpec};

    fn base() -> WorldSpec {
        WorldSpec {
            population: 0,
            seed: 7,
            half_extent_x: 500.0,
            half_extent_y: 500.0,
            distribution: Distribution::Uniform,
        }
    }

    #[test]
    fn sweep_returns_one_point_per_population_in_order() {
        let pts = sweep_populations(&base(), &[100, 300, 500], 1, 3, 0.05, TICK_BUDGET_US);
        assert_eq!(pts.len(), 3);
        assert_eq!(pts[0].population, 100);
        assert_eq!(pts[1].population, 300);
        assert_eq!(pts[2].population, 500);
    }

    #[test]
    fn sweep_records_real_nonzero_wall_time() {
        let pts = sweep_populations(&base(), &[200, 400], 1, 3, 0.05, TICK_BUDGET_US);
        assert!(pts.iter().all(|p| p.wall_p99_us > 0.0));
    }

    #[test]
    fn sweep_small_populations_are_within_generous_budget() {
        let pts = sweep_populations(&base(), &[100], 1, 3, 0.05, TICK_BUDGET_US);
        assert!(pts[0].within_budget, "100 creatures must fit a 50ms budget");
    }
}
