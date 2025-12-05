# Spatial Grid Cache Performance Analysis

**Status:** Performance regression investigation
**Branch:** `feat-spatial-grid`
**Problem:** FxHashMap spatial grid slower than O(N²) brute-force for dense populations

---

## Hypothesis: The HashMap Tax

### Expected vs Actual

**Expected:** O(N²) → O(N×M) where M = entities per cell
**Actual:** No performance improvement, increased L3 cache usage

**Root Cause:** Random memory access pattern in HashMap destroys CPU cache locality.

---

## Memory Access Pattern Analysis

### Brute-Force (Bad Complexity, Good Cache)

```rust
// O(N²) but SEQUENTIAL memory access
for entity_a in query.iter() {           // Linear scan of Archetype
    for entity_b in query.iter() {       // Another linear scan
        // Distance check (all data is HOT in L1/L2)
    }
}
```

**Cache Behavior:**
- Archetype data is contiguous in memory (Bevy's table storage)
- Sequential reads = perfect prefetcher behavior
- High L1 hit rate (~95%+)
- IPC likely 2.0+ (good SIMD utilization)

**Why it's fast at small N:**
- All entity data fits in L3 (32MB)
- At 10K creatures × 64 bytes/entity = 640KB (fits in L2!)

### HashMap Grid (Good Complexity, Bad Cache)

```rust
// O(N×M) but RANDOM memory access
for entity in query.iter() {
    for cell_key in query_cells(radius) {    // 9 cells (3×3)
        if let Some(cell) = grid.cells.get(&cell_key) {  // RANDOM LOOKUP
            for &(other_entity, x, y, r) in cell.iter() {
                // Distance check
            }
        }
    }
}
```

**Cache Behavior:**
- `FxHashMap::get()` computes hash, probes bucket array (random index)
- Each cell lookup = L3 cache line fetch (60-100 cycles)
- 9 cell lookups per entity = 9× cache misses
- Entities scattered across many Vec allocations (heap fragmentation)

**Memory Layout:**
```
HashMap Bucket Array:  [ptr₀][ptr₁][ptr₂]...[ptrₙ]  ← Random access
                          ↓     ↓     ↓
Cell Vecs (heap):      [Vec][Vec][Vec]...          ← Pointer chasing
                         ↓     ↓     ↓
EntityData tuples:    [(E,x,y,r)][(E,x,y,r)]...    ← Fragmented
```

**Estimated Cost:**
- HashMap probe: ~50-100 cycles (L3 miss)
- Vec dereference: ~10-20 cycles (L3 hit if lucky)
- Total per cell: ~70-120 cycles
- Per entity (9 cells): ~630-1080 cycles

Compare to brute-force:
- Sequential read: ~4 cycles (L1 hit)
- SIMD distance check: ~10 cycles
- Per comparison: ~14 cycles

**Break-even point:** HashMap wins only when M < N/80 (very sparse grids).

---

## Profiling Commands

### 1. Health Check (Baseline Metrics)

```bash
cd /home/dev/dev/speciate/apps/simulation
cargo build --release

perf stat -e instructions,cycles,L1-dcache-loads,L1-dcache-load-misses,LLC-loads,LLC-load-misses,branch-misses \
    timeout 10s ./target/release/sim_app
```

**Key Metrics:**
- **IPC (Instructions/Cycle):** Should be > 1.0. If < 0.8, you're memory bound.
- **L1 Miss Rate:** `L1-dcache-load-misses / L1-dcache-loads`. Should be < 5%.
- **LLC Miss Rate:** `LLC-load-misses / LLC-loads`. Should be < 1%.

**Interpretation:**
```
IPC < 0.8         → CPU stalled on memory
L1 Miss > 5%      → Poor data locality (cold structs)
LLC Miss > 1%     → Random access (HashMap pointer chasing)
```

### 2. L3 Cache Miss Hotspot

```bash
perf record --call-graph dwarf -e LLC-load-misses \
    timeout 10s ./target/release/sim_app

perf report --stdio --sort=symbol --percent-limit=1
```

**Look for:**
- `hashbrown::raw::RawTable::find` (HashMap lookup)
- `core::hash::BuildHasher::hash` (FxHash computation)
- `alloc::vec::Vec::as_ptr` (Cell Vec dereference)

**Red Flag:** If these functions account for >5% of LLC misses, HashMap is the culprit.

### 3. Flamegraph (Visual Analysis)

```bash
cargo install samply  # If not installed
samply record timeout 10s ./target/release/sim_app
```

**Opens Firefox Profiler** showing call stack timing.

**Look for:**
- Wide bars in `update_perception_system`
- Deep call stacks through `FxHashMap::get`
- Self-time (orange) in hash/lookup functions

### 4. Instruction-Level Drill-Down (Advanced)

```bash
perf record --call-graph dwarf -e cycles:pp \
    timeout 10s ./target/release/sim_app

perf annotate --stdio update_perception_system
```

Shows assembly with cycle counts per instruction. Look for stalls at:
- `mov` from dereferenced pointers (cache miss)
- Unpredictable branches (hash collision handling)

---

## Alternative Spatial Data Structures

### Option 1: Flat 2D Array (Best Cache, Moderate Complexity)

**Concept:** Pre-allocate a 2D grid array covering the world bounds.

```rust
pub struct DenseGrid {
    cells: Vec<Vec<EntityData>>,  // Flat allocation
    width: usize,                 // Grid dimensions
    height: usize,
    cell_size: f32,
}

impl DenseGrid {
    pub fn new(world_width: f32, world_height: f32, cell_size: f32) -> Self {
        let width = (world_width / cell_size).ceil() as usize;
        let height = (world_height / cell_size).ceil() as usize;
        let cells = vec![Vec::new(); width * height];  // Pre-allocate
        Self { cells, width, height, cell_size }
    }

    #[inline]
    fn cell_index(&self, x: f32, y: f32) -> usize {
        let cx = (x / self.cell_size).floor() as usize;
        let cy = (y / self.cell_size).floor() as usize;
        cy * self.width + cx  // DIRECT ADDRESSING (no hash!)
    }

    pub fn query_radius(&self, x: f32, y: f32, radius: f32) -> impl Iterator<Item = &EntityData> {
        let center_idx = self.cell_index(x, y);
        let cells_radius = (radius / self.cell_size).ceil() as i32;

        // Convert linear index to 2D coordinates
        let center_cx = (center_idx % self.width) as i32;
        let center_cy = (center_idx / self.width) as i32;

        (-cells_radius..=cells_radius)
            .flat_map(move |dy| {
                (-cells_radius..=cells_radius).map(move |dx| {
                    let cx = center_cx + dx;
                    let cy = center_cy + dy;
                    if cx >= 0 && cx < self.width as i32 && cy >= 0 && cy < self.height as i32 {
                        Some(cy as usize * self.width + cx as usize)
                    } else {
                        None
                    }
                })
            })
            .flatten()
            .filter_map(move |idx| self.cells.get(idx))
            .flatten()
    }
}
```

**Pros:**
- O(1) cell lookup (arithmetic, not hashing)
- Sequential array access = perfect prefetching
- Predictable memory layout

**Cons:**
- Wastes memory for sparse worlds (100×100 grid = 10K cells)
- Fixed world bounds (must be defined upfront)

**Cache Performance:**
- Cell lookup: ~4 cycles (L1 hit - array index is predictable)
- 9 cell queries: ~36 cycles (vs ~630 cycles for HashMap)

**Break-even:** Wins whenever entities are somewhat clustered.

---

### Option 2: Sort-and-Sweep (Best for Dense)

**Concept:** Sort entities by X coordinate, then sweep for neighbors.

```rust
pub fn query_neighbors_sweep(
    entities: &[(Entity, f32, f32, f32)],  // Pre-sorted by X
    query_x: f32,
    query_y: f32,
    radius: f32,
) -> impl Iterator<Item = &(Entity, f32, f32, f32)> {
    let min_x = query_x - radius;
    let max_x = query_x + radius;

    // Binary search for start index
    let start = entities.partition_point(|(_, x, _, _)| *x < min_x);

    entities[start..]
        .iter()
        .take_while(move |(_, x, _, _)| *x <= max_x)  // Linear scan until X exceeds
        .filter(move |(_, x, y, r)| {
            let dx = x - query_x;
            let dy = y - query_y;
            dx * dx + dy * dy <= (radius + r) * (radius + r)
        })
}
```

**Pros:**
- O(N log N + M) where M = candidates in X range
- Perfectly cache-friendly (linear scan)
- No spatial structure overhead

**Cons:**
- Requires sorting every tick (O(N log N))
- Still O(N) worst case for large radius

**Best Use Case:** Dense, uniform distributions (all creatures clustered).

---

### Option 3: Bounding Volume Hierarchy (BVH)

**Concept:** Tree structure where each node bounds its children.

**Pros:**
- O(log N + M) query time
- Industry-standard for collision detection

**Cons:**
- Complex to implement correctly
- Requires rebalancing (expensive)
- Poor cache locality (tree traversal = pointer chasing)

**Verdict:** Overkill for 2D creature simulation. Use for 3D physics engines.

---

## Recommended Next Steps

### Immediate: Run the Profiling Script

```bash
cd /home/dev/dev/speciate/apps/simulation
chmod +x profile_spatial_grid.sh
./profile_spatial_grid.sh
```

This will generate:
- `perf_results_YYYYMMDD_HHMMSS/01_health_check.txt` (IPC, cache metrics)
- `perf_results_YYYYMMDD_HHMMSS/03_llc_misses_report.txt` (HashMap hotspots)
- `perf_results_YYYYMMDD_HHMMSS/04_cpu_profile.json` (Flamegraph)

### Decision Matrix

**If LLC Miss Rate > 1%:**
→ HashMap is thrashing L3. Switch to Flat 2D Array (Option 1).

**If IPC < 0.8 but LLC Miss Rate < 1%:**
→ Different bottleneck (not spatial grid). Check Archetype fragmentation.

**If L1 Miss Rate > 5%:**
→ Component layout issue. Entities are scattered across many Archetypes.

---

## Dev UI Instrumentation

To expose cache metrics in real-time, emit these counters from Rust:

```rust
// In perception system
#[cfg(feature = "dev-tools")]
{
    let cache_metrics = serde_json::json!({
        "type": "cache_metrics",
        "timestamp": SystemTime::now(),
        "perception": {
            "entities_queried": query.iter().count(),
            "grid_cells_accessed": grid.cells.len(),
            "avg_entities_per_cell": grid.cells.values().map(|v| v.len()).sum::<usize>() / grid.cells.len().max(1),
        }
    });
    println!("{}", cache_metrics);
}
```

Add to `apps/dev-ui/src/components/StateDisplay.tsx`:

```tsx
interface CacheMetrics {
  entities_queried: number;
  grid_cells_accessed: number;
  avg_entities_per_cell: number;
}

// In component
<div className="cache-metrics">
  <h3>Spatial Grid</h3>
  <div>Cells Accessed: {metrics.grid_cells_accessed}</div>
  <div>Avg Density: {metrics.avg_entities_per_cell.toFixed(1)}</div>
  <div className={metrics.avg_entities_per_cell > 50 ? 'warning' : ''}>
    {metrics.avg_entities_per_cell > 50 && '⚠️ High cell density - grid too coarse'}
  </div>
</div>
```

---

## References

- **Linux perf tutorial:** https://perf.wiki.kernel.org/index.php/Tutorial
- **Intel optimization manual:** Section 3.5 "Memory Access Optimization"
- **Rust Performance Book:** https://nnethercote.github.io/perf-book/
- **Bevy ECS internals:** https://bevyengine.org/learn/book/ecs/

---

**Document Owner:** cache-carl (Performance Analyst)
**Last Updated:** 2025-12-04
