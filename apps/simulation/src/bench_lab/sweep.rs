use crate::bench_lab::growth::fit_growth_exponent;
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

/// A population sweep plus its fitted growth-rate exponent, so the downstream
/// reader (the cloud triage workflow) gets the big-O classification straight
/// from the artifact instead of re-deriving it. `growth_exponent` is `None`
/// when the fit is undefined (see [`fit_growth_exponent`]).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SweepReport {
    pub points: Vec<SweepPoint>,
    pub growth_exponent: Option<f64>,
}

impl SweepReport {
    /// Bundle sweep points with the exponent fitted from them.
    pub fn from_points(points: Vec<SweepPoint>) -> Self {
        let growth_exponent = fit_growth_exponent(&points);
        SweepReport { points, growth_exponent }
    }
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

    /// Consumer boundary: the cloud workflow reads the sweep JSON, so prove the
    /// fitted exponent survives serialization to the downstream reader — not just
    /// that Rust computed it. Linear synthetic points must round-trip to b≈1.
    #[test]
    fn sweep_report_serializes_growth_exponent_for_the_reader() {
        let points: Vec<SweepPoint> = [1000usize, 2000, 4000, 10000]
            .iter()
            .map(|&n| SweepPoint {
                population: n,
                wall_mean_us: 0.5 * n as f64,
                wall_p99_us: 0.5 * n as f64,
                within_budget: true,
            })
            .collect();
        let report = SweepReport::from_points(points);
        let json = serde_json::to_string(&report).expect("serialize sweep report");
        assert!(json.contains("growthExponent"), "reader must see the exponent field: {json}");

        let round: SweepReport = serde_json::from_str(&json).expect("reader parses report");
        let b = round.growth_exponent.expect("linear data yields a finite exponent");
        assert!(b.is_finite(), "exponent must be finite, got {b}");
        assert!((b - 1.0).abs() < 1e-6, "linear sweep must read back as b≈1, got {b}");
    }
}
