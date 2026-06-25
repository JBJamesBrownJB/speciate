pub mod budget;
pub mod ramp;
pub mod report;
pub mod sampler;
pub mod stats;
pub mod sweep;
pub mod verdict;
pub mod world;

pub use budget::{within_budget, BudgetMetric, TICK_BUDGET_US};
pub use ramp::{find_max_pop, MaxPopResult, RampConfig};
pub use report::{diff_reports, LabReport, PhaseDelta};
pub use sweep::{sweep_populations, SweepPoint};
pub use sampler::{sample_ticks, PhaseSamples};
pub use stats::{summarize, TickStats};
pub use verdict::{classify, ChangeEvidence, Verdict};
pub use world::{build_world, Distribution, WorldSpec};

use serde::{Deserialize, Serialize};

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

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MultiSeedReport {
    pub label: String,
    pub population: usize,
    pub seeds: Vec<u64>,
    // Per-seed detail is bulky and not needed to compute a verdict; keep it in
    // memory but omit from the serialized A/B artifacts the workflow compares.
    #[serde(skip)]
    pub per_seed: Vec<LabReport>,
    pub wall_p99_across_seeds: TickStats,
    pub wall_mean_across_seeds: TickStats,
    /// Per-phase distribution of each phase's p99 across the seeds. The `std_dev`
    /// of each field is that phase's noise floor — the precise yardstick the
    /// verdict classifier judges a phase-targeted change against.
    pub per_phase_p99_across_seeds: PhaseNoiseFloors,
    /// Per-seed p99s (wall + each phase) in seed order, so a baseline-vs-candidate
    /// verdict can pair the arms seed-for-seed. `default` keeps older artifacts that
    /// predate this field deserializable (verdict then falls back to across-seed std).
    #[serde(default)]
    pub per_seed_p99: PerSeedP99s,
}

/// Per-seed p99 values (µs) for the wall and each time phase, in seed order.
/// WHY: baseline and candidate run on the SAME seeds, so each seed's world-to-world
/// cost is common to both arms. Pairing the per-seed p99s (Common Random Numbers)
/// cancels that shared variance, leaving only the change's own effect — exposing
/// real wins the much larger across-seed spread would otherwise bury as noise.
/// Serialized (unlike `per_seed`) because the cross-process A/B verdict needs it.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PerSeedP99s {
    pub wall: Vec<f64>,
    pub perception: Vec<f64>,
    pub steering: Vec<f64>,
    pub movement: Vec<f64>,
    pub spatial_grid_rebuild: Vec<f64>,
    pub l1_aggregation: Vec<f64>,
    pub behavior_transition: Vec<f64>,
}

/// Across-seed p99 distribution per phase. `std_dev` = the phase's noise floor,
/// `mean` = its typical p99. Mirrors the phases in [`PhaseSamples`] that carry
/// real timings (export/total_tick read 0 in the lab, so they are omitted).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PhaseNoiseFloors {
    pub perception: TickStats,
    pub steering: TickStats,
    pub movement: TickStats,
    pub spatial_grid_rebuild: TickStats,
    pub l1_aggregation: TickStats,
    pub behavior_transition: TickStats,
    pub cells_queried: TickStats,
}

/// The phases a change can target, used to pick which noise floor a verdict is
/// judged against. Time phases only — `cells_queried` is a causal proxy, not ms.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Phase {
    Perception,
    Steering,
    Movement,
    GridRebuild,
    L1Aggregation,
    Behavior,
}

impl Phase {
    /// All time phases, for scanning regressions across the whole tick.
    pub const ALL: [Phase; 6] = [
        Phase::Perception,
        Phase::Steering,
        Phase::Movement,
        Phase::GridRebuild,
        Phase::L1Aggregation,
        Phase::Behavior,
    ];

    pub fn parse(name: &str) -> Option<Phase> {
        Some(match name {
            "perception" => Phase::Perception,
            "steering" => Phase::Steering,
            "movement" => Phase::Movement,
            "grid_rebuild" | "spatial_grid_rebuild" => Phase::GridRebuild,
            "l1_aggregation" | "l1" => Phase::L1Aggregation,
            "behavior" | "behavior_transition" => Phase::Behavior,
            _ => return None,
        })
    }

    fn select<'a>(&self, floors: &'a PhaseNoiseFloors) -> &'a TickStats {
        match self {
            Phase::Perception => &floors.perception,
            Phase::Steering => &floors.steering,
            Phase::Movement => &floors.movement,
            Phase::GridRebuild => &floors.spatial_grid_rebuild,
            Phase::L1Aggregation => &floors.l1_aggregation,
            Phase::Behavior => &floors.behavior_transition,
        }
    }

    fn select_seeds<'a>(&self, s: &'a PerSeedP99s) -> &'a [f64] {
        match self {
            Phase::Perception => &s.perception,
            Phase::Steering => &s.steering,
            Phase::Movement => &s.movement,
            Phase::GridRebuild => &s.spatial_grid_rebuild,
            Phase::L1Aggregation => &s.l1_aggregation,
            Phase::Behavior => &s.behavior_transition,
        }
    }
}

/// Build the verdict inputs from a baseline vs candidate A/B, both measured the
/// same way (same pop, same seeds, quiet machine). The representative p99 is the
/// across-seed mean-of-p99s; the noise floor is the baseline's across-seed std.
pub fn evidence_from_reports(
    baseline: &MultiSeedReport,
    candidate: &MultiSeedReport,
    target: Phase,
) -> ChangeEvidence {
    let base_phase = target.select(&baseline.per_phase_p99_across_seeds);
    let cand_phase = target.select(&candidate.per_phase_p99_across_seeds);

    let worst_phase_regression_us = Phase::ALL
        .iter()
        .map(|p| {
            let b = p.select(&baseline.per_phase_p99_across_seeds).mean;
            let c = p.select(&candidate.per_phase_p99_across_seeds).mean;
            c - b
        })
        .fold(f64::NEG_INFINITY, f64::max);

    // Pair the arms seed-for-seed (Common Random Numbers): baseline and candidate
    // ran the SAME seeds, so each seed's world-to-world cost is shared and cancels
    // in the per-seed difference. The signal (mean of diffs) is unchanged, but the
    // noise floor collapses from the across-seed spread to the much smaller paired
    // std — the across-seed variance was never attributable to the change. Falls
    // back to the across-seed std for summary-only inputs (older artifacts/fixtures).
    let (phase_delta_p99_us, phase_noise_floor_us) =
        paired_delta_and_noise(target.select_seeds(&baseline.per_seed_p99), target.select_seeds(&candidate.per_seed_p99))
            .unwrap_or((cand_phase.mean - base_phase.mean, base_phase.std_dev));
    let (tick_delta_p99_us, tick_noise_floor_us) =
        paired_delta_and_noise(&baseline.per_seed_p99.wall, &candidate.per_seed_p99.wall)
            .unwrap_or((
                candidate.wall_p99_across_seeds.mean - baseline.wall_p99_across_seeds.mean,
                baseline.wall_p99_across_seeds.std_dev,
            ));

    ChangeEvidence {
        phase_delta_p99_us,
        phase_noise_floor_us,
        tick_delta_p99_us,
        tick_noise_floor_us,
        worst_phase_regression_us,
    }
}

/// Paired difference of two seed-aligned p99 vectors: `(mean(cand−base), std)`.
/// `std` is the population std of the per-seed differences (matching [`summarize`]),
/// i.e. the noise that survives after the shared world variance cancels. Returns
/// `None` when the arms cannot be paired (mismatched/under-two seeds), so callers
/// fall back to the unpaired across-seed floor.
fn paired_delta_and_noise(base: &[f64], cand: &[f64]) -> Option<(f64, f64)> {
    if base.len() != cand.len() || base.len() < 2 {
        return None;
    }
    let diffs: Vec<f64> = cand.iter().zip(base).map(|(c, b)| c - b).collect();
    let n = diffs.len() as f64;
    let mean = diffs.iter().sum::<f64>() / n;
    let variance = diffs.iter().map(|d| (d - mean).powi(2)).sum::<f64>() / n;
    Some((mean, variance.sqrt()))
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

    // Summarize each phase's per-seed p99 into its own across-seed distribution.
    let phase_p99 = |select: fn(&PhaseSamples) -> &TickStats| -> TickStats {
        let xs: Vec<u64> = per_seed
            .iter()
            .map(|r| select(&r.samples).p99.round() as u64)
            .collect();
        summarize(&xs)
    };
    let per_phase_p99_across_seeds = PhaseNoiseFloors {
        perception: phase_p99(|s| &s.perception),
        steering: phase_p99(|s| &s.steering),
        movement: phase_p99(|s| &s.movement),
        spatial_grid_rebuild: phase_p99(|s| &s.spatial_grid_rebuild),
        l1_aggregation: phase_p99(|s| &s.l1_aggregation),
        behavior_transition: phase_p99(|s| &s.behavior_transition),
        cells_queried: phase_p99(|s| &s.cells_queried),
    };

    // Raw (unrounded) per-seed p99s in seed order, so the verdict can pair arms.
    let seed_p99 = |select: fn(&PhaseSamples) -> &TickStats| -> Vec<f64> {
        per_seed.iter().map(|r| select(&r.samples).p99).collect()
    };
    let per_seed_p99 = PerSeedP99s {
        wall: seed_p99(|s| &s.wall_total),
        perception: seed_p99(|s| &s.perception),
        steering: seed_p99(|s| &s.steering),
        movement: seed_p99(|s| &s.movement),
        spatial_grid_rebuild: seed_p99(|s| &s.spatial_grid_rebuild),
        l1_aggregation: seed_p99(|s| &s.l1_aggregation),
        behavior_transition: seed_p99(|s| &s.behavior_transition),
    };

    MultiSeedReport {
        label: cfg.label.clone(),
        population: cfg.spec.population,
        seeds: seeds.to_vec(),
        wall_p99_across_seeds: summarize(&p99s),
        wall_mean_across_seeds: summarize(&means),
        per_phase_p99_across_seeds,
        per_seed_p99,
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
    fn multi_seed_reports_per_phase_noise_floor_structure() {
        let cfg = LabConfig {
            label: "phase-noise".to_string(),
            spec: small_spec(1000),
            warmup: 1,
            samples: 5,
            dt: 0.05,
            budget_us: TICK_BUDGET_US,
            metric: BudgetMetric::P99,
            find_max: None,
        };
        let report = run_lab_multi_seed(&cfg, &[11, 42, 99]);
        let nf = &report.per_phase_p99_across_seeds;
        // One p99 sample per seed; its std is that phase's noise floor.
        assert_eq!(nf.perception.count, 3, "one phase p99 per seed");
        assert_eq!(nf.steering.count, 3);
        assert!(nf.perception.std_dev.is_finite(), "phase noise floor must be finite");
        assert!(nf.steering.std_dev.is_finite());
        assert!(nf.cells_queried.std_dev.is_finite());
    }

    #[test]
    #[cfg(feature = "dev-tools")]
    fn multi_seed_per_phase_noise_floor_is_populated() {
        let cfg = LabConfig {
            label: "phase-noise-dev".to_string(),
            spec: small_spec(1000),
            warmup: 1,
            samples: 5,
            dt: 0.05,
            budget_us: TICK_BUDGET_US,
            metric: BudgetMetric::P99,
            find_max: None,
        };
        let report = run_lab_multi_seed(&cfg, &[11, 42, 99]);
        assert!(
            report.per_phase_p99_across_seeds.perception.mean > 0.0,
            "perception phase p99 must be real under dev-tools"
        );
    }

    fn stats(mean: f64, std_dev: f64) -> TickStats {
        TickStats { count: 5, mean, std_dev, p99: mean, ..TickStats::default() }
    }

    /// A report where every time phase shares `phase` stats and the wall uses
    /// `wall` stats — enough to exercise the evidence builder deterministically.
    /// No per-seed vectors → exercises the across-seed (unpaired) fallback path.
    fn report_with(perception: TickStats, wall: TickStats) -> MultiSeedReport {
        MultiSeedReport {
            label: "t".into(),
            population: 900_000,
            seeds: vec![11, 42, 99, 137, 2025],
            per_seed: vec![],
            per_seed_p99: PerSeedP99s::default(),
            wall_p99_across_seeds: wall,
            wall_mean_across_seeds: TickStats::default(),
            per_phase_p99_across_seeds: PhaseNoiseFloors {
                perception,
                ..PhaseNoiseFloors::default()
            },
        }
    }

    /// A report carrying real per-seed steering & wall p99s, with the across-seed
    /// summaries derived from them — mirrors a real run so the paired path engages.
    fn report_with_seeds(steering: &[f64], wall: &[f64]) -> MultiSeedReport {
        let u64s = |xs: &[f64]| xs.iter().map(|&x| x.round() as u64).collect::<Vec<_>>();
        MultiSeedReport {
            label: "t".into(),
            population: 900_000,
            seeds: vec![11, 42, 99],
            per_seed: vec![],
            per_seed_p99: PerSeedP99s {
                wall: wall.to_vec(),
                steering: steering.to_vec(),
                ..PerSeedP99s::default()
            },
            wall_p99_across_seeds: summarize(&u64s(wall)),
            wall_mean_across_seeds: TickStats::default(),
            per_phase_p99_across_seeds: PhaseNoiseFloors {
                steering: summarize(&u64s(steering)),
                ..PhaseNoiseFloors::default()
            },
        }
    }

    #[test]
    fn evidence_pairs_seeds_so_a_consistent_win_clears_cross_seed_noise() {
        // Worlds differ wildly seed-to-seed (steering p99 10ms→18ms across seeds;
        // across-seed std ≈ 3.27ms), but the candidate is consistently ~0.5ms
        // faster on EVERY seed and the wall ~2.5ms faster on every seed. The
        // shared world variance must cancel: judged against the across-seed std
        // this is invisible (Ditch); paired, it is a clean Keep.
        let base = report_with_seeds(&[10_000.0, 14_000.0, 18_000.0], &[48_000.0, 52_000.0, 56_000.0]);
        let cand = report_with_seeds(&[9_500.0, 13_400.0, 17_600.0], &[45_400.0, 49_600.0, 53_500.0]);
        let ev = evidence_from_reports(&base, &cand, Phase::Steering);

        // Signal is unchanged (mean of diffs = diff of means).
        assert!((ev.phase_delta_p99_us - -500.0).abs() < 1e-6, "phase signal");
        assert!((ev.tick_delta_p99_us - -2_500.0).abs() < 1e-6, "wall signal");
        // Noise is the PAIRED-difference std (≈82us), not the across-seed std (≈3266us).
        assert!(ev.phase_noise_floor_us < 200.0, "paired phase noise, got {}", ev.phase_noise_floor_us);
        assert!(ev.tick_noise_floor_us < 200.0, "paired wall noise, got {}", ev.tick_noise_floor_us);
        // The whole point: a win the unpaired gate threw away now banks.
        assert_eq!(classify(&ev), Verdict::Keep);
    }

    #[test]
    fn evidence_uses_baseline_noise_and_phase_delta() {
        // Perception p99 16000→13000 (−3ms), baseline phase noise 200us;
        // wall 48000→45500 (−2.5ms), baseline wall noise 300us.
        let base = report_with(stats(16_000.0, 200.0), stats(48_000.0, 300.0));
        let cand = report_with(stats(13_000.0, 150.0), stats(45_500.0, 250.0));
        let ev = evidence_from_reports(&base, &cand, Phase::Perception);
        assert!((ev.phase_delta_p99_us - -3_000.0).abs() < 1e-6, "phase delta from candidate");
        assert!((ev.phase_noise_floor_us - 200.0).abs() < 1e-6, "noise floor is the BASELINE std");
        assert!((ev.tick_delta_p99_us - -2_500.0).abs() < 1e-6);
        assert!((ev.tick_noise_floor_us - 300.0).abs() < 1e-6);
    }

    #[test]
    fn evidence_reports_worst_phase_regression_across_all_phases() {
        // Baseline all phases at 5000us; candidate regresses steering to 8000us
        // (+3ms) while perception improves. Worst regression must surface +3ms.
        let mut base = report_with(stats(5_000.0, 100.0), stats(40_000.0, 200.0));
        for p in [
            &mut base.per_phase_p99_across_seeds.steering,
            &mut base.per_phase_p99_across_seeds.movement,
            &mut base.per_phase_p99_across_seeds.spatial_grid_rebuild,
            &mut base.per_phase_p99_across_seeds.l1_aggregation,
            &mut base.per_phase_p99_across_seeds.behavior_transition,
        ] {
            *p = stats(5_000.0, 100.0);
        }
        let mut cand = base.clone();
        cand.per_phase_p99_across_seeds.perception = stats(2_000.0, 100.0);
        cand.per_phase_p99_across_seeds.steering = stats(8_000.0, 100.0);
        let ev = evidence_from_reports(&base, &cand, Phase::Perception);
        assert!(
            (ev.worst_phase_regression_us - 3_000.0).abs() < 1e-6,
            "steering's +3ms must be the worst regression, got {}",
            ev.worst_phase_regression_us
        );
    }

    #[test]
    fn per_seed_p99_survives_json_round_trip_so_verdict_can_pair() {
        // The verdict runs in a separate process off serialized A/B artifacts. If
        // per-seed p99s don't survive serde (the original `per_seed` bug), pairing
        // silently degrades to the unpaired floor. Guard the boundary.
        let r = report_with_seeds(&[10_000.0, 14_000.0, 18_000.0], &[48_000.0, 52_000.0, 56_000.0]);
        let round: MultiSeedReport = serde_json::from_str(&serde_json::to_string(&r).unwrap()).unwrap();
        assert_eq!(round.per_seed_p99.steering, vec![10_000.0, 14_000.0, 18_000.0]);
        assert_eq!(round.per_seed_p99.wall, vec![48_000.0, 52_000.0, 56_000.0]);
    }

    #[test]
    fn multi_seed_populates_per_seed_p99_in_seed_order() {
        let cfg = LabConfig {
            label: "per-seed".to_string(),
            spec: small_spec(1000),
            warmup: 1,
            samples: 5,
            dt: 0.05,
            budget_us: TICK_BUDGET_US,
            metric: BudgetMetric::P99,
            find_max: None,
        };
        let report = run_lab_multi_seed(&cfg, &[11, 42, 99]);
        assert_eq!(report.per_seed_p99.wall.len(), 3, "one wall p99 per seed for pairing");
        assert_eq!(report.per_seed_p99.steering.len(), 3);
        assert!(report.per_seed_p99.wall.iter().all(|&x| x > 0.0), "real wall p99s");
    }

    #[test]
    fn evidence_feeds_classify_end_to_end() {
        // A clean perception win that also moves the wall → KEEP through the
        // real classifier, proving the two pieces compose.
        let base = report_with(stats(16_000.0, 200.0), stats(48_000.0, 300.0));
        let cand = report_with(stats(13_000.0, 150.0), stats(45_000.0, 300.0));
        let ev = evidence_from_reports(&base, &cand, Phase::Perception);
        assert_eq!(classify(&ev), Verdict::Keep);
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
