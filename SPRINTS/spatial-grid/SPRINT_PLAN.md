# Sprint 16: Spatial Grid for Scalable Perception

**Theme:** Break the O(N²) perception bottleneck to enable 150K+ creature populations

**Goal:** Replace brute-force neighbor detection with spatial partitioning, achieving O(N×k) complexity where k ≈ 180 neighbors instead of N comparisons.

---

## Sprint Status: CORE COMPLETE

**Achievements:**
- 150K+ creature population achieved
- 5 systems parallelized with Rayon (perception, seek, wander, avoidance, transitions)
- Spatial grid with 50m cells, full rebuild per tick
- Grid visualization ('G' key) with queried cell highlighting
- **Real cell tracking** - Debug overlay shows ACTUAL queried vs skipped cells from perception execution
- Movement optimized with NoiseTable (eliminated 200K allocations/tick)
- 238 unit tests passing

---

## Completed Phases

| Phase | Description |
|-------|-------------|
| Phase 1 | Spatial grid data structure (50m cells, std HashMap) |
| Phase 1.5 | Grid visualization ('G' key toggle) |
| Phase 2 | Two-phase perception pattern with Rayon |
| Phase 2.1 | Queried cells visualization (green/yellow highlights) |
| Phase 2.2 | **Real cell tracking** - Debug target processed separately with instrumentation, captures ACTUAL queried vs skipped cells |
| Phase 3 | Rayon validation (all 5 steering systems parallelized) |

---

## Remaining Work

### 1. ~~Cache Locality Optimizations~~ SKIPPED

**Status:** ❌ Failed performance test - sorting overhead > cache benefit

**Attempted:** Sort entities by grid cell + `par_chunks_mut(2000)` chunked parallelism.

**Result:** Perception time increased from 14ms → 20ms (43% regression). The O(n log n) sorting overhead (~6ms for 150K entities) and reduced parallelism (75 chunks vs 150K fine-grained tasks) outweighed any cache locality gains.

**Conclusion:** Fine-grained `par_iter_mut` remains optimal. The bottleneck is memory bandwidth from scattered grid access, but sorting doesn't help because the sorting cost exceeds the cache benefit. Future alternatives: stochastic perception (reduce work) or cell-level FOV culling (reduce candidates)

### 2. Code Cleanup

- [ ] **Hot/Cold Perception split** - Split `Perception` into `PerceptionConfig` (16B) + `PerceptionResult` (reduces cache traffic)
- [x] **Remove PerceptionScratchBuffer** - Removed (was unused since spatial grid replaced brute-force)
- [ ] **Fix test compilation errors** - `instrumentation_test.rs`, `trial_integration.rs`

### 3. ~~Neighbor Sorting Strategy~~ ✅ COMPLETE

**Status:** Implemented topological sort (distance-based)

**Solution:** Collect all FOV-passing candidates with distance², sort by distance, take closest MAX_PERCEIVED_NEIGHBORS.

**Result:** Biologically accurate - creatures perceive closest neighbors first. No more ID bias.

**Performance:** O(k log k) where k ≈ candidates per creature. Negligible overhead at 150K (candidates typically < 50 per creature).

**Files:** `perception/systems.rs:49-86`

### 4. Grid Rebuild Optimizations

**Current cost:** ~4ms per tick for full rebuild @ 150K creatures

#### 4.1 Static Component (Immediate Win)

**Concept:** Exclude non-moving entities from grid rebuild entirely.

```rust
#[derive(Component, Default)]
pub struct Static;  // Marker for entities that never move

// Grid rebuild only processes moving entities
fn rebuild_spatial_grid(
    query: Query<(Entity, &Position, &BodySize), Without<Static>>,
    ...
)
```

**Use cases for `Static`:**
- Catatonic creatures (sleeping, waiting)
- Terrain obstacles (rocks, trees)
- Food patches, resource nodes
- Plants, flora

**Impact:** If 50% of entities are static → 2ms saved per tick

- [ ] Add `Static` component to `core/components.rs`
- [ ] Update `CritBuilder` to optionally add `Static`
- [ ] Filter grid rebuild query with `Without<Static>`
- [ ] Filter movement system with `Without<Static>`

#### 4.2 Incremental Grid Update (Major Optimization)

**Concept:** Don't clear and rebuild entire grid. Update in place.

**Current approach (expensive):**
```rust
grid.clear();  // Throw away everything
for (entity, pos, size) in query.iter() {
    grid.insert(entity, pos, size);  // Rebuild from scratch
}
```

**Incremental approach (cheap):**
```rust
// Track previous cell for each entity
#[derive(Component)]
pub struct GridCell {
    pub prev_x: i32,
    pub prev_y: i32,
}

// Only update when entity crosses cell boundary
let (new_cx, new_cy) = grid.world_to_cell(pos.x, pos.y);
if new_cx != cell.prev_x || new_cy != cell.prev_y {
    grid.remove(entity, cell.prev_x, cell.prev_y);
    grid.insert(entity, new_cx, new_cy);
    cell.prev_x = new_cx;
    cell.prev_y = new_cy;
}
// Also update position within proxy (cheap)
grid.update_position(entity, pos.x, pos.y);
```

**Impact:** Most creatures stay in same cell per tick. With 50m cells and 50 unit/s max speed, creatures cross boundaries every ~22 ticks on average. Only ~5% of entities need cell updates per tick.

**Estimated savings:** 4ms → 0.5ms (87% reduction)

- [ ] Add `GridCell` component tracking previous cell coords
- [ ] Implement `grid.remove()` and `grid.update_position()` methods
- [ ] Change rebuild to incremental update
- [ ] Handle entity spawn/despawn edge cases

---

### 5. Workload Reduction Optimizations

**Goal:** Less work per tick through temporal amortization and algorithmic culling.

#### 5.1 Stochastic Perception (60-75% reduction)

**Concept:** Not every creature needs fresh perception every tick. DNA-driven "alertness" determines update frequency.

**Implementation:**
```rust
#[derive(Component)]
pub struct PerceptionCadence {
    pub next_update_tick: u32,
    pub interval: u8,  // 1-8 ticks, from DNA "alertness" gene
}

// Only process creatures due for update THIS tick
let due_for_update: Vec<_> = query
    .iter_mut()
    .filter(|(.., cadence)| cadence.next_update_tick == current_tick)
    .collect();
```

**DNA Integration:**
- Predators: interval 1-2 (high alertness, expensive)
- Prey: interval 2-4 (reactive)
- Grazers: interval 4-8 (docile, cheap)

**Gameplay Benefit:** Creates emergent "surprise attacks" - alertness becomes a survival trait subject to natural selection.

- [ ] Add `PerceptionCadence` component
- [ ] Filter perception system by due tick
- [ ] Add jitter to prevent sync waves
- [ ] Integrate with DNA system

#### 5.2 Cell-Level FOV Culling (25-50% candidate reduction)

**Concept:** Skip entire grid cells that are behind the creature before examining any proxies.

```rust
// Before examining proxies, check if cell center is behind creature
let cell_dir_dot = (cell_center_x - x) * facing_x + (cell_center_y - y) * facing_y;
if cell_dir_dot < -cell_size {
    continue; // Entire cell is behind, skip all its proxies
}
```

**Impact by FOV:**
- 180° FOV: ~50% cells skipped
- 90° FOV (predators): ~75% cells skipped

- [ ] Add `query_visible_cells()` method to SpatialGrid
- [ ] Integrate with perception system
- [ ] Verify correctness at FOV boundaries

### 6. Stretch Goals (Future Sprints)

- [ ] Double-buffer grid architecture
- [ ] FxHashMap vs std HashMap benchmark
- [ ] Parallel grid rebuild
- [ ] SIMD batch distance calculations (AVX2)

---

## Key Implementation Details

### Spatial Grid
- **Cell size:** 50m (1.5× max perception range of ~35m)
- **Storage:** `HashMap<(i32, i32), Vec<PerceptionProxy>>`
- **Rebuild:** Full rebuild every tick (~4ms @ 150K) - see 4.2 for incremental optimization
- **Query:** `query_radius()` returns all entities in overlapping cells

### Two-Phase Perception Pattern
```rust
// Phase 1: Collect read-only inputs
let inputs: Vec<_> = query.iter().filter(active).collect();

// Phase 2: Parallel grid queries (Rayon)
let results: Vec<_> = inputs.par_iter().map(|...| {
    grid.query_radius(x, y, range)
        .filter(|n| passes_fov_check(...))
        .take(MAX_PERCEIVED_NEIGHBORS)
        .collect()
}).collect();

// Phase 3: Sequential write-back
for (entity, neighbors) in results {
    perception.set_neighbors(neighbors);
}
```

### Movement NoiseTable
- Pre-computed 65536 noise values (~256KB)
- Ring buffer access via `(entity_id + tick + axis) & mask`
- Eliminated per-call Perlin::new() allocation overhead

---

## Files Modified This Sprint

**Rust (simulation):**
- `src/simulation/spatial/` - New module (grid, systems)
- `src/simulation/perception/systems.rs` - Two-phase Rayon pattern
- `src/simulation/perception/components.rs` - PerceptionDebugSnapshot with cells
- `src/simulation/movement/noise.rs` - NoiseTable resource
- `src/simulation/behaviors/*/systems.rs` - Rayon parallelization
- `src/ipc/bridge/perception_debug_buffer.rs` - Cell data export

**TypeScript (portal):**
- `src/rendering/SpatialGridOverlay.ts` - Grid + queried cell rendering
- `src/types/GameState.ts` - QueriedCell interface
- `src/infrastructure/ipc/ElectronIPCClient.ts` - Cell data parsing
