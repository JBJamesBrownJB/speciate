# Spatial Grid Parallelization - Quick Reference

## TL;DR

**Current:** 5.5ms rebuild time (150K entities)
**Target:** 1.6ms rebuild time (150K entities)
**Speedup:** 3.3x overall

## Strategy

```
┌─────────────────────────────────────────────────────┐
│ PHASE 1: HISTOGRAM (2.5ms → 0.3ms) = 8x speedup    │
│ - Thread-local counts (no contention)              │
│ - Map-reduce pattern                                │
│ - Risk: LOW                                         │
└─────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────┐
│ PHASE 2: PREFIX SUM (0.3μs) = KEEP SEQUENTIAL      │
│ - Sequential dependency chain                       │
│ - Too small to parallelize                          │
└─────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────┐
│ PHASE 3: SCATTER (2.0ms → 0.4ms) = 5x speedup      │
│ - Atomic fetch_add counters                         │
│ - Cache line bouncing possible                      │
│ - Risk: MEDIUM                                      │
└─────────────────────────────────────────────────────┘
```

## Code Template

```rust
use rayon::prelude::*;
use std::sync::atomic::{AtomicU32, Ordering};

// Phase 1: Thread-local histogram
const CHUNK_SIZE: usize = 4096;
let local_histograms: Vec<Vec<u32>> = entity_scratch
    .par_chunks(CHUNK_SIZE)
    .map(|chunk| {
        let mut local_counts = vec![0u32; total_cells];
        for (_, x, y, _) in chunk {
            local_counts[cell_index(x, y)] += 1;
        }
        local_counts
    })
    .collect();

// Merge (sequential)
for local_hist in &local_histograms {
    for (i, &count) in local_hist.iter().enumerate() {
        cells[i].1 += count;
    }
}

// Phase 2: Prefix sum (sequential)
let mut offset = 0;
for cell in &mut cells {
    cell.0 = offset;
    offset += cell.1;
    cell.1 = 0;
}

// Phase 3: Atomic scatter
let atomic_cells: Vec<(u32, AtomicU32)> = cells
    .iter()
    .map(|(offset, _)| (*offset, AtomicU32::new(0)))
    .collect();

entity_scratch.par_iter().for_each(|&(entity, x, y, radius)| {
    let idx = cell_index(x, y);
    let (start, counter) = &atomic_cells[idx];
    let local_count = counter.fetch_add(1, Ordering::Relaxed);
    let write_pos = (*start + local_count) as usize;
    unsafe {
        *proxies_ptr.add(write_pos) = PerceptionProxy { x, y, radius, entity };
    }
});
```

## Validation Checklist

- [ ] Correctness test: Compare cell counts with sequential
- [ ] Correctness test: Compare sorted proxy arrays with sequential
- [ ] Stress test: 50K entities in single cell (worst case contention)
- [ ] Perf stat: IPC > 1.8 (up from 1.2)
- [ ] Perf stat: Lock loads < 15%
- [ ] Benchmark: Speedup > 3.0x at 150K entities

## Perf Commands

```bash
# Before
perf stat -e instructions,cycles,L1-dcache-load-misses,LLC-load-misses \
    timeout 10s ./target/release/sim_app

# After
perf stat -e instructions,cycles,L1-dcache-load-misses,LLC-load-misses,mem_inst_retired.lock_loads \
    timeout 10s ./target/release/sim_app

# Expected changes:
# - IPC: 1.2 → 2.4 (2x better)
# - Cycles: 1B → 625M (37% reduction)
# - Lock loads: 0 → 5-10% (atomic operations)
```

## Risk Mitigation

**If lock_loads > 15%:**
→ Severe atomic contention (clustered entities)
→ Revert Phase 3, keep Phase 1 (still get 2.5x speedup)

**If LLC miss > 10%:**
→ Memory bandwidth saturation
→ Reduce thread count from 16 to 8

**If any test fails:**
→ Full rollback to sequential

## Expected Results

| Metric | Before | After | Change |
|--------|--------|-------|--------|
| Rebuild Time | 5.5ms | 1.6ms | -3.9ms |
| Frame Budget % | 22% | 6.4% | -15.6% |
| IPC | 1.2 | 2.4 | +100% |
| Speedup | 1.0x | 3.3x | +230% |

## Files

**Strategy:** `/home/dev/dev/speciate/SPRINTS/spatial-grid/parallel-rebuild-strategy.md`
**Reference:** `/home/dev/dev/speciate/SPRINTS/spatial-grid/parallel-implementation-reference.rs`
**Analysis:** `/home/dev/dev/speciate/SPRINTS/spatial-grid/parallel-speedup-analysis.md`
**Implementation:** `/home/dev/dev/speciate/apps/simulation/src/simulation/spatial/grid.rs`

## SIMD Verdict

**NOT RECOMMENDED**

- Histogram: Random memory access (no vectorization benefit)
- Scatter: Dynamic write positions (not SIMD-friendly)
- Expected gain: < 10% (not worth complexity)

**Focus on Rayon parallelization instead.**
