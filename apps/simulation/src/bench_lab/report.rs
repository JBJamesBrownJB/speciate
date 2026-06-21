use crate::bench_lab::sampler::PhaseSamples;
use crate::bench_lab::stats::TickStats;
use crate::bench_lab::world::WorldSpec;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LabReport {
    pub label: String,
    pub spec: WorldSpec,
    pub budget_us: u64,
    pub within_budget: bool,
    pub max_pop: Option<usize>,
    pub samples: PhaseSamples,
    pub build_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PhaseDelta {
    pub name: String,
    pub before_us: f64,
    pub after_us: f64,
    pub delta_us: f64,
    pub pct: f64,
}

fn delta(name: &str, before: &TickStats, after: &TickStats) -> PhaseDelta {
    let before_us = before.mean;
    let after_us = after.mean;
    let delta_us = after_us - before_us;
    let pct = if before_us == 0.0 { 0.0 } else { delta_us / before_us * 100.0 };
    PhaseDelta { name: name.to_string(), before_us, after_us, delta_us, pct }
}

pub fn diff_reports(before: &LabReport, after: &LabReport) -> Vec<PhaseDelta> {
    let b = &before.samples;
    let a = &after.samples;
    vec![
        delta("total_tick", &b.total_tick, &a.total_tick),
        delta("perception", &b.perception, &a.perception),
        delta("steering", &b.steering, &a.steering),
        delta("movement", &b.movement, &a.movement),
        delta("spatial_grid_rebuild", &b.spatial_grid_rebuild, &a.spatial_grid_rebuild),
        delta("l1_aggregation", &b.l1_aggregation, &a.l1_aggregation),
        delta("behavior_transition", &b.behavior_transition, &a.behavior_transition),
        delta("export_positions", &b.export_positions, &a.export_positions),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bench_lab::world::Distribution;

    fn mean_stats(mean: f64) -> TickStats {
        TickStats { count: 7, min: mean, max: mean, mean, std_dev: 0.0, p50: mean, p95: mean, p99: mean }
    }

    fn report(label: &str, perception_mean: f64) -> LabReport {
        let mut samples = PhaseSamples::default();
        samples.total_tick = mean_stats(perception_mean + 1000.0);
        samples.perception = mean_stats(perception_mean);
        LabReport {
            label: label.to_string(),
            spec: WorldSpec {
                population: 200_000,
                seed: 1,
                half_extent_x: 2500.0,
                half_extent_y: 2000.0,
                distribution: Distribution::Uniform,
            },
            budget_us: 50_000,
            within_budget: true,
            max_pop: None,
            samples,
            build_type: "release".to_string(),
        }
    }

    #[test]
    fn diff_reports_computes_perception_delta() {
        let before = report("baseline", 15_000.0);
        let after = report("optimized", 12_000.0);
        let deltas = diff_reports(&before, &after);

        let perception = deltas.iter().find(|d| d.name == "perception").unwrap();
        assert_eq!(perception.before_us, 15_000.0);
        assert_eq!(perception.after_us, 12_000.0);
        assert_eq!(perception.delta_us, -3_000.0);
        assert!((perception.pct - (-20.0)).abs() < 1e-6);
    }

    #[test]
    fn report_serializes_to_camel_case_json() {
        let json = serde_json::to_string(&report("x", 15_000.0)).unwrap();
        assert!(json.contains("withinBudget"));
        assert!(json.contains("buildType"));
        assert!(json.contains("totalTick"));
    }
}
