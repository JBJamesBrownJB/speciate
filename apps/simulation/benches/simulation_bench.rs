use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use speciate::{CritBuilder, Simulation, SimulationBuilder};
use speciate::simulation::creatures::dna::Dna;

// 22.2Hz tick rate → ~45ms per tick (matches TARGET_SIMULATION_HZ in napi_addon)
const TICK_DELTA: f32 = 0.045;

// Creature counts for scaling benchmarks (matches realistic production targets)
// 1K = baseline, 10K = current target, 50K/100K = stress, 200K = capacity limit
const SCALING_COUNTS: [usize; 5] = [1_000, 10_000, 50_000, 100_000, 200_000];

// Spawn extent matching NAPI spawn_creatures (±500 = 1000×1000 area)
const SPAWN_EXTENT: f32 = 500.0;

// Large spawn extent for 100K+ random DNA benchmarks (~5km x 4km realistic gameplay)
const LARGE_SPAWN_EXTENT_X: f32 = 2500.0;
const LARGE_SPAWN_EXTENT_Y: f32 = 2000.0;

fn create_simulation_with_creatures(count: usize) -> Simulation {
    use rand::Rng;
    let mut sim = SimulationBuilder::new()
        .set_boundaries(SPAWN_EXTENT * 2.0, SPAWN_EXTENT * 2.0)
        .build();
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

/// Create simulation with randomized DNA (size 0.1-10m, FOV narrow/medium/wide)
/// Uses larger spawn extent (~5km x 4km) matching realistic 100K gameplay scenarios
fn create_simulation_with_random_dna(count: usize) -> Simulation {
    use rand::Rng;
    let mut sim = SimulationBuilder::new()
        .set_boundaries(LARGE_SPAWN_EXTENT_X * 2.0, LARGE_SPAWN_EXTENT_Y * 2.0)
        .build();
    let mut rng = rand::thread_rng();

    for _ in 0..count {
        let x = (rng.gen::<f32>() - 0.5) * (LARGE_SPAWN_EXTENT_X * 2.0);
        let y = (rng.gen::<f32>() - 0.5) * (LARGE_SPAWN_EXTENT_Y * 2.0);

        // Random DNA: size gene and FOV gene both 0.0-1.0
        // This exercises the full range of creature sizes and perception configs
        let dna = Dna::random();

        let builder = CritBuilder::new()
            .at(x, y)
            .with_dna(dna)
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
                x,
                y,
                radius,
                rotation.cos(),
                rotation.sin(),
                range,
                cos_half_fov_sq,
            ));
        }

        // Use rebuild_parallel with fixed bounds (rebuild is cfg(test) only)
        grid.rebuild_parallel(
            entities_data
                .iter()
                .map(|(e, x, y, r, _, _, _, _)| (*e, *x, *y, 0.0, 0.0, *r)),
        );

        const CAPACITY: usize = 8;
        const MAX_OTHER_RADIUS: f32 = 5.0;

        group.bench_with_input(
            BenchmarkId::new("first_k_neighbors", count),
            &(&grid, &entities_data),
            |b, (grid, entities)| {
                b.iter(|| {
                    let mut total_neighbors = 0usize;
                    let mut cells: Vec<(f32, usize)> = Vec::with_capacity(64);

                    for (entity, x, y, self_radius, facing_x, facing_y, range, cos_half_fov_sq) in
                        entities.iter()
                    {
                        let query_radius = range + self_radius + MAX_OTHER_RADIUS;
                        let mut neighbors: [Entity; CAPACITY] = [Entity::PLACEHOLDER; CAPACITY];
                        let mut count = 0usize;

                        grid.collect_cells_sorted(
                            *x,
                            *y,
                            query_radius,
                            *facing_x,
                            *facing_y,
                            &mut cells,
                        );

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

// Benchmark for sorting creature export buffer by CritId
// This measures the cost of stable ordering for ghost-crits fix (Sprint 16)
fn bench_export_sort(c: &mut Criterion) {
    use rayon::prelude::*;

    let mut group = c.benchmark_group("export_sort");

    // Test at realistic population sizes including 400K
    for count in [10_000, 50_000, 100_000, 200_000, 400_000] {
        // Simulate export_positions data: (CritId, x, y, rotation)
        // CritIds are sequential but query order is scrambled (simulating ECS reordering)
        let mut rng = rand::thread_rng();
        use rand::Rng;

        let mut data: Vec<(u64, f32, f32, f32)> = (0..count as u64)
            .map(|id| {
                (
                    id,
                    (rng.gen::<f32>() - 0.5) * 1000.0,
                    (rng.gen::<f32>() - 0.5) * 1000.0,
                    rng.gen::<f32>() * std::f32::consts::TAU,
                )
            })
            .collect();

        // Scramble to simulate ECS query order instability
        use rand::seq::SliceRandom;
        data.shuffle(&mut rng);

        // Sequential sort
        group.bench_with_input(BenchmarkId::new("sequential", count), &data, |b, data| {
            b.iter_batched(
                || data.clone(),
                |mut d| {
                    d.sort_unstable_by_key(|(id, _, _, _)| *id);
                    black_box(d)
                },
                criterion::BatchSize::LargeInput,
            );
        });

        // Parallel sort (Rayon)
        group.bench_with_input(BenchmarkId::new("parallel", count), &data, |b, data| {
            b.iter_batched(
                || data.clone(),
                |mut d| {
                    d.par_sort_unstable_by_key(|(id, _, _, _)| *id);
                    black_box(d)
                },
                criterion::BatchSize::LargeInput,
            );
        });
    }

    group.finish();
}

/// 100K creatures with randomized DNA - primary optimization target
/// Uses realistic creature distribution (size 0.1-10m, varied FOV)
/// and large world (~5km x 4km) matching production scenarios
fn bench_100k_random_dna(c: &mut Criterion) {
    let mut group = c.benchmark_group("random_dna_100k");

    // Configure for heavy benchmark
    group.sample_size(20);
    group.measurement_time(std::time::Duration::from_secs(30));

    let mut sim = create_simulation_with_random_dna(100_000);

    // Warm-up: stabilize ECS archetypes and spatial grid
    for _ in 0..10 {
        sim.update(TICK_DELTA);
    }

    group.bench_function("tick", |b| {
        b.iter(|| {
            sim.update(black_box(TICK_DELTA));
        });
    });

    group.finish();
}

/// Per-system breakdown at 100K for optimization hunting
/// Run with: cargo bench --bench simulation_bench -- random_dna_scaling
fn bench_random_dna_scaling(c: &mut Criterion) {
    let mut group = c.benchmark_group("random_dna_scaling");

    for count in [50_000, 100_000, 150_000] {
        let mut sim = create_simulation_with_random_dna(count);

        // Warm-up
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

/// Dense crowd tick benchmark — regression guard for the 180° oscillation fix.
///
/// Packs creatures into a tight grid so many are simultaneously below STOPPED_THRESHOLD
/// and facing forces in conflicting directions. Before the fix, these creatures would
/// 180°-snap every tick (NaN bypass). After the fix, heading is rate-limited from
/// rotation.radians, eliminating the spam. This bench catches any regression where
/// turn-rate-limiting is removed or disabled for stopped creatures.
///
/// Run with: cargo bench --bench simulation_bench -- dense_crowd
fn bench_dense_crowd_turn_limiting(c: &mut Criterion) {
    let mut group = c.benchmark_group("dense_crowd");
    group.sample_size(20);
    group.measurement_time(std::time::Duration::from_secs(15));

    // 5 000 creatures packed into a 100×100 grid (2 m spacing) — guaranteed crowd collisions
    let count = 5_000;
    let spacing = 2.0_f32;
    let side = (count as f32).sqrt() as usize;
    let world_size = side as f32 * spacing * 2.0;

    let mut sim = SimulationBuilder::new()
        .set_boundaries(world_size, world_size)
        .build();

    for row in 0..side {
        for col in 0..side {
            let x = (col as f32 - side as f32 * 0.5) * spacing;
            let y = (row as f32 - side as f32 * 0.5) * spacing;
            let builder = CritBuilder::new()
                .at(x, y)
                .with_all_capabilities()
                .in_behavior(speciate::BehaviorMode::Wandering);
            sim.spawn_crit(builder);
        }
    }

    // Warm-up: let creatures interact and reach sub-threshold speeds
    for _ in 0..20 {
        sim.update(TICK_DELTA);
    }

    group.bench_function("tick_5k_packed", |b| {
        b.iter(|| {
            sim.update(black_box(TICK_DELTA));
        });
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_tick_scaling,
    bench_spawn,
    bench_vector_ops,
    bench_perception,
    bench_export_sort,
    bench_100k_random_dna,
    bench_random_dna_scaling,
    bench_dense_crowd_turn_limiting
);
criterion_main!(benches);
