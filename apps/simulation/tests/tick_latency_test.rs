//! Tick latency profiling test.
//!
//! Replicates the user's scenario: 200k world spread + 160k medium density = 360k total
//! Target: <50ms per tick
//!
//! Build: cargo build --release --features test-helpers --test tick_latency_test
//! Run:   ./target/release/deps/tick_latency_test-* --nocapture

use rand::Rng;
use speciate::simulation::creatures::dna::Dna;
use speciate::{BehaviorMode, CritBuilder, SimulationBuilder};
use std::time::Instant;

const WORLD_SPREAD_COUNT: usize = 200_000;
const MEDIUM_DENSITY_COUNT: usize = 160_000;
const WARMUP_TICKS: usize = 10;
const MEASURE_TICKS: usize = 50;
const TICK_DELTA: f32 = 0.045;

#[test]
fn measure_tick_latency_360k() {
    eprintln!("=== Tick Latency Test (360k creatures) ===");

    let mut sim = SimulationBuilder::new()
        .set_boundaries(5000.0, 5000.0)
        .build();
    let mut rng = rand::thread_rng();

    // Phase 1: World spread (200k) - spread across entire world WITH RANDOM DNA
    eprintln!("Spawning {} world-spread creatures (random DNA)...", WORLD_SPREAD_COUNT);
    for i in 0..WORLD_SPREAD_COUNT {
        let x = (rng.gen::<f32>() - 0.5) * 10000.0;
        let y = (rng.gen::<f32>() - 0.5) * 10000.0;
        let builder = CritBuilder::new()
            .at(x, y)
            .with_dna(Dna::random())
            .with_all_capabilities()
            .in_behavior(BehaviorMode::Wandering);
        sim.spawn_crit(builder);

        if i % 50000 == 0 && i > 0 {
            eprintln!("  Spawned {}/{}", i, WORLD_SPREAD_COUNT);
        }
    }

    // Phase 2: Medium density (160k) - clustered in central region WITH RANDOM DNA
    eprintln!("Spawning {} medium-density creatures (random DNA)...", MEDIUM_DENSITY_COUNT);
    for i in 0..MEDIUM_DENSITY_COUNT {
        // Medium density: spawn in a 4000x4000 central region
        let x = (rng.gen::<f32>() - 0.5) * 4000.0;
        let y = (rng.gen::<f32>() - 0.5) * 4000.0;
        let builder = CritBuilder::new()
            .at(x, y)
            .with_dna(Dna::random())
            .with_all_capabilities()
            .in_behavior(BehaviorMode::Wandering);
        sim.spawn_crit(builder);

        if i % 50000 == 0 && i > 0 {
            eprintln!("  Spawned {}/{}", i, MEDIUM_DENSITY_COUNT);
        }
    }

    let total = WORLD_SPREAD_COUNT + MEDIUM_DENSITY_COUNT;
    eprintln!("Total creatures: {}", total);

    // Warmup
    eprintln!("Warmup ({} ticks)...", WARMUP_TICKS);
    for _ in 0..WARMUP_TICKS {
        sim.update(TICK_DELTA);
    }

    // Measure (including simulated export overhead)
    eprintln!("Measuring {} ticks (with export simulation)...", MEASURE_TICKS);
    let mut tick_times: Vec<f64> = Vec::with_capacity(MEASURE_TICKS);

    // Pre-allocate export buffer to simulate real app behavior
    let mut export_buffer: Vec<f32> = vec![0.0; total * 5]; // ID, X, Y, Rot, Size

    for i in 0..MEASURE_TICKS {
        let start = Instant::now();

        // 1. Simulation update
        sim.update(TICK_DELTA);

        // 2. Simulate export overhead (collect, sort, copy)
        // This approximates what export_positions does in the real app
        let world = sim.world_mut();
        use speciate::simulation::core::components::{Position, Rotation, BodySize};
        use speciate::simulation::creatures::components::CritId;
        use rayon::prelude::*;

        let mut query = world.query::<(&CritId, &Position, &Rotation, &BodySize)>();
        let mut entities: Vec<_> = query.iter(world).collect();
        entities.par_sort_unstable_by_key(|(id, _, _, _)| id.0);

        // Simulate buffer copy
        for (idx, (id, pos, rot, size)) in entities.iter().enumerate() {
            if idx >= export_buffer.len() / 5 { break; }
            export_buffer[idx] = id.0 as f32;
            export_buffer[total + idx] = pos.x;
            export_buffer[total * 2 + idx] = pos.y;
            export_buffer[total * 3 + idx] = rot.radians;
            export_buffer[total * 4 + idx] = size.length;
        }

        let elapsed = start.elapsed().as_secs_f64() * 1000.0;
        tick_times.push(elapsed);

        if i % 10 == 0 {
            eprintln!("  Tick {}: {:.2}ms", i, elapsed);
        }
    }

    // Statistics
    tick_times.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let min = tick_times[0];
    let max = tick_times[tick_times.len() - 1];
    let median = tick_times[tick_times.len() / 2];
    let avg: f64 = tick_times.iter().sum::<f64>() / tick_times.len() as f64;
    let p95 = tick_times[(tick_times.len() as f64 * 0.95) as usize];
    let p99 = tick_times[(tick_times.len() as f64 * 0.99) as usize];

    eprintln!("\n=== Results ===");
    eprintln!("Creatures: {}", total);
    eprintln!("Min:    {:.2}ms", min);
    eprintln!("Max:    {:.2}ms", max);
    eprintln!("Median: {:.2}ms", median);
    eprintln!("Avg:    {:.2}ms", avg);
    eprintln!("P95:    {:.2}ms", p95);
    eprintln!("P99:    {:.2}ms", p99);
    eprintln!("Target: <50ms");
    eprintln!("Status: {}", if median < 50.0 { "PASS" } else { "FAIL" });
}

#[test]
fn measure_tick_latency_200k() {
    eprintln!("=== Tick Latency Test (200k creatures) ===");

    let mut sim = SimulationBuilder::new()
        .set_boundaries(5000.0, 5000.0)
        .build();
    let mut rng = rand::thread_rng();

    eprintln!("Spawning 200k world-spread creatures...");
    for i in 0..200_000 {
        let x = (rng.gen::<f32>() - 0.5) * 10000.0;
        let y = (rng.gen::<f32>() - 0.5) * 10000.0;
        let builder = CritBuilder::new()
            .at(x, y)
            .with_all_capabilities()
            .in_behavior(BehaviorMode::Wandering);
        sim.spawn_crit(builder);
    }

    // Warmup
    for _ in 0..WARMUP_TICKS {
        sim.update(TICK_DELTA);
    }

    // Measure
    let mut tick_times: Vec<f64> = Vec::with_capacity(MEASURE_TICKS);
    for _ in 0..MEASURE_TICKS {
        let start = Instant::now();
        sim.update(TICK_DELTA);
        tick_times.push(start.elapsed().as_secs_f64() * 1000.0);
    }

    tick_times.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let median = tick_times[tick_times.len() / 2];
    let avg: f64 = tick_times.iter().sum::<f64>() / tick_times.len() as f64;
    let p99 = tick_times[(tick_times.len() as f64 * 0.99) as usize];

    eprintln!("\n=== Results (200k) ===");
    eprintln!("Median: {:.2}ms, Avg: {:.2}ms, P99: {:.2}ms", median, avg, p99);
}
