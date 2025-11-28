# Sprint 16: Spatial Grid + Parallel Perception

**Branch:** (TBD - will create when sprint starts)
**Status:** PLANNED
**Prerequisites:** Sprint 15 complete (ECS optimizations)
**Duration:** 5 days
**Owner:** ecs-emma + rusty-ron + instrumentation-ian

---

## Sprint Goal

Break the O(N²) perception bottleneck via:
1. **Spatial Grid:** O(N²) → O(N×k) complexity reduction (833x fewer operations @ 150K)
2. **Rayon Parallelization:** Multi-core acceleration on grid queries

**Target:** 150K-200K creatures @ <45ms tick time with perception <10ms

---

## Decision Point

Sprint 16 is **CONDITIONAL** based on Sprint 15 Phase 2D validation results:

**Trigger A:** Sprint 15 fails to achieve 150K creatures @ <45ms tick time
→ Spatial grid is MANDATORY (algorithmic bottleneck must be fixed)

**Trigger B:** Perception still >40% of frame budget @ 150K after all Sprint 15 optimizations
→ Spatial grid recommended for scaling headroom

**Skip Condition:** Sprint 15 achieves 200K @ <45ms with perception <30%
→ Defer spatial grid, proceed with other features (organic shaders, etc.)

---

## Critical Context: Why Rayon Alone Fails

### The O(N²) Problem

**Current Algorithm (Sequential):**
```rust
// Build scratch buffer - O(n)
for entity in all_creatures {
    scratch.positions.push((entity, pos.x, pos.y, radius));
}

// Check every creature against every other creature - O(n²)
for entity in all_creatures {
    for other in scratch.positions {  // Inner loop: n iterations
        if distance_check(entity, other) {
            perception.add_neighbor(other);
        }
    }
}
```

**Performance at Scale:**

| Creatures | Comparisons | Sequential | Parallel (8 cores) | Budget | Status |
|-----------|-------------|------------|--------------------| -------|--------|
| 5K | 25M | 34ms | ~5ms | 45ms | ✅ OK |
| 25K | 625M | ~850ms | ~106ms | 45ms | ❌ 2.4x over |
| 50K | 2.5B | ~3,400ms | ~425ms | 45ms | ❌ 9.4x over |
| 150K | 22.5B | ~30,600ms | ~3,825ms | 45ms | ❌ **85x over** |

**Conclusion:** Even with perfect 8-core Rayon parallelization, O(N²) fails at 25K creatures. Must fix algorithmic complexity BEFORE parallelizing.

---

## Architecture: Spatial Grid

**See:** `docs/architecture/spatial-partitioning.md` for full specification

### Complexity Reduction

- **Current:** O(n²) = 150K × 150K = 22.5 billion comparisons
- **With Grid:** O(n × k) where k ≈ 180 avg neighbors = 150K × 180 = 27 million comparisons
- **Reduction:** 833x fewer operations

### Data Structure

```rust
use rustc_hash::FxHashMap;

#[derive(Resource)]
pub struct SpatialGrid {
    cell_size: f32,              // 200.0m
    inv_cell_size: f32,          // 1.0 / 200.0 (multiply is faster than divide)
    cells: FxHashMap<(i32, i32), Vec<Entity>>,  // Grid cells
    positions: FxHashMap<Entity, (f32, f32, f32)>,  // (x, y, radius) cache
}

impl SpatialGrid {
    pub fn query_radius_iter(&self, x: f32, y: f32, radius: f32)
        -> impl Iterator<Item = Entity> + '_
    {
        // Calculate grid cell range (typically 3×3 = 9 cells)
        let min_cell = self.world_to_cell(x - radius, y - radius);
        let max_cell = self.world_to_cell(x + radius, y + radius);

        // Iterate entities in nearby cells only
        (min_cell.0..=max_cell.0)
            .flat_map(|cx| {
                (min_cell.1..=max_cell.1).filter_map(move |cy| {
                    self.cells.get(&(cx, cy))
                })
            })
            .flatten()
            .copied()
    }

    #[inline]
    pub fn world_to_cell(&self, x: f32, y: f32) -> (i32, i32) {
        (
            (x * self.inv_cell_size).floor() as i32,
            (y * self.inv_cell_size).floor() as i32,
        )
    }

    pub fn get_position(&self, entity: Entity) -> (f32, f32, f32) {
        self.positions[&entity]
    }
}
```

### Optimized Perception System

```rust
pub fn update_vision_system(
    query: Query<(Entity, &Position, &BodySize, &mut Perception, &CreatureState)>,
    grid: Res<SpatialGrid>,
) {
    // Collect entity data for parallel processing (O(n), ~1ms @ 150K)
    let entity_data: Vec<_> = query
        .iter()
        .map(|(e, pos, size, _, state)| {
            (e, pos.x, pos.y, size.radius(), state.behavior.is_active())
        })
        .collect();

    // Parallel perception computation (O(n × k), ~20ms @ 150K on 8 cores)
    let results: Vec<_> = entity_data
        .par_iter()  // Rayon parallel iterator
        .filter(|(_, _, _, _, is_active)| *is_active)
        .map(|&(entity, pos_x, pos_y, self_radius, _)| {
            let range = /* perception range */;
            let mut neighbors = [Entity::PLACEHOLDER; MAX_PERCEIVED_NEIGHBORS];
            let mut count: u8 = 0;

            // Spatial query: O(9 cells × ~20 entities/cell) = ~180 checks
            for candidate in grid.query_radius_iter(pos_x, pos_y, range) {
                if candidate == entity || count >= MAX_PERCEIVED_NEIGHBORS as u8 {
                    continue;
                }
                // Distance check (from grid's position cache)
                let (cx, cy, c_radius) = grid.get_position(candidate);
                let dx = cx - pos_x;
                let dy = cy - pos_y;
                let max_dist = range + c_radius;
                if dx * dx + dy * dy <= max_dist * max_dist {
                    neighbors[count as usize] = candidate;
                    count += 1;
                }
            }
            (entity, neighbors, count)
        })
        .collect();

    // Apply results (sequential, O(n), ~2ms @ 150K)
    for (entity, neighbors, count) in results {
        if let Ok((_, _, _, mut perception, _)) = query.get_mut(entity) {
            perception.set_neighbors_raw(neighbors, count);
        }
    }
}
```

---

## Implementation Plan (TDD)

### Day 1-2: Spatial Grid Foundation

**RED Phase:**

```rust
#[test]
fn test_spatial_grid_query_returns_nearby_entities() {
    let mut grid = SpatialGrid::new(200.0);

    let e1 = Entity::from_raw(1);
    let e2 = Entity::from_raw(2);
    let e3 = Entity::from_raw(3);

    grid.insert(e1, 0.0, 0.0, 5.0);
    grid.insert(e2, 50.0, 0.0, 5.0);   // Within 100m
    grid.insert(e3, 500.0, 0.0, 5.0);  // Outside 100m

    let results: Vec<_> = grid.query_radius_iter(0.0, 0.0, 100.0).collect();

    assert!(results.contains(&e1));
    assert!(results.contains(&e2));
    assert!(!results.contains(&e3));
}

#[test]
fn test_spatial_grid_empty_query_returns_nothing() {
    let grid = SpatialGrid::new(200.0);
    let results: Vec<_> = grid.query_radius_iter(0.0, 0.0, 100.0).collect();
    assert!(results.is_empty());
}

#[test]
fn test_spatial_grid_boundary_conditions() {
    let mut grid = SpatialGrid::new(200.0);
    let e1 = Entity::from_raw(1);

    // Entity at cell boundary (199.9, 0.0)
    grid.insert(e1, 199.9, 0.0, 5.0);

    // Query from origin should find it
    let results: Vec<_> = grid.query_radius_iter(0.0, 0.0, 200.0).collect();
    assert!(results.contains(&e1));
}
```

**GREEN Phase - Tasks:**

- [ ] Implement `SpatialGrid::new(cell_size: f32)`
- [ ] Implement `insert(entity, x, y, radius)` - Add to grid
- [ ] Implement `remove(entity)` - Remove from grid
- [ ] Implement `query_radius_iter(x, y, radius)` - Iterator over nearby entities
- [ ] Implement `clear()` - Reset grid (per-tick)
- [ ] Implement `world_to_cell(x, y)` - Coordinate conversion
- [ ] Implement `get_position(entity)` - Position cache lookup

**REFACTOR Phase - Optimizations:**

- [ ] Use `FxHashMap` for cell storage (2-5x faster than std HashMap)
- [ ] Cache `inv_cell_size` (avoid division)
- [ ] Pre-allocate cell vectors with `with_capacity(32)`
- [ ] Unit test grid at 1K, 10K, 100K entities
- [ ] Benchmark grid insertion/query performance

**Owner:** rusty-ron + ecs-emma

---

### Day 3: Integration with Perception System

**RED Phase:**

```rust
#[test]
fn test_perception_with_grid_handles_25k_under_20ms() {
    let mut world = setup_world_with_grid(25_000);

    let start = Instant::now();
    update_perception_system(/* ... */);
    let elapsed = start.elapsed();

    assert!(elapsed < Duration::from_millis(20),
        "Perception took {:?}, expected <20ms", elapsed);
}

#[test]
fn test_grid_perception_matches_naive_results() {
    // Verify grid produces identical neighbor lists to naive O(n²)
    let world = setup_test_world(100);

    let naive_results = run_naive_perception(&world);
    let grid_results = run_grid_perception(&world);

    assert_eq!(naive_results, grid_results);
}
```

**GREEN Phase - Tasks:**

- [ ] Create `rebuild_spatial_grid_system()` (runs BEFORE perception)
  - Clears grid
  - Repopulates from Position/BodySize query
- [ ] Add `SpatialGrid` as Bevy `Resource`
- [ ] Update `update_perception_system()` to use `grid.query_radius_iter()`
- [ ] Remove `PerceptionScratchBuffer` (obsolete)
- [ ] Update system ordering: `rebuild_spatial_grid_system.before(update_perception_system)`

**REFACTOR Phase - Optimizations:**

- [ ] Batch position updates
- [ ] Measure rebuild cost (target: <2ms @ 150K)
- [ ] Verify behavior matches old system (no regressions)
- [ ] Benchmark perception time @ 25K, 50K, 100K

**Owner:** ecs-emma + rusty-ron

---

### Day 4: Add Rayon Parallelization

**Setup:**

Add to `apps/simulation/Cargo.toml`:
```toml
[dependencies]
rayon = "1.10"
rustc-hash = "2.0"  # FxHashMap (faster than std HashMap)
```

**RED Phase:**

```rust
#[test]
fn test_parallel_perception_deterministic() {
    // Same seed should produce identical results
    let results1 = run_perception_parallel(seed: 42);
    let results2 = run_perception_parallel(seed: 42);
    assert_eq!(results1, results2);
}

#[test]
fn test_parallel_perception_no_race_conditions() {
    // Run 100 times, verify consistent results
    let expected = run_perception_parallel(seed: 42);
    for _ in 0..100 {
        let result = run_perception_parallel(seed: 42);
        assert_eq!(result, expected);
    }
}
```

**GREEN Phase - Tasks:**

- [ ] Collect entity data into `Vec<(Entity, f32, f32, f32, bool)>`
- [ ] Replace `.iter()` with `.par_iter()` on entity_data
- [ ] Ensure grid is read-only during parallel phase
- [ ] Collect results into `Vec<(Entity, [Entity; 40], u8)>`
- [ ] Sequential apply phase (write to `Perception` component)

**REFACTOR Phase - Tuning:**

- [ ] Test `min_batch_size(64)` for <10K creatures
- [ ] Test `min_batch_size(256)` for 10K-50K creatures
- [ ] Test `min_batch_size(512)` for 150K+ creatures
- [ ] Choose optimal based on benchmarks
- [ ] Verify determinism (same seed = same results)

**Owner:** rusty-ron + instrumentation-ian

---

### Day 5: Benchmarking & Validation

**Create Benchmark Suite:**

File: `apps/simulation/benches/perception.rs`

```rust
use criterion::{criterion_group, criterion_main, Criterion, BenchmarkId};

fn perception_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("perception");

    for size in [1_000, 5_000, 25_000, 100_000, 150_000] {
        // Baseline: O(n²) sequential
        group.bench_with_input(
            BenchmarkId::new("sequential_naive", size),
            &size,
            |b, &size| {
                let world = setup_world_naive(size);
                b.iter(|| run_perception_naive(&world));
            },
        );

        // Optimized: Spatial grid sequential
        group.bench_with_input(
            BenchmarkId::new("sequential_grid", size),
            &size,
            |b, &size| {
                let world = setup_world_with_grid(size);
                b.iter(|| run_perception_with_grid(&world));
            },
        );

        // Optimized: Spatial grid + Rayon parallel
        group.bench_with_input(
            BenchmarkId::new("parallel_grid", size),
            &size,
            |b, &size| {
                let world = setup_world_with_grid(size);
                b.iter(|| run_perception_parallel(&world));
            },
        );
    }

    group.finish();
}

criterion_group!(benches, perception_benchmark);
criterion_main!(benches);
```

**Run Command:**
```bash
cd apps/simulation
cargo bench --bench perception
```

**Expected Output:**
```
perception/sequential_naive/150000    time: [30.6 s]
perception/sequential_grid/150000     time: [40 ms]    (765x speedup)
perception/parallel_grid/150000       time: [7 ms]     (4,371x speedup)
```

**Validation Tasks:**

- [ ] Create benchmark suite (criterion)
- [ ] Benchmark @ 1K, 5K, 25K, 50K, 100K, 150K, 200K creatures
- [ ] Capture hardware metrics (IPC, L1/L2 cache hit rates, CPU utilization)
- [ ] Verify 150K @ <10ms perception (parallel grid)
- [ ] Verify no behavior regressions (all tests pass)
- [ ] Verify no memory leaks (grid cleanup)
- [ ] Update `docs/performance/optimization-backlog.md`
- [ ] Create Sprint 16 completion report

**Owner:** instrumentation-ian + ecs-emma

---

## Performance Projections

### Spatial Grid Alone (Sequential)

| Creatures | Comparisons | Time | vs Current | Fits Budget? |
|-----------|-------------|------|------------|--------------|
| 5K | 900K | ~1ms | 34x faster | ✅ Yes |
| 25K | 4.5M | ~6ms | 142x faster | ✅ Yes |
| 50K | 9M | ~12ms | 283x faster | ✅ Yes |
| 150K | 27M | ~40ms | 765x faster | ✅ Yes (barely) |

### Spatial Grid + Rayon (8 cores)

| Creatures | Comparisons | Time (8 cores) | vs Current | Headroom |
|-----------|-------------|----------------|------------|----------|
| 5K | 900K | ~0.5ms | 68x faster | 90x budget |
| 25K | 4.5M | ~2ms | 425x faster | 22x budget |
| 50K | 9M | ~3ms | 1,133x faster | 15x budget |
| 150K | 27M | ~7ms | 4,371x faster | **6.4x budget** |
| 200K | 36M | ~10ms | 8,500x faster | **4.5x budget** |

### With Stochastic Vision (10% per tick)

**If combined with Sprint 15's stochastic vision updates:**

- 150K creatures: ~0.7ms perception (64x budget remaining!)
- 200K creatures: ~1.0ms perception (45x budget remaining!)

---

## Success Criteria

**Sprint 16 Complete When:**

- [ ] Spatial grid handles 150K creatures @ <40ms perception (sequential)
- [ ] Parallel grid handles 150K creatures @ <10ms perception (8 cores)
- [ ] Benchmarks show 500x+ speedup vs naive O(n²) at 150K
- [ ] Zero regression in perception behavior (all tests pass)
- [ ] Cache hit rates improved (hardware profiling confirms)
- [ ] No memory leaks (grid properly cleared each tick)
- [ ] 200K creatures achievable @ <50ms tick time

**Target Metrics:**

| Metric | Before (O(n²)) | After (Grid + Rayon) | Improvement |
|--------|----------------|----------------------|-------------|
| Perception @ 5K | 34ms | <1ms | 34x faster |
| Perception @ 25K | ~850ms (projected) | ~2ms | 425x faster |
| Perception @ 150K | ~30,600ms (projected) | ~7ms | 4,371x faster |
| Max creatures @ 45ms | ~5K | 200K+ | **40x capacity** |

---

## ECS Performance Considerations

### Archetype Stability

- Spatial grid is a `Resource`, not a `Component` - no archetype impact
- Grid rebuild happens once per tick (before perception) - no per-entity churn

### Cache Coherency

- Grid stores position cache: `FxHashMap<Entity, (f32, f32, f32)>`
- Sequential access pattern during grid rebuild (good cache locality)
- Random access during query (mitigated by small cell count, L1 cache friendly)

### Bevy Batch Size Tuning

- For 150K entities, use `min_batch_size(512)` to reduce work-stealing overhead
- Below 10K entities, use `min_batch_size(64)` for better load balancing

### Trade-offs

- **Memory:** ~10-20MB for grid at 150K creatures (acceptable)
- **Rebuild cost:** ~1-2ms per tick to rebuild grid (small compared to 40ms savings)
- **Complexity:** More code to maintain (but critical for scaling)

---

## Risks & Mitigations

**Risk:** Grid rebuild overhead negates gains

- **Mitigation:** Benchmark rebuild separately, target <2ms @ 150K
- **Fallback:** Incremental grid updates (only move changed entities)
- **Reference:** `docs/architecture/spatial-partitioning.md` describes incremental updates

**Risk:** Cell size incorrect (too large/small)

- **Mitigation:** Empirical testing with 100m, 200m, 300m, 500m
- **Validation:** Benchmark perception time at each cell size
- **Expected:** 200m is optimal per existing spec

**Risk:** Rayon overhead exceeds benefits at low creature counts

- **Mitigation:** Conditional parallelization (only above 10K creatures)
- **Code:** `if entity_count > 10_000 { par_iter } else { iter }`

**Risk:** Race conditions in parallel perception

- **Mitigation:** Grid is read-only during parallel phase
- **Testing:** Determinism tests, ThreadSanitizer in CI

---

## Cell Size Tuning (Empirical)

**Tasks:**

- [ ] Test cell size 100m (many cells, more overhead)
- [ ] Test cell size 150m
- [ ] Test cell size 200m (spec recommendation)
- [ ] Test cell size 300m
- [ ] Test cell size 500m (fewer cells, more entities per cell)
- [ ] Choose optimal based on 150K benchmark
- [ ] Document rationale in `docs/architecture/spatial-partitioning.md`

**Expected:** 200m is optimal (existing spec is correct)

**Trade-off:**
- Too small (50m): More cells, overhead from checking many empty cells
- Too large (500m): Fewer cells, but more entities per cell (approaches O(n²))
- Sweet spot: ~200m (2× max perception range)

---

## Alternative Approaches

### Option A: Stochastic Vision + Spatial Grid (RECOMMENDED)

Combine 10% per-tick updates (Sprint 15) with spatial grid:
- 150K × 10% × 180 neighbors = 2.7M comparisons
- 2.7M / 8 cores = ~340K per core = **~0.7ms**
- **Headroom:** 64x under budget!

### Option B: Hierarchical Spatial Grid (Quadtree)

For very large worlds (>10,000m × 10,000m):
- Adaptive cell sizing based on entity density
- More complex, only if linear grid insufficient
- Defer to Sprint 17+ if needed

### Option C: GPU-Accelerated Perception

For 1M+ creatures (future research):
- Compute shader for distance checks
- Entire perception system parallel on GPU
- Requires major architecture change (Sprint 18+)

---

## System Ordering

**Bevy System Configuration:**

```rust
app.add_systems(Update, (
    rebuild_spatial_grid_system
        .before(update_vision_system),
    update_vision_system,
    // ... rest of AI systems
).chain());
```

**Critical:** Grid rebuild MUST run before perception to ensure fresh data.

---

## References

**Architecture:**
- `docs/architecture/spatial-partitioning.md` - Full spatial grid specification
- `docs/architecture/electron-architecture.md` - IPC considerations

**Sprint Context:**
- Sprint 15 completion metrics (prerequisite)
- `SPRINT_DOCS/SPRINT_PLAN_sprint-15-ecs-optimizations.md`

**Biology:**
- `docs/biology/biology-notes.md` - Perception range, reaction times

**Performance:**
- `docs/performance/optimization-backlog.md` - Ongoing optimization tracking

**Dependencies:**
- Rayon docs: https://docs.rs/rayon/latest/rayon/
- FxHashMap: https://docs.rs/rustc-hash/latest/rustc_hash/

---

## Future Work (Sprint 17+)

**If Sprint 16 successful but further optimization needed:**

- Incremental grid updates (avoid full rebuild)
- Hierarchical spatial grid (quadtree for massive worlds)
- Spatial hash optimization (perfect hashing for fixed world size)
- GPU-accelerated perception (compute shaders)
- Viewport culling (only update on-screen creatures)
- DNA-driven `neural_speed` gene (reaction time variation)
- Variable LOD based on zoom level
