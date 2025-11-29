use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use speciate::{CritBuilder, Simulation, SimulationBuilder};

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

fn bench_full_tick(c: &mut Criterion) {
    let mut group = c.benchmark_group("full_tick");

    for count in [100, 1_000, 5_000, 10_000] {
        let mut sim = create_simulation_with_creatures(count);

        for _ in 0..10 {
            sim.update(0.045);
        }

        group.bench_with_input(BenchmarkId::new("creatures", count), &count, |b, _| {
            b.iter(|| {
                sim.update(black_box(0.045));
            });
        });
    }

    group.finish();
}

fn bench_spawn(c: &mut Criterion) {
    let mut group = c.benchmark_group("spawn");

    group.bench_function("single_creature", |b| {
        let mut sim = SimulationBuilder::new().build();
        let mut x = 0.0_f32;

        b.iter(|| {
            let builder = CritBuilder::new()
                .at(x, 0.0)
                .with_all_capabilities();
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

fn bench_movement_scaling(c: &mut Criterion) {
    let mut group = c.benchmark_group("movement_scaling");

    for count in [1_000, 5_000, 10_000, 20_000] {
        let mut sim = create_simulation_with_creatures(count);

        for _ in 0..5 {
            sim.update(0.045);
        }

        group.bench_with_input(BenchmarkId::new("creatures", count), &count, |b, _| {
            b.iter(|| {
                sim.update(black_box(0.045));
            });
        });
    }

    group.finish();
}

fn bench_vector_ops(c: &mut Criterion) {
    use speciate::simulation::math::{clamp_force, magnitude, magnitude_sq, normalize};

    let mut group = c.benchmark_group("vector_ops");

    group.bench_function("magnitude_sq", |b| {
        b.iter(|| {
            black_box(magnitude_sq(black_box(3.0), black_box(4.0)))
        });
    });

    group.bench_function("magnitude", |b| {
        b.iter(|| {
            black_box(magnitude(black_box(3.0), black_box(4.0)))
        });
    });

    group.bench_function("normalize", |b| {
        b.iter(|| {
            black_box(normalize(black_box(3.0), black_box(4.0)))
        });
    });

    group.bench_function("clamp_force_under", |b| {
        b.iter(|| {
            black_box(clamp_force(black_box(3.0), black_box(4.0), black_box(10.0)))
        });
    });

    group.bench_function("clamp_force_over", |b| {
        b.iter(|| {
            black_box(clamp_force(black_box(6.0), black_box(8.0), black_box(5.0)))
        });
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_full_tick,
    bench_spawn,
    bench_movement_scaling,
    bench_vector_ops
);
criterion_main!(benches);
