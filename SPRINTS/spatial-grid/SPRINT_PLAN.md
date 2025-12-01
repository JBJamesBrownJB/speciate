# Sprint 16: Spatial Grid for Scalable Perception

**Theme:** Break the O(N²) perception bottleneck to enable 150K+ creature populations

**Goal:** Replace brute-force neighbor detection with spatial partitioning, achieving O(N×k) complexity where k ≈ 180 neighbors instead of N comparisons.

**Prerequisites:** Sprint 15 complete (Rayon parallelization, vision split queries)

**Expected Duration:** 5 days

**Target Performance:** 150K creatures @ <45ms tick, perception <10ms

---

## Team Review Summary

**Reviewed by:** ecs-emma, rusty-ron, architect-andy (2025-12-01)

**Verdict:** APPROVED with required changes

### Critical Fixes (P0)

| Issue | Fix |
|-------|-----|
| System ordering not explicit | Add `.chain()` between grid rebuild and perception |
| Mutable borrow blocks Rayon | Two-phase pattern: collect read-only → parallel query → sequential write-back |

### High Priority (P1)

| Issue | Impact |
|-------|--------|
| Hot/Cold Perception split | 192B → 16B read during queries. Saves ~35MB cache traffic/tick @ 200K |
| Delete `obstacles: Vec<Entity>` | Dead heap allocation - free performance |
| Replace fxhash with XorShift32 | 3x faster for random offset generation |
| Pre-allocate grid capacity | Avoid rehashing during rebuild |

### Confirmed Decisions

| Decision | Verdict |
|----------|---------|
| Separate `rebuild_grid_system` | ✅ Correct |
| `Res<SpatialGrid>` immutable | ✅ Correct |
| Random offset over sorting | ✅ Correct |
| 50m cell size | ✅ Valid (with assertion) |
| FxHashMap | ✅ Correct |

---

## Phase Structure

### Phase 0: Pre-Sprint Cleanup (Day 0)

**Outcome:** Clean component architecture ready for grid integration

**Tasks:**
- [ ] Delete `obstacles: Vec<Entity>` from Perception (dead heap allocation)
- [ ] Split `Perception` into `PerceptionConfig` + `PerceptionResult` (Hot/Cold)
- [ ] Remove `PerceptionScratchBuffer` (replaced by SpatialGrid)
- [ ] Add explicit system ordering documentation

**Hot/Cold Component Split:**

```rust
// HOT: Read during spatial queries (16 bytes, fits 1/4 cache line)
#[derive(Component)]
pub struct PerceptionConfig {
    pub range: f32,
    pub cos_half_fov_sq: f32,
    pub fov_angle: f32,
    _padding: f32,
}

// COLD: Write-only output (72 bytes)
#[derive(Component)]
pub struct PerceptionResult {
    pub neighbor_count: u8,
    neighbors: [Entity; MAX_PERCEIVED_NEIGHBORS],
}
```

**Impact:** During read phase, load 16B instead of 192B. Saves ~35MB cache traffic/tick @ 200K.

---

### Phase 1: Spatial Grid Data Structure (Day 1-2)

**Outcome:** FxHashMap-based grid with 50m cells, rebuilt every frame

**Key Decisions:**
- Cell size: **50m** (max perception ~35m, use 1.5× for safety)
- Use FxHashMap via `rustc-hash` crate (2-5× faster than std HashMap)
- Store `(Entity, x, y, radius)` in cells to avoid component double-lookup
- **Full rebuild per frame** - simpler, ~1-2ms overhead acceptable
- **Pre-allocate capacity** to avoid rehashing during rebuild

**Grid API:**

```rust
#[derive(Resource)]
pub struct SpatialGrid {
    cells: FxHashMap<(i32, i32), Vec<(Entity, f32, f32, f32)>>,
    cell_size: f32,
    inv_cell_size: f32,
}

impl SpatialGrid {
    pub fn new(cell_size: f32, expected_cells: usize) -> Self;
    pub fn clear_and_reuse(&mut self);  // Preserves Vec capacities
    pub fn insert(&mut self, entity: Entity, x: f32, y: f32, radius: f32);
    pub fn query_radius(&self, x: f32, y: f32, radius: f32) -> impl Iterator<Item = &(Entity, f32, f32, f32)>;
}
```

**Rebuild System:**

```rust
pub fn rebuild_spatial_grid_system(
    mut grid: ResMut<SpatialGrid>,
    query: Query<(Entity, &Position, &BodySize)>,
) {
    grid.clear_and_reuse();  // Preserve Vec capacities
    for (entity, pos, size) in query.iter() {
        grid.insert(entity, pos.x, pos.y, size.radius());
    }
}
```

**System Ordering (Phase 1-3, single-buffer):**

```rust
app.add_systems(Update, (
    rebuild_spatial_grid_system,
    update_perception_system.after(rebuild_spatial_grid_system),
));
```

**Note:** `.after()` ensures perception reads fresh grid. In Phase 5, double-buffer removes this dependency - systems run in parallel.

---

### Phase 2: Two-Phase Perception Pattern (Day 2-3)

**Outcome:** Perception system uses grid with Rayon-compatible architecture

**Critical Pattern:** The current `&mut Perception` borrow blocks Rayon. Must use two-phase:

```rust
pub fn update_perception_system(
    grid: Res<SpatialGrid>,
    perceivers: Query<(Entity, &Position, &Rotation, &BodySize, &PerceptionConfig, &CreatureState)>,
    mut results: Query<&mut PerceptionResult>,
) {
    // Phase 1: Collect read-only data for parallel processing
    let inputs: Vec<_> = perceivers.iter()
        .filter(|(_, _, _, _, _, state)| state.behavior.is_active())
        .collect();

    // Phase 2: Parallel perception queries (Rayon)
    let perception_results: Vec<(Entity, Vec<Entity>)> = inputs.par_iter()
        .map(|(entity, pos, rotation, size, config, _)| {
            let mut neighbors = Vec::with_capacity(MAX_PERCEIVED_NEIGHBORS);

            // Random offset via XorShift32 (3x faster than fxhash)
            let mut rng = XorShift32::from_seed(entity.index());
            let candidates: Vec<_> = grid.query_radius(pos.x, pos.y, config.range).collect();

            if !candidates.is_empty() {
                let offset = rng.next() as usize % candidates.len();
                for i in 0..candidates.len() {
                    let idx = (offset + i) % candidates.len();
                    let (other_entity, ox, oy, _) = candidates[idx];

                    if *other_entity == *entity { continue; }

                    // FOV + distance checks (existing optimized logic)
                    if passes_fov_check(pos, rotation, config, *ox, *oy) {
                        neighbors.push(*other_entity);
                        if neighbors.len() >= MAX_PERCEIVED_NEIGHBORS { break; }
                    }
                }
            }
            (*entity, neighbors)
        })
        .collect();

    // Phase 3: Sequential write-back (<1ms @ 200K)
    for (entity, neighbors) in perception_results {
        if let Ok(mut result) = results.get_mut(entity) {
            result.clear();
            for neighbor in neighbors {
                result.add_neighbor(neighbor);
            }
        }
    }
}
```

**XorShift32 RNG (faster than fxhash):**

```rust
pub struct XorShift32(u32);

impl XorShift32 {
    #[inline]
    pub fn from_seed(seed: u32) -> Self {
        Self(seed.wrapping_add(1))  // Avoid zero
    }

    #[inline]
    pub fn next(&mut self) -> u32 {
        let mut x = self.0;
        x ^= x << 13;
        x ^= x >> 17;
        x ^= x << 5;
        self.0 = x;
        x
    }
}
```

**Breaking Entity ID Bias - Cost Analysis @ 200K:**

| Approach | Overhead | Notes |
|----------|----------|-------|
| Grid rebuild only | ~2ms | Baseline |
| + Global sort | +15-20ms | O(n log n) ❌ |
| + Per-cell sort | +5-8ms | ⚠️ Acceptable but slow |
| + Random offset (XorShift32) | +0ms | ✅ Free |

---

### Phase 3: Rayon Validation (Day 3-4)

**Outcome:** Confirm multi-core engagement, benchmark at scale

**Validation Checklist:**
- [ ] Rayon engages all CPU cores (check with `htop` or instrumentation)
- [ ] No lock contention on grid reads
- [ ] Benchmark perception time at 50K, 100K, 150K, 200K creatures
- [ ] Compare parallel vs sequential for regression detection

**Expected Performance:**

| Creatures | Current (O(n²)) | With Grid (Sequential) | With Grid (Parallel) |
|-----------|-----------------|------------------------|----------------------|
| 20K | 50ms | ~3-5ms | ~1ms |
| 50K | 425ms | ~12ms | ~2ms |
| 100K | 1,700ms | ~25ms | ~4ms |
| 150K | 3,825ms | ~40ms | ~7ms |
| 200K | 6,800ms | ~55ms | ~10ms |

---

### Phase 4: Staggered Perception (Day 4-5)

**Outcome:** DNA-driven perception update frequency for 5-10x additional reduction

**Rationale:** Real animals don't perceive every instant. Reaction times:
- Insects: ~50ms (every tick)
- Small mammals: ~100ms (every 2nd tick)
- Large mammals: ~300-500ms (every 5-10 ticks)

**Component:**

```rust
#[derive(Component)]
pub struct PerceptionCadence {
    pub interval_ticks: u8,   // 1 = every tick, 5 = every 5th tick
    pub last_update_tick: u32,
}
```

**System Modification:**

```rust
// Only process creatures whose perception is "due"
let due_creatures: Vec<_> = perceivers.iter()
    .filter(|(_, _, _, _, _, state, cadence)| {
        state.behavior.is_active() &&
        (current_tick - cadence.last_update_tick) >= cadence.interval_ticks as u32
    })
    .collect();
```

**Impact:** At 200K creatures with average 5-tick interval, only ~40K process per tick. 5x reduction.

**DNA Integration:** `perception_cadence` gene controls `interval_ticks`. Small/fast creatures = 1, large/slow = 5-10.

---

### Phase 5: Double-Buffer + Staggered Grid Updates (Day 5)

**Outcome:** Grid rebuild decoupled from perception via double-buffer, updates every Nth tick

**Double-Buffer Architecture:**

```rust
#[derive(Resource)]
pub struct SpatialGrids {
    active: SpatialGrid,    // Perception reads (immutable during tick)
    inactive: SpatialGrid,  // Rebuild writes (mutated during tick)
}

impl SpatialGrids {
    pub fn swap(&mut self) {
        std::mem::swap(&mut self.active, &mut self.inactive);  // Zero-cost pointer swap
    }

    pub fn active(&self) -> &SpatialGrid {
        &self.active
    }

    pub fn inactive_mut(&mut self) -> &mut SpatialGrid {
        &mut self.inactive
    }
}
```

**Staggered Rebuild (every Nth tick):**

```rust
const GRID_REBUILD_INTERVAL: u32 = 2;  // Rebuild every 2nd tick

pub fn rebuild_spatial_grid_system(
    mut grids: ResMut<SpatialGrids>,
    tick: Res<SimulationTick>,
    query: Query<(Entity, &Position, &BodySize)>,
) {
    // Modulo check - simpler, no overflow issues
    if tick.get() % GRID_REBUILD_INTERVAL != 0 {
        return;  // Skip rebuild, perception uses active buffer
    }

    let grid = grids.inactive_mut();

    // clear() preserves Vec capacity - avoids reallocation churn
    grid.clear_preserving_capacity();

    for (entity, pos, size) in query.iter() {
        grid.insert(entity, pos.x, pos.y, size.radius());
    }
}

// End of tick: swap buffers (only if rebuild happened)
pub fn swap_grid_buffers_system(
    mut grids: ResMut<SpatialGrids>,
    tick: Res<SimulationTick>,
) {
    if tick.get() % GRID_REBUILD_INTERVAL == 0 {
        grids.swap();
    }
}
```

**Grid Clear Implementation:**

```rust
impl SpatialGrid {
    /// Clears all cells but preserves Vec allocations to avoid churn
    pub fn clear_preserving_capacity(&mut self) {
        for cell in self.cells.values_mut() {
            cell.clear();  // Keeps capacity, just sets len=0
        }
    }
}
```

**System Ordering:**

```rust
app.add_systems(Update, (
    rebuild_spatial_grid_system,    // Writes to inactive buffer
    update_perception_system,        // Reads from active buffer
));

// Swap happens at end of frame
app.add_systems(PostUpdate, swap_grid_buffers_system);
```

**Parallelism Clarification:**

Bevy sees `ResMut<SpatialGrids>` (rebuild) vs resource access (perception) as a potential conflict. The **actual benefit** comes from:
1. **Staggered skip** - rebuild doesn't run on perception-only ticks
2. **Buffer isolation** - no data races, clean separation
3. **PostUpdate swap** - isolated from both systems

**Why This Works:**

| Tick | Rebuild (inactive) | Perception (active) | Swap? |
|------|-------------------|---------------------|-------|
| 0 | Build grid v0 | Empty (bootstrap) | Yes |
| 1 | Skip | Read v0 | No |
| 2 | Build grid v1 | Read v0 | Yes |
| 3 | Skip | Read v1 | No |
| 4 | Build grid v2 | Read v1 | Yes |

**Staleness Analysis:**

At 50 m/s max speed and 22Hz tick rate:
- Distance per tick: 2.27m
- Distance per 2 ticks: 4.54m
- Cell size: 50m

Creatures move <10% of cell size between rebuilds. Entities stay in correct cells 99%+ of the time.

**Performance Impact @ 200K:**

| Configuration | Rebuild | Perception | Total |
|---------------|---------|------------|-------|
| Sequential, every tick | 2ms | 10ms | 12ms |
| Staggered (every 2nd tick) | 1ms avg | 10ms | 11ms |

**Net savings:** ~1ms/tick average from staggered rebuild

**Memory overhead:** 2× grid size = ~6MB @ 200K (negligible)

---

### Phase 5 Stretch Goal: Parallel Grid Rebuild

**Outcome:** Reduce grid rebuild from ~2ms to <0.5ms via Rayon

**Pattern:** Chunk entities into thread-local buckets, merge into grid:

```rust
pub fn rebuild_spatial_grid_parallel(
    mut grids: ResMut<SpatialGrids>,
    tick: Res<SimulationTick>,
    query: Query<(Entity, &Position, &BodySize)>,
) {
    if tick.get() % GRID_REBUILD_INTERVAL != 0 {
        return;
    }

    let grid = grids.inactive_mut();
    grid.clear_preserving_capacity();

    // Collect for Rayon
    let entities: Vec<_> = query.iter().collect();

    // Parallel: build thread-local cell maps
    let chunk_size = (entities.len() / rayon::current_num_threads()).max(1000);
    let local_maps: Vec<FxHashMap<(i32, i32), Vec<(Entity, f32, f32, f32)>>> =
        entities.par_chunks(chunk_size)
            .map(|chunk| {
                let mut local = FxHashMap::default();
                for (entity, pos, size) in chunk {
                    let cell = grid.position_to_cell(pos.x, pos.y);
                    local.entry(cell)
                        .or_insert_with(Vec::new)
                        .push((*entity, pos.x, pos.y, size.radius()));
                }
                local
            })
            .collect();

    // Sequential merge (fast - just extend vecs)
    for local_map in local_maps {
        for (cell, entities) in local_map {
            grid.cells.entry(cell)
                .or_insert_with(Vec::new)
                .extend(entities);
        }
    }
}
```

**Expected Impact:**
- Before: ~2ms rebuild @ 200K
- After: ~0.3-0.5ms rebuild @ 200K
- **4-6x speedup** on multi-core

**When to implement:** Only if profiling shows rebuild > 1ms is a concern. Serial rebuild is likely fast enough.

---

## Guidance Notes

### Cell Size Rationale

**50m chosen based on actual perception range analysis:**

```
PERCEPTION_MULTIPLIER = 10.0  (base_range = body_size × 10)
FOV_RANGE_EXPONENT = 0.4
Max body_size = 2.0 → base_range = 20m
Max FOV bonus (45° narrow) = 1.74× → max_range = 34.8m
```

**Cell size = 50m (1.5× max perception):**
- 3×3 query = 9 cells
- At uniform distribution: ~100 creatures/cell
- ~900 comparisons per query

**Validation:** Add debug assertion:
```rust
debug_assert!(perception.range <= CELL_SIZE, "Perception {} exceeds cell size {}", perception.range, CELL_SIZE);
```

### Pre-Sprint FOV Optimizations (Preserve These)

1. **Sqrt-free FOV check:** `rough_dot² >= cos_half_fov_sq × center_dist_sq`
2. **Cached `cos_half_fov_sq`:** Pre-computed at construction
3. **Early-exit for behind:** `if rough_dot <= 0.0 { continue; }`
4. **Dot product FOV:** Replaced atan2

### Biological Context

Spatial grids mirror real animal cognition - creatures don't evaluate every entity in the world, only those in local proximity. Combined with staggered perception, this models realistic reaction times and attention limitations.

---

## Success Criteria

### Core Requirements
- [ ] Spatial grid supports 150K creatures @ <45ms total tick time
- [ ] Perception system uses <10ms @ 150K (down from 70% of budget)
- [ ] Grid rebuild overhead <3ms @ 200K (full rebuild per tick)
- [ ] All existing tests pass (zero behavioral regression)
- [ ] Rayon parallel queries engage all CPU cores

### Architecture Requirements
- [ ] Hot/Cold Perception split implemented
- [ ] Two-phase perception pattern (collect → parallel → write-back)
- [ ] Explicit system ordering with `.chain()`
- [ ] XorShift32 for random offset (not fxhash)
- [ ] Grid pre-allocates capacity

### Validation
- [ ] Benchmarked at 50K, 100K, 150K, 200K creatures
- [ ] Determinism test: same seed → same perception results
- [ ] Cell size validated (50m default, assertion if exceeded)

### Stretch Goals
- [ ] Staggered perception (Phase 4) reduces load 5x
- [ ] Double-buffer with staggered grid rebuild saves ~1ms avg
- [ ] Parallel grid rebuild (Rayon) reduces rebuild from 2ms to <0.5ms
- [ ] Validated at 200K creatures sustained

---

## Dependencies

```toml
# Add to Cargo.toml
rustc-hash = "2.0"  # FxHashMap for fast integer hashing
```

---

## Files to Create/Modify

```
apps/simulation/src/simulation/spatial/mod.rs       (NEW)
apps/simulation/src/simulation/spatial/grid.rs      (NEW)
apps/simulation/src/simulation/spatial/systems.rs   (NEW)
apps/simulation/src/simulation/spatial/rng.rs       (NEW - XorShift32)
apps/simulation/src/simulation/perception/components.rs (MODIFY - Hot/Cold split)
apps/simulation/src/simulation/perception/systems.rs    (MODIFY - Two-phase)
apps/simulation/src/simulation/mod.rs               (MODIFY - system ordering)
```

---

## Risk Mitigation

| Risk | Mitigation |
|------|------------|
| Cell size too small | Start at 50m, add assertion, tune based on profiling |
| Grid rebuild too slow | Pre-allocate capacity, use `clear_and_reuse()` |
| Rayon doesn't engage | Two-phase pattern ensures read-only grid access |
| Staggered perception breaks behavior | Make cadence=1 default, DNA-driven is opt-in |
| Memory pressure | Grid is ~3MB @ 200K, negligible |
