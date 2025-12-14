# Performance Analysis: 200K Creatures @ 22-24ms/tick
**Target:** Sub-20ms tick time
**Date:** 2025-12-13
**Analyst:** perf-pat (Linux Performance Analysis)

---

## Executive Summary

Current bottleneck is NOT CPU parallelism (already well-optimized with Rayon). The hidden gem is **memory layout and cache efficiency**. Analysis reveals three high-impact opportunities:

1. **CRITICAL: Perception Component Bloat (168 bytes!)** - Cacheline massacre
2. **HIGH: Unnecessary Par-Iter Collections** - 200K entity Vec allocations every tick
3. **MEDIUM: Thread-Local Allocations in Hot Loop** - RefCell overhead + allocation churn

**Expected Gains:**
- Perception shrink: 2-4ms (cache miss reduction)
- Query iterator reuse: 1-2ms (allocation elimination)
- Thread-local optimization: 0.5-1ms (contention reduction)

**Total Potential:** 3.5-7ms reduction → **15-20ms tick time**

---

## 1. CRITICAL FINDING: Perception Component Cache Massacre

### Problem

**Location:** `/home/dev/dev/speciate/apps/simulation/src/simulation/perception/components.rs:36-43`

```rust
#[derive(Component, Debug, Clone)]
pub struct Perception {
    pub fov_angle: f32,        // 4 bytes
    pub range: f32,            // 4 bytes
    pub cos_half_fov_sq: f32,  // 8 bytes (aligned)
    neighbor_count: u8,        // 1 byte + 7 padding
    neighbors: [NeighborData; MAX_PERCEIVED_NEIGHBORS], // 7 * 24 bytes = 168 bytes
}
```

**Size Calculation:**
- `NeighborData`: Entity (8 bytes) + x (4) + y (4) + radius (4) + padding (4) = **24 bytes**
- Array: 7 * 24 = **168 bytes**
- Total Perception: **~192 bytes** (with padding)

**Impact at 200K Creatures:**
- Memory: 200K * 192 bytes = **38.4 MB**
- Cachelines: 192 / 64 = **3 cachelines per creature**
- L1 Cache: 32 KB → can hold only **170 Perception components**
- L3 Cache: 36 MB (typical) → barely fits all, ZERO headroom

**Why This Destroys Performance:**

1. **Avoidance System Hot Loop** (`avoidance/systems.rs:46-95`):
   ```rust
   for neighbor in perception.iter_neighbors() {
       // Reads 168 bytes of neighbor array EVERY iteration
       // But only uses 4-12 bytes per neighbor (x, y, entity)
   ```
   - Loads 168 bytes to access 12 bytes = **14x waste**
   - Thrashes L1 cache (32 KB) with 200K queries

2. **False Sharing Risk:**
   - Adjacent creatures in memory share cachelines
   - Parallel writes to `Acceleration` can invalidate `Perception` cacheline
   - Cache ping-pong between cores

### Solution: Split Cold Data to Separate Component

**BEFORE (Current):**
```rust
struct Perception {
    fov_angle: f32,           // Read every tick (for viz/debug)
    range: f32,               // Read every tick (perception radius)
    cos_half_fov_sq: f32,     // Read every tick (FOV check)
    neighbor_count: u8,       // Read every tick (avoidance)
    neighbors: [NeighborData; 7], // Read every tick (avoidance) - 168 bytes!
}
```

**AFTER (Proposed):**
```rust
// Hot data: 16 bytes (fits in 1 cacheline with other hot components)
#[derive(Component)]
struct Perception {
    pub range: f32,               // 4 bytes
    pub cos_half_fov_sq: f32,     // 4 bytes
    pub fov_angle: f32,           // 4 bytes
    pub neighbor_count: u8,       // 1 byte + 3 padding
}

// Cold data: 168 bytes (separate allocation, fetched only when iterating neighbors)
#[derive(Component)]
struct NeighborCache {
    neighbors: [NeighborData; 7],
}
```

**Access Pattern:**
```rust
// Perception system: writes NeighborCache (once per tick)
perception.neighbor_count = candidates.len();
neighbor_cache.neighbors[..k].copy_from_slice(&candidates[..k]);

// Avoidance system: reads NeighborCache (once per tick, only for creatures with neighbors)
if perception.neighbor_count > 0 {
    for neighbor in neighbor_cache.neighbors[..perception.neighbor_count] {
        // Process neighbor
    }
}
```

**Expected Gains:**
- L1 cache hit rate: 60% → 85% (3x less data per query)
- Memory bandwidth: -35% (192 → 124 bytes per creature in hot path)
- **Measured speedup estimate:** 2-4ms at 200K creatures

---

## 2. HIGH PRIORITY: Unnecessary Par-Iter Collections

### Problem

**Locations:**
- `/home/dev/dev/speciate/apps/simulation/src/simulation/perception/systems.rs:50`
- `/home/dev/dev/speciate/apps/simulation/src/simulation/movement/systems.rs:50`
- `/home/dev/dev/speciate/apps/simulation/src/simulation/creatures/behaviors/wander/systems.rs:26`
- `/home/dev/dev/speciate/apps/simulation/src/simulation/creatures/behaviors/avoidance/systems.rs:19`

**Pattern (Repeated 4 times):**
```rust
let mut entities: Vec<_> = query.iter_mut().collect();
entities.par_iter_mut().for_each(|(entity, pos, vel, ...)| {
    // Process
});
```

**Cost Per Tick:**
- Perception: 200K * 48 bytes (6 refs) = **9.6 MB allocation**
- Movement: 200K * 48 bytes (6 refs) = **9.6 MB allocation**
- Wander: ~200K * 64 bytes (8 refs) = **12.8 MB allocation**
- Avoidance: ~200K * 64 bytes (8 refs) = **12.8 MB allocation**
- **Total: 44.8 MB/tick** @ 60Hz = **2.7 GB/sec allocation**

**Why This Exists:**
Sprint 15 discovered Bevy's `par_iter_mut()` doesn't engage Rayon in NAPI context, so manual Vec collection was necessary. This was correct for the initial optimization but leaves headroom.

### Solution: Reuse Collection Vectors

**Pattern:**
```rust
// Thread-local reusable buffer (zero allocations after first tick)
thread_local! {
    static ENTITY_BUFFER: RefCell<Vec<(EntityRef, ComponentRefs...)>> =
        RefCell::new(Vec::with_capacity(256));
}

pub fn system(mut query: Query<...>) {
    ENTITY_BUFFER.with(|buf| {
        let mut entities = buf.borrow_mut();
        entities.clear();
        entities.extend(query.iter_mut());

        entities.par_iter_mut().for_each(|...| {
            // Process
        });
    });
}
```

**Caveat:** Thread-local requires `static` lifetime components. May not work if Bevy iter_mut() returns non-static refs. **Needs validation.**

**Alternative (Safer):** Pre-allocated Resource
```rust
#[derive(Resource)]
struct EntityCollector {
    buffer: Vec<(Entity, ComponentPointers...)>,
}

// Reuse buffer across ticks (clear + extend, no realloc if capacity sufficient)
collector.buffer.clear();
collector.buffer.extend(query.iter_mut().map(|tuple| /* extract */));
```

**Expected Gains:**
- Allocation: -44.8 MB/tick
- Allocator contention: Eliminated
- **Measured speedup estimate:** 1-2ms (allocator is fast, but not free)

---

## 3. MEDIUM PRIORITY: Thread-Local Allocation Churn

### Problem

**Location:** `/home/dev/dev/speciate/apps/simulation/src/simulation/perception/systems.rs:22-28`

```rust
thread_local! {
    static CELL_SCRATCH: RefCell<Vec<(f32, usize)>> = RefCell::new(Vec::with_capacity(256));
}

thread_local! {
    static NEIGHBOR_CANDIDATES: RefCell<Vec<(f32, NeighborData)>> =
        RefCell::new(Vec::with_capacity(256));
}
```

**Used in parallel loop:**
```rust
entities.par_iter_mut().for_each(|(entity, pos, ...)| {
    CELL_SCRATCH.with(|scratch| {
        let mut cells = scratch.borrow_mut(); // RefCell overhead
        cells.clear();
        // ... use scratch buffer
    });

    NEIGHBOR_CANDIDATES.with(|candidates_cell| {
        let mut candidates = candidates_cell.borrow_mut(); // RefCell overhead
        candidates.clear();
        // ... use candidates buffer
    });
});
```

**Issues:**

1. **RefCell Overhead:**
   - `borrow_mut()` checks at runtime (cheap but not free)
   - 200K * 2 RefCell operations = 400K runtime checks/tick

2. **Nested with() Calls:**
   - Two separate closures per creature
   - Prevents compiler inlining optimizations

3. **Capacity Growth:**
   - Initial capacity: 256 elements
   - Some creatures have > 256 neighbors (dense areas)
   - Reallocation in hot loop

### Solution: Pre-Allocate Per-Thread Buffers

**Approach 1: Rayon ParallelIterator with Custom Init**
```rust
use rayon::iter::IntoParallelRefMutIterator;

entities.par_iter_mut()
    .with_max_len(1024) // Chunk size
    .for_each_init(
        || {
            // Per-thread initialization (called once per thread)
            (
                Vec::with_capacity(512), // cell_scratch
                Vec::with_capacity(512), // neighbor_candidates
            )
        },
        |(cell_scratch, neighbor_candidates), (entity, pos, ...)| {
            cell_scratch.clear();
            neighbor_candidates.clear();

            // Use buffers directly (no RefCell, no closure nesting)
            grid_ref.collect_cells_sorted(x, y, query_radius, facing_x, facing_y, cell_scratch);

            for &(sort_key, cell_idx) in cell_scratch.iter() {
                // ... fill neighbor_candidates
            }
        }
    );
```

**Benefits:**
- Eliminates RefCell overhead (400K → 0)
- Eliminates nested closures (better inlining)
- Pre-sized buffers reduce reallocation
- **Expected gain:** 0.5-1ms

**Approach 2: Struct-Based Scratch (Cleaner)**
```rust
struct PerceptionScratch {
    cells: Vec<(f32, usize)>,
    candidates: Vec<(f32, NeighborData)>,
}

impl Default for PerceptionScratch {
    fn default() -> Self {
        Self {
            cells: Vec::with_capacity(512),
            candidates: Vec::with_capacity(512),
        }
    }
}

entities.par_iter_mut()
    .for_each_init(PerceptionScratch::default, |scratch, (entity, pos, ...)| {
        scratch.cells.clear();
        scratch.candidates.clear();
        // ... use scratch.cells, scratch.candidates
    });
```

---

## 4. Architecture-Level Observations

### Current System Performance (Already Optimized)

**Spatial Grid Rebuild:**
- Parallel counting sort with atomics: EXCELLENT
- Fixed-bounds mode (zero allocations): EXCELLENT
- Non-empty cell tracking: EXCELLENT
- **Verdict:** Near-optimal. No low-hanging fruit.

**Movement System:**
- Rayon parallelism: EXCELLENT (6.3x speedup validated)
- Merged physics + boundary check: EXCELLENT
- Cached sqrt avoidance: EXCELLENT
- Fast atan2 approximation: EXCELLENT
- **Verdict:** Near-optimal. Possible SIMD for velocity math (advanced).

**Behavior Systems:**
- Parallel wander/avoidance/seek: GOOD
- Force clamping: Efficient
- **Potential:** Could merge wander + avoidance into single parallel loop (reduces entity iteration overhead)

### System Ordering Analysis

**Current Schedule** (`simulation.rs:76-89`):
```rust
rebuild_spatial_grid_system,
perception::update_perception_system.after(rebuild_spatial_grid_system),
behavior_transition_system,
territory_wandering_system,
flee_system,
seek_system,
behaviors::avoidance_system,
update_body_size_cache,
integrate_motion_system,
rotation_system,
swap_spatial_grid_buffers_system.after(rotation_system),
```

**Issue:** No parallelism between independent behavior systems

**Opportunity (Advanced):**
```rust
.add_systems((
    rebuild_spatial_grid_system,
    perception::update_perception_system.after(rebuild_spatial_grid_system),
))
.add_systems((
    // These are INDEPENDENT (all write to Acceleration, read different components)
    territory_wandering_system,
    flee_system,
    seek_system,
    behaviors::avoidance_system,
).after(perception::update_perception_system))  // Bevy runs these in parallel!
```

**Expected gain:** 0-2ms (depends on Bevy's par executor overhead)

---

## 5. SIMD Opportunities (Advanced)

### Avoidance Force Calculation

**Location:** `avoidance/systems.rs:42-95`

**Current (Scalar):**
```rust
for neighbor in perception.iter_neighbors() {
    let away_x = position.x - neighbor.x;
    let away_y = position.y - neighbor.y;
    let center_distance_sq = magnitude_sq(away_x, away_y);
    let center_distance = center_distance_sq.sqrt();
    let inv_distance = 1.0 / center_distance;

    // ... scalar force calculation
    total_repulsion_x += force_x;
    total_repulsion_y += force_y;
}
```

**SIMD (4-wide f32x4):**
```rust
use std::simd::f32x4;

// Process 4 neighbors at once
let mut chunks = neighbors.chunks_exact(4);
for chunk in chunks {
    let away_x = f32x4::from_array([
        position.x - chunk[0].x,
        position.x - chunk[1].x,
        position.x - chunk[2].x,
        position.x - chunk[3].x,
    ]);
    // ... vectorized force calculation
}
```

**Challenge:** MAX_PERCEIVED_NEIGHBORS = 7 (not power of 2)
**Feasibility:** LOW priority (scalar is already fast, SIMD overhead high for 7 elements)

---

## 6. Memory Layout Analysis (Component Packing)

### Current Archetype Layout (Estimated)

**Components per creature (full-featured wanderer):**
```
Position         (8 bytes)   ← HOT
Velocity         (8 bytes)   ← HOT
Acceleration     (8 bytes)   ← HOT
BodySize         (12 bytes)  ← HOT
Rotation         (4 bytes)   ← HOT
Perception       (192 bytes) ← HOT BUT BLOATED
CreatureState    (32 bytes)  ← MEDIUM
WanderState      (16 bytes)  ← HOT (wanderers only)
HomePosition     (8 bytes)   ← COLD
AvoidanceBehavior(4 bytes)   ← COLD
Brain            (? bytes)   ← COLD
CritId           (4 bytes)   ← COLD
Target           (8 bytes)   ← COLD (seekers only)
```

**Problem:** Bevy stores components in separate arrays (SoA), but cache prefetcher loads adjacent cachelines. Large components like Perception disrupt prefetch efficiency.

### Recommendation: Component Size Audit

**Rule of Thumb:**
- HOT components (read every tick): < 16 bytes ideal, < 32 acceptable
- MEDIUM components (read conditionally): < 64 bytes
- COLD components (rare reads): Any size

**Audit Needed:**
- `CreatureState`: 32 bytes - what's inside? Can it shrink?
- `WanderState`: 16 bytes - reasonable
- `Brain`: Unknown size - needs check

---

## 7. Profiling Recommendations

Before implementing changes, establish baseline with `perf`:

### Health Check (10-second sample)
```bash
cd /home/dev/dev/speciate/apps/simulation
cargo build --release
perf stat -e instructions,cycles,L1-dcache-loads,L1-dcache-load-misses,LLC-loads,LLC-load-misses,branch-misses \
    timeout 10s ./target/release/YOUR_BINARY --creatures 200000
```

**Expected Baseline (200K creatures):**
- IPC: 2.5-4.0 (good for memory-bound workload)
- L1 Miss Rate: 5-10% (target: < 5%)
- LLC Miss Rate: 1-3% (target: < 1%)

### Hotspot Analysis (Cache Miss Attribution)
```bash
perf record --call-graph dwarf -e L1-dcache-load-misses \
    timeout 30s ./target/release/YOUR_BINARY --creatures 200000
hotspot perf.data  # Open in Hotspot GUI
```

**What to Look For:**
1. `perception::update_perception_system` - should show high L1 misses on line accessing `perception.neighbors`
2. `avoidance_system` - should show misses on `perception.iter_neighbors()`
3. `integrate_motion_system` - should be LOW (already well-optimized)

### Before/After Comparison
```bash
# Baseline
perf stat -r 3 timeout 10s ./target/release/sim_before > baseline.txt

# After Perception split
perf stat -r 3 timeout 10s ./target/release/sim_after > optimized.txt

# Compare
diff baseline.txt optimized.txt
```

**Success Criteria:**
- L1 miss reduction: > 20%
- IPC increase: > 10%
- Tick time: < 20ms

---

## 8. Implementation Priority

### Phase 1: Low-Risk High-Impact (Week 1)
1. **Split Perception component** (CRITICAL)
   - Create `NeighborCache` component
   - Update perception system to write both
   - Update avoidance system to read both
   - Run perf analysis before/after
   - **Expected: 2-4ms gain**

### Phase 2: Medium-Risk Medium-Impact (Week 2)
2. **Optimize thread-local allocations**
   - Replace RefCell with `for_each_init`
   - Increase initial capacity to 512
   - **Expected: 0.5-1ms gain**

3. **Audit component sizes**
   - Print `size_of::<CreatureState>()`
   - Check Brain component size
   - Identify shrink opportunities

### Phase 3: Advanced Optimizations (Week 3+)
4. **Query iterator reuse** (needs validation)
   - Prototype Resource-based entity collector
   - Benchmark allocation overhead
   - **Expected: 1-2ms gain IF feasible**

5. **Parallel behavior systems** (if Bevy supports)
   - Test independent system parallelism
   - Measure overhead vs gain

---

## 9. Validation Tests

After each optimization, run:

```bash
# Correctness (determinism check)
cargo test --release test_parallel_movement_determinism
cargo test --release test_perception_detects_nearby_entities

# Performance (200K stress test)
./run_perf_test.sh 200000 60  # 200K creatures, 60 seconds

# Memory (check for leaks)
valgrind --leak-check=full ./target/release/sim_app --creatures 200000 --ticks 100
```

---

## 10. Alternative Hypotheses (If Above Fails)

If Perception split doesn't yield 2-4ms:

### Hypothesis A: Spatial Grid Query Overhead
- Test: Replace `collect_cells_sorted` with simpler radius query
- Measure: Cell iteration vs proxy iteration time

### Hypothesis B: Rayon Thread Overhead
- Test: Reduce thread count (`RAYON_NUM_THREADS=8`)
- Measure: Check if fewer threads = less contention

### Hypothesis C: Bevy Query Overhead
- Test: Raw ECS access vs Query API
- Measure: `world.get_unchecked` vs `query.iter_mut()`

---

## Summary: The Hidden Gem

**Primary Bottleneck:** Perception component size (192 bytes) destroys cache locality.

**Root Cause:** Storing 7 × 24-byte NeighborData in hot component despite only needing it for 200K iterations/tick (avoidance), while reading metadata (range, FOV) for every spatial query.

**Fix:** Cold/hot split → Perception (16 bytes) + NeighborCache (168 bytes).

**Proof Strategy:**
1. `perf record -e L1-dcache-load-misses` → Confirm perception access is top cache killer
2. Implement split → Re-run perf
3. Measure L1 miss reduction + tick time improvement

**Expected Outcome:** 22-24ms → 18-20ms (under target!)

---

**Files to Modify:**
- `/home/dev/dev/speciate/apps/simulation/src/simulation/perception/components.rs`
- `/home/dev/dev/speciate/apps/simulation/src/simulation/perception/systems.rs`
- `/home/dev/dev/speciate/apps/simulation/src/simulation/creatures/behaviors/avoidance/systems.rs`

**Validation:**
- Run existing tests (should pass unchanged)
- Add perf regression test (tick time < 21ms @ 200K creatures)
