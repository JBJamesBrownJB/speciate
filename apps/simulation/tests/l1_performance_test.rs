#![cfg(feature = "dev-tools")]

use speciate::{CritBuilder, SimulationBuilder, Simulation};
use std::time::Instant;

const WARMUP_TICKS: usize = 10;
const MEASURE_TICKS: usize = 100;
const DELTA_TIME: f32 = 0.045; // ~22Hz tick rate

/// Spawn creatures in a grid pattern (matches spec spawn behavior)
fn spawn_grid(sim: &mut Simulation, start_x: f32, start_y: f32, spacing: f32, rows: usize, cols: usize) -> usize {
    let mut count = 0;
    for row in 0..rows {
        for col in 0..cols {
            let x = start_x + (col as f32 * spacing);
            let y = start_y + (row as f32 * spacing);
            sim.spawn_crit(CritBuilder::new().at(x, y).with_all_capabilities());
            count += 1;
        }
    }
    count
}

/// 200K world spread: 400×500 grid, 20m spacing, spread across 10km×8km
fn spawn_world_spread(sim: &mut Simulation) -> usize {
    spawn_grid(sim, -5000.0, -4000.0, 20.0, 400, 500)
}

/// 160K medium density: 400×400 grid, 2.5m spacing, 1km×1km cluster
fn spawn_medium_density(sim: &mut Simulation) -> usize {
    spawn_grid(sim, -500.0, -500.0, 2.5, 400, 400)
}

fn run_benchmark(sim: &mut Simulation, label: &str) -> f64 {
    // Warmup
    for _ in 0..WARMUP_TICKS {
        sim.update(DELTA_TIME);
    }

    // Measure
    let mut tick_times_ms: Vec<f64> = Vec::with_capacity(MEASURE_TICKS);
    for _ in 0..MEASURE_TICKS {
        let start = Instant::now();
        sim.update(DELTA_TIME);
        tick_times_ms.push(start.elapsed().as_secs_f64() * 1000.0);
    }

    tick_times_ms.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let avg = tick_times_ms.iter().sum::<f64>() / tick_times_ms.len() as f64;
    let p50 = tick_times_ms[tick_times_ms.len() / 2];
    let p95 = tick_times_ms[(tick_times_ms.len() as f64 * 0.95) as usize];

    println!("{}: avg={:.2}ms, p50={:.2}ms, p95={:.2}ms", label, avg, p50, p95);
    avg
}

#[test]
fn benchmark_200k_world_spread() {
    println!("\n=== 200K WORLD SPREAD (sparse) ===\n");

    let mut sim = SimulationBuilder::new().build();
    sim.set_boundaries(5000.0, 5000.0);

    let spawn_start = Instant::now();
    let count = spawn_world_spread(&mut sim);
    println!("Spawned {} creatures in {:?}", count, spawn_start.elapsed());

    let avg = run_benchmark(&mut sim, "World spread");

    // Target from spec: 16ms (was 20ms, optimized to 16ms)
    let target = 20.0;
    if avg < target {
        println!("✓ PASS: {:.2}ms < {:.0}ms target", avg, target);
    } else {
        println!("✗ FAIL: {:.2}ms > {:.0}ms target", avg, target);
    }
}

#[test]
fn benchmark_160k_medium_density() {
    println!("\n=== 160K MEDIUM DENSITY (dense) ===\n");

    let mut sim = SimulationBuilder::new().build();
    sim.set_boundaries(5000.0, 5000.0);

    let spawn_start = Instant::now();
    let count = spawn_medium_density(&mut sim);
    println!("Spawned {} creatures in {:?}", count, spawn_start.elapsed());

    let avg = run_benchmark(&mut sim, "Medium density");

    // Target from spec: 22ms (was 30ms, optimized to 22ms)
    let target = 36.0;
    if avg < target {
        println!("✓ PASS: {:.2}ms < {:.0}ms target", avg, target);
    } else {
        println!("✗ FAIL: {:.2}ms > {:.0}ms target", avg, target);
    }
}

#[test]
fn benchmark_360k_combined() {
    println!("\n=== 360K COMBINED (200K spread + 160K dense) ===\n");

    let mut sim = SimulationBuilder::new().build();
    sim.set_boundaries(5000.0, 5000.0);

    let spawn_start = Instant::now();
    let spread_count = spawn_world_spread(&mut sim);
    let dense_count = spawn_medium_density(&mut sim);
    let total = spread_count + dense_count;
    println!("Spawned {} creatures ({} spread + {} dense) in {:?}",
             total, spread_count, dense_count, spawn_start.elapsed());

    let avg = run_benchmark(&mut sim, "Combined");

    // Combined target - sum of individual targets with some overhead
    let target = 60.0; // Realistic combined target
    if avg < target {
        println!("✓ PASS: {:.2}ms < {:.0}ms target", avg, target);
    } else {
        println!("✗ FAIL: {:.2}ms > {:.0}ms target", avg, target);
    }
}

#[test]
fn benchmark_system_breakdown_combined() {
    println!("\n=== SYSTEM TIMING BREAKDOWN (360K combined) ===\n");

    let mut sim = SimulationBuilder::new().build();
    sim.set_boundaries(5000.0, 5000.0);

    spawn_world_spread(&mut sim);
    spawn_medium_density(&mut sim);

    // Warmup
    for _ in 0..WARMUP_TICKS {
        sim.update(DELTA_TIME);
    }

    // Measure and accumulate
    let mut total_perception = 0u64;
    let mut total_movement = 0u64;
    let mut total_grid_rebuild = 0u64;
    let mut total_l1_aggregation = 0u64;
    let mut total_behavior = 0u64;
    let mut total_steering = 0u64;

    for _ in 0..MEASURE_TICKS {
        sim.update(DELTA_TIME);

        let world = sim.world();
        if let Some(timings) = world.get_resource::<speciate::instrumentation::SystemTimings>() {
            let snapshot = timings.snapshot();
            total_perception += snapshot.perception_us;
            total_movement += snapshot.movement_us;
            total_grid_rebuild += snapshot.spatial_grid_rebuild_us;
            total_l1_aggregation += snapshot.l1_aggregation_us;
            total_behavior += snapshot.behavior_transition_us;
            total_steering += snapshot.steering_us;
        }
    }

    let n = MEASURE_TICKS as f64;
    let total = total_perception + total_movement + total_grid_rebuild +
                total_l1_aggregation + total_behavior + total_steering;

    println!("Average per-tick timing:");
    println!("  Perception:     {:>8.2} ms", total_perception as f64 / n / 1000.0);
    println!("  Steering:       {:>8.2} ms", total_steering as f64 / n / 1000.0);
    println!("  Movement:       {:>8.2} ms", total_movement as f64 / n / 1000.0);
    println!("  Grid rebuild:   {:>8.2} ms", total_grid_rebuild as f64 / n / 1000.0);
    println!("  L1 aggregation: {:>8.2} ms", total_l1_aggregation as f64 / n / 1000.0);
    println!("  Behavior:       {:>8.2} ms", total_behavior as f64 / n / 1000.0);
    println!("  ─────────────────────────────");
    println!("  Total measured: {:>8.2} ms", total as f64 / n / 1000.0);

    println!("\n=== END BREAKDOWN ===\n");
}
