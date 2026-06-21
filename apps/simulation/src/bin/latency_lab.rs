use speciate::bench_lab::budget::TICK_BUDGET_US;
use speciate::bench_lab::ramp::RampConfig;
use speciate::bench_lab::world::{Distribution, WorldSpec};
use speciate::bench_lab::{run_lab, BudgetMetric, LabConfig};

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

fn main() {
    let args: Vec<String> = std::env::args().collect();

    let pop: usize = arg(&args, "--pop", 200_000);
    let seed: u64 = arg(&args, "--seed", 1);
    let samples: usize = arg(&args, "--samples", 60);
    let warmup: usize = arg(&args, "--warmup", 20);
    let half_x: f32 = arg(&args, "--half-x", 2500.0);
    let half_y: f32 = arg(&args, "--half-y", 2000.0);
    let dt: f32 = arg(&args, "--dt", 0.05);

    let distribution = if flag(&args, "--clustered") {
        Distribution::Clustered {
            clusters: arg(&args, "--clusters", 32),
            spread: arg(&args, "--spread", 150.0),
        }
    } else {
        Distribution::Uniform
    };

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
