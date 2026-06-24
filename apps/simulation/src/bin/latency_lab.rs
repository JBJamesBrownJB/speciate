use speciate::bench_lab::budget::TICK_BUDGET_US;
use speciate::bench_lab::ramp::RampConfig;
use speciate::bench_lab::sweep::sweep_populations;
use speciate::bench_lab::world::{Distribution, WorldSpec};
use speciate::bench_lab::{
    classify, evidence_from_reports, run_lab, run_lab_multi_seed, BudgetMetric, LabConfig,
    MultiSeedReport, Phase, Verdict,
};

fn arg<T: std::str::FromStr>(args: &[String], key: &str, default: T) -> T {
    args.iter()
        .position(|a| a == key)
        .and_then(|i| args.get(i + 1))
        .and_then(|v| v.parse().ok())
        .unwrap_or(default)
}

fn flag(args: &[String], key: &str) -> bool {
    args.iter().any(|a| a == key)
}

/// Compare two multi-seed A/B artifacts with the tested `classify()` and print
/// the verdict. The workflow's serial measurement loop calls this so the gate —
/// not an LLM — decides KEEP/DEFER/DITCH. Exit code: 0 KEEP, 2 DEFER, 3 DITCH.
fn run_verdict(args: &[String]) -> ! {
    let read = |key: &str| -> MultiSeedReport {
        let path = args
            .iter()
            .position(|a| a == key)
            .and_then(|i| args.get(i + 1))
            .unwrap_or_else(|| panic!("--verdict requires {key} <path>"));
        let json = std::fs::read_to_string(path).unwrap_or_else(|e| panic!("read {path}: {e}"));
        serde_json::from_str(&json).unwrap_or_else(|e| panic!("parse {path}: {e}"))
    };
    let baseline = read("--baseline");
    let candidate = read("--candidate");
    let phase_name = args
        .iter()
        .position(|a| a == "--phase")
        .and_then(|i| args.get(i + 1))
        .map(String::as_str)
        .unwrap_or("perception");
    let phase = Phase::parse(phase_name)
        .unwrap_or_else(|| panic!("unknown --phase {phase_name}"));

    let ev = evidence_from_reports(&baseline, &candidate, phase);
    let verdict = classify(&ev);
    println!(
        "VERDICT={:?} phase={phase_name} dPhaseP99={:.0}us phaseNoise={:.0}us dWallP99={:.0}us wallNoise={:.0}us worstPhaseRegression={:.0}us",
        verdict, ev.phase_delta_p99_us, ev.phase_noise_floor_us,
        ev.tick_delta_p99_us, ev.tick_noise_floor_us, ev.worst_phase_regression_us,
    );
    std::process::exit(match verdict {
        Verdict::Keep => 0,
        Verdict::Defer => 2,
        Verdict::Ditch => 3,
    });
}

fn main() {
    let args: Vec<String> = std::env::args().collect();

    if flag(&args, "--verdict") {
        run_verdict(&args);
    }

    let pop: usize = arg(&args, "--pop", 200_000);
    let seed: u64 = arg(&args, "--seed", 1);
    let samples: usize = arg(&args, "--samples", 60);
    let warmup: usize = arg(&args, "--warmup", 20);
    let half_x: f32 = arg(&args, "--half-x", 5000.0);
    let half_y: f32 = arg(&args, "--half-y", 5000.0);
    let dt: f32 = arg(&args, "--dt", 0.05);

    let distribution = if flag(&args, "--clustered") {
        Distribution::Clustered {
            clusters: arg(&args, "--clusters", 32),
            spread: arg(&args, "--spread", 150.0),
        }
    } else if flag(&args, "--realistic-dna") {
        Distribution::RealisticSize { median_meters: 0.5, sigma_log10: 0.45 }
    } else {
        Distribution::Uniform
    };

    if let Some(i) = args.iter().position(|a| a == "--seeds") {
        let seeds: Vec<u64> = args.get(i + 1).map(|s| s.split(',').filter_map(|x| x.trim().parse().ok()).collect()).unwrap_or_default();
        if !seeds.is_empty() {
            let base = LabConfig {
                label: format!("pop{pop}"),
                spec: WorldSpec { population: pop, seed: seeds[0], half_extent_x: half_x, half_extent_y: half_y, distribution: distribution.clone() },
                warmup, samples, dt, budget_us: TICK_BUDGET_US, metric: BudgetMetric::P99, find_max: None,
            };
            let r = run_lab_multi_seed(&base, &seeds);
            eprintln!("[{} pop={}] seeds={:?}", r.label, r.population, r.seeds);
            eprintln!("  wall p99: mean-of-p99s={:.0}us  NOISE-FLOOR(std)={:.0}us  worst={:.0}us",
                r.wall_p99_across_seeds.mean, r.wall_p99_across_seeds.std_dev, r.wall_p99_across_seeds.max);
            eprintln!("  wall mean: mean-of-means={:.0}us", r.wall_mean_across_seeds.mean);
            // Per-phase p99 + its own noise floor: detection of a phase-targeted
            // change is judged against THIS phase's noise, not the wall's.
            let nf = &r.per_phase_p99_across_seeds;
            for (name, s) in [
                ("perception", &nf.perception),
                ("steering", &nf.steering),
                ("movement", &nf.movement),
                ("grid_rebuild", &nf.spatial_grid_rebuild),
                ("l1_aggregation", &nf.l1_aggregation),
                ("behavior", &nf.behavior_transition),
                ("cells_queried", &nf.cells_queried),
            ] {
                eprintln!("  phase {name:<14} p99={:.0}  NOISE-FLOOR(std)={:.0}  (detect bar: >{:.0})",
                    s.mean, s.std_dev, 2.0 * s.std_dev);
            }
            if let Some(i) = args.iter().position(|a| a == "--out") {
                if let Some(path) = args.get(i + 1) {
                    let json = serde_json::to_string_pretty(&r).expect("serialize multi-seed");
                    std::fs::write(path, json).expect("write multi-seed report");
                    eprintln!("wrote {path}");
                }
            }
            return;
        }
    }

    if flag(&args, "--sweep") {
        let from: usize = arg(&args, "--sweep-from", 100_000);
        let to: usize = arg(&args, "--sweep-to", 1_000_000);
        let step: usize = arg(&args, "--sweep-step", 100_000);
        let pops: Vec<usize> = (from..=to).step_by(step.max(1)).collect();
        let base = WorldSpec {
            population: from,
            seed,
            half_extent_x: half_x,
            half_extent_y: half_y,
            distribution: distribution.clone(),
        };
        let points = sweep_populations(&base, &pops, warmup, samples, dt, TICK_BUDGET_US);
        eprintln!("population,wall_mean_us,wall_p99_us,within_budget");
        for p in &points {
            eprintln!("{},{:.0},{:.0},{}", p.population, p.wall_mean_us, p.wall_p99_us, p.within_budget);
        }
        if let Some(i) = args.iter().position(|a| a == "--out") {
            if let Some(path) = args.get(i + 1) {
                let json = serde_json::to_string_pretty(&points).expect("serialize sweep");
                std::fs::write(path, json).expect("write sweep");
                eprintln!("wrote {path}");
            }
        }
        return;
    }

    let find_max = if flag(&args, "--find-max") {
        Some(RampConfig {
            low: arg(&args, "--low", 200_000),
            high: arg(&args, "--high", 1_200_000),
            coarse_step: arg(&args, "--coarse-step", 100_000),
            tolerance: arg(&args, "--tolerance", 25_000),
            budget_us: TICK_BUDGET_US,
            metric: BudgetMetric::P99,
        })
    } else {
        None
    };

    let cfg = LabConfig {
        label: args
            .iter()
            .position(|a| a == "--label")
            .and_then(|i| args.get(i + 1))
            .cloned()
            .unwrap_or_else(|| format!("pop{pop}_seed{seed}")),
        spec: WorldSpec {
            population: pop,
            seed,
            half_extent_x: half_x,
            half_extent_y: half_y,
            distribution,
        },
        warmup,
        samples,
        dt,
        budget_us: TICK_BUDGET_US,
        metric: BudgetMetric::P99,
        find_max,
    };

    let report = run_lab(&cfg);

    eprintln!(
        "[{}] pop={} build={} within_budget={} p99_total={:.0}us wall_p99={:.0}us max_pop={:?}",
        report.label,
        report.spec.population,
        report.build_type,
        report.within_budget,
        report.samples.total_tick.p99,
        report.samples.wall_total.p99,
        report.max_pop,
    );

    if let Some(i) = args.iter().position(|a| a == "--out") {
        if let Some(path) = args.get(i + 1) {
            let json = serde_json::to_string_pretty(&report).expect("serialize report");
            std::fs::write(path, json).expect("write report");
            eprintln!("wrote {path}");
        }
    }
}
