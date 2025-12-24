use bevy_ecs::prelude::*;
use rayon::prelude::*;
use std::sync::atomic::{AtomicU32, Ordering};

use super::constants::{
    CELL_SIZE, COS_15_DEG, DISTANCE_EPSILON, NON_ADJACENT_OFFSET, SIN_15_DEG, SMALL_SORT_THRESHOLD,
    WIDE_FOV_THRESHOLD,
};
use crate::simulation::core::MAX_WORLD_SIZE;

/// Wrapper to allow raw pointer in parallel iteration.
/// SAFETY: Each thread writes to a unique position via atomic increment.
#[derive(Clone, Copy)]
struct SyncPtr<T>(*mut T);
unsafe impl<T> Send for SyncPtr<T> {}
unsafe impl<T> Sync for SyncPtr<T> {}

impl<T> SyncPtr<T> {
    /// Write value at offset.
    /// SAFETY: Caller must ensure unique write access to this position.
    #[inline(always)]
    unsafe fn write_at(self, offset: usize, value: T) {
        std::ptr::write(self.0.add(offset), value);
    }
}

#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct PerceptionProxy {
    pub x: f32,
    pub y: f32,
    pub vx: f32,
    pub vy: f32,
    pub radius: f32,
    pub entity: Entity,
}

impl Default for PerceptionProxy {
    fn default() -> Self {
        Self {
            x: 0.0,
            y: 0.0,
            vx: 0.0,
            vy: 0.0,
            radius: 0.0,
            entity: Entity::PLACEHOLDER,
        }
    }
}

/// Cell query bounds for iteration.
/// Returned by `query_bounds()` to avoid recalculating in each method.
#[derive(Clone, Copy)]
struct CellQueryBounds {
    min_qx: i32,
    max_qx: i32,
    min_qy: i32,
    max_qy: i32,
    center_cx: i32,
    center_cy: i32,
}

/// DOD Spatial Grid with contiguous buffer storage.
///
/// Uses counting sort to bin entities into a single Vec for cache-friendly access.
/// Zero pointer chasing during queries - all data is contiguous in memory.
///
/// Fixed-bounds mode: Pre-allocates grid for world bounds, eliminating per-tick allocations.
#[derive(Resource)]
pub struct SpatialGrid {
    // Single contiguous buffer of all proxies
    proxies: Vec<PerceptionProxy>,

    // Cell -> slice mapping: (start_index, count)
    // Index = (cy - min_cell_y) * width + (cx - min_cell_x)
    cells: Vec<(u32, u32)>,

    // Reusable scratch buffer for rebuild (avoids allocation each tick)
    // Format: (entity, x, y, vx, vy, radius)
    entity_scratch: Vec<(Entity, f32, f32, f32, f32, f32)>,

    // Pre-allocated atomic counters for scatter phase (reused each tick)
    atomic_counters: Vec<AtomicU32>,

    // Track non-empty cells from previous tick for efficient clearing
    prev_non_empty: Vec<usize>,

    // Fixed grid bounds (derived from world bounds + cell size)
    min_cell_x: i32,
    min_cell_y: i32,
    width: usize,
    height: usize,

    // World bounds for fixed-grid mode
    world_min_x: f32,
    world_max_x: f32,
    world_min_y: f32,
    world_max_y: f32,
    fixed_bounds: bool,

    cell_size: f32,
    inv_cell_size: f32,
}

impl SpatialGrid {
    pub fn new(cell_size: f32) -> Self {
        Self {
            proxies: Vec::new(),
            cells: Vec::new(),
            entity_scratch: Vec::new(),
            atomic_counters: Vec::new(),
            prev_non_empty: Vec::new(),
            min_cell_x: 0,
            min_cell_y: 0,
            width: 0,
            height: 0,
            world_min_x: 0.0,
            world_max_x: 0.0,
            world_min_y: 0.0,
            world_max_y: 0.0,
            fixed_bounds: false,
            cell_size,
            inv_cell_size: 1.0 / cell_size,
        }
    }

    /// Create grid with fixed world bounds (pre-allocated, no per-tick allocations).
    pub fn with_fixed_bounds(
        cell_size: f32,
        min_x: f32,
        max_x: f32,
        min_y: f32,
        max_y: f32,
    ) -> Self {
        let mut grid = Self::new(cell_size);
        grid.set_world_bounds(min_x, max_x, min_y, max_y);
        grid
    }

    /// Create grid with default cell size and MAX_WORLD_SIZE bounds.
    pub fn with_default_bounds() -> Self {
        Self::with_fixed_bounds(
            CELL_SIZE,
            -MAX_WORLD_SIZE,
            MAX_WORLD_SIZE,
            -MAX_WORLD_SIZE,
            MAX_WORLD_SIZE,
        )
    }

    pub fn with_default_cell_size() -> Self {
        Self::new(CELL_SIZE)
    }

    /// Set fixed world bounds. Pre-allocates grid cells and atomic counters.
    /// Call this once at startup or when world/cell size changes.
    pub fn set_world_bounds(&mut self, min_x: f32, max_x: f32, min_y: f32, max_y: f32) {
        self.world_min_x = min_x;
        self.world_max_x = max_x;
        self.world_min_y = min_y;
        self.world_max_y = max_y;
        self.fixed_bounds = true;
        self.recalculate_grid_dimensions();
    }

    /// Change cell size and recalculate grid dimensions.
    pub fn set_cell_size(&mut self, cell_size: f32) {
        self.cell_size = cell_size;
        self.inv_cell_size = 1.0 / cell_size;
        if self.fixed_bounds {
            self.recalculate_grid_dimensions();
        }
    }

    /// Recalculate grid dimensions from world bounds and cell size.
    fn recalculate_grid_dimensions(&mut self) {
        let min_cx = (self.world_min_x * self.inv_cell_size).floor() as i32 - 1;
        let max_cx = (self.world_max_x * self.inv_cell_size).ceil() as i32 + 1;
        let min_cy = (self.world_min_y * self.inv_cell_size).floor() as i32 - 1;
        let max_cy = (self.world_max_y * self.inv_cell_size).ceil() as i32 + 1;

        self.min_cell_x = min_cx;
        self.min_cell_y = min_cy;
        self.width = (max_cx - min_cx + 1) as usize;
        self.height = (max_cy - min_cy + 1) as usize;

        let total_cells = self.width * self.height;

        // Pre-allocate cells and atomic counters
        self.cells = vec![(0, 0); total_cells];
        self.atomic_counters = (0..total_cells).map(|_| AtomicU32::new(0)).collect();

        log::info!(
            "SpatialGrid: {}x{} = {} cells for world ({:.0},{:.0}) to ({:.0},{:.0}) @ {:.1}m",
            self.width,
            self.height,
            total_cells,
            self.world_min_x,
            self.world_min_y,
            self.world_max_x,
            self.world_max_y,
            self.cell_size
        );
    }

    pub fn cell_size(&self) -> f32 {
        self.cell_size
    }

    /// Get grid width in cells.
    #[inline]
    pub fn width(&self) -> usize {
        self.width
    }

    /// Get grid height in cells.
    #[inline]
    pub fn height(&self) -> usize {
        self.height
    }

    /// Get indices of non-empty cells from the previous rebuild.
    /// Use for efficient iteration (O(non-empty) not O(total cells)).
    #[inline]
    pub fn non_empty_cells(&self) -> &[usize] {
        &self.prev_non_empty
    }

    #[inline(always)]
    pub fn world_to_cell(&self, x: f32, y: f32) -> (i32, i32) {
        (
            (x * self.inv_cell_size).floor() as i32,
            (y * self.inv_cell_size).floor() as i32,
        )
    }

    #[cfg(test)]
    pub fn cell_to_world_min(&self, cell_x: i32, cell_y: i32) -> (f32, f32) {
        (
            cell_x as f32 * self.cell_size,
            cell_y as f32 * self.cell_size,
        )
    }

    /// Test-only convenience: dynamically computes bounds then delegates to rebuild_parallel().
    #[cfg(test)]
    pub fn rebuild(&mut self, entities: impl Iterator<Item = (Entity, f32, f32, f32, f32, f32)>) {
        let entities: Vec<_> = entities.collect();

        if entities.is_empty() {
            self.proxies.clear();
            for cell in &mut self.cells {
                *cell = (0, 0);
            }
            return;
        }

        // Find bounds from entities
        let (min_x, max_x, min_y, max_y) = entities.iter().fold(
            (f32::MAX, f32::MIN, f32::MAX, f32::MIN),
            |(min_x, max_x, min_y, max_y), (_, x, y, _, _, _)| {
                (min_x.min(*x), max_x.max(*x), min_y.min(*y), max_y.max(*y))
            },
        );

        // Set bounds with padding
        self.set_world_bounds(min_x - self.cell_size, max_x + self.cell_size, min_y - self.cell_size, max_y + self.cell_size);

        // Delegate to parallel rebuild
        self.rebuild_parallel(entities.into_iter());
    }

    /// Parallel rebuild using Rayon.
    ///
    /// Requires fixed bounds (call `set_world_bounds()` first).
    /// Zero allocations per tick, uses pre-allocated buffers.
    ///
    /// # Panics
    /// Panics if `set_world_bounds()` has not been called.
    pub fn rebuild_parallel(
        &mut self,
        entities: impl Iterator<Item = (Entity, f32, f32, f32, f32, f32)>,
    ) {
        assert!(
            self.fixed_bounds,
            "rebuild_parallel() requires fixed bounds - call set_world_bounds() first"
        );

        // Phase 0: Collect into scratch buffer
        self.entity_scratch.clear();
        self.entity_scratch.extend(entities);

        let n = self.entity_scratch.len();
        if n == 0 {
            self.proxies.clear();
            for cell in &mut self.cells {
                *cell = (0, 0);
            }
            return;
        }

        self.rebuild_parallel_fixed_bounds(n);
    }

    /// Fast path: Fixed bounds, no allocations per tick.
    /// Uses two-pass algorithm: parallel count then parallel scatter.
    /// Optimized: Tracks populated cells during counting (avoids 1M atomic scan).
    fn rebuild_parallel_fixed_bounds(&mut self, n: usize) {
        let inv_cell_size = self.inv_cell_size;
        let min_cell_x = self.min_cell_x;
        let min_cell_y = self.min_cell_y;
        let width = self.width;
        let height = self.height;

        // Resize proxies buffer (no realloc if capacity sufficient)
        self.proxies.resize(n, PerceptionProxy::default());

        // Clear cells that were non-empty last tick (parallel, O(N) instead of O(cells))
        let cells_ptr = SyncPtr(self.cells.as_mut_ptr());
        let atomic_counters = &self.atomic_counters;
        self.prev_non_empty.par_iter().for_each(|&idx| {
            // SAFETY: Each index is unique (deduped), so no data races
            unsafe {
                cells_ptr.write_at(idx, (0, 0));
            }
            atomic_counters[idx].store(0, Ordering::Relaxed);
        });

        // Phase 1: PARALLEL count entities per cell using atomics
        // Track newly populated cells during counting (avoids scanning all 1M atomics)
        let atomic_counters = &self.atomic_counters;
        let newly_populated: Vec<usize> = self
            .entity_scratch
            .par_chunks(4096)
            .flat_map(|chunk| {
                // Thread-local buffer for newly populated cell indices
                let mut local_new: Vec<usize> = Vec::with_capacity(chunk.len() / 4);
                for &(_, x, y, _, _, _) in chunk {
                    let cx = (x * inv_cell_size).floor() as i32;
                    let cy = (y * inv_cell_size).floor() as i32;
                    let lx = ((cx - min_cell_x) as usize).min(width - 1);
                    let ly = ((cy - min_cell_y) as usize).min(height - 1);
                    let idx = ly * width + lx;
                    // fetch_add returns OLD value - if 0, this cell just became populated
                    let old_count = atomic_counters[idx].fetch_add(1, Ordering::Relaxed);
                    if old_count == 0 {
                        local_new.push(idx);
                    }
                }
                local_new
            })
            .collect();

        // Merge newly populated indices (already collected, no 1M scan needed!)
        let mut non_empty = std::mem::take(&mut self.prev_non_empty);
        non_empty.clear();
        non_empty.extend(newly_populated);

        // Prefix sum ONLY on non-empty cells (200K instead of 1M iterations!)
        let mut offset = 0u32;
        for &idx in &non_empty {
            let count = self.atomic_counters[idx].load(Ordering::Relaxed);
            self.cells[idx] = (offset, count);
            offset += count;
        }

        // Reset atomics for scatter phase (parallel, only non-empty cells)
        non_empty.par_iter().for_each(|&idx| {
            atomic_counters[idx].store(0, Ordering::Relaxed);
        });

        // Save non-empty list for next tick's clearing
        self.prev_non_empty = non_empty;

        // Phase 3: Parallel atomic scatter
        let proxies_ptr = SyncPtr(self.proxies.as_mut_ptr());
        let cells_ref = &self.cells;
        let atomic_counters = &self.atomic_counters;
        let height = self.height;

        // SAFETY: Each entity writes to a unique position via atomic increment
        self.entity_scratch
            .par_iter()
            .for_each(|&(entity, x, y, vx, vy, radius)| {
                let cx = (x * inv_cell_size).floor() as i32;
                let cy = (y * inv_cell_size).floor() as i32;
                let lx = (cx - min_cell_x) as usize;
                let ly = (cy - min_cell_y) as usize;

                // Bounds check (entities outside world are clamped to edge cells)
                let lx = lx.min(width - 1);
                let ly = ly.min(height - 1);
                let idx = ly * width + lx;

                let start = cells_ref[idx].0;
                let local_offset = atomic_counters[idx].fetch_add(1, Ordering::Relaxed);
                let write_pos = (start + local_offset) as usize;

                // SAFETY: Unique write position guaranteed by atomic increment
                unsafe {
                    proxies_ptr.write_at(
                        write_pos,
                        PerceptionProxy {
                            x,
                            y,
                            vx,
                            vy,
                            radius,
                            entity,
                        },
                    );
                }
            });
    }

    /// Query entities within radius with cell-level FOV culling.
    /// Skips entire cells that are behind the creature before examining any proxies.
    /// This can reduce candidates by 25-50% depending on FOV.
    #[inline(always)]
    pub fn query_radius_fov(
        &self,
        x: f32,
        y: f32,
        radius: f32,
        facing_x: f32,
        facing_y: f32,
    ) -> impl Iterator<Item = &PerceptionProxy> {
        let (center_cx, center_cy) = self.world_to_cell(x, y);
        let cells_radius = (radius * self.inv_cell_size).ceil() as i32;
        let cell_size = self.cell_size;
        let half_cell = cell_size * 0.5;

        let min_qx = (center_cx - cells_radius).max(self.min_cell_x);
        let max_qx = (center_cx + cells_radius).min(self.min_cell_x + self.width as i32 - 1);
        let min_qy = (center_cy - cells_radius).max(self.min_cell_y);
        let max_qy = (center_cy + cells_radius).min(self.min_cell_y + self.height as i32 - 1);

        (min_qy..=max_qy).flat_map(move |cy| {
            (min_qx..=max_qx)
                .filter_map(move |cx| {
                    // Cell center in world coordinates
                    let cell_center_x = (cx as f32 * cell_size) + half_cell;
                    let cell_center_y = (cy as f32 * cell_size) + half_cell;

                    // Check if cell is behind creature (conservative: use cell_size threshold)
                    let cell_dir_dot =
                        (cell_center_x - x) * facing_x + (cell_center_y - y) * facing_y;
                    if cell_dir_dot < -cell_size {
                        return None; // Entire cell is behind, skip
                    }

                    let idx = ((cy - self.min_cell_y) as usize) * self.width
                        + ((cx - self.min_cell_x) as usize);
                    // SAFETY: idx is within bounds because cx/cy are clamped to grid bounds
                    let (start, count) = unsafe { *self.cells.get_unchecked(idx) };
                    if count == 0 {
                        None
                    } else {
                        // SAFETY: (start, count) pair is valid because rebuild() maintains invariant
                        Some(unsafe {
                            self.proxies
                                .get_unchecked(start as usize..(start + count) as usize)
                        })
                    }
                })
                .flatten()
        })
    }

    // ========================================================================
    // Helper methods for cell queries (reduce duplication)
    // ========================================================================

    /// Calculate cell iteration bounds for a radius query.
    #[inline(always)]
    fn query_bounds(&self, x: f32, y: f32, radius: f32) -> CellQueryBounds {
        let (center_cx, center_cy) = self.world_to_cell(x, y);
        let cells_radius = (radius * self.inv_cell_size).ceil() as i32;

        CellQueryBounds {
            min_qx: (center_cx - cells_radius).max(self.min_cell_x),
            max_qx: (center_cx + cells_radius).min(self.min_cell_x + self.width as i32 - 1),
            min_qy: (center_cy - cells_radius).max(self.min_cell_y),
            max_qy: (center_cy + cells_radius).min(self.min_cell_y + self.height as i32 - 1),
            center_cx,
            center_cy,
        }
    }

    /// Calculate cell index from cell coordinates.
    /// SAFETY: Caller must ensure cx/cy are within grid bounds.
    #[inline(always)]
    fn cell_index_unchecked(&self, cx: i32, cy: i32) -> usize {
        ((cy - self.min_cell_y) as usize) * self.width + ((cx - self.min_cell_x) as usize)
    }

    /// Calculate cell center in world coordinates.
    #[inline(always)]
    fn cell_center(&self, cx: i32, cy: i32) -> (f32, f32) {
        let half_cell = self.cell_size * 0.5;
        (
            (cx as f32 * self.cell_size) + half_cell,
            (cy as f32 * self.cell_size) + half_cell,
        )
    }

    /// Get cell occupancy count (0 if empty).
    /// SAFETY: Caller must ensure cell_idx is within bounds.
    #[inline(always)]
    unsafe fn cell_count_unchecked(&self, cell_idx: usize) -> u32 {
        (*self.cells.get_unchecked(cell_idx)).1
    }

    // ========================================================================
    // Cell collection methods
    // ========================================================================

    /// Collect cell indices sorted with adjacent cells FIRST, then by distance.
    /// Output: Vec of (sort_key, cell_index) pairs.
    /// Adjacent cells (3x3 around creature) are always examined first, never FOV-culled.
    /// Non-adjacent cells behind creature are FOV-culled.
    /// Use `get_cell_proxies` to retrieve the actual proxies for each cell.
    #[inline(always)]
    pub fn collect_cells_sorted(
        &self,
        x: f32,
        y: f32,
        radius: f32,
        facing_x: f32,
        facing_y: f32,
        out: &mut Vec<(f32, usize)>,
    ) {
        out.clear();

        let bounds = self.query_bounds(x, y, radius);

        for cy in bounds.min_qy..=bounds.max_qy {
            for cx in bounds.min_qx..=bounds.max_qx {
                let (cell_center_x, cell_center_y) = self.cell_center(cx, cy);

                // Adjacent cells (3x3 grid) - skip FOV check, always include, examine FIRST
                let is_adjacent =
                    (cx - bounds.center_cx).abs() <= 1 && (cy - bounds.center_cy).abs() <= 1;

                if !is_adjacent {
                    // Only apply FOV culling for distant cells
                    let cell_dir_dot =
                        (cell_center_x - x) * facing_x + (cell_center_y - y) * facing_y;
                    if cell_dir_dot < -self.cell_size {
                        continue;
                    }
                }

                let idx = self.cell_index_unchecked(cx, cy);
                // SAFETY: idx is within bounds because cx/cy are clamped to grid bounds
                let count = unsafe { self.cell_count_unchecked(idx) };
                if count > 0 {
                    let dx = cell_center_x - x;
                    let dy = cell_center_y - y;
                    let dist_sq = dx * dx + dy * dy;

                    // Adjacent cells get raw distance, non-adjacent get offset so they sort after
                    let sort_key = if is_adjacent {
                        dist_sq
                    } else {
                        dist_sq + NON_ADJACENT_OFFSET
                    };
                    out.push((sort_key, idx));
                }
            }
        }

        // Sort by sort_key: adjacent cells first (by distance), then non-adjacent (by distance)
        // Skip sort for small sets (adjacent 3x3 grid) - order doesn't matter
        if out.len() > SMALL_SORT_THRESHOLD {
            out.sort_unstable_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal));
        }
    }

    /// Collect cell indices with range AND FOV culling.
    /// Cells whose nearest edge is beyond perception_range are skipped.
    /// Non-adjacent cells outside the creature's FOV cone are also skipped.
    #[inline(always)]
    pub fn collect_cells_sorted_fov(
        &self,
        x: f32,
        y: f32,
        radius: f32,
        perception_range: f32,
        facing_x: f32,
        facing_y: f32,
        cos_half_fov: f32,
        out: &mut Vec<(f32, usize)>,
    ) {
        out.clear();

        let bounds = self.query_bounds(x, y, radius);
        let half_cell = self.cell_size * 0.5;

        // Skip FOV culling for very wide FOVs (>= 300°, cos_half_fov <= WIDE_FOV_THRESHOLD)
        let adjusted_cos = Self::adjusted_fov_cos(cos_half_fov);

        for cy in bounds.min_qy..=bounds.max_qy {
            for cx in bounds.min_qx..=bounds.max_qx {
                let (cell_center_x, cell_center_y) = self.cell_center(cx, cy);

                // Own cell is ALWAYS included (can't skip where we are)
                let is_own_cell = cx == bounds.center_cx && cy == bounds.center_cy;

                if !is_own_cell {
                    // FOV culling: skip cells outside FOV cone (applies to ALL cells except own)
                    let dx = cell_center_x - x;
                    let dy = cell_center_y - y;
                    let dist = (dx * dx + dy * dy).sqrt();

                    if dist > DISTANCE_EPSILON {
                        let cos_angle = (dx * facing_x + dy * facing_y) / dist;
                        if cos_angle < adjusted_cos {
                            continue;
                        }
                    }
                }

                // Range check: skip cells whose nearest edge is beyond perception range
                let dx_to_center = (cell_center_x - x).abs();
                let dy_to_center = (cell_center_y - y).abs();
                let dx_to_edge = (dx_to_center - half_cell).max(0.0);
                let dy_to_edge = (dy_to_center - half_cell).max(0.0);
                let dist_to_nearest_edge = dx_to_edge.hypot(dy_to_edge);

                if dist_to_nearest_edge > perception_range {
                    continue;
                }

                let idx = self.cell_index_unchecked(cx, cy);
                // SAFETY: idx is within bounds because cx/cy are clamped to grid bounds
                let count = unsafe { self.cell_count_unchecked(idx) };

                if count > 0 {
                    let dx = cell_center_x - x;
                    let dy = cell_center_y - y;
                    let dist_sq = dx * dx + dy * dy;
                    out.push((dist_sq, idx));
                }
            }
        }
    }

    /// Calculate adjusted FOV cosine with safety margin.
    /// Returns -2.0 for wide FOVs (>= 300°) to effectively disable culling.
    #[inline(always)]
    fn adjusted_fov_cos(cos_half_fov: f32) -> f32 {
        if cos_half_fov <= WIDE_FOV_THRESHOLD {
            -2.0 // Effectively disables culling (no cos_angle can be < -2)
        } else {
            // 15° safety margin: expand FOV cone slightly to avoid edge cases
            // cos(fov + 15°) = cos(fov)*cos(15°) - sin(fov)*sin(15°)
            let sin_half_fov = (1.0 - cos_half_fov * cos_half_fov).sqrt();
            cos_half_fov * COS_15_DEG - sin_half_fov * SIN_15_DEG
        }
    }

    /// Get proxies for a cell by index. Use after collect_cells_sorted.
    ///
    /// # Safety
    /// Caller must ensure cell_idx came from collect_cells_sorted (valid index).
    #[inline(always)]
    pub fn get_cell_proxies(&self, cell_idx: usize) -> &[PerceptionProxy] {
        // SAFETY: cell_idx is from collect_cells_sorted which only returns valid indices
        let (start, count) = unsafe { *self.cells.get_unchecked(cell_idx) };
        // SAFETY: (start, count) pair is valid because rebuild() maintains this invariant
        unsafe {
            self.proxies
                .get_unchecked(start as usize..(start + count) as usize)
        }
    }

    /// Get cell coordinates (cx, cy) from a cell index.
    #[inline(always)]
    pub fn get_cell_coords_by_index(&self, cell_idx: usize) -> (i32, i32) {
        let local_y = cell_idx / self.width;
        let local_x = cell_idx % self.width;
        (
            local_x as i32 + self.min_cell_x,
            local_y as i32 + self.min_cell_y,
        )
    }

    /// Get cell index from cell coordinates (cx, cy).
    /// Returns None if coordinates are outside grid bounds.
    #[inline(always)]
    pub fn get_cell_index_by_coords(&self, cx: i32, cy: i32) -> Option<usize> {
        let local_x = cx - self.min_cell_x;
        let local_y = cy - self.min_cell_y;

        if local_x < 0
            || local_y < 0
            || local_x >= self.width as i32
            || local_y >= self.height as i32
        {
            return None;
        }

        Some((local_y as usize) * self.width + (local_x as usize))
    }

    /// Query entities within radius. Returns iterator over contiguous slices.
    #[inline(always)]
    pub fn query_radius(
        &self,
        x: f32,
        y: f32,
        radius: f32,
    ) -> impl Iterator<Item = &PerceptionProxy> {
        let (center_cx, center_cy) = self.world_to_cell(x, y);
        let cells_radius = (radius * self.inv_cell_size).ceil() as i32;

        // Pre-compute valid cell range (clamped to grid bounds)
        let min_qx = (center_cx - cells_radius).max(self.min_cell_x);
        let max_qx = (center_cx + cells_radius).min(self.min_cell_x + self.width as i32 - 1);
        let min_qy = (center_cy - cells_radius).max(self.min_cell_y);
        let max_qy = (center_cy + cells_radius).min(self.min_cell_y + self.height as i32 - 1);

        // Row-major iteration for cache locality
        (min_qy..=max_qy).flat_map(move |cy| {
            (min_qx..=max_qx)
                .filter_map(move |cx| {
                    let idx = ((cy - self.min_cell_y) as usize) * self.width
                        + ((cx - self.min_cell_x) as usize);
                    // SAFETY: idx is within bounds because cx/cy are clamped to grid bounds
                    let (start, count) = unsafe { *self.cells.get_unchecked(idx) };
                    if count == 0 {
                        None
                    } else {
                        // SAFETY: (start, count) pair is valid because rebuild() maintains invariant
                        Some(unsafe {
                            self.proxies
                                .get_unchecked(start as usize..(start + count) as usize)
                        })
                    }
                })
                .flatten()
        })
    }

    #[cfg(test)]
    pub fn get_query_cells(&self, x: f32, y: f32, radius: f32) -> Vec<(i32, i32)> {
        let (center_cx, center_cy) = self.world_to_cell(x, y);
        let cells_radius = (radius * self.inv_cell_size).ceil() as i32;

        let capacity = ((2 * cells_radius + 1) * (2 * cells_radius + 1)) as usize;
        let mut cells = Vec::with_capacity(capacity);

        for dy in -cells_radius..=cells_radius {
            for dx in -cells_radius..=cells_radius {
                cells.push((center_cx + dx, center_cy + dy));
            }
        }

        cells
    }

    pub fn entity_count(&self) -> usize {
        self.proxies.len()
    }

    pub fn dimensions(&self) -> (usize, usize) {
        (self.width, self.height)
    }

    pub fn bounds(&self) -> (i32, i32) {
        (self.min_cell_x, self.min_cell_y)
    }

    pub fn clear(&mut self) {
        self.proxies.clear();
        for cell in &mut self.cells {
            *cell = (0, 0);
        }
    }
}

impl Default for SpatialGrid {
    fn default() -> Self {
        Self::with_default_cell_size()
    }
}

/// Double-buffered spatial grid for latency hiding.
///
/// Perception reads from front buffer while rebuild writes to back buffer.
/// Swap at end of tick to publish new data. 1-tick staleness is acceptable
/// for perception (biologically plausible as "reaction time").
#[derive(Resource)]
pub struct DoubleBufferedSpatialGrid {
    grids: [SpatialGrid; 2],
    /// 0 or 1 - which grid is currently the "front" (read) buffer
    front_index: usize,
}

impl DoubleBufferedSpatialGrid {
    pub fn new(cell_size: f32) -> Self {
        Self {
            grids: [SpatialGrid::new(cell_size), SpatialGrid::new(cell_size)],
            front_index: 0,
        }
    }

    /// Create with fixed world bounds (pre-allocated, no per-tick allocations).
    pub fn with_fixed_bounds(
        cell_size: f32,
        min_x: f32,
        max_x: f32,
        min_y: f32,
        max_y: f32,
    ) -> Self {
        Self {
            grids: [
                SpatialGrid::with_fixed_bounds(cell_size, min_x, max_x, min_y, max_y),
                SpatialGrid::with_fixed_bounds(cell_size, min_x, max_x, min_y, max_y),
            ],
            front_index: 0,
        }
    }

    /// Create with default cell size and MAX_WORLD_SIZE bounds.
    pub fn with_default_bounds() -> Self {
        Self::with_fixed_bounds(
            CELL_SIZE,
            -MAX_WORLD_SIZE,
            MAX_WORLD_SIZE,
            -MAX_WORLD_SIZE,
            MAX_WORLD_SIZE,
        )
    }

    pub fn with_default_cell_size() -> Self {
        Self::new(CELL_SIZE)
    }

    /// Set fixed world bounds on both grids. Call once at startup.
    pub fn set_world_bounds(&mut self, min_x: f32, max_x: f32, min_y: f32, max_y: f32) {
        self.grids[0].set_world_bounds(min_x, max_x, min_y, max_y);
        self.grids[1].set_world_bounds(min_x, max_x, min_y, max_y);
    }

    /// Change cell size on both grids.
    pub fn set_cell_size(&mut self, cell_size: f32) {
        self.grids[0].set_cell_size(cell_size);
        self.grids[1].set_cell_size(cell_size);
    }

    /// Get the front buffer for reading (perception queries this)
    #[inline]
    pub fn read_grid(&self) -> &SpatialGrid {
        &self.grids[self.front_index]
    }

    /// Get the back buffer for writing (rebuild writes here)
    #[inline]
    pub fn write_grid(&mut self) -> &mut SpatialGrid {
        &mut self.grids[1 - self.front_index]
    }

    /// Swap front and back buffers (call at end of tick)
    #[inline]
    pub fn swap(&mut self) {
        self.front_index = 1 - self.front_index;
    }

    /// Get cell size (same for both buffers)
    pub fn cell_size(&self) -> f32 {
        self.grids[0].cell_size()
    }
}

impl Default for DoubleBufferedSpatialGrid {
    fn default() -> Self {
        Self::with_default_cell_size()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_world_to_cell_positive_coords() {
        let grid = SpatialGrid::new(50.0);

        assert_eq!(grid.world_to_cell(0.0, 0.0), (0, 0));
        assert_eq!(grid.world_to_cell(25.0, 25.0), (0, 0));
        assert_eq!(grid.world_to_cell(49.9, 49.9), (0, 0));
        assert_eq!(grid.world_to_cell(50.0, 50.0), (1, 1));
        assert_eq!(grid.world_to_cell(100.0, 150.0), (2, 3));
    }

    #[test]
    fn test_world_to_cell_negative_coords() {
        let grid = SpatialGrid::new(50.0);

        assert_eq!(grid.world_to_cell(-1.0, -1.0), (-1, -1));
        assert_eq!(grid.world_to_cell(-50.0, -50.0), (-1, -1));
        assert_eq!(grid.world_to_cell(-50.1, -50.1), (-2, -2));
        assert_eq!(grid.world_to_cell(-100.0, -100.0), (-2, -2));
    }

    #[test]
    fn test_world_to_cell_mixed_coords() {
        let grid = SpatialGrid::new(50.0);

        assert_eq!(grid.world_to_cell(25.0, -25.0), (0, -1));
        assert_eq!(grid.world_to_cell(-25.0, 25.0), (-1, 0));
    }

    #[test]
    fn test_cell_to_world_min() {
        let grid = SpatialGrid::new(50.0);

        assert_eq!(grid.cell_to_world_min(0, 0), (0.0, 0.0));
        assert_eq!(grid.cell_to_world_min(1, 1), (50.0, 50.0));
        assert_eq!(grid.cell_to_world_min(-1, -1), (-50.0, -50.0));
        assert_eq!(grid.cell_to_world_min(2, -3), (100.0, -150.0));
    }

    #[test]
    fn test_rebuild_and_query() {
        let mut grid = SpatialGrid::new(50.0);

        let entity1 = Entity::from_raw(1);
        let entity2 = Entity::from_raw(2);
        let entity3 = Entity::from_raw(3);

        let entities = vec![
            (entity1, 25.0, 25.0, 0.0, 0.0, 1.0),
            (entity2, 75.0, 25.0, 0.0, 0.0, 1.0),
            (entity3, 25.0, 75.0, 0.0, 0.0, 1.0),
        ];

        grid.rebuild(entities.into_iter());

        assert_eq!(grid.entity_count(), 3);

        let nearby: Vec<_> = grid.query_radius(25.0, 25.0, 30.0).collect();
        assert!(nearby.iter().any(|p| p.entity == entity1));
    }

    #[test]
    fn test_get_query_cells_small_radius() {
        let grid = SpatialGrid::new(50.0);

        let cells = grid.get_query_cells(25.0, 25.0, 30.0);

        assert_eq!(cells.len(), 9);
        assert!(cells.contains(&(0, 0)));
        assert!(cells.contains(&(-1, -1)));
        assert!(cells.contains(&(1, 1)));
    }

    #[test]
    fn test_get_query_cells_larger_radius() {
        let grid = SpatialGrid::new(50.0);

        let cells = grid.get_query_cells(25.0, 25.0, 75.0);

        assert_eq!(cells.len(), 25);
    }

    #[test]
    fn test_rebuild_clears_previous() {
        let mut grid = SpatialGrid::new(50.0);

        // First rebuild
        let entities1 = vec![
            (Entity::from_raw(1), 25.0, 25.0, 0.0, 0.0, 1.0),
            (Entity::from_raw(2), 75.0, 25.0, 0.0, 0.0, 1.0),
        ];
        grid.rebuild(entities1.into_iter());
        assert_eq!(grid.entity_count(), 2);

        // Second rebuild with different entities
        let entities2 = vec![(Entity::from_raw(3), 125.0, 125.0, 0.0, 0.0, 1.0)];
        grid.rebuild(entities2.into_iter());
        assert_eq!(grid.entity_count(), 1);
    }

    #[test]
    fn test_large_world_coords() {
        let grid = SpatialGrid::new(50.0);

        assert_eq!(grid.world_to_cell(100_000.0, 100_000.0), (2000, 2000));
        assert_eq!(grid.world_to_cell(-100_000.0, -100_000.0), (-2000, -2000));
    }

    #[test]
    fn test_bounds_tracking() {
        let mut grid = SpatialGrid::new(50.0);

        let entities = vec![
            (Entity::from_raw(1), 0.0, 0.0, 0.0, 0.0, 1.0), // cell (0, 0)
            (Entity::from_raw(2), 100.0, 100.0, 0.0, 0.0, 1.0), // cell (2, 2)
            (Entity::from_raw(3), -50.0, -50.0, 0.0, 0.0, 1.0), // cell (-1, -1)
        ];

        grid.rebuild(entities.into_iter());

        assert!(grid.width >= 5);
        assert!(grid.height >= 5);
    }

    #[test]
    fn test_query_respects_bounds() {
        let mut grid = SpatialGrid::new(50.0);

        let entities = vec![(Entity::from_raw(1), 25.0, 25.0, 0.0, 0.0, 1.0)];

        grid.rebuild(entities.into_iter());

        // Query far outside the grid should return empty
        let far_away: Vec<_> = grid.query_radius(10000.0, 10000.0, 30.0).collect();
        assert!(far_away.is_empty());

        // Query near the entity should find it
        let nearby: Vec<_> = grid.query_radius(25.0, 25.0, 30.0).collect();
        assert_eq!(nearby.len(), 1);
    }

    #[test]
    fn test_empty_rebuild() {
        let mut grid = SpatialGrid::new(50.0);

        // First add some entities
        let entities = vec![(Entity::from_raw(1), 25.0, 25.0, 0.0, 0.0, 1.0)];
        grid.rebuild(entities.into_iter());
        assert_eq!(grid.entity_count(), 1);

        // Then rebuild with empty
        grid.rebuild(std::iter::empty());
        assert_eq!(grid.entity_count(), 0);
    }

    #[test]
    fn test_perception_proxy_size() {
        // Entity requires 8-byte alignment, so struct is 32 bytes
        // (5 f32s = 20 bytes + Entity = 8 bytes + 4 bytes padding = 32)
        // 2 proxies per cache line - key benefit is contiguous buffer
        assert_eq!(std::mem::size_of::<PerceptionProxy>(), 32);
    }

    #[test]
    fn test_contiguous_buffer() {
        let mut grid = SpatialGrid::new(50.0);

        let entities = vec![
            (Entity::from_raw(1), 25.0, 25.0, 0.0, 0.0, 1.0),
            (Entity::from_raw(2), 26.0, 26.0, 0.0, 0.0, 1.0), // Same cell
            (Entity::from_raw(3), 27.0, 27.0, 0.0, 0.0, 1.0), // Same cell
        ];

        grid.rebuild(entities.into_iter());

        // All 3 entities should be in contiguous memory
        assert_eq!(grid.proxies.len(), 3);

        // Query should return all 3
        let nearby: Vec<_> = grid.query_radius(25.0, 25.0, 10.0).collect();
        assert_eq!(nearby.len(), 3);
    }

    #[test]
    fn test_query_radius_fov_skips_cells_behind() {
        let mut grid = SpatialGrid::new(50.0);

        // Entity at origin facing right (+X direction)
        // Place targets in front (right) and behind (left)
        let entities = vec![
            (Entity::from_raw(1), 100.0, 0.0, 0.0, 0.0, 1.0), // In front (right)
            (Entity::from_raw(2), -100.0, 0.0, 0.0, 0.0, 1.0), // Behind (left)
            (Entity::from_raw(3), 0.0, 100.0, 0.0, 0.0, 1.0), // To the side (up)
        ];

        grid.rebuild(entities.into_iter());

        // Query from origin, facing right (+X), large radius
        let facing_x = 1.0;
        let facing_y = 0.0;
        let results: Vec<_> = grid
            .query_radius_fov(0.0, 0.0, 150.0, facing_x, facing_y)
            .collect();

        // Should find entity in front and to the side, but NOT the one behind
        let found_ids: Vec<u32> = results.iter().map(|p| p.entity.index()).collect();

        assert!(found_ids.contains(&1), "Should find entity in front");
        assert!(
            !found_ids.contains(&2),
            "Should NOT find entity behind (cell culled)"
        );
        assert!(found_ids.contains(&3), "Should find entity to the side");
    }

    #[test]
    fn test_query_radius_fov_vs_regular_query() {
        let mut grid = SpatialGrid::new(50.0);

        let entities = vec![
            (Entity::from_raw(1), 75.0, 25.0, 0.0, 0.0, 1.0), // Front-right
            (Entity::from_raw(2), -75.0, 25.0, 0.0, 0.0, 1.0), // Back-left
            (Entity::from_raw(3), 75.0, -25.0, 0.0, 0.0, 1.0), // Front-right
            (Entity::from_raw(4), -75.0, -25.0, 0.0, 0.0, 1.0), // Back-left
        ];

        grid.rebuild(entities.into_iter());

        // Regular query finds all 4
        let all_results: Vec<_> = grid.query_radius(0.0, 0.0, 100.0).collect();
        assert_eq!(all_results.len(), 4);

        // FOV query facing right should find only front entities
        let fov_results: Vec<_> = grid.query_radius_fov(0.0, 0.0, 100.0, 1.0, 0.0).collect();
        assert!(
            fov_results.len() < all_results.len(),
            "FOV query should return fewer results"
        );

        // Entities 1 and 3 are in front, 2 and 4 are behind
        let found_ids: Vec<u32> = fov_results.iter().map(|p| p.entity.index()).collect();
        assert!(found_ids.contains(&1), "Should find front-right entity");
        assert!(found_ids.contains(&3), "Should find front-right entity");
    }

    #[test]
    fn test_collect_cells_sorted_includes_adjacent_cells_behind() {
        // Regression test: Adjacent cells should NOT be culled even when behind creature
        // This tests the fix for cell-center FOV approximation breaking at cell boundaries
        let mut grid = SpatialGrid::new(50.0);

        // Creature at (25, 25) - center of cell (0, 0) facing right (+X)
        // Entity directly behind at (-25, 25) - center of cell (-1, 0) - ADJACENT cell
        // Entity far behind at (-125, 25) - center of cell (-3, 0) - NOT adjacent
        let entities = vec![
            (Entity::from_raw(1), -25.0, 25.0, 0.0, 0.0, 1.0), // Adjacent cell behind (-1, 0)
            (Entity::from_raw(2), -125.0, 25.0, 0.0, 0.0, 1.0), // Far cell behind (-3, 0)
        ];

        grid.rebuild(entities.into_iter());

        let mut cells = Vec::new();
        grid.collect_cells_sorted(25.0, 25.0, 200.0, 1.0, 0.0, &mut cells);

        // Convert cell indices back to coords
        let cell_coords: Vec<(i32, i32)> = cells
            .iter()
            .map(|&(_, idx)| grid.get_cell_coords_by_index(idx))
            .collect();

        // Adjacent cell (-1, 0) should be included despite being behind
        assert!(
            cell_coords.contains(&(-1, 0)),
            "Adjacent cell behind creature should NOT be culled. Found: {:?}",
            cell_coords
        );

        // Far cell (-3, 0) should be culled (it's behind and not adjacent)
        assert!(
            !cell_coords.contains(&(-3, 0)),
            "Far cell behind creature SHOULD be culled. Found: {:?}",
            cell_coords
        );
    }

    #[test]
    fn test_collect_cells_sorted_adjacent_cells_examined_first() {
        // Adjacent cells (3x3) should ALWAYS be examined before non-adjacent cells
        // This ensures creatures are always aware of immediate surroundings
        let mut grid = SpatialGrid::new(50.0);

        // Creature at (48, 25) - near right edge of cell (0, 0) facing right (+X)
        // This position makes cell (2, 0) closer than cell (-1, 0) by raw distance
        // But (-1, 0) is adjacent and should still be examined FIRST
        let entities = vec![
            (Entity::from_raw(1), -25.0, 25.0, 0.0, 0.0, 1.0), // Adjacent behind (-1, 0), dist ~73
            (Entity::from_raw(2), 125.0, 25.0, 0.0, 0.0, 1.0), // Non-adjacent in front (2, 0), dist ~77
            (Entity::from_raw(3), 75.0, 25.0, 0.0, 0.0, 1.0),  // Adjacent in front (1, 0), dist ~27
        ];

        grid.rebuild(entities.into_iter());

        let mut cells = Vec::new();
        grid.collect_cells_sorted(48.0, 25.0, 200.0, 1.0, 0.0, &mut cells);

        // Convert to coords preserving order
        let cell_coords: Vec<(i32, i32)> = cells
            .iter()
            .map(|&(_, idx)| grid.get_cell_coords_by_index(idx))
            .collect();

        // Find positions in the sorted order
        let adjacent_behind_pos = cell_coords.iter().position(|&c| c == (-1, 0));
        let adjacent_front_pos = cell_coords.iter().position(|&c| c == (1, 0));
        let non_adjacent_pos = cell_coords.iter().position(|&c| c == (2, 0));

        assert!(
            adjacent_behind_pos.is_some(),
            "Adjacent cell behind should be included"
        );
        assert!(
            adjacent_front_pos.is_some(),
            "Adjacent cell in front should be included"
        );
        assert!(
            non_adjacent_pos.is_some(),
            "Non-adjacent cell in front should be included"
        );

        // Adjacent cells should come BEFORE non-adjacent cells
        assert!(
            adjacent_behind_pos.unwrap() < non_adjacent_pos.unwrap(),
            "Adjacent cell behind ({:?}) should be examined BEFORE non-adjacent cell ({:?}). Order: {:?}",
            adjacent_behind_pos, non_adjacent_pos, cell_coords
        );
        assert!(
            adjacent_front_pos.unwrap() < non_adjacent_pos.unwrap(),
            "Adjacent cell in front ({:?}) should be examined BEFORE non-adjacent cell ({:?}). Order: {:?}",
            adjacent_front_pos, non_adjacent_pos, cell_coords
        );
    }

    // ============================================================================
    // PERCEPTION RANGE CULLING TESTS
    // ============================================================================

    #[test]
    fn test_small_perception_at_cell_center_culls_diagonals() {
        // Creature at center of cell (10, 10) with 5m perception
        // CELL_SIZE = 20m, so diagonal cell nearest edge is sqrt(10² + 10²) = 14.14m
        // 5m perception should cull diagonal cells
        let mut grid = SpatialGrid::new(20.0); // 20m cells

        // Put entity in each diagonal cell to make them non-empty
        let entities = vec![
            (Entity::from_raw(1), 30.0, 30.0, 0.0, 0.0, 1.0), // Diagonal (+1, +1)
            (Entity::from_raw(2), -10.0, 30.0, 0.0, 0.0, 1.0), // Diagonal (-1, +1)
            (Entity::from_raw(3), 30.0, -10.0, 0.0, 0.0, 1.0), // Diagonal (+1, -1)
            (Entity::from_raw(4), -10.0, -10.0, 0.0, 0.0, 1.0), // Diagonal (-1, -1)
            (Entity::from_raw(5), 10.0, 10.0, 0.0, 0.0, 1.0), // Own cell
        ];
        grid.rebuild(entities.into_iter());

        let mut cells = Vec::new();

        // Creature at (10, 10) - center of cell, with 5m perception, facing right
        // cos_half_fov = -1.0 (full 360° FOV) so FOV culling doesn't affect range test
        grid.collect_cells_sorted_fov(10.0, 10.0, 200.0, 5.0, 1.0, 0.0, -1.0, &mut cells);

        // Should get own cell (0,0) only - diagonal cells are ~14m away, beyond 5m perception
        // Cardinal cells are 10m away, also beyond 5m perception
        assert_eq!(
            cells.len(),
            1,
            "Creature at cell center with 5m perception should only query own cell. Found: {:?}",
            cells.iter().map(|&(_, idx)| grid.get_cell_coords_by_index(idx)).collect::<Vec<_>>()
        );
    }

    #[test]
    fn test_small_perception_at_cell_corner_includes_adjacent() {
        // Creature at corner of cell (19, 19) with 5m perception
        // Adjacent cardinal cells have nearest edge at ~1m, diagonal at ~1.4m
        // 5m perception should include all adjacent cells
        let mut grid = SpatialGrid::new(20.0);

        // Put entity in each adjacent cell
        let entities = vec![
            (Entity::from_raw(1), 30.0, 30.0, 0.0, 0.0, 1.0), // Diagonal (+1, +1)
            (Entity::from_raw(2), -10.0, 30.0, 0.0, 0.0, 1.0), // Diagonal (-1, +1)
            (Entity::from_raw(3), 30.0, -10.0, 0.0, 0.0, 1.0), // Diagonal (+1, -1)
            (Entity::from_raw(4), -10.0, -10.0, 0.0, 0.0, 1.0), // Diagonal (-1, -1)
            (Entity::from_raw(5), 30.0, 10.0, 0.0, 0.0, 1.0), // Cardinal (+1, 0)
            (Entity::from_raw(6), -10.0, 10.0, 0.0, 0.0, 1.0), // Cardinal (-1, 0)
            (Entity::from_raw(7), 10.0, 30.0, 0.0, 0.0, 1.0), // Cardinal (0, +1)
            (Entity::from_raw(8), 10.0, -10.0, 0.0, 0.0, 1.0), // Cardinal (0, -1)
            (Entity::from_raw(9), 19.0, 19.0, 0.0, 0.0, 1.0), // Own cell at corner
        ];
        grid.rebuild(entities.into_iter());

        let mut cells = Vec::new();

        // Creature at (19, 19) - corner of cell, with 5m perception, facing right
        // cos_half_fov = -1.0 (full 360° FOV) so FOV culling doesn't affect range test
        grid.collect_cells_sorted_fov(19.0, 19.0, 200.0, 5.0, 1.0, 0.0, -1.0, &mut cells);

        // At corner (19,19), distances to adjacent cells are much shorter:
        // - Own cell: 0m (always included)
        // - Cardinal (1,0) nearest edge at x=20: dx = 20-19-10 = -9 -> 0, dist = 0 (inside cell overlap)
        // Wait, let me recalculate. Cell (1,0) has center at (30, 10), half_cell = 10
        // dx_to_center = |30-19| = 11, dx_to_edge = 11-10 = 1
        // dy_to_center = |10-19| = 9, dy_to_edge = max(0, 9-10) = 0
        // dist = sqrt(1² + 0²) = 1m ✓ Passes 5m perception

        // Should include multiple adjacent cells (those within 5m of nearest edge)
        assert!(
            cells.len() >= 4,
            "Creature at cell corner with 5m perception should include multiple adjacent cells. Found: {} cells: {:?}",
            cells.len(),
            cells.iter().map(|&(_, idx)| grid.get_cell_coords_by_index(idx)).collect::<Vec<_>>()
        );
    }

    #[test]
    fn test_own_cell_always_included() {
        // Even with tiny 1m perception, own cell should always be included
        let mut grid = SpatialGrid::new(20.0);

        let entities = vec![(Entity::from_raw(1), 10.0, 10.0, 0.0, 0.0, 1.0)];
        grid.rebuild(entities.into_iter());

        let mut cells = Vec::new();

        // 1m perception - should still get own cell (facing right)
        // cos_half_fov = -1.0 (full 360° FOV) so FOV culling doesn't affect range test
        grid.collect_cells_sorted_fov(10.0, 10.0, 200.0, 1.0, 1.0, 0.0, -1.0, &mut cells);

        assert_eq!(
            cells.len(),
            1,
            "Own cell should always be included regardless of perception range"
        );
    }

    #[test]
    fn test_large_perception_includes_all_adjacent() {
        // With 50m perception, all 9 adjacent cells should be included
        let mut grid = SpatialGrid::new(20.0);

        // Put entity in every cell of 3x3 grid around origin
        let mut entities = Vec::new();
        let mut id = 1;
        for cx in -1..=1 {
            for cy in -1..=1 {
                let x = cx as f32 * 20.0 + 10.0;
                let y = cy as f32 * 20.0 + 10.0;
                entities.push((Entity::from_raw(id), x, y, 0.0, 0.0, 1.0));
                id += 1;
            }
        }
        grid.rebuild(entities.into_iter());

        let mut cells = Vec::new();

        // 50m perception - should get all 9 cells (facing right)
        // cos_half_fov = -1.0 (full 360° FOV) so FOV culling doesn't affect range test
        grid.collect_cells_sorted_fov(10.0, 10.0, 200.0, 50.0, 1.0, 0.0, -1.0, &mut cells);

        assert_eq!(
            cells.len(),
            9,
            "Large perception should include all 9 adjacent cells. Found: {:?}",
            cells.iter().map(|&(_, idx)| grid.get_cell_coords_by_index(idx)).collect::<Vec<_>>()
        );
    }
}
