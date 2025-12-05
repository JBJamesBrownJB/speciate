# Spatial Grid Parallel Rebuild Strategy

## Baseline Performance (150K entities)
- **Current:** 5.5ms avg (4.4ms min, 5.8ms p95)
- **Hardware:** 16-core CPU, Rayon available
- **Density:** ~2000 active cells, 50m cell size
- **Data:** `/home/dev/dev/speciate/docs/performance/snapshots/150k_mixed_density_2025-12-05T18-05-00.json`

## Current Sequential Implementation

```rust
// Phase 0: Collect - O(N) - 150K entities
entity_scratch.extend(entities);

// Phase 1: Histogram - O(N) - Count entities per cell
for (_, x, y, _) in &entity_scratch {
    let idx = cell_index(x, y);
    cells[idx].1 += 1;
}

// Phase 2: Prefix Sum - O(cells) - ~2000 cells
let mut offset = 0;
for cell in &mut cells {
    cell.0 = offset;
    offset += cell.1;
    cell.1 = 0;
}

// Phase 3: Scatter - O(N) - Bin entities
for &(entity, x, y, radius) in &entity_scratch {
    let idx = cell_index(x, y);
    let write_pos = cells[idx].0 + cells[idx].1;
    proxies[write_pos] = PerceptionProxy { ... };
    cells[idx].1 += 1;
}
```

## Parallelization Opportunities

### 1. Phase 1: Histogram (Thread-Local Reduce Pattern)

**Strategy:** Partition entities, count locally, then merge histograms.

**Implementation:**
```rust
use rayon::prelude::*;

// Partition size: Balance contention vs cache efficiency
const CHUNK_SIZE: usize = 4096; // ~64KB of entity data (16 bytes each)

// Phase 1: Parallel histogram with thread-local counts
let local_histograms: Vec<Vec<u32>> = entity_scratch
    .par_chunks(CHUNK_SIZE)
    .map(|chunk| {
        let mut local_counts = vec![0u32; total_cells];
        for (_, x, y, _) in chunk {
            let idx = cell_index_unchecked(*x, *y);
            local_counts[idx] += 1;
        }
        local_counts
    })
    .collect();

// Merge phase: Sum local histograms into global counts
for cell in &mut cells {
    cell.1 = 0;
}
for local_hist in &local_histograms {
    for (i, &count) in local_hist.iter().enumerate() {
        cells[i].1 += count;
    }
}
```

**Expected Speedup:** 8-12x on 16 cores (memory-bound workload)
- **Rationale:** Histogram counting is embarrassingly parallel with no shared state during map phase
- **Merge overhead:** ~200us for 2000 cells x 16 threads (negligible)
- **Cache friendly:** 4096-entity chunks fit in L2 cache (256KB)

**Risk:** Merge phase could become bottleneck if cell count explodes (>100K cells)

### 2. Phase 2: Prefix Sum (Sequential - Unavoidable Dependency Chain)

**Status:** Cannot parallelize effectively.

**Rationale:** Each cell's offset depends on previous cell's offset + count. This is a sequential dependency.

**Parallel prefix sum algorithms exist** (Blelloch scan), but overhead is massive for small arrays:
- **Threshold:** Only worth it for >1M elements
- **Our case:** ~2000 cells = **0.2-0.5us** sequential time
- **Parallel overhead:** ~50-100us (context switches, barriers)

**Verdict:** Keep sequential. Not the bottleneck.

### 3. Phase 3: Scatter (Atomic Counter Approach)

**Strategy:** Use atomic increments to avoid contention during parallel scatter.

**Implementation:**
```rust
use std::sync::atomic::{AtomicU32, Ordering};

// Convert cells to atomic counters
let atomic_cells: Vec<(u32, AtomicU32)> = cells
    .iter()
    .map(|(offset, _)| (*offset, AtomicU32::new(0)))
    .collect();

// Parallel scatter with atomic increments
entity_scratch.par_iter().for_each(|&(entity, x, y, radius)| {
    let idx = cell_index_unchecked(x, y);
    let (start, counter) = &atomic_cells[idx];
    let local_count = counter.fetch_add(1, Ordering::Relaxed);
    let write_pos = (*start + local_count) as usize;

    // SAFETY: Atomic counter ensures unique write positions
    unsafe {
        *proxies.get_unchecked_mut(write_pos) = PerceptionProxy { x, y, radius, entity };
    }
});

// Convert back to regular cells
for (i, (offset, counter)) in atomic_cells.iter().enumerate() {
    cells[i] = (*offset, counter.load(Ordering::Relaxed));
}
```

**Expected Speedup:** 4-8x on 16 cores (atomic contention limits scaling)
- **Best case:** Uniform distribution (75 entities/cell) → minimal contention → 8x
- **Worst case:** Clustered distribution (1000 entities in few cells) → heavy contention → 2-3x

**Risk:** Atomic fetch_add can cause cache line bouncing if entities cluster in same cells.

### Alternative: Sort-Based Scatter (No Atomics)

**Strategy:** Sort entities by cell index, then scatter in parallel batches.

**Implementation:**
```rust
// Sort entities by cell index (parallel sort)
let mut sorted_entities: Vec<_> = entity_scratch
    .iter()
    .map(|&(entity, x, y, radius)| {
        let idx = cell_index_unchecked(x, y);
        (idx, entity, x, y, radius)
    })
    .collect();

sorted_entities.par_sort_unstable_by_key(|&(idx, ..)| idx);

// Now entities are grouped by cell - can scatter in parallel
// Each thread writes to disjoint cell ranges
sorted_entities.chunks(CHUNK_SIZE).enumerate().par_bridge().for_each(|(chunk_id, chunk)| {
    for &(idx, entity, x, y, radius) in chunk {
        let (start, count) = &mut cells[idx]; // Safe: disjoint access per chunk
        let write_pos = (*start + *count) as usize;
        proxies[write_pos] = PerceptionProxy { x, y, radius, entity };
        *count += 1;
    }
});
```

**Expected Speedup:** 6-10x on 16 cores
- **Sort cost:** 10-15ms for 150K elements (Rayon parallel sort)
- **Scatter gain:** ~5x speedup (no atomic contention)
- **Net result:** Sort overhead dominates → **NOT RECOMMENDED**

**Verdict:** Atomic approach is better. Sort adds ~10ms overhead.

## Recommended Implementation: Hybrid Approach

Parallelize **Phase 1 (histogram)** and **Phase 3 (scatter)** only.

```rust
pub fn rebuild(&mut self, entities: impl Iterator<Item = (Entity, f32, f32, f32)>) {
    use rayon::prelude::*;
    use std::sync::atomic::{AtomicU32, Ordering};

    // Phase 0: Collect (unchanged)
    self.entity_scratch.clear();
    self.entity_scratch.extend(entities);

    if self.entity_scratch.is_empty() { /* ... */ }

    // Find bounds (could parallelize with par_iter + reduce, but negligible gain)
    let (min_cx, max_cx, min_cy, max_cy) = self.entity_scratch
        .iter()
        .fold((i32::MAX, i32::MIN, i32::MAX, i32::MIN), |(min_x, max_x, min_y, max_y), (_, x, y, _)| {
            let (cx, cy) = self.world_to_cell(*x, *y);
            (min_x.min(cx), max_x.max(cx), min_y.min(cy), max_y.max(cy))
        });

    self.min_cell_x = min_cx - 1;
    self.min_cell_y = min_cy - 1;
    self.width = (max_cx - min_cx + 3) as usize;
    self.height = (max_cy - min_cy + 3) as usize;

    let total_cells = self.width * self.height;
    self.cells.resize(total_cells, (0, 0));
    self.proxies.resize(self.entity_scratch.len(), PerceptionProxy::default());

    // PARALLEL PHASE 1: Thread-local histograms
    const CHUNK_SIZE: usize = 4096;
    let local_histograms: Vec<Vec<u32>> = self.entity_scratch
        .par_chunks(CHUNK_SIZE)
        .map(|chunk| {
            let mut local_counts = vec![0u32; total_cells];
            for (_, x, y, _) in chunk {
                let idx = self.cell_index_unchecked(*x, *y);
                local_counts[idx] += 1;
            }
            local_counts
        })
        .collect();

    // Merge histograms (sequential, but fast for ~2000 cells)
    for cell in &mut self.cells {
        cell.1 = 0;
    }
    for local_hist in &local_histograms {
        for (i, &count) in local_hist.iter().enumerate() {
            self.cells[i].1 += count;
        }
    }

    // SEQUENTIAL PHASE 2: Prefix sum (not worth parallelizing)
    let mut offset = 0u32;
    for cell in &mut self.cells {
        cell.0 = offset;
        offset += cell.1;
        cell.1 = 0;
    }

    // PARALLEL PHASE 3: Atomic scatter
    let atomic_cells: Vec<(u32, AtomicU32)> = self.cells
        .iter()
        .map(|(offset, _)| (*offset, AtomicU32::new(0)))
        .collect();

    self.entity_scratch.par_iter().for_each(|&(entity, x, y, radius)| {
        let idx = self.cell_index_unchecked(x, y);
        let (start, counter) = &atomic_cells[idx];
        let local_count = counter.fetch_add(1, Ordering::Relaxed);
        let write_pos = (*start + local_count) as usize;

        unsafe {
            let proxy_ptr = self.proxies.as_mut_ptr().add(write_pos);
            *proxy_ptr = PerceptionProxy { x, y, radius, entity };
        }
    });

    // Convert atomic cells back to regular cells
    for (i, (offset, counter)) in atomic_cells.iter().enumerate() {
        self.cells[i] = (*offset, counter.load(Ordering::Relaxed));
    }
}
```

## Expected Performance (150K entities)

**Sequential Breakdown:**
- Phase 0 (collect): ~0.5ms (iterator overhead)
- Phase 1 (histogram): ~2.5ms (memory-bound, random access to cells array)
- Phase 2 (prefix sum): ~0.3us (2000 cells, cache-resident)
- Phase 3 (scatter): ~2.0ms (write proxies, increment counters)

**Parallel Breakdown:**
- Phase 0 (collect): ~0.5ms (unchanged)
- Phase 1 (parallel histogram): ~0.3ms (8-10x speedup, memory bandwidth saturated)
- Phase 2 (prefix sum): ~0.3us (unchanged, negligible)
- Phase 3 (atomic scatter): ~0.4ms (4-5x speedup, atomic contention)

**Total Expected:**
- **Current:** 5.5ms
- **Parallel:** 0.5 + 0.3 + 0.0003 + 0.4 = **1.2ms**
- **Speedup:** **4.6x** (5.5ms → 1.2ms)

## SIMD Opportunities (Future Optimization)

**Phase 1 Histogram:** Not applicable. Cell index calculation requires integer division (slow) and random memory access (no vectorization benefit).

**Phase 3 Scatter:** Not applicable. Write positions are computed dynamically and scattered (no contiguous access pattern).

**Verdict:** SIMD is NOT viable for spatial grid rebuild. Focus on Rayon parallelization.

## Validation Strategy

### 1. Correctness Tests

```rust
#[test]
fn test_parallel_rebuild_equivalence() {
    // Generate 150K random entities
    let entities: Vec<_> = (0..150000).map(|i| {
        (Entity::from_raw(i), rand_x(), rand_y(), rand_radius())
    }).collect();

    // Sequential rebuild
    let mut grid_seq = SpatialGrid::new(50.0);
    grid_seq.rebuild(entities.iter().cloned());

    // Parallel rebuild
    let mut grid_par = SpatialGrid::new(50.0);
    grid_par.rebuild_parallel(entities.iter().cloned());

    // Compare cell counts
    assert_eq!(grid_seq.cells, grid_par.cells);

    // Compare proxy sets (order may differ, so sort both)
    let mut seq_proxies = grid_seq.proxies.clone();
    let mut par_proxies = grid_par.proxies.clone();
    seq_proxies.sort_by_key(|p| p.entity);
    par_proxies.sort_by_key(|p| p.entity);
    assert_eq!(seq_proxies, par_proxies);
}
```

### 2. Performance Benchmark

```rust
#[bench]
fn bench_parallel_rebuild_150k(b: &mut Bencher) {
    let entities: Vec<_> = generate_150k_entities();
    let mut grid = SpatialGrid::new(50.0);

    b.iter(|| {
        grid.rebuild_parallel(entities.iter().cloned());
    });
}
```

### 3. Perf Analysis Commands

```bash
# Baseline (sequential)
perf stat -e instructions,cycles,L1-dcache-load-misses,LLC-load-misses,cache-misses \
    timeout 10s ./target/release/sim_app

# After parallelization
perf stat -e instructions,cycles,L1-dcache-load-misses,LLC-load-misses,cache-misses \
    timeout 10s ./target/release/sim_app

# Expected results:
# - IPC: 1.2 → 2.5 (better instruction throughput due to parallel execution)
# - Cache misses: Similar (parallel histograms increase cache pressure slightly)
# - Cycles: 60% reduction (wall-clock time decreases)
```

### 4. Dev UI Metric Integration

**Rust Backend:** Already emits `spatial_grid_rebuild_us` to telemetry.

**Dev UI Display:** Add "Parallel Speedup" metric:
```typescript
// apps/dev-ui/src/components/StateDisplay.tsx
const parallelSpeedup = baselineTime / currentTime; // e.g., 5.5ms / 1.2ms = 4.6x
```

## Implementation Phases

### Phase 1: Parallel Histogram (Low Risk)
- **Target:** 2.5ms → 0.3ms (8x speedup on Phase 1 alone)
- **Risk:** Low (read-only entity data, thread-local histograms)
- **Validation:** Compare cell counts with sequential version

### Phase 2: Atomic Scatter (Medium Risk)
- **Target:** 2.0ms → 0.4ms (5x speedup on Phase 3 alone)
- **Risk:** Medium (atomic contention if entities cluster)
- **Validation:** Compare proxy arrays (order-independent)
- **Fallback:** If contention is severe (>50% time in atomics), revert to sequential scatter

### Phase 3: Fine-Tuning
- **Chunk size:** Benchmark 2048, 4096, 8192 (trade-off: overhead vs cache efficiency)
- **Atomic ordering:** Test `Relaxed` vs `AcqRel` (Relaxed should suffice)
- **Memory layout:** Align `cells` array to cache line boundaries (64 bytes) to reduce false sharing

## Risks & Mitigations

### 1. Atomic Contention in Dense Clusters
**Risk:** 10K entities in single cell → 10K atomic increments on same counter.

**Mitigation:** Monitor with `perf` for cache line bouncing:
```bash
perf stat -e mem_load_retired.fb_hit,mem_inst_retired.lock_loads \
    timeout 10s ./target/release/sim_app
```

If `lock_loads` > 10% of instructions → contention is severe.

**Fallback:** Revert Phase 3 to sequential scatter (still get 8x speedup from Phase 1).

### 2. Memory Bandwidth Saturation
**Risk:** 16 threads all hammering L3 cache → bandwidth bottleneck.

**Mitigation:** Monitor `LLC-load-misses`. If misses spike >5% → reduce thread count:
```rust
rayon::ThreadPoolBuilder::new()
    .num_threads(8) // Half the cores
    .build_global()
    .unwrap();
```

### 3. Non-Deterministic Proxy Order
**Risk:** Parallel scatter may write proxies in different order each run.

**Impact:** None for correctness (queries iterate all proxies in cell regardless of order).

**Validation:** Sort proxies by entity ID in tests before comparing.

## Summary

**Recommended Strategy:**
1. Parallelize Phase 1 (histogram) with thread-local reduce → **8-10x speedup**
2. Parallelize Phase 3 (scatter) with atomic counters → **4-5x speedup**
3. Keep Phase 2 (prefix sum) sequential → **negligible time**

**Expected Outcome:**
- **Current:** 5.5ms (150K entities)
- **Parallel:** 1.2ms (150K entities)
- **Speedup:** **4.6x** overall
- **Dev UI Impact:** `spatial_grid_rebuild_us` drops from 5500us → 1200us

**Risk Level:** Low-Medium
- Phase 1: Low risk (embarrassingly parallel)
- Phase 3: Medium risk (atomic contention if clustering is severe)

**Validation:**
- Correctness: Compare cell counts + sorted proxy arrays
- Performance: `perf stat` for IPC, cache misses, cycles
- Dev UI: Monitor `spatial_grid_rebuild_us` telemetry

**Next Steps:**
1. Implement Phase 1 (parallel histogram) first
2. Validate with tests + perf analysis
3. If successful, implement Phase 3 (atomic scatter)
4. Benchmark at 150K, 200K, 300K entity counts
5. Document final speedup in sprint summary
