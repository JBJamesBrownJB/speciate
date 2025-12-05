// Parallel Spatial Grid Rebuild Implementation
// Reference implementation showing exact Rayon patterns used in movement systems

use rayon::prelude::*;
use std::sync::atomic::{AtomicU32, Ordering};

impl SpatialGrid {
    /// Parallel rebuild using thread-local histograms + atomic scatter.
    /// Expected speedup: 4.6x (5.5ms → 1.2ms for 150K entities)
    pub fn rebuild_parallel(&mut self, entities: impl Iterator<Item = (Entity, f32, f32, f32)>) {
        // ========== Phase 0: Collect ==========
        self.entity_scratch.clear();
        self.entity_scratch.extend(entities);

        if self.entity_scratch.is_empty() {
            self.proxies.clear();
            for cell in &mut self.cells {
                *cell = (0, 0);
            }
            return;
        }

        // ========== Find Bounds (Sequential - Fast Enough) ==========
        let mut min_cx = i32::MAX;
        let mut max_cx = i32::MIN;
        let mut min_cy = i32::MAX;
        let mut max_cy = i32::MIN;

        for (_, x, y, _) in &self.entity_scratch {
            let (cx, cy) = self.world_to_cell(*x, *y);
            min_cx = min_cx.min(cx);
            max_cx = max_cx.max(cx);
            min_cy = min_cy.min(cy);
            max_cy = max_cy.max(cy);
        }

        self.min_cell_x = min_cx - 1;
        self.min_cell_y = min_cy - 1;
        self.width = (max_cx - min_cx + 3) as usize;
        self.height = (max_cy - min_cy + 3) as usize;

        let total_cells = self.width * self.height;
        self.cells.resize(total_cells, (0, 0));
        self.proxies.resize(self.entity_scratch.len(), PerceptionProxy::default());

        // ========== PARALLEL PHASE 1: Thread-Local Histograms ==========
        // Chunk size: Balance overhead vs cache efficiency
        // 4096 entities = ~64KB of data (16 bytes per entity tuple)
        const CHUNK_SIZE: usize = 4096;

        // Map phase: Each thread counts entities in its chunk
        let local_histograms: Vec<Vec<u32>> = self.entity_scratch
            .par_chunks(CHUNK_SIZE)
            .map(|chunk| {
                // Thread-local histogram (no contention)
                let mut local_counts = vec![0u32; total_cells];

                for (_, x, y, _) in chunk {
                    let idx = self.cell_index_unchecked(*x, *y);
                    local_counts[idx] += 1;
                }

                local_counts
            })
            .collect();

        // Reduce phase: Merge local histograms (sequential but fast)
        // For 2000 cells x 16 threads = 32K operations = ~200us
        for cell in &mut self.cells {
            cell.1 = 0;
        }
        for local_hist in &local_histograms {
            for (i, &count) in local_hist.iter().enumerate() {
                self.cells[i].1 += count;
            }
        }

        // ========== SEQUENTIAL PHASE 2: Prefix Sum ==========
        // Cannot parallelize (sequential dependency chain)
        // Cost: ~0.3us for 2000 cells (negligible)
        let mut offset = 0u32;
        for cell in &mut self.cells {
            cell.0 = offset;
            offset += cell.1;
            cell.1 = 0; // Reset count for scatter phase
        }

        // ========== PARALLEL PHASE 3: Atomic Scatter ==========
        // Use atomic counters to avoid race conditions during parallel writes

        // Convert cells to atomic counters
        let atomic_cells: Vec<(u32, AtomicU32)> = self.cells
            .iter()
            .map(|(offset, _)| (*offset, AtomicU32::new(0)))
            .collect();

        // Get raw pointer for unsafe parallel writes
        // SAFETY: Each write position is unique (guaranteed by atomic counter)
        let proxies_ptr = self.proxies.as_mut_ptr();

        // Parallel scatter (each thread processes subset of entities)
        self.entity_scratch.par_iter().for_each(|&(entity, x, y, radius)| {
            let idx = self.cell_index_unchecked(x, y);
            let (start, counter) = &atomic_cells[idx];

            // Atomic increment: Each thread gets unique count value
            let local_count = counter.fetch_add(1, Ordering::Relaxed);
            let write_pos = (*start + local_count) as usize;

            // SAFETY:
            // 1. write_pos is within bounds (guaranteed by prefix sum)
            // 2. write_pos is unique (guaranteed by atomic counter)
            // 3. No two threads write to same position
            unsafe {
                *proxies_ptr.add(write_pos) = PerceptionProxy { x, y, radius, entity };
            }
        });

        // Convert atomic cells back to regular cells (for query compatibility)
        for (i, (offset, counter)) in atomic_cells.iter().enumerate() {
            self.cells[i] = (*offset, counter.load(Ordering::Relaxed));
        }
    }
}

// ========== TESTS ==========

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parallel_rebuild_correctness() {
        // Generate test entities
        let entities: Vec<_> = (0..100000)
            .map(|i| {
                let x = (i % 1000) as f32 * 10.0;
                let y = (i / 1000) as f32 * 10.0;
                (Entity::from_raw(i as u32), x, y, 5.0)
            })
            .collect();

        // Sequential rebuild
        let mut grid_seq = SpatialGrid::with_default_cell_size();
        grid_seq.rebuild(entities.iter().copied());

        // Parallel rebuild
        let mut grid_par = SpatialGrid::with_default_cell_size();
        grid_par.rebuild_parallel(entities.iter().copied());

        // Verify cell metadata matches
        assert_eq!(grid_seq.min_cell_x, grid_par.min_cell_x);
        assert_eq!(grid_seq.min_cell_y, grid_par.min_cell_y);
        assert_eq!(grid_seq.width, grid_par.width);
        assert_eq!(grid_seq.height, grid_par.height);
        assert_eq!(grid_seq.cells, grid_par.cells);

        // Verify proxy count matches
        assert_eq!(grid_seq.proxies.len(), grid_par.proxies.len());

        // Verify all proxies exist (order may differ)
        let mut seq_proxies = grid_seq.proxies.clone();
        let mut par_proxies = grid_par.proxies.clone();

        seq_proxies.sort_by_key(|p| p.entity.to_bits());
        par_proxies.sort_by_key(|p| p.entity.to_bits());

        assert_eq!(seq_proxies, par_proxies);
    }

    #[test]
    fn test_parallel_rebuild_clustered_entities() {
        // Stress test: All entities in same cell (worst case for atomic contention)
        let entities: Vec<_> = (0..50000)
            .map(|i| (Entity::from_raw(i), 100.0, 100.0, 5.0))
            .collect();

        let mut grid = SpatialGrid::with_default_cell_size();
        grid.rebuild_parallel(entities.iter().copied());

        // Should still work correctly despite contention
        assert_eq!(grid.proxies.len(), 50000);

        // All entities should be in same cell
        let (cx, cy) = grid.world_to_cell(100.0, 100.0);
        let idx = ((cy - grid.min_cell_y) as usize) * grid.width + ((cx - grid.min_cell_x) as usize);
        let (start, count) = grid.cells[idx];
        assert_eq!(count, 50000);
    }
}

// ========== BENCHMARK VALIDATION ==========

#[cfg(test)]
mod bench_setup {
    use super::*;

    /// Generate realistic entity distribution for benchmarking
    pub fn generate_150k_entities() -> Vec<(Entity, f32, f32, f32)> {
        use rand::{Rng, SeedableRng};
        use rand_pcg::Pcg64;

        let mut rng = Pcg64::seed_from_u64(42); // Deterministic for reproducibility

        (0..150000)
            .map(|i| {
                // Mixed density: 70% clustered, 30% sparse
                let (x, y) = if rng.gen::<f32>() < 0.7 {
                    // Clustered (typical creature distribution)
                    let cluster_x = rng.gen_range(0.0..2000.0);
                    let cluster_y = rng.gen_range(0.0..2000.0);
                    let offset_x = rng.gen_range(-100.0..100.0);
                    let offset_y = rng.gen_range(-100.0..100.0);
                    (cluster_x + offset_x, cluster_y + offset_y)
                } else {
                    // Sparse (wanderers at world edges)
                    (rng.gen_range(0.0..5000.0), rng.gen_range(0.0..5000.0))
                };

                let radius = rng.gen_range(5.0..30.0);
                (Entity::from_raw(i), x, y, radius)
            })
            .collect()
    }
}

// ========== PERF ANALYSIS COMMANDS ==========

/*
# Baseline sequential rebuild
perf stat -e instructions,cycles,L1-dcache-load-misses,LLC-load-misses,cache-misses \
    -e mem_inst_retired.lock_loads \
    timeout 10s ./target/release/sim_app

# Expected baseline (150K entities):
# - Time: ~5.5ms
# - IPC: ~1.2 (memory-bound)
# - L1 misses: ~15-20% (random cell access)
# - LLC misses: ~2-3% (working set fits in L3)
# - Lock loads: 0 (no atomics)

# After parallel rebuild
perf stat -e instructions,cycles,L1-dcache-load-misses,LLC-load-misses,cache-misses \
    -e mem_inst_retired.lock_loads \
    timeout 10s ./target/release/sim_app

# Expected parallel (150K entities):
# - Time: ~1.2ms (4.6x speedup)
# - IPC: ~2.5 (better throughput due to parallelism)
# - L1 misses: ~18-25% (slightly higher due to parallel access)
# - LLC misses: ~3-4% (more cache pressure from threads)
# - Lock loads: ~5-10% (atomic fetch_add in scatter phase)

# If lock_loads > 15%: Atomic contention is severe (clustered entities)
# Mitigation: Reduce thread count or revert Phase 3 to sequential

# Monitor cache line bouncing (false sharing)
perf stat -e mem_load_retired.fb_hit \
    timeout 10s ./target/release/sim_app

# If fb_hit (forward buffer hits) > 20%: False sharing detected
# Mitigation: Align atomic_cells to cache line boundaries (64 bytes)
*/

// ========== DEV UI INTEGRATION ==========

/*
Rust backend already emits spatial_grid_rebuild_us to telemetry.
No changes needed - metric will automatically reflect parallel speedup.

Expected Dev UI display:
- Before: spatial_grid_rebuild_us: 5500 (5.5ms)
- After:  spatial_grid_rebuild_us: 1200 (1.2ms)

Add visual indicator in Dev UI:
```typescript
// apps/dev-ui/src/components/PerformanceMetrics.tsx

const BASELINE_REBUILD_US = 5500; // 150K entities sequential
const parallelSpeedup = BASELINE_REBUILD_US / telemetry.spatialGridRebuildUs;

<div>
  Spatial Grid Rebuild: {telemetry.spatialGridRebuildUs}μs
  {parallelSpeedup > 2 && <span>(🚀 {parallelSpeedup.toFixed(1)}x faster)</span>}
</div>
```
*/

// ========== IMPLEMENTATION PHASES ==========

/*
Phase 1: Parallel Histogram Only (Low Risk)
- Implement map-reduce histogram counting
- Expected gain: 2.5ms → 0.3ms (2ms saved)
- Validate: Compare cell counts with sequential
- Perf check: IPC should increase, LLC misses should stay similar

Phase 2: Atomic Scatter (Medium Risk)
- Add atomic counter scatter
- Expected gain: 2.0ms → 0.4ms (1.6ms saved)
- Validate: Compare sorted proxy arrays with sequential
- Perf check: Monitor lock_loads < 15%

Phase 3: Fine-Tuning
- Benchmark chunk sizes: 2048, 4096, 8192
- Test atomic ordering: Relaxed vs AcqRel (Relaxed should suffice)
- Profile with different entity densities (clustered vs uniform)

Rollback Criteria:
- If lock_loads > 20%: Revert Phase 2, keep Phase 1
- If LLC misses spike > 10%: Reduce thread count
- If any correctness test fails: Revert entire feature
*/

// ========== ALTERNATIVE: SIMD EXPLORATION (NOT RECOMMENDED) ==========

/*
SIMD is NOT viable for spatial grid rebuild because:

1. Phase 1 (Histogram):
   - Random memory access to cells[idx] (no vectorization benefit)
   - Cell index calculation requires integer division (slow, not SIMD-friendly)
   - Write pattern is scatter (not contiguous)

2. Phase 3 (Scatter):
   - Write positions computed dynamically from atomic counters
   - No predictable access pattern for SIMD prefetching

3. Vectorization Potential: < 10% (not worth complexity)

VERDICT: Focus on Rayon parallelization (4.6x speedup is achievable).
*/
