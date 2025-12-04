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

### 1. Neighbor Sorting Strategy (ID Bias Problem)

**Current State:** Neighbors are returned in arbitrary order based on grid cell iteration and entity spawn order. This creates "ID bias" where lower-ID creatures are consistently perceived first when `MAX_PERCEIVED_NEIGHBORS` is hit.

**Options to evaluate:**

| Approach | Complexity | Accuracy | Cache | Notes |
|----------|------------|----------|-------|-------|
| **ID Bias (current)** | O(1) | Poor | Poor | Lower IDs always win when capacity hit |
| **Random Offset** | O(1) | Fair | Poor | Start iteration at random index, breaks bias |
| **Topological Sort** | O(k log k) | Best | Poor | Sort by distance, closest neighbors first |
| **Morton Sort** | O(k log k) | Fair | Best | Z-order curve, cache-friendly spatial locality |

**Key distinction:**
- **Topological** = "Who's closest to ME?" (per-query distance sort, biologically accurate)
- **Morton** = "Keep spatially nearby entities adjacent in memory" (cache optimization)

**Decision needed:**
- [ ] **Trial random offset** - Add XorShift32 random start index per query
- [ ] **Trial topological sort** - Sort candidates by distance before FOV check
- [ ] **Trial Morton sort** - Sort by Z-order code for cache locality
- [ ] **Benchmark all three** - Measure overhead at 150K creatures
- [ ] **Implement chosen approach**

**Random Offset Implementation (O(1)):**
```rust
let offset = xorshift32(entity.index()) as usize % candidates.len();
for i in 0..candidates.len() {
    let idx = (offset + i) % candidates.len();
    // process candidates[idx]
}
```

**Topological Sort Implementation (O(k log k)):**
```rust
candidates.sort_by(|a, b| {
    let dist_a = (a.x - pos.x).powi(2) + (a.y - pos.y).powi(2);
    let dist_b = (b.x - pos.x).powi(2) + (b.y - pos.y).powi(2);
    dist_a.partial_cmp(&dist_b).unwrap()
});
```

**Morton Sort Implementation (O(k log k)):**
```rust
// Morton code: interleave bits of x and y for Z-order curve
fn morton_code(x: f32, y: f32, cell_size: f32) -> u32 {
    let ix = ((x / cell_size) as i32).max(0) as u32;
    let iy = ((y / cell_size) as i32).max(0) as u32;
    interleave_bits(ix, iy)
}

fn interleave_bits(x: u32, y: u32) -> u32 {
    let x = (x | (x << 8)) & 0x00FF00FF;
    let x = (x | (x << 4)) & 0x0F0F0F0F;
    let x = (x | (x << 2)) & 0x33333333;
    let x = (x | (x << 1)) & 0x55555555;

    let y = (y | (y << 8)) & 0x00FF00FF;
    let y = (y | (y << 4)) & 0x0F0F0F0F;
    let y = (y | (y << 2)) & 0x33333333;
    let y = (y | (y << 1)) & 0x55555555;

    x | (y << 1)
}

// Sort candidates by Morton code
candidates.sort_by_key(|c| morton_code(c.x, c.y, CELL_SIZE));
```

**Morton can also apply at grid rebuild time:**
```rust
// Sort entities within each cell by Morton code during rebuild
for cell in grid.cells.values_mut() {
    cell.sort_by_key(|(_, x, y, _)| morton_code(*x, *y, CELL_SIZE));
}
```

### 2. Code Cleanup

- [ ] **Hot/Cold Perception split** - Split `Perception` into `PerceptionConfig` (16B) + `PerceptionResult` (reduces cache traffic)
- [ ] **Remove PerceptionScratchBuffer** - No longer needed with spatial grid
- [ ] **Fix test compilation errors** - `instrumentation_test.rs`, `trial_integration.rs`

### 3. Stretch Goals (Future Sprints)

- [ ] Staggered perception (DNA-driven cadence)
- [ ] Double-buffer grid architecture
- [ ] FxHashMap vs std HashMap benchmark
- [ ] Parallel grid rebuild

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
