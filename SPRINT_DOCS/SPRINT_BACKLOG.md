# Sprint 15: ECS Optimizations - Backlog

**Branch:** `feat/sprint-15-ecs-optimizations`
**Status:** IN PROGRESS
**Duration:** 6 days

---

## Sprint Goal

Scale backend ECS simulation to 150K-200K creatures through:
1. **Uber-struct pattern** (stable archetypes, hot/cold split, cache-friendly)
2. **Vision system refactor** (remove Vec allocation bottleneck, FOV, stochastic updates)
3. **Vec2 vector math** (SIMD optimization)
4. **Parallelization** (multi-core utilization)

---

## Baseline Metrics (Actual @ 5K Active Wanderers)
**Source:** `docs/performance/snapshots/5k_wanderers_2025-11-28T14-33-50.json`

| Metric | Current Value | Problem |
|--------|---------------|---------|
| Total tick time | **50ms** | **AT BUDGET LIMIT** |
| Perception system | **34ms (67%)** | **CRITICAL BOTTLENECK** |
| Movement systems | 13ms (26%) | Acceptable |
| Avoidance | 3.4ms (7%) | Acceptable |
| Vec allocations | 3.2MB/frame | 64MB/sec memory churn |
| Max active creatures | **~5K** | Limited by O(N²) perception |
| CPU utilization | 17% (1 core) | 7 cores idle |
| IPC | 3.42 | Good (hardware efficient) |

---

## Phase 1: Archetype Churn Trial (Day 1 - VALIDATION)

### Purpose
Validate whether archetype fragmentation is a measurable performance problem before investing in uber-struct refactor.

### Trial Design
| Scenario | Description | Expected Outcome |
|----------|-------------|------------------|
| A (Stable) | 2.5K wanderers, no behavior changes | Baseline metrics |
| B (Churning) | 2.5K creatures, constant behavior transitions | Measure degradation |

### Metrics to Capture
- Tick time (ms)
- Archetype count (should grow in B if fragmentation)
- IPC (should drop in B if cache thrashing)
- L1/L2 cache miss rates

### Decision Point
- **B >> A (>20% slower):** Proceed to Phase 1b (uber-struct)
- **B ≈ A (<10% difference):** Skip to Phase 2A (vision optimization)

### Tasks
- [ ] Design Scenario A: stable 2.5K wanderers
- [ ] Design Scenario B: churning behavior mechanism (user to specify)
- [ ] Run Scenario A, capture snapshot
- [ ] Run Scenario B, capture snapshot
- [ ] Compare metrics, make decision

**Owner:** ecs-emma + instrumentation-ian

---

## Phase 1b: Uber-Struct Refactor (Day 2 - CONDITIONAL)

**Only proceed if Phase 1 trial shows >20% degradation.**

### Expected Metrics
| Metric | Before | After | Gain |
|--------|--------|-------|------|
| Tick time @ 5K | 50ms | 45ms | 5-10% |
| Active creature capacity | 5K | 6K | +20% |
| L1 cache hit rate | Baseline | +5-10% | Measurable |
| Archetype fragmentation | High | Stable | Eliminated |

**Note:** Modest gains. Real wins come from Phase 2 (vision optimization).

### Tasks
- [ ] Design uber-struct component architecture
- [ ] Implement `PhysicalTraits` (size, speed, color)
- [ ] Implement `BehaviorState` (current behavior, timers)
- [ ] Remove `Catatonic` component → `BehaviorMode::Catatonic` enum
- [ ] Migrate existing component-per-field to uber-structs
- [ ] Verify stable archetype (no add/remove component churn)
- [ ] Run benchmarks to validate cache locality improvement
- [ ] **BENCHMARK:** Capture tick time @ 5K before/after

**Owner:** ecs-emma + rusty-ron
**Validation:** instrumentation-ian (cache metrics)

---

## Phase 2A: Vision Split Queries (Day 3 - CRITICAL)

### Expected Metrics
| Metric | Before | After | Gain |
|--------|--------|-------|------|
| Vec allocations/frame | 3.2MB | 0 bytes | **100% eliminated** |
| Perception time @ 5K | 34ms | ~15ms | **50-60% reduction** |
| Perception time @ 20K | N/A (>100ms) | ~25ms | Now possible |
| Active creature capacity | 6K | 15-20K | **3x increase** |
| Memory churn | 64MB/sec | 0 | GC eliminated |

**CRITICAL:** This is the single most important optimization.

### Tasks
- [ ] Rename `Perception` → `Vision` (biological naming)
- [ ] Add `VisionTiming` component (stochastic updates)
- [ ] Add `Visible` marker component (zero-cost filter)
- [ ] **CRITICAL:** Remove `.collect()` Vec allocation
- [ ] Implement split queries (observers mut, targets immut)
- [ ] Implement FOV dot product check (blind spots)
- [ ] Add stochastic vision updates (reaction time gating)
- [ ] **BENCHMARK:** Verify zero allocations with profiler
- [ ] **BENCHMARK:** Vision time @ 50K, 100K creatures

**Owner:** ecs-emma + rusty-ron
**Validation:** instrumentation-ian, zoologist-tom (FOV biology)

---

## Phase 2B: Changed<T> Filters + Vec2 (Day 4)

### Expected Metrics
| Metric | Before | After | Gain |
|--------|--------|-------|------|
| Rotation iterations @ 20K | 20K/frame | 2-4K/frame | **80-90% reduction** |
| Vector math speed | Scalar f32 | SIMD Vec2 | 10-20% faster |
| Tick time @ 20K | ~35ms | ~32ms | 2-3ms saved |
| Active creature capacity | 15-20K | 25K | +25% |

### Tasks
- [ ] Add `Changed<Velocity>` filter to rotation system
- [ ] Audit all systems for `Changed<T>` opportunities
- [ ] Migrate `Position { x, y }` → `Position(Vec2)`
- [ ] Migrate `Velocity { vx, vy }` → `Velocity(Vec2)`
- [ ] Migrate `Acceleration { ax, ay }` → `Acceleration(Vec2)`
- [ ] Update all vector math to use glam Vec2
- [ ] Verify SIMD optimization in compiler output
- [ ] **BENCHMARK:** Compare rotation system iterations before/after
- [ ] **BENCHMARK:** Vector operation microbenchmarks

**Owner:** rusty-ron + ecs-emma

---

## Phase 2C: Parallelization (Day 5)

### Expected Metrics
| Metric | Before | After | Gain |
|--------|--------|-------|------|
| CPU cores utilized | 1 | 4-8 | **4-8x potential** |
| Realistic speedup | 1x | 2-3x | Amdahl's law |
| Movement systems time @ 25K | ~13ms | ~5ms | 5-8ms saved |
| Tick time @ 25K | ~32ms | ~25ms | 20% reduction |
| Active creature capacity | 25K | 40-50K | **+60-100%** |

**Note:** Parallelization helps movement but NOT perception (O(N²)).

### Tasks
- [ ] Analyze system dependencies for parallel execution
- [ ] Add `par_iter_mut()` to rotation system
- [ ] Add `par_iter_mut()` to seek system
- [ ] Add `par_iter_mut()` to wander system
- [ ] Replace `thread_rng()` with `fastrand` (thread-safe)
- [ ] Test determinism (parallel order shouldn't matter)
- [ ] **BENCHMARK:** Measure speedup on 4-core, 8-core CPUs
- [ ] Profile for race conditions

**Owner:** rusty-ron + instrumentation-ian

**Cannot parallelize (entity lookups):**
- Vision system (O(N²) interactions)
- Avoidance system (queries other entities)

---

## Phase 2D: Stochastic Vision + Validation (Day 6)

### Expected Metrics After Stochastic Vision
| Metric | Before | After | Gain |
|--------|--------|-------|------|
| Vision updates/tick | 100% creatures | ~10% creatures | **10x reduction** |
| Perception time @ 50K | ~25ms | ~3ms | **90% reduction** |
| Active creature capacity | 40-50K | 100-200K | **2-4x increase** |

### Target Metrics (Final)
| Active Creatures | Target Tick | Perception Budget | Confidence |
|-----------------|-------------|-------------------|------------|
| 5K | <25ms | <10ms | 99% |
| 20K | <35ms | <15ms | 90% |
| 50K | <45ms | <20ms | 75% |
| 100K | <50ms | <25ms | 50% |
| 200K | <50ms | <30ms | 30% (stretch) |

### Tasks
- [ ] **BENCHMARK:** Full simulation @ 50K (baseline comparison)
- [ ] **BENCHMARK:** Full simulation @ 100K
- [ ] **BENCHMARK:** Full simulation @ 150K (primary target)
- [ ] **BENCHMARK:** Full simulation @ 200K (stretch goal)
- [ ] Capture hardware metrics snapshot (IPC, cache rates)
- [ ] Validate 22.2Hz tick rate maintained
- [ ] Profile remaining bottlenecks
- [ ] Generate performance report
- [ ] Update `docs/performance/optimization-backlog.md`

**Owner:** instrumentation-ian + ecs-emma

---

## Success Metrics Summary

### Performance KPIs
| KPI | Baseline (5K) | Target | Phase |
|-----|---------------|--------|-------|
| Max active creatures | 5K | 50-100K | All |
| Perception % of frame | 67% | <50% | 2A+2D |
| Vec allocations/frame | 3.2MB | 0 | 2A |
| Perception time @ 5K | 34ms | <10ms | 2A+2D |
| CPU utilization | 17% | 50%+ | 2C |
| IPC | 3.42 | >3.5 | 1+2 |

### Cumulative Capacity by Phase (Active Creatures)
```
Baseline:     ██ 5K (at budget limit!)
Phase 1:      ██ 6K (+20%)
Phase 2A:     ████████ 15-20K (+200%) ← CRITICAL
Phase 2B:     ██████████ 25K (+25%)
Phase 2C:     ████████████████ 40-50K (+60-100%)
Phase 2D:     ████████████████████████████████████████ 100-200K (+100-300%) ← Stochastic vision
```

---

## Biological Validation (zoologist-tom)

### Size-Based Reaction Times
| Creature Size | Reaction Time | Updates/sec | Behavior |
|---------------|---------------|-------------|----------|
| 0.5m (small) | 68ms | ~15/sec | Twitchy prey |
| 1.0m (medium) | 100ms | ~10/sec | Baseline |
| 5.0m (large) | 500ms | ~2/sec | Deliberate |
| 10.0m (huge) | 1632ms | ~0.6/sec | Ponderous |

### FOV Validation
- [ ] Verify blind spots (entities outside FOV not detected)
- [ ] Test predator sneaking from behind
- [ ] Balance FOV width (start 180°, tune from gameplay)

---

## Notes

- Sprint 14 delivered GPU interpolation (165 FPS achieved)
- Frontend ready for high entity counts (200K+)
- Backend is the bottleneck (vision Vec allocations)
- Focus: zero-allocation, cache-friendly, parallel architecture

---

## Phase 3: Spatial Grid + Parallel Perception (DEFERRED TO SPRINT 16+)

**Status:** NOT IN SPRINT 15 SCOPE
**Trigger:** If Phase 2D validation shows perception still bottleneck at 150K creatures
**Duration:** 4-5 days
**Owner:** ecs-emma + rusty-ron + instrumentation-ian

### ⚠️ Critical Context: Why Rayon Alone Fails

**The Math:**
- 150K creatures O(n²): 22.5 billion comparisons
- With Rayon (8 cores): 2.8 billion comparisons per core
- Estimated time: **3,825ms** (85x over 45ms budget)

**Conclusion:** Must fix algorithmic complexity BEFORE parallelizing.

### Spatial Grid Architecture

**Complexity Reduction:**
- Current: O(n²) = 150K × 150K = 22.5B comparisons
- With Grid: O(n × k) = 150K × 180 = 27M comparisons
- **Reduction:** 833x fewer operations

**Expected Performance:**
- Sequential grid @ 150K: ~40ms
- Parallel grid (8 cores) @ 150K: **~7ms**
- With stochastic vision (10% per tick): **~0.7ms**

---

### Day 1-2: Spatial Grid Foundation

**Tasks:**

- [ ] **Design:** Review `docs/architecture/spatial-partitioning.md` spec
- [ ] **RED:** Write `test_spatial_grid_query_returns_nearby_entities()`
- [ ] **RED:** Write `test_spatial_grid_empty_query_returns_nothing()`
- [ ] **RED:** Write `test_spatial_grid_boundary_conditions()`
- [ ] **GREEN:** Implement `SpatialGrid` struct
  - [ ] `new(cell_size: f32)` - Initialize with 200m cells
  - [ ] `insert(entity, x, y, radius)` - Add to grid
  - [ ] `remove(entity)` - Remove from grid
  - [ ] `query_radius_iter(x, y, radius)` - Iterator over nearby entities
  - [ ] `clear()` - Reset grid (per-tick)
  - [ ] `world_to_cell(x, y)` - Coordinate conversion
  - [ ] `get_position(entity)` - Position cache lookup
- [ ] **REFACTOR:** Optimize data structures
  - [ ] Use `FxHashMap` for cell storage
  - [ ] Cache `inv_cell_size` (avoid division)
  - [ ] Pre-allocate cell vectors with `with_capacity(20)`
- [ ] **Test:** Unit test grid at 1K, 10K, 100K entities
- [ ] **Benchmark:** Grid insertion/query performance

**Owner:** rusty-ron + ecs-emma

---

### Day 3: Integration with Perception System

**Tasks:**

- [ ] **RED:** Write `test_perception_with_grid_handles_25k_under_20ms()`
- [ ] **RED:** Write `test_grid_perception_matches_naive_results()`
- [ ] **Design:** Create `rebuild_spatial_grid_system()`
  - Runs BEFORE perception in schedule
  - Clears grid, repopulates from Position/BodySize query
- [ ] **GREEN:** Integrate grid into perception
  - [ ] Add `SpatialGrid` as `Resource`
  - [ ] Update `update_perception_system()` to use `grid.query_radius_iter()`
  - [ ] Remove `PerceptionScratchBuffer` (obsolete)
  - [ ] Update perception system ordering (grid rebuild → perception)
- [ ] **REFACTOR:** Optimize grid rebuild
  - [ ] Batch position updates
  - [ ] Measure rebuild cost (target: <2ms @ 150K)
- [ ] **Test:** Verify behavior matches old system (no regressions)
- [ ] **Benchmark:** Perception time @ 25K, 50K, 100K

**Owner:** ecs-emma + rusty-ron

---

### Day 4: Add Rayon Parallelization

**Tasks:**

- [ ] **Setup:** Add `rayon = "1.10"` to `Cargo.toml`
- [ ] **RED:** Write `test_parallel_perception_deterministic()`
- [ ] **RED:** Write `test_parallel_perception_no_race_conditions()`
- [ ] **GREEN:** Parallelize perception computation
  - [ ] Collect entity data into `Vec<(Entity, f32, f32, f32, bool)>`
  - [ ] Replace `.iter()` with `.par_iter()` on entity_data
  - [ ] Ensure grid is read-only during parallel phase
  - [ ] Collect results into `Vec<(Entity, [Entity; 40], u8)>`
  - [ ] Sequential apply phase (write to `Perception` component)
- [ ] **REFACTOR:** Tune batch sizes
  - [ ] Test `min_batch_size(64)`, `min_batch_size(256)`, `min_batch_size(512)`
  - [ ] Choose optimal based on 150K creature benchmark
- [ ] **Test:** Determinism (same seed = same results)
- [ ] **Benchmark:** Speedup on 4-core, 8-core, 16-core CPUs

**Owner:** rusty-ron + instrumentation-ian

---

### Day 5: Benchmarking & Validation

**Tasks:**

- [ ] **Benchmark:** Create `apps/simulation/benches/perception.rs`
  - [ ] Sequential naive (O(n²) baseline)
  - [ ] Sequential grid (O(n×k))
  - [ ] Parallel grid (O(n×k) / 8 cores)
- [ ] **Run benchmarks:**
  - [ ] 1K creatures (baseline)
  - [ ] 5K creatures (current production scale)
  - [ ] 25K creatures (Phase 2 target)
  - [ ] 50K creatures (intermediate)
  - [ ] 100K creatures (stress test)
  - [ ] 150K creatures (primary target)
  - [ ] 200K creatures (stretch goal)
- [ ] **Hardware profiling:**
  - [ ] Capture IPC before/after
  - [ ] Measure L1/L2 cache hit rates
  - [ ] CPU utilization (should be 50%+ on 8-core)
  - [ ] Memory allocation overhead (grid rebuild)
- [ ] **Validation:**
  - [ ] Verify 150K @ <10ms perception (parallel grid)
  - [ ] Verify no behavior regressions (all tests pass)
  - [ ] Verify no memory leaks (grid cleanup)
- [ ] **Documentation:**
  - [ ] Update `docs/performance/optimization-backlog.md`
  - [ ] Create Phase 3 completion report
  - [ ] Document remaining bottlenecks

**Owner:** instrumentation-ian + ecs-emma

---

### Expected Metrics (Phase 3 Complete)

| Metric | Before (O(n²)) | After (Grid) | After (Grid + Rayon) | Improvement |
|--------|----------------|--------------|----------------------|-------------|
| Perception @ 5K | 34ms | ~1ms | ~0.5ms | 68x faster |
| Perception @ 25K | ~850ms | ~6ms | ~2ms | 425x faster |
| Perception @ 50K | ~3,400ms | ~12ms | ~3ms | 1,133x faster |
| Perception @ 150K | ~30,600ms | ~40ms | **~7ms** | 4,371x faster |
| Perception @ 200K | N/A | ~53ms | **~10ms** | Enables 200K! |
| Max creatures @ 45ms | ~5K | ~50K | **200K+** | **40x capacity** |

### Cell Size Tuning (Empirical)

**Tasks:**

- [ ] Test cell size 100m (many cells, more overhead)
- [ ] Test cell size 150m
- [ ] Test cell size 200m (spec recommendation)
- [ ] Test cell size 300m
- [ ] Test cell size 500m (fewer cells, more entities per cell)
- [ ] Choose optimal based on 150K benchmark
- [ ] Document rationale in `docs/architecture/spatial-partitioning.md`

**Expected:** 200m is optimal (spec is correct)

---

### Success Criteria

**Phase 3 Complete When:**

- [ ] Spatial grid handles 150K creatures @ <40ms perception (sequential)
- [ ] Parallel grid handles 150K creatures @ <10ms perception (8 cores)
- [ ] Benchmarks show 500x+ speedup vs naive O(n²) at 150K
- [ ] Zero regression in perception behavior (tests pass)
- [ ] Cache hit rates improved (hardware profiling confirms)
- [ ] No memory leaks (grid properly cleared each tick)
- [ ] 200K creatures achievable @ <50ms tick time

---

### Dependencies

**Cargo.toml:**
```toml
[dependencies]
rayon = "1.10"
rustc-hash = "1.1"  # FxHashMap (faster than std HashMap)
```

**System Ordering:**
```
rebuild_spatial_grid_system
  .before(update_perception_system)
```

---

### Risks & Mitigations

**Risk:** Grid rebuild overhead negates gains
- **Mitigation:** Benchmark rebuild separately, target <2ms @ 150K
- **Fallback:** Incremental grid updates (only move changed entities)

**Risk:** Cell size incorrect (too large/small)
- **Mitigation:** Empirical testing with 100m, 200m, 300m, 500m
- **Validation:** Benchmark perception time at each cell size

**Risk:** Rayon overhead exceeds benefits at low creature counts
- **Mitigation:** Conditional parallelization (only above 10K creatures)
- **Code:** `if entity_count > 10_000 { par_iter } else { iter }`

**Risk:** Race conditions in parallel perception
- **Mitigation:** Grid is read-only during parallel phase
- **Testing:** Determinism tests, ThreadSanitizer in CI

---

### Alternative Approaches (If Phase 3 Insufficient)

**Option A: Stochastic Vision + Spatial Grid**
- Combine 10% per-tick updates with spatial grid
- 150K × 10% × 180 neighbors = 2.7M comparisons
- 2.7M / 8 cores = ~340K per core = **~0.7ms**
- **Headroom:** 64x under budget!

**Option B: Hierarchical Spatial Grid (Quadtree)**
- For very large worlds (>10,000m × 10,000m)
- Adaptive cell sizing based on entity density
- More complex, only if linear grid insufficient

**Option C: GPU-Accelerated Perception**
- Compute shader for distance checks
- 1 million+ creatures possible
- Requires major architecture change (Sprint 18+)

---

### References

- **Spec:** `docs/architecture/spatial-partitioning.md`
- **Emma's Analysis:** SPRINT_DOCS/SPRINT_PLAN_sprint-15-ecs-optimizations.md (Phase 3)
- **Rayon Docs:** https://docs.rs/rayon/latest/rayon/
- **FxHashMap:** https://docs.rs/rustc-hash/latest/rustc_hash/
