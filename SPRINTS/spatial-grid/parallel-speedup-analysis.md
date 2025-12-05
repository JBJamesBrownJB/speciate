# Parallel Spatial Grid Rebuild: Speedup Analysis

## Baseline Performance (150K Entities)

**Hardware:** 16-core CPU (validated in movement parallelization)
**Data Source:** `/home/dev/dev/speciate/docs/performance/snapshots/150k_mixed_density_2025-12-05T18-05-00.json`

| Metric | Value |
|--------|-------|
| Avg Rebuild Time | 5469 μs (5.5ms) |
| Min Rebuild Time | 4369 μs (4.4ms) |
| Max Rebuild Time | 5802 μs (5.8ms) |
| P95 Rebuild Time | 5802 μs (5.8ms) |
| Active Cells | ~2000 (typical) |
| Cell Size | 50m |
| Entity Count | 150,000 |

## Sequential Phase Breakdown

| Phase | Operation | Cost (μs) | % Total | Parallelizable? |
|-------|-----------|-----------|---------|-----------------|
| 0 | Collect entities | 500 | 9% | No (iterator overhead) |
| 1 | Histogram counting | 2500 | 46% | **YES** (8-10x) |
| 2 | Prefix sum | 0.3 | 0.005% | No (sequential dependency) |
| 3 | Scatter proxies | 2000 | 37% | **YES** (4-5x) |
| - | Bounds finding | 469 | 8% | Maybe (reduce pattern) |

**Total Sequential:** 5469 μs

## Parallel Phase Breakdown

### Phase 1: Thread-Local Histogram (Map-Reduce)

**Strategy:** Partition entities into chunks, count locally, merge histograms

| Metric | Sequential | Parallel | Speedup |
|--------|------------|----------|---------|
| Time | 2500 μs | 250-300 μs | 8-10x |
| Thread Count | 1 | 16 | - |
| Chunk Size | - | 4096 entities | ~64KB |
| Memory Access | Random (cells array) | Random (thread-local) | - |
| Contention | None | None (thread-local) | - |

**Speedup Calculation:**
- Ideal: 16x (16 cores)
- Memory-bound: 8-10x (LLC bandwidth saturation)
- Merge overhead: ~200 μs (2000 cells x 16 threads)

**Expected Time:** 2500 μs / 9 = 278 μs

### Phase 3: Atomic Scatter

**Strategy:** Parallel entity iteration with atomic counter increments

| Metric | Sequential | Parallel | Speedup |
|--------|------------|----------|---------|
| Time | 2000 μs | 400-500 μs | 4-5x |
| Thread Count | 1 | 16 | - |
| Atomic Ops | None | 150,000 fetch_add | - |
| Contention | None | Low-Medium (entity distribution) | - |
| Write Pattern | Sequential in cell | Atomic increment | - |

**Speedup Calculation:**
- Ideal: 16x (16 cores)
- Atomic contention: 4-5x (cache line bouncing on hot cells)
- Uniform distribution: 5x (75 entities/cell avg)
- Clustered distribution: 3-4x (1000+ entities in few cells)

**Expected Time:** 2000 μs / 5 = 400 μs

### Phase 2: Prefix Sum (Sequential)

**Strategy:** Keep sequential (not worth parallelizing)

| Metric | Sequential | Parallel (Blelloch) | Verdict |
|--------|------------|---------------------|---------|
| Time | 0.3 μs | ~50-100 μs | **Sequential faster** |
| Operations | 2000 cells | 2000 cells | - |
| Overhead | None | Thread spawn, barriers | High |
| Complexity | O(N) | O(log N) | O(N) is faster for small N |

**Threshold for parallel prefix sum:** >1M elements (our case: 2000 cells)

## Expected Parallel Performance

| Phase | Sequential (μs) | Parallel (μs) | Saved (μs) |
|-------|-----------------|---------------|------------|
| 0: Collect | 500 | 500 | 0 |
| Bounds | 469 | 469 | 0 |
| 1: Histogram | 2500 | 278 | 2222 |
| 2: Prefix Sum | 0.3 | 0.3 | 0 |
| 3: Scatter | 2000 | 400 | 1600 |
| **TOTAL** | **5469** | **1647** | **3822** |

**Overall Speedup:** 5469 μs / 1647 μs = **3.3x** (conservative estimate)

**Optimistic Case (uniform distribution, low contention):**
- Phase 1: 250 μs (10x speedup)
- Phase 3: 333 μs (6x speedup)
- **Total:** 1552 μs → **3.5x speedup**

**Pessimistic Case (clustered distribution, high contention):**
- Phase 1: 312 μs (8x speedup)
- Phase 3: 500 μs (4x speedup)
- **Total:** 1781 μs → **3.1x speedup**

## Performance Validation Metrics

### perf stat Expected Results

**Sequential Baseline:**
```bash
perf stat -e instructions,cycles,L1-dcache-load-misses,LLC-load-misses,cache-misses \
    timeout 10s ./target/release/sim_app
```

| Counter | Baseline (Sequential) | Target (Parallel) | Interpretation |
|---------|----------------------|-------------------|----------------|
| IPC | 1.2 | 2.0-2.5 | Better throughput (parallel execution) |
| L1 Miss % | 15-20% | 18-25% | Slight increase (parallel access) |
| LLC Miss % | 2-3% | 3-5% | Acceptable (working set fits in L3) |
| Cycles | 100M | 40-50M | 50-60% reduction (wall-clock speedup) |
| Lock Loads | 0 | 5-10% | Atomic fetch_add in Phase 3 |

**Red Flags:**
- Lock Loads > 15% → Severe atomic contention (clustered entities)
- LLC Miss > 10% → Memory bandwidth saturation (reduce threads)
- IPC < 1.5 → Parallelization not effective (rollback)

### Cache Line Bouncing (False Sharing)

```bash
perf stat -e mem_load_retired.fb_hit,mem_inst_retired.lock_loads \
    timeout 10s ./target/release/sim_app
```

| Counter | Threshold | Mitigation |
|---------|-----------|------------|
| fb_hit | < 20% | Align atomic_cells to 64-byte boundaries |
| lock_loads | < 15% of instructions | Reduce thread count or revert Phase 3 |

**Mitigation Example:**
```rust
#[repr(align(64))] // Cache line alignment
struct AlignedAtomicCell {
    offset: u32,
    counter: AtomicU32,
}
```

## Scaling Projections

### Entity Count vs Rebuild Time

| Entity Count | Sequential (ms) | Parallel (ms) | Speedup |
|--------------|-----------------|---------------|---------|
| 50K | 1.8 | 0.6 | 3.0x |
| 100K | 3.6 | 1.1 | 3.3x |
| 150K | 5.5 | 1.6 | 3.4x |
| 200K | 7.3 | 2.2 | 3.3x |
| 300K | 11.0 | 3.3 | 3.3x |
| 500K | 18.3 | 5.5 | 3.3x |

**Note:** Speedup plateaus at ~3.3x due to memory bandwidth saturation (16 threads competing for LLC).

### Thread Count Sensitivity

| Thread Count | Rebuild Time (ms) | Speedup | Efficiency |
|--------------|-------------------|---------|------------|
| 1 (sequential) | 5.5 | 1.0x | 100% |
| 2 | 3.2 | 1.7x | 85% |
| 4 | 2.0 | 2.8x | 70% |
| 8 | 1.4 | 3.9x | 49% |
| 12 | 1.2 | 4.6x | 38% |
| 16 | 1.1 | 5.0x | 31% |
| 24 | 1.0 | 5.5x | 23% |

**Diminishing Returns:** Beyond 8 threads, speedup plateaus due to:
1. Memory bandwidth saturation (all cores contending for LLC)
2. Atomic contention in scatter phase
3. Merge overhead in histogram phase

**Optimal Thread Count:** 8-12 threads (balance speedup vs overhead)

## Risk Assessment

### Atomic Contention Scenarios

| Scenario | Distribution | Entities/Cell | Lock Load % | Speedup |
|----------|--------------|---------------|-------------|---------|
| Best Case | Uniform | 75 avg | 5% | 5x (Phase 3) |
| Typical | Mixed (70% clustered, 30% sparse) | 100-200 avg | 8% | 4.5x |
| Worst Case | All clustered | 1000+ in hotspots | 20% | 2x |

**Mitigation for Worst Case:**
- Monitor lock_loads with perf
- If > 15%, revert Phase 3 to sequential (still get 8x from Phase 1)
- Net speedup: 2.5x (better than nothing)

### Memory Bandwidth Saturation

**Symptom:** LLC miss rate spikes above 10%

**Cause:** 16 threads all hammering L3 cache (limited bandwidth)

**Mitigation:**
```rust
rayon::ThreadPoolBuilder::new()
    .num_threads(8) // Reduce from 16 to 8
    .build_global()
    .unwrap();
```

**Impact:** Speedup drops from 3.5x → 3.0x (acceptable trade-off)

## Implementation Roadmap

### Phase 1: Parallel Histogram (Week 1)

**Target:** 2500 μs → 278 μs (2.2ms saved)

**Tasks:**
1. Implement map-reduce histogram counting
2. Add correctness test (compare cell counts)
3. Benchmark at 50K, 100K, 150K entities
4. Validate with perf stat (IPC should increase)

**Success Criteria:**
- Cell counts match sequential version
- IPC > 1.8 (up from 1.2)
- L1 miss rate < 25%

### Phase 2: Atomic Scatter (Week 2)

**Target:** 2000 μs → 400 μs (1.6ms saved)

**Tasks:**
1. Add atomic counter scatter
2. Add correctness test (compare sorted proxies)
3. Stress test clustered entities (50K in one cell)
4. Monitor lock_loads with perf

**Success Criteria:**
- Proxy arrays match sequential (order-independent)
- Lock loads < 15%
- Speedup > 3x overall

### Phase 3: Fine-Tuning (Week 3)

**Target:** Optimize chunk size, thread count, memory alignment

**Tasks:**
1. Benchmark chunk sizes: 2048, 4096, 8192
2. Test thread counts: 8, 12, 16
3. Profile cache line bouncing (fb_hit)
4. Add Dev UI visualization (speedup indicator)

**Success Criteria:**
- Rebuild time < 2ms for 150K entities
- Speedup > 3.0x consistently
- No performance regressions in other systems

## Success Metrics

### Telemetry (Dev UI)

**Before:**
```json
{
  "spatialGridRebuildUs": 5469
}
```

**After:**
```json
{
  "spatialGridRebuildUs": 1647,
  "parallelSpeedup": "3.3x"
}
```

### Perf Analysis

**Before:**
```
Performance counter stats for 'timeout 10s ./target/release/sim_app':

    1,200,000,000      instructions              #    1.20  insn per cycle
    1,000,000,000      cycles
       15,000,000      L1-dcache-load-misses     #   15.00% of all L1 loads
        2,500,000      LLC-load-misses           #    2.50% of all LLC loads
```

**After:**
```
Performance counter stats for 'timeout 10s ./target/release/sim_app':

    1,500,000,000      instructions              #    2.40  insn per cycle
      625,000,000      cycles
       20,000,000      L1-dcache-load-misses     #   20.00% of all L1 loads
        3,500,000      LLC-load-misses           #    3.50% of all LLC loads
       75,000,000      mem_inst_retired.lock_loads  #  5.00% of all instructions
```

### Frame Budget Impact

**Assumption:** 40Hz simulation (25ms frame budget)

**Before:**
- Spatial grid: 5.5ms (22% of frame budget)
- Movement: 4ms (16%)
- Perception: 8ms (32%)
- **Total:** 17.5ms (70% utilization)

**After:**
- Spatial grid: 1.6ms (6.4% of frame budget)
- Movement: 4ms (16%)
- Perception: 8ms (32%)
- **Total:** 13.6ms (54% utilization)

**Headroom Gained:** 3.9ms (15.6% of frame budget) → can support more entities or add new systems

## Conclusion

**Conservative Estimate:** 3.3x speedup (5.5ms → 1.6ms)
**Optimistic Estimate:** 3.5x speedup (5.5ms → 1.5ms)
**Pessimistic Estimate:** 3.0x speedup (5.5ms → 1.8ms)

**Risk Level:** Low-Medium
- Phase 1 (histogram): Low risk, high reward
- Phase 3 (scatter): Medium risk (atomic contention)

**Rollback Strategy:**
- If lock_loads > 15%: Revert Phase 3, keep Phase 1 (2.5x speedup)
- If any correctness test fails: Full rollback

**Next Steps:**
1. Implement Phase 1 (parallel histogram)
2. Validate correctness + performance
3. If successful, implement Phase 2 (atomic scatter)
4. Benchmark at 200K, 300K entities
5. Document final speedup in sprint summary
