# Flat 2D Array Spatial Grid - Cache-Optimized Implementation

**Status:** Recommended replacement for FxHashMap grid
**Performance Target:** <1% LLC miss rate, IPC > 1.5
**Complexity:** O(N×M) with excellent cache locality

---

## Architecture Overview

### Core Concept

Replace `FxHashMap<(i32, i32), Vec<EntityData>>` with a single flat `Vec<Vec<EntityData>>` indexed by arithmetic (no hashing).

**Key Insight:** CPU cache prefetching loves predictable memory access. Array indexing is predictable. HashMap probing is not.

---

## Implementation

File: `/home/dev/dev/speciate/apps/simulation/src/simulation/spatial/flat_grid.rs`

```rust
use bevy::prelude::*;
use crate::simulation::creatures::components::Position;

/// Flat 2D spatial grid with direct array indexing
///
/// Cache-friendly alternative to HashMap-based grid.
/// Trades memory (pre-allocates cells) for speed (O(1) cell lookup).
pub struct FlatSpatialGrid {
    /// Flat array of cells (row-major order)
    cells: Vec<Vec<EntityData>>,

    /// Grid dimensions
    width: usize,
    height: usize,

    /// Cell size in world units
    cell_size: f32,
    inv_cell_size: f32,

    /// World bounds (for clamping)
    world_width: f32,
    world_height: f32,
}

pub type EntityData = (Entity, f32, f32, f32);

impl FlatSpatialGrid {
    /// Create grid covering world bounds
    ///
    /// Memory usage: width × height × sizeof(Vec<EntityData>)
    /// For 200×200 grid: ~320KB overhead (negligible)
    pub fn new(world_width: f32, world_height: f32, cell_size: f32) -> Self {
        let width = (world_width / cell_size).ceil() as usize;
        let height = (world_height / cell_size).ceil() as usize;

        // Pre-allocate all cells (empty Vecs have ~24 bytes overhead each)
        let cells = vec![Vec::new(); width * height];

        Self {
            cells,
            width,
            height,
            cell_size,
            inv_cell_size: 1.0 / cell_size,
            world_width,
            world_height,
        }
    }

    /// Clear all cells (reuse allocations)
    #[inline]
    pub fn clear(&mut self) {
        for cell in &mut self.cells {
            cell.clear();  // Keeps Vec capacity
        }
    }

    /// Insert entity into grid
    #[inline]
    pub fn insert(&mut self, entity: Entity, x: f32, y: f32, radius: f32) {
        if let Some(idx) = self.world_to_index(x, y) {
            // Direct array access (no hashing, no collision handling)
            self.cells[idx].push((entity, x, y, radius));
        }
    }

    /// Convert world coordinates to flat array index
    ///
    /// This is the KEY optimization: arithmetic instead of hashing
    #[inline]
    fn world_to_index(&self, x: f32, y: f32) -> Option<usize> {
        // Clamp to world bounds
        if x < 0.0 || x >= self.world_width || y < 0.0 || y >= self.world_height {
            return None;
        }

        let cx = (x * self.inv_cell_size).floor() as usize;
        let cy = (y * self.inv_cell_size).floor() as usize;

        // Row-major order: index = row × width + col
        Some(cy * self.width + cx)
    }

    /// Query entities within radius (iterator-based, zero-copy)
    pub fn query_radius(&self, x: f32, y: f32, radius: f32) -> impl Iterator<Item = &EntityData> + '_ {
        // Calculate cell bounds
        let min_x = (x - radius).max(0.0);
        let max_x = (x + radius).min(self.world_width);
        let min_y = (y - radius).max(0.0);
        let max_y = (y + radius).min(self.world_height);

        let min_cx = (min_x * self.inv_cell_size).floor() as usize;
        let max_cx = (max_x * self.inv_cell_size).floor() as usize;
        let min_cy = (min_y * self.inv_cell_size).floor() as usize;
        let max_cy = (max_y * self.inv_cell_size).floor() as usize;

        // Iterator over rectangular region
        (min_cy..=max_cy)
            .flat_map(move |cy| {
                (min_cx..=max_cx).map(move |cx| cy * self.width + cx)
            })
            .filter(move |&idx| idx < self.cells.len())
            .flat_map(move |idx| &self.cells[idx])
    }

    /// Get cell bounds for visualization
    pub fn get_query_cells(&self, x: f32, y: f32, radius: f32) -> Vec<(i32, i32)> {
        let min_x = (x - radius).max(0.0);
        let max_x = (x + radius).min(self.world_width);
        let min_y = (y - radius).max(0.0);
        let max_y = (y + radius).min(self.world_height);

        let min_cx = (min_x * self.inv_cell_size).floor() as i32;
        let max_cx = (max_x * self.inv_cell_size).floor() as i32;
        let min_cy = (min_y * self.inv_cell_size).floor() as i32;
        let max_cy = (max_y * self.inv_cell_size).floor() as i32;

        let capacity = ((max_cy - min_cy + 1) * (max_cx - min_cx + 1)) as usize;
        let mut cells = Vec::with_capacity(capacity);

        for cy in min_cy..=max_cy {
            for cx in min_cx..=max_cx {
                cells.push((cx, cy));
            }
        }

        cells
    }

    /// Get grid statistics
    pub fn stats(&self) -> GridStats {
        let occupied = self.cells.iter().filter(|v| !v.is_empty()).count();
        let total_entities: usize = self.cells.iter().map(|v| v.len()).sum();

        GridStats {
            total_cells: self.cells.len(),
            occupied_cells: occupied,
            total_entities,
            avg_entities_per_occupied_cell: if occupied > 0 {
                total_entities as f32 / occupied as f32
            } else {
                0.0
            },
        }
    }
}

#[derive(Debug, Clone)]
pub struct GridStats {
    pub total_cells: usize,
    pub occupied_cells: usize,
    pub total_entities: usize,
    pub avg_entities_per_occupied_cell: f32,
}

impl Default for FlatSpatialGrid {
    fn default() -> Self {
        // Default: 1000×1000 world, 50m cells
        Self::new(1000.0, 1000.0, 50.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_flat_grid_indexing() {
        let grid = FlatSpatialGrid::new(1000.0, 1000.0, 50.0);

        // Grid should be 20×20 = 400 cells
        assert_eq!(grid.cells.len(), 400);

        // Test coordinate to index conversion
        let idx = grid.world_to_index(25.0, 25.0).unwrap();
        assert_eq!(idx, 0); // Top-left cell

        let idx = grid.world_to_index(975.0, 975.0).unwrap();
        assert_eq!(idx, 399); // Bottom-right cell

        // Test bounds clamping
        assert!(grid.world_to_index(-10.0, 500.0).is_none());
        assert!(grid.world_to_index(1100.0, 500.0).is_none());
    }

    #[test]
    fn test_query_radius() {
        let mut grid = FlatSpatialGrid::new(1000.0, 1000.0, 50.0);

        // Insert test entities
        let e1 = Entity::from_raw(1);
        let e2 = Entity::from_raw(2);
        let e3 = Entity::from_raw(3);

        grid.insert(e1, 100.0, 100.0, 5.0);
        grid.insert(e2, 150.0, 150.0, 5.0);
        grid.insert(e3, 500.0, 500.0, 5.0);

        // Query near e1 (should find e1 and e2)
        let results: Vec<_> = grid.query_radius(100.0, 100.0, 100.0).collect();
        assert!(results.len() >= 2);

        // Query near e3 (should only find e3)
        let results: Vec<_> = grid.query_radius(500.0, 500.0, 50.0).collect();
        assert_eq!(results.len(), 1);
    }

    #[test]
    fn test_clear_reuses_capacity() {
        let mut grid = FlatSpatialGrid::new(1000.0, 1000.0, 50.0);

        // Insert many entities
        for i in 0..100 {
            grid.insert(Entity::from_raw(i), i as f32 * 10.0, i as f32 * 10.0, 5.0);
        }

        // Get capacity before clear
        let capacities_before: Vec<_> = grid.cells.iter().map(|v| v.capacity()).collect();

        // Clear
        grid.clear();

        // Capacities should be preserved
        let capacities_after: Vec<_> = grid.cells.iter().map(|v| v.capacity()).collect();
        assert_eq!(capacities_before, capacities_after);
    }
}
```

---

## Performance Analysis

### Cache Behavior

#### Cell Lookup Cost

```
HashMap (old):
1. Compute FxHash(cell_key)        →  ~10 cycles
2. Probe bucket array              →  ~60-100 cycles (L3 miss)
3. Compare keys (collision?)       →  ~5 cycles
4. Dereference Vec pointer         →  ~10-20 cycles
Total: ~85-135 cycles per cell

Flat Array (new):
1. Compute index (cy × width + cx) →  ~3 cycles (arithmetic)
2. Array access                    →  ~4 cycles (L1 hit - predictable!)
Total: ~7 cycles per cell
```

**Speedup:** 12-20× faster per cell lookup

#### Multi-Cell Query Cost

Typical perception query: 9 cells (3×3 region)

```
HashMap: 9 × 120 cycles = 1080 cycles
Flat Array: 9 × 7 cycles = 63 cycles
Speedup: 17×
```

At 10K creatures:
```
HashMap: 10,800,000 cycles = ~5.4ms @ 2GHz
Flat Array: 630,000 cycles = ~0.3ms @ 2GHz
Savings: 5.1ms per tick
```

### Memory Overhead

```
HashMap:
- Bucket array: ~8 bytes × num_buckets (load factor ~0.7)
- Entry overhead: ~24 bytes per entry (key + value + metadata)
- Vec allocations: ~24 bytes per Vec + data

Flat Array:
- Pre-allocated cells: 24 bytes × (width × height)
- For 20×20 grid: 400 × 24 = 9.6 KB (negligible)
```

**Verdict:** Flat array uses slightly more memory (predictable, static) but gains massive cache locality.

---

## Migration Guide

### 1. Add New Module

```rust
// In src/simulation/spatial/mod.rs
pub mod flat_grid;
pub use flat_grid::FlatSpatialGrid;
```

### 2. Replace Resource

```rust
// In src/simulation/core/simulation.rs

// Old:
// .insert_resource(SpatialGrid::with_default_cell_size())

// New:
.insert_resource(FlatSpatialGrid::new(
    WORLD_WIDTH,   // 1000.0 or from config
    WORLD_HEIGHT,  // 1000.0 or from config
    CELL_SIZE,     // 50.0
))
```

### 3. Update System Signatures

```rust
// Perception system
pub fn update_perception_system(
    grid: Res<FlatSpatialGrid>,  // Changed from SpatialGrid
    // ... rest unchanged
) {
    // API is identical (query_radius returns same iterator type)
}
```

### 4. Update Rebuild System

```rust
// Spatial grid rebuild
pub fn rebuild_spatial_grid_system(
    mut grid: ResMut<FlatSpatialGrid>,  // Changed type
    query: Query<(Entity, &Position, &BodySize)>,
    #[cfg(feature = "dev-tools")] timings: Res<SystemTimings>,
) {
    #[cfg(feature = "dev-tools")]
    crate::time_system!(timings, "spatial_grid_rebuild");

    grid.clear();  // Same API

    for (entity, pos, size) in query.iter() {
        grid.insert(entity, pos.x, pos.y, size.radius);  // Same API
    }
}
```

---

## Validation

### Before Switching

Run baseline profiling:
```bash
cd /home/dev/dev/speciate/apps/simulation
./profile_spatial_grid.sh
```

Record:
- IPC (should be < 1.0 with HashMap)
- LLC miss rate (should be > 1% with HashMap)
- Perception system time (from Dev UI)

### After Switching

Re-run profiling:
```bash
./profile_spatial_grid.sh
```

**Expected Improvements:**
- IPC: 0.8 → 1.5+ (CPU less stalled)
- LLC miss rate: 2% → 0.5% (75% reduction)
- Perception time: 5ms → 0.5ms (10× faster)

### Validation Tests

```rust
#[test]
fn test_flat_vs_hashmap_equivalence() {
    // Insert same entities into both grids
    let mut flat = FlatSpatialGrid::new(1000.0, 1000.0, 50.0);
    let mut hash = SpatialGrid::with_default_cell_size();

    for i in 0..1000 {
        let entity = Entity::from_raw(i);
        let x = (i as f32 * 10.0) % 1000.0;
        let y = (i as f32 * 10.0) % 1000.0;

        flat.insert(entity, x, y, 5.0);
        hash.insert(entity, x, y, 5.0);
    }

    // Query same location in both
    let flat_results: HashSet<Entity> = flat.query_radius(500.0, 500.0, 100.0)
        .map(|&(e, _, _, _)| e)
        .collect();

    let hash_results: HashSet<Entity> = hash.query_radius(500.0, 500.0, 100.0)
        .map(|&(e, _, _, _)| e)
        .collect();

    // Should return identical entities
    assert_eq!(flat_results, hash_results);
}
```

---

## Alternative: Hybrid Approach

If world bounds are unknown or extremely large (e.g., procedural worlds), use a hybrid:

```rust
pub struct HybridGrid {
    active_chunks: FxHashMap<(i32, i32), FlatSpatialGrid>,  // Sparse chunk allocation
}
```

Each chunk is a dense 10×10 sub-grid. Only allocate chunks that contain entities.

**Best of both worlds:**
- Sparse allocation (HashMap at chunk level)
- Dense access (flat array within chunks)

---

## Summary

**Problem:** FxHashMap spatial grid causes cache thrashing (random access)

**Solution:** Flat 2D array with arithmetic indexing (predictable access)

**Trade-off:** Fixed memory overhead (~10KB for 20×20 grid) for 10-20× speedup

**Next Steps:**
1. Run profiling script to confirm HashMap is the bottleneck
2. Implement FlatSpatialGrid (copy code above)
3. Swap resource in simulation setup
4. Re-profile and verify improvements

---

**Document Owner:** cache-carl (Performance Analyst)
**Last Updated:** 2025-12-04
