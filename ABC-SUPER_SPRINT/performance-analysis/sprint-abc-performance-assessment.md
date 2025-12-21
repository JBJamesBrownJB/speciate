# Performance Assessment: Sprint A-B-C (Dual Grid + Drive Simplex + Frequency Control)

**Analyst:** claude-perf
**Date:** 2025-12-21
**Implementation Order:** Phase A → Phase C → Phase B

---

## Executive Summary

**Recommendation:** Proceed with implementation order A → C → B with mandatory performance gates.

**High-Risk Items:**
1. L1 aggregation overhead (Phase A): Target < 0.5ms at 360K creatures
2. Drive computation scalability (Phase B): Per-creature L1 scan cost unknown
3. Throttling overhead verification (Phase C): Must prove zero-cost at divisor=1

**Low-Risk Items:**
- Early-exit optimization (Phase A): Expected latency reduction in sparse areas
- Frequency bucketing (Phase C): Proven pattern from prior art

---

## Baseline Performance Context

### Current State (360K Creatures @ 20Hz)

**Source:** `/home/dev/dev/speciate/docs/performance/snapshots/NOW.json`

```
Total Tick:     29.3ms avg (p95: 29.9ms)
├─ Steering:     9.4ms (32% of tick)
├─ Movement:     6.0ms (21%)  [Rayon-parallelized, IPC 4.25]
├─ Perception:   5.9ms (20%)  [Rayon-parallelized]
├─ Grid Rebuild: 4.2ms (14%)
└─ Behavior:     2.0ms (7%)

Hardware Profile:
├─ IPC:              1.68 (memory-bound, below optimal)
├─ L1D Miss Rate:    3.4% (acceptable)
├─ LLC Miss Rate:    3.0% (high - DRAM stalls)
├─ Branch Miss Rate: 0.02% (excellent)
└─ CPU Utilization:  45% (11 cores engaged)
```

**Key Constraints:**
- World: 10km × 10km (±5km)
- L0 Cell Size: 10m
- Total L0 Cells: 1,000,000 (1000 × 1000)
- Non-Empty L0 Cells: ~1,000 at 20K creatures (estimated from density)
- Tick Budget: 50ms @ 20Hz (currently using 29ms = 58% budget)

---

## Phase A: Dual Spatial Grid

### Architecture

**L1 Grid:**
- Cell Size: 30m (3×3 L0 cells)
- Total L1 Cells: 111,111 (333 × 333)
- BioSignature: `{ total_mass: f32, max_size: f32 }`

**Aggregation Algorithm:**
```
For each non-empty L0 cell:
  1. Compute parent L1 cell index (integer division)
  2. Accumulate total_mass += creature_mass
  3. Update max_size = max(max_size, creature_size)
```

### Performance Concern 1: L1 Aggregation Overhead

**Question:** What is the per-tick cost of L0 → L1 reduction?

**Baseline Estimate:**

At 360K creatures with uniform distribution:
- Non-empty L0 cells: ~360,000 ÷ 1 = 360,000 (worst case: 1 creature per cell)
- Realistic: ~50K-100K non-empty cells (3-6 creatures per cell average)
- Operation per L0 cell: 1 integer division, 1 float add, 1 float max
- Sequential cost: 100K × 10ns = 1ms
- Parallel cost: With 16 cores, ~0.1ms

**Target:** < 0.5ms at 360K creatures

**Measurement Plan:**

```bash
# Add instrumentation to L1 aggregation system
# File: apps/simulation/src/simulation/spatial/systems.rs

pub fn aggregate_l1_biosignatures(
    tick: Res<PhysicsTick>,
    mut grid: ResMut<HierarchicalGrid>,
    mut timings: ResMut<SystemTimings>,
) {
    let start = std::time::Instant::now();

    // Clear L1 grid
    grid.l1.clear();

    // Aggregate L0 → L1
    for (l0_idx, entities) in grid.l0.non_empty_cells() {
        let l1_idx = l0_idx_to_l1_idx(l0_idx);
        for proxy in entities {
            grid.l1[l1_idx].total_mass += proxy.radius.powi(2);  // Approx mass
            grid.l1[l1_idx].max_size = grid.l1[l1_idx].max_size.max(proxy.radius);
        }
    }

    timings.l1_aggregation_us = start.elapsed().as_micros() as u32;
}
```

**Validation:**
- Run `perf stat -e L1-dcache-load-misses` during aggregation
- L1 cache miss target: < 1% (sequential scan of non-empty cells is cache-friendly)
- Compare against steering system (9.4ms) - aggregation should be ~20× faster

**Risk Level:** LOW
- Simple reduction operation
- Cache-friendly access pattern (sequential scan)
- Parallelization opportunity if needed (Rayon over non-empty cells)

---

### Performance Concern 2: Early-Exit Optimization

**Claim:** Skip L0 scan when L1 cell is "empty" (`total_mass < perception_threshold`)

**Latency Model:**

Current perception (no early-exit):
```
For each creature:
  1. collect_cells_sorted_fov() → 100-500 cells depending on range
  2. Scan each cell's proxies → 0-10 entities per cell
  3. Filter by FOV, distance → K nearest neighbors
```

With early-exit:
```
For each creature:
  1. Check L1 cell: if total_mass < threshold → DONE (0 neighbors)
  2. Otherwise: proceed with L0 scan
```

**Expected Improvement:**

| Scenario | Current | With Early-Exit | Speedup |
|----------|---------|-----------------|---------|
| Isolated creature (empty area) | 5.9ms | ~0.1ms (threshold check only) | 59× |
| Dense crowd (20+ neighbors) | 5.9ms | 5.9ms (no early-exit) | 1× |
| Mixed (50% isolated) | 5.9ms | ~3ms | 2× |

**Measurement Plan:**

```bash
# Add metric to track early-exit hit rate
# File: apps/simulation/src/simulation/perception/systems.rs

let mut early_exit_count = 0;
let mut full_scan_count = 0;

entities.par_iter_mut().for_each(|(..., perception)| {
    let l1_cell = grid.l1.get_cell(pos.x, pos.y);
    if l1_cell.total_mass < perception.threshold {
        early_exit_count.fetch_add(1, Ordering::Relaxed);
        return;
    }
    full_scan_count.fetch_add(1, Ordering::Relaxed);
    // ... L0 scan
});

// Emit to dev-ui:
// early_exit_rate = early_exit_count / (early_exit + full_scan)
```

**Validation:**
- Spawn 10K creatures in corner (0-1000, 0-1000)
- Rest of world empty (90% of cells)
- Measure perception system time: expect 70-80% reduction
- Run determinism test: `cargo test test_deterministic_simulation_20k`

**Risk Level:** LOW
- Early-exit is pure optimization (doesn't change behavior when disabled)
- Threshold check is O(1) overhead
- IPC measurement: expect increase from 1.68 → 2.0+ (less memory-bound)

---

### Performance Concern 3: Size Domination Asymmetry

**Biological Claim:** Large creatures don't "see" small ones (below perception threshold)

**Performance Impact:** POSITIVE (fewer neighbors to process)

**Example:**
- Giant (mass 1000, threshold 50)
- Mouse (mass 1, threshold 0.05)
- Giant ignores Mouse (Mouse mass < 50)
- Mouse sees Giant (Giant mass > 0.05)

**Result:** Giants have smaller neighbor sets → faster perception

**Measurement:**
- Spawn 1K giants (size 5m) + 10K mice (size 0.5m)
- Measure perception time for giants vs mice
- Expect: Giant perception 5-10× faster (fewer neighbors pass threshold)

**Risk Level:** NONE (emergent benefit)

---

## Phase C: System Update Frequency

### Architecture

**Bucketing Algorithm:**
```rust
let l1_cell_idx = grid.l1.position_to_cell_index(pos.x, pos.y);
let current_bucket = (tick.get() as usize) % divisor;

if l1_cell_idx % divisor != current_bucket {
    return;  // Skip this creature this tick
}
```

**Key Properties:**
- Spatial coherence: Nearby creatures update together (same L1 cell)
- Uniform distribution: L1 cell count (111K) >> typical divisor (10)
- Zero overhead at divisor=1: Early return in conditional

---

### Performance Concern 4: Zero-Overhead Claim (divisor=1)

**Claim:** "When divisor=1, zero overhead"

**Reality Check:**

With divisor=1, the code becomes:
```rust
if l1_cell_idx % 1 != (tick % 1) {  // Always false
    return;
}
```

**Compiler Optimization Analysis:**

Option 1: Add fast path (RECOMMENDED):
```rust
if divisor == 1 {
    // Existing code path, unchanged
    entities.par_iter_mut().for_each(|...| { /* work */ });
    return;
}

// Throttled path
let current_bucket = (tick.get() as usize) % divisor;
entities.par_iter_mut().for_each(|(entity, pos, ...)| {
    let l1_cell = grid.l1.position_to_cell_index(pos.x, pos.y);
    if l1_cell % divisor != current_bucket {
        return;
    }
    // ... work
});
```

Option 2: Trust compiler (NOT RECOMMENDED):
```rust
// Compiler may not optimize modulo checks
let current_bucket = (tick.get() as usize) % divisor;  // divisor=1 → always 0
entities.par_iter_mut().for_each(|...| {
    if l1_cell % divisor != current_bucket {  // Adds 1 div + 1 cmp per creature
        return;
    }
});
```

**Overhead Estimate (Option 2):**
- Per-creature cost: 1 integer division (~10 cycles) + 1 branch (~1 cycle)
- 360K creatures: 3.96M operations
- Cost: 3.96M × 11 cycles ÷ 3.2GHz = 0.014ms
- Verdict: Negligible even WITHOUT fast path

**Recommendation:** Use fast path (Option 1) for clarity, not performance.

**Validation:**
```bash
# Benchmark divisor=1 vs baseline (no frequency control)
cargo bench --bench simulation_bench -- --exact "perception_360k"

# Compare:
# - Baseline (no frequency control code)
# - Divisor=1 with fast path
# - Divisor=1 WITHOUT fast path (compiler-only)

# Expectation: All three within 2% margin of error
```

**Risk Level:** NONE (negligible overhead even in worst case)

---

### Performance Concern 5: Frequency Bucketing Granularity

**Question:** Is L1 cell bucketing (111K cells, divisor 10) the right granularity?

**Spatial Distribution Analysis:**

At divisor=10:
- Creatures per bucket: 360K ÷ 10 = 36K
- L1 cells per bucket: 111K ÷ 10 = 11.1K
- Average creatures per active L1 cell: 36K ÷ 11.1K = 3.2

**Comparison to Entity-Based Bucketing:**

| Method | Creatures/Bucket | Spatial Locality |
|--------|------------------|------------------|
| Entity ID % divisor | 36K | None (random) |
| L1 cell % divisor | 36K | High (nearby update together) |

**Benefit of Spatial Bucketing:**
- Shared L0 cache: Nearby creatures in same bucket → reuse L0 scan results
- Example: 10 creatures in same L1 cell scan same 9 L0 cells
- Cache-friendly: L1 scan for drive computation hits hot cache lines

**Measurement:**
```bash
# Compare cache miss rates: entity-based vs L1-based bucketing
perf stat -e L1-dcache-load-misses,LLC-load-misses \
  timeout 10s ./target/release/sim_app

# Expectation: L1-based bucketing reduces cache misses by 20-40%
```

**Risk Level:** LOW (spatial bucketing strictly better than random)

---

### Throttling Budget Analysis

**Scenario:** Perception at divisor=10 (10Hz effective rate)

**Current Cost:**
- Perception @ 20Hz: 5.9ms per tick
- Perception @ 10Hz: 5.9ms ÷ 2 = 2.95ms per tick (50% reduction)

**Savings:**
- Time freed: 5.9ms - 2.95ms = 2.95ms
- Available for: More creatures, complex behaviors, or lower tick budget

**Target Scenarios:**

| Creatures | Perception Hz | Tick Budget | Result |
|-----------|---------------|-------------|--------|
| 360K | 20 (divisor=1) | 29.3ms | Current baseline |
| 360K | 10 (divisor=2) | ~26ms | 11% improvement |
| 500K | 10 (divisor=2) | ~36ms | 28% more creatures |
| 360K | 5 (divisor=4) | ~24ms | 18% improvement |

**Validation:**
- Run 360K creatures with divisor=1,2,4,8
- Measure: totalTickUs, perceptionUs
- Verify: Linear scaling (divisor=2 → 50% reduction)

---

## Phase B: Simple Drive Simplex

### Architecture

**Drive Computation (per creature):**
```rust
// Layer 1: L1 Navigation Drive
let creature_l1_cell = grid.l1.position_to_cell(pos.x, pos.y);
let perception_l1_radius = perception_range / L1_CELL_SIZE;  // ~3-5 cells

for neighbor_l1_cell in l1_grid.query_radius(creature_l1_cell, perception_l1_radius) {
    let bio = neighbor_l1_cell.biosignature;

    // Repulsion from large crits
    if bio.max_size > self_size {
        let repulsion_vec = compute_repulsion(bio, self_pos);
        drive += repulsion_vec;
    }

    // Attraction to empty space
    if bio.total_mass < threshold {
        let attraction_vec = compute_attraction(bio, self_pos);
        drive += attraction_vec;
    }
}

// Layer 2: L0 Avoidance (existing system, unchanged)
for neighbor in l0_neighbors {
    avoidance_vec += lateral_dodge(neighbor);
}
```

---

### Performance Concern 6: Drive Computation Cost

**Question:** What is the per-creature cost of L1 drive computation?

**Complexity Analysis:**

For creature with perception range 100m:
- L1 radius: 100m ÷ 30m = 3.33 cells
- L1 cells scanned: π × (3.33)² ≈ 35 cells
- Operations per L1 cell:
  - Load BioSignature (2 floats, 8 bytes) → 1 cache line
  - Compare max_size, total_mass → 2 float cmps
  - Compute repulsion/attraction vector → 10 flops
- Total per creature: 35 cells × 12 ops = 420 ops

**Cost Estimate:**
- 360K creatures × 420 ops = 151M operations
- At 3.2GHz, IPC 2.0: 151M ÷ (3.2GHz × 2.0) = 23.6ms
- Parallelized (16 cores): 23.6ms ÷ 16 = 1.5ms

**Comparison to Current Behavior System:**
- Current behavior: 2.0ms (includes state transitions, brain updates)
- Drive system: 1.5ms (simple force accumulation)
- Verdict: COMPETITIVE (may be faster due to simpler logic)

**Risk Level:** MEDIUM
- Unknown: L1 cache miss rate during BioSignature scan
- Mitigation: BioSignatures are small (8 bytes), likely cache-friendly
- Need measurement before claiming performance win

---

### Measurement Plan: Drive System Baseline

**Benchmark:**
```bash
# Create isolated test for L1 drive computation
# File: apps/simulation/benches/drive_bench.rs

fn bench_l1_drive_computation(c: &mut Criterion) {
    let mut grid = setup_l1_grid_with_360k_creatures();
    let creatures = setup_creatures_with_perception_100m();

    c.bench_function("drive_l1_scan_360k", |b| {
        b.iter(|| {
            for creature in creatures.iter() {
                let drive = compute_l1_drive(&grid, creature);
                black_box(drive);
            }
        });
    });
}
```

**perf Profile:**
```bash
# Run with hardware counters
perf stat -e cycles,instructions,L1-dcache-load-misses,LLC-load-misses \
  cargo bench drive_l1_scan_360k

# Target metrics:
# - IPC: > 2.0 (compute-bound, not memory-bound)
# - L1 Miss Rate: < 2% (BioSignatures fit in L1 cache)
# - LLC Miss Rate: < 0.5% (no random access)
```

**Validation Gates:**
1. Drive computation < 2ms at 360K creatures (must not regress vs current behavior)
2. IPC > 1.8 (should be compute-bound, not memory-bound)
3. Determinism test passes (drive forces reproducible)

---

### Rayon Parallelization Strategy

**Current Pattern (Movement System):**
```rust
let mut entities: Vec<_> = query.iter_mut().collect();
entities.par_iter_mut().for_each(|(entity, ...)| {
    // Physics logic runs in parallel
});
```

**Drive System Pattern:**
```rust
let mut entities: Vec<_> = query.iter_mut().collect();
entities.par_iter_mut().for_each(|(entity, pos, perception, drive_state)| {
    // Read-only access to L1 grid (shared across threads)
    let drive = compute_l1_drive(&grid.l1, pos, perception);
    drive_state.direction = drive;  // Write to component
});
```

**Key Difference:**
- Movement: Reads/writes own components only
- Drive: Reads shared L1 grid (immutable during drive computation)

**Thread Safety:**
- L1 grid is Resource (read-only during drive system)
- Each thread writes to disjoint drive_state components
- No synchronization required

**Expected Scaling:**
- Single-threaded: 23.6ms (estimated)
- 16 cores (perfect scaling): 1.5ms
- Realistic (Rayon overhead): 2.0ms
- Matches current IPC 4.25 baseline (from movement system)

**Validation:**
```bash
# Compare single-threaded vs Rayon
cargo bench drive_single_threaded
cargo bench drive_rayon_parallel

# Measure with `perf stat`:
# - cpuCoresActive: should be 16
# - IPC: should match movement system (4.0-4.5)
```

---

## Recommended Benchmarking Strategy

### Phase A Benchmarks (Dual Grid)

**Test 1: L1 Aggregation Overhead**
```bash
# File: apps/simulation/benches/l1_aggregation_bench.rs

// Scenarios:
// - 20K creatures (baseline)
// - 100K creatures
// - 360K creatures (max)

// Metrics:
// - Aggregation time (target: < 0.5ms at 360K)
// - L1 cache miss rate (target: < 1%)
// - Parallelization factor (if using Rayon)
```

**Test 2: Early-Exit Effectiveness**
```bash
# File: apps/simulation/specs/performance/early_exit_sparse.toml

[[spawn]]
species = "wanderer"
min = 10000
max = 10000
# Cluster in corner (0-1000, 0-1000)
x_distribution = { min = 0, max = 1000 }
y_distribution = { min = 0, max = 1000 }

# Expectation: perception_system time 70-80% reduction vs uniform distribution
```

**Test 3: Size Domination**
```bash
# Spawn 1K giants + 10K mice, measure perception time per species
# Expectation: Giants process faster (fewer neighbors pass threshold)
```

---

### Phase C Benchmarks (Frequency Control)

**Test 1: Zero Overhead (divisor=1)**
```bash
# Compare three implementations:
# A. Baseline (no frequency control code)
# B. Frequency control with divisor=1 (fast path)
# C. Frequency control with divisor=1 (compiler-only, no fast path)

# Expectation: All three within 2% margin
```

**Test 2: Throttled Performance (divisor=2,4,8)**
```bash
# Measure totalTickUs at different divisors
# Expectation: Linear scaling (divisor=2 → 50% perception time reduction)
```

**Test 3: Spatial Bucketing vs Entity Bucketing**
```bash
# Compare cache miss rates:
# - L1 cell % divisor (spatial locality)
# - Entity ID % divisor (random distribution)

# Expectation: Spatial bucketing 20-40% fewer cache misses
```

---

### Phase B Benchmarks (Drive Simplex)

**Test 1: L1 Drive Computation Baseline**
```bash
# Isolated benchmark of L1 scan + force accumulation
# Target: < 2ms at 360K creatures (parallel)
# IPC target: > 1.8 (compute-bound)
```

**Test 2: Drive System vs Behavior State Machine**
```bash
# Compare:
# - Old: BehaviorMode transitions + wandering system
# - New: Drive simplex (L1 repulsion/attraction)

# Expectation: Drive system competitive or faster (simpler logic)
```

**Test 3: Emergent Behavior Validation**
```bash
# Visual validation in portal:
# - Creatures disperse from crowded areas (attraction to low mass)
# - Small creatures avoid large ones (repulsion from max_size)
# - Equilibrium: creatures rest when gradients balanced

# Performance: Measure resting rate (no drive → no acceleration → cheaper)
```

---

## Hardware Counter Targets

### Phase A (Dual Grid)

| Metric | Current (360K) | Target (with L1) | Rationale |
|--------|----------------|------------------|-----------|
| IPC | 1.68 | 1.8-2.0 | Early-exit reduces memory stalls |
| L1D Miss Rate | 3.4% | 3.0% | L1 aggregation is cache-friendly |
| LLC Miss Rate | 3.0% | 2.5% | Fewer L0 scans (early-exit) |
| Perception Time | 5.9ms | 3-4ms | 50% sparse, 50% dense (2× speedup in sparse) |
| L1 Aggregation Time | N/A | < 0.5ms | New system, must be negligible |

---

### Phase C (Frequency Control)

| Metric | Divisor=1 | Divisor=2 | Divisor=4 | Divisor=10 |
|--------|-----------|-----------|-----------|------------|
| Perception Time | 5.9ms | 3.0ms | 1.5ms | 0.6ms |
| Total Tick | 29.3ms | 26.4ms | 24.9ms | 23.6ms |
| Creatures Supported | 360K | 500K | 700K | 1M+ |

---

### Phase B (Drive Simplex)

| Metric | Current (Behavior) | Target (Drive) | Rationale |
|--------|--------------------|--------------------|-----------|
| Behavior/Drive Time | 2.0ms | < 2.0ms | Simpler logic (no state machine) |
| IPC | 1.68 | 2.0+ | L1 scan is compute-bound |
| L1D Miss Rate | 3.4% | 2.5% | BioSignatures are cache-friendly |

---

## Risk Mitigation Strategy

### High-Risk Item 1: L1 Aggregation Overhead

**Risk:** L1 aggregation becomes a new bottleneck (> 1ms)

**Mitigation:**
1. Benchmark FIRST before implementation (create test grid, measure reduction time)
2. Use Rayon if sequential scan exceeds 0.5ms
3. Track non-empty L0 cells during rebuild (avoid scanning all 1M cells)

**Abort Criteria:** If aggregation > 1ms at 360K, reconsider L1 grid design

---

### High-Risk Item 2: Drive Computation Scalability

**Risk:** L1 drive scan is memory-bound (poor IPC, high cache misses)

**Mitigation:**
1. Benchmark L1 scan in isolation BEFORE removing behavior state machine
2. Profile with `perf record --call-graph dwarf -e L1-dcache-load-misses`
3. Optimize BioSignature layout if cache misses high (e.g., pack into single cache line)

**Abort Criteria:** If drive computation > 3ms at 360K, keep behavior state machine as fallback

---

### Medium-Risk Item: Throttling Breaks Determinism

**Risk:** Frequency control introduces non-determinism (creatures updated in different order)

**Mitigation:**
1. Run determinism test at each divisor value: `cargo test test_deterministic_simulation_20k`
2. Ensure bucketing is stable (same L1 cell → same bucket across ticks)
3. Test save/load: serialized state must be identical regardless of divisor

**Validation:**
```bash
# Run 1000 ticks with divisor=1,2,4,8
# Save state at tick 1000
# Compare: all four states must be bit-identical
```

---

## Dev-UI Instrumentation Requirements

### Phase A Metrics

**L1 Grid Overlay:**
- Heatmap of `total_mass` per L1 cell (color intensity)
- Outline of L1 cell boundaries (30m grid)
- G key cycles: Off → L0 → L1 → Heatmap

**Performance Panel:**
- `l1_aggregation_us`: Time spent aggregating L0 → L1
- `early_exit_rate`: % of creatures skipping L0 scan
- `l1_cells_non_empty`: Count of L1 cells with mass > 0

---

### Phase C Metrics

**Frequency Control Panel:**
- Slider for each system: `perception_divisor`, `behavior_divisor`, `steering_divisor`
- Real-time update: sparkline changes immediately when divisor adjusted
- Display: "Effective Hz" = 20 / divisor (e.g., divisor=2 → 10Hz)

**Performance Impact:**
- Show: `creatures_processed_this_tick` vs `total_creatures`
- Example: 360K creatures, divisor=10 → ~36K processed

---

### Phase B Metrics

**Drive State Visualization:**
- Arrow overlay: Drive direction (L1 repulsion + attraction)
- Color code: Drive magnitude (red = strong, green = weak)
- Debug mode: Show L1 cells scanned for drive computation

**Performance Panel:**
- `drive_computation_us`: Time spent computing L1 forces
- `avg_l1_cells_per_creature`: Average L1 cells scanned per drive update
- `resting_creatures`: Count of creatures with zero drive (equilibrium)

---

## Recommended Implementation Gates

### Phase A: Dual Grid

**Gate 1: L1 Aggregation Performance**
- [ ] L1 aggregation < 0.5ms at 360K creatures
- [ ] L1 cache miss rate < 1% during aggregation
- [ ] Dev-UI shows L1 heatmap correctly

**Gate 2: Early-Exit Validation**
- [ ] Early-exit reduces perception time by 50%+ in sparse scenario
- [ ] Determinism test passes with early-exit enabled
- [ ] IPC increases from 1.68 → 1.8+

**Gate 3: Size Domination**
- [ ] Giants process faster than mice (fewer neighbors)
- [ ] Visual confirmation: Giants walk through mice, mice avoid giants

---

### Phase C: Frequency Control

**Gate 1: Zero Overhead**
- [ ] Divisor=1 within 2% of baseline (no frequency control code)
- [ ] Compiler optimization verified (assembly inspection or profiling)

**Gate 2: Throttling Accuracy**
- [ ] Divisor=2 reduces perception time by 50%
- [ ] Divisor=4 reduces perception time by 75%
- [ ] Linear scaling confirmed

**Gate 3: Determinism**
- [ ] Determinism test passes at divisor=1,2,4,8
- [ ] Save/load produces identical state regardless of divisor

---

### Phase B: Drive Simplex

**Gate 1: Drive Computation Performance**
- [ ] Drive computation < 2ms at 360K creatures (parallel)
- [ ] IPC > 1.8 during drive computation
- [ ] L1D miss rate < 3%

**Gate 2: Behavior Parity**
- [ ] Creatures disperse (visual confirmation)
- [ ] Small creatures avoid large ones (visual confirmation)
- [ ] No jittering at equilibrium (visual confirmation)

**Gate 3: State Machine Removal**
- [ ] All tests pass with BehaviorMode enum removed
- [ ] Wandering system deleted, no references remain

---

## Conclusion

**Proceed with implementation order A → C → B.**

**Confidence Levels:**
- Phase A (Dual Grid): HIGH (simple aggregation, proven early-exit pattern)
- Phase C (Frequency Control): HIGH (trivial bucketing, zero-cost at divisor=1)
- Phase B (Drive Simplex): MEDIUM (need to validate L1 scan performance)

**Critical Path:**
1. Phase A Gate 1 (L1 aggregation < 0.5ms) → MUST PASS before proceeding
2. Phase B Gate 1 (drive computation < 2ms) → MUST PASS before removing behavior state machine

**Expected Outcome:**
- 360K creatures: 29.3ms → 24ms (18% improvement from throttling)
- 500K creatures: Possible at divisor=2 (perception 10Hz)
- IPC: 1.68 → 2.0+ (less memory-bound due to early-exit)

**Final Validation:**
- Run full spec suite: `cargo test --release --package simulation --test specs`
- Run 20K determinism test: `cargo test test_deterministic_simulation_20k`
- Visual smoke test in portal: 10K creatures for 60 seconds, no crashes
