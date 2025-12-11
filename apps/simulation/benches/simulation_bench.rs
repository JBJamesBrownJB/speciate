use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use speciate::{CritBuilder, Simulation, SimulationBuilder};

// 22.2Hz tick rate → ~45ms per tick (matches TARGET_SIMULATION_HZ in napi_addon)
const TICK_DELTA: f32 = 0.045;

// Creature counts for scaling benchmarks (matches realistic production targets)
// 1K = baseline, 10K = current target, 50K/100K = stress, 200K = capacity limit
const SCALING_COUNTS: [usize; 5] = [1_000, 10_000, 50_000, 100_000, 200_000];

// Spawn extent matching NAPI spawn_creatures (±500 = 1000×1000 area)
const SPAWN_EXTENT: f32 = 500.0;

fn create_simulation_with_creatures(count: usize) -> Simulation {
    use rand::Rng;
    let mut sim = SimulationBuilder::new().build();
    let mut rng = rand::thread_rng();

    // Random spawn matching NAPI: (rand - 0.5) * 1000 = ±500 units
    // This matches apps/simulation/src/ipc/bridge/bevy_app.rs:108-109
    for _ in 0..count {
        let x = (rng.gen::<f32>() - 0.5) * (SPAWN_EXTENT * 2.0);
        let y = (rng.gen::<f32>() - 0.5) * (SPAWN_EXTENT * 2.0);

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

        // Warm-up: stabilize ECS archetypes (random spawn already spreads creatures)
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

// Perception neighbor selection benchmark (pseudo-random: first-k neighbors found)
// Sprint 16: Chose pseudo-random over topological (~10% faster, acceptable accuracy)
fn bench_perception(c: &mut Criterion) {
    use bevy_ecs::prelude::*;
    use speciate::simulation::spatial::SpatialGrid;

    let mut group = c.benchmark_group("perception");

    for count in [5_000, 10_000, 20_000] {
        let mut grid = SpatialGrid::with_default_bounds();
        let mut rng = rand::thread_rng();
        use rand::Rng;

        let mut entities_data: Vec<(Entity, f32, f32, f32, f32, f32, f32, f32)> = Vec::new();

        for i in 0..count {
            let x = (rng.gen::<f32>() - 0.5) * 1000.0;
            let y = (rng.gen::<f32>() - 0.5) * 1000.0;
            let radius = 1.0 + rng.gen::<f32>() * 2.0;
            let rotation = rng.gen::<f32>() * std::f32::consts::TAU;
            let range = 50.0 + rng.gen::<f32>() * 50.0;
            let fov = 120.0_f32.to_radians();
            let cos_half_fov = (fov / 2.0).cos();
            let cos_half_fov_sq = cos_half_fov * cos_half_fov;

            entities_data.push((
                Entity::from_raw(i as u32),
                x, y, radius,
                rotation.cos(), rotation.sin(),
                range, cos_half_fov_sq
            ));
        }

        grid.rebuild(entities_data.iter().map(|(e, x, y, r, _, _, _, _)| (*e, *x, *y, *r)));

        const CAPACITY: usize = 8;
        const MAX_OTHER_RADIUS: f32 = 5.0;

        group.bench_with_input(
            BenchmarkId::new("first_k_neighbors", count),
            &(&grid, &entities_data),
            |b, (grid, entities)| {
                b.iter(|| {
                    let mut total_neighbors = 0usize;
                    let mut cells: Vec<(f32, usize)> = Vec::with_capacity(64);

                    for (entity, x, y, self_radius, facing_x, facing_y, range, cos_half_fov_sq) in entities.iter() {
                        let query_radius = range + self_radius + MAX_OTHER_RADIUS;
                        let mut neighbors: [Entity; CAPACITY] = [Entity::PLACEHOLDER; CAPACITY];
                        let mut count = 0usize;

                        grid.collect_cells_sorted(*x, *y, query_radius, *facing_x, *facing_y, &mut cells);

                        'cell_loop: for &(_, cell_idx) in cells.iter() {
                            for proxy in grid.get_cell_proxies(cell_idx) {
                                if *entity == proxy.entity {
                                    continue;
                                }

                                let dx = proxy.x - x;
                                let dy = proxy.y - y;
                                let center_dist_sq = dx * dx + dy * dy;

                                let max_dist = range + self_radius + proxy.radius;
                                if center_dist_sq > max_dist * max_dist {
                                    continue;
                                }

                                let rough_dot = dx * facing_x + dy * facing_y;
                                if rough_dot <= 0.0 {
                                    continue;
                                }

                                if rough_dot * rough_dot >= cos_half_fov_sq * center_dist_sq {
                                    neighbors[count] = proxy.entity;
                                    count += 1;
                                    if count == CAPACITY {
                                        break 'cell_loop;
                                    }
                                }
                            }
                        }
                        total_neighbors += count;
                    }
                    black_box(total_neighbors)
                });
            },
        );
    }

    group.finish();
}

criterion_group!(benches, bench_tick_scaling, bench_spawn, bench_vector_ops, bench_perception);
criterion_main!(benches);
