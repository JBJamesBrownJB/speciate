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
| Phase 3 | Rayon validation (all 5 steering systems parallelized) |

---

## Remaining Work

### 1. Cache Locality & Throughput Optimizations

**Context:** Profiling at 150K creatures shows 88% frontend stalls, 24% L3 miss rate, IPC of 1.74. The bottleneck is memory bandwidth, not compute or branch prediction (0.76% miss rate is healthy).

#### 1.1 Stochastic Perception (HIGHEST PRIORITY - 60-75% reduction)

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

#### 1.2 Sort Entities by Grid Cell (30-40% L3 reduction)

**Problem:** Rayon threads randomly access different grid regions → cache thrashing.

**Solution:** Pre-sort entities by spatial locality before parallel processing:

```rust
// Sort by grid cell so nearby creatures process together
let mut sorted: Vec<_> = entities.into_iter()
    .map(|e| (grid.world_to_cell_idx(e.pos.x, e.pos.y), e))
    .collect();
sorted.sort_unstable_by_key(|(idx, _)| *idx);

// Process in chunks - each chunk shares cached grid data
sorted.par_chunks_mut(2000).for_each(|chunk| { ... });
```

- [ ] Add `world_to_cell_idx()` method to SpatialGrid
- [ ] Implement entity sorting before parallel phase
- [ ] Benchmark L3 miss rate before/after

#### 1.3 Cell-Level FOV Culling (25-50% candidate reduction)

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

#### 1.4 Chunked Parallelism (10-20% IPC improvement)

**Quick win:** Replace fine-grained `par_iter_mut` with chunked processing for better prefetching.

```rust
// BEFORE: Per-entity parallelism
entities.par_iter_mut().for_each(|e| { ... });

// AFTER: Chunked parallelism (better cache reuse)
entities.par_chunks_mut(1000).for_each(|chunk| {
    for e in chunk { ... }
});
```

- [ ] Update perception system to use `par_chunks_mut`
- [ ] Tune chunk size (1000-2000 entities)
- [ ] Measure IPC improvement

### 2. Code Cleanup

- [ ] **Hot/Cold Perception split** - Split `Perception` into `PerceptionConfig` (16B) + `PerceptionResult` (reduces cache traffic)
- [ ] **Remove PerceptionScratchBuffer** - No longer needed with spatial grid
- [ ] **Fix test compilation errors** - `instrumentation_test.rs`, `trial_integration.rs`

### 3. Neighbor Sorting Strategy (ID Bias Problem)

**Current State:** Neighbors are returned in arbitrary order based on grid cell iteration and entity spawn order. This creates "ID bias" where lower-ID creatures are consistently perceived first when `MAX_PERCEIVED_NEIGHBORS` is hit.

**Priority:** Lower than cache optimizations - this is a correctness/fairness issue, not a performance bottleneck.

**Options to evaluate:**

| Approach | Complexity | Accuracy | Cache | Notes |
|----------|------------|----------|-------|-------|
| **ID Bias (current)** | O(1) | Poor | Poor | Lower IDs always win when capacity hit |
| **Random Offset** | O(1) | Fair | Poor | Start iteration at random index, breaks bias |
| **Topological Sort** | O(k log k) | Best | Poor | Sort by distance, closest neighbors first |

**Recommendation:** Topological sort (distance-based) is biologically accurate - creatures should perceive their closest neighbors first.

**Decision needed:**
- [ ] **Trial topological sort** - Sort candidates by distance before FOV check
- [ ] **Trial random offset** - Add XorShift32 random start index per query (simpler fallback)
- [ ] **Benchmark chosen approach** - Measure overhead at 150K creatures

**Topological Sort Implementation (O(k log k)):**
```rust
candidates.sort_by(|a, b| {
    let dist_a = (a.x - pos.x).powi(2) + (a.y - pos.y).powi(2);
    let dist_b = (b.x - pos.x).powi(2) + (b.y - pos.y).powi(2);
    dist_a.partial_cmp(&dist_b).unwrap()
});
```

**Random Offset Implementation (O(1)):**
```rust
let offset = xorshift32(entity.index()) as usize % candidates.len();
for i in 0..candidates.len() {
    let idx = (offset + i) % candidates.len();
    // process candidates[idx]
}
```

**Note:** Morton grid indexing was tried for cache locality but showed no improvement - the bottleneck is scattered Rayon thread access, not grid cell ordering.

### 4. Stretch Goals (Future Sprints)

- [ ] Double-buffer grid architecture
- [ ] FxHashMap vs std HashMap benchmark
- [ ] Parallel grid rebuild
- [ ] SIMD batch distance calculations (AVX2)

---

## Key Implementation Details

### Spatial Grid
- **Cell size:** 50m (1.5× max perception range of ~35m)
- **Storage:** `HashMap<(i32, i32), Vec<PerceptionProxy>>`
- **Rebuild:** Full rebuild every tick (~2ms @ 150K)
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
