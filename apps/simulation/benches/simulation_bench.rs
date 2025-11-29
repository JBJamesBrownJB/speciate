use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use speciate::{CritBuilder, Simulation, SimulationBuilder};

// 22.2Hz tick rate → ~45ms per tick (matches TARGET_SIMULATION_HZ in napi_addon)
const TICK_DELTA: f32 = 0.045;

// Creature counts for scaling benchmarks
// 1K = baseline, 10K = current target, 20K = stress test
const SCALING_COUNTS: [usize; 4] = [1_000, 5_000, 10_000, 20_000];

fn create_simulation_with_creatures(count: usize) -> Simulation {
    let mut sim = SimulationBuilder::new().build();

    for i in 0..count {
        let x = (i % 100) as f32 * 10.0;
        let y = (i / 100) as f32 * 10.0;

        let builder = CritBuilder::new()
            .at(x, y)
            .with_all_capabilities()
            .in_behavior(speciate::BehaviorMode::Wandering);
        sim.spawn_crit(builder);
    }

    sim
}

// Measures full simulation tick time as creature count scales.
// Key metric for capacity planning - 45ms budget means we can hit 22Hz.
fn bench_tick_scaling(c: &mut Criterion) {
    let mut group = c.benchmark_group("tick_scaling");

    for count in SCALING_COUNTS {
        let mut sim = create_simulation_with_creatures(count);

        // Warm-up: let ECS stabilize archetypes
        for _ in 0..10 {
            sim.update(TICK_DELTA);
        }

        group.bench_with_input(BenchmarkId::new("creatures", count), &count, |b, _| {
            b.iter(|| {
                sim.update(black_box(TICK_DELTA));
            });
        });
    }

    group.finish();
}

// Measures entity creation overhead (ECS archetype allocation).
// Single spawn = amortized cost, batch = bulk spawn performance.
fn bench_spawn(c: &mut Criterion) {
    let mut group = c.benchmark_group("spawn");

    group.bench_function("single_creature", |b| {
        let mut sim = SimulationBuilder::new().build();
        let mut x = 0.0_f32;

        b.iter(|| {
            let builder = CritBuilder::new().at(x, 0.0).with_all_capabilities();
            let id = sim.spawn_crit(builder);
            x += 1.0;
            black_box(id)
        });
    });

    group.bench_function("batch_100", |b| {
        b.iter(|| {
            let mut sim = SimulationBuilder::new().build();
            for i in 0..100 {
                let builder = CritBuilder::new()
                    .at(i as f32 * 10.0, 0.0)
                    .with_all_capabilities();
                sim.spawn_crit(builder);
            }
            black_box(sim.creature_count())
        });
    });

    group.finish();
}

// Microbenchmarks for hot-path math operations.
// These run millions of times per tick - nanoseconds matter.
fn bench_vector_ops(c: &mut Criterion) {
    use speciate::simulation::math::{clamp_force, magnitude, magnitude_sq, normalize};

    let mut group = c.benchmark_group("vector_ops");

    group.bench_function("magnitude_sq", |b| {
        b.iter(|| black_box(magnitude_sq(black_box(3.0), black_box(4.0))));
    });

    group.bench_function("magnitude", |b| {
        b.iter(|| black_box(magnitude(black_box(3.0), black_box(4.0))));
    });

    group.bench_function("normalize", |b| {
        b.iter(|| black_box(normalize(black_box(3.0), black_box(4.0))));
    });

    // Clamp when under limit (fast path - no clamping needed)
    group.bench_function("clamp_force_under", |b| {
        b.iter(|| black_box(clamp_force(black_box(3.0), black_box(4.0), black_box(10.0))));
    });

    // Clamp when over limit (slow path - needs normalization)
    group.bench_function("clamp_force_over", |b| {
        b.iter(|| black_box(clamp_force(black_box(6.0), black_box(8.0), black_box(5.0))));
    });

    group.finish();
}

criterion_group!(benches, bench_tick_scaling, bench_spawn, bench_vector_ops);
criterion_main!(benches);
