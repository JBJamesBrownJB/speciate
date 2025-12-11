use bevy_ecs::prelude::*;
use rayon::prelude::*;
use std::sync::atomic::{AtomicU32, Ordering};

use super::constants::{CELL_SIZE, NON_ADJACENT_OFFSET};
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
    pub radius: f32,
    pub entity: Entity,
}

impl Default for PerceptionProxy {
    fn default() -> Self {
        Self {
            x: 0.0,
            y: 0.0,
            radius: 0.0,
            entity: Entity::PLACEHOLDER,
        }
    }
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
    entity_scratch: Vec<(Entity, f32, f32, f32)>,

    // Pre-allocated atomic counters for scatter phase (reused each tick)
    atomic_counters: Vec<AtomicU32>,

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
    pub fn with_fixed_bounds(cell_size: f32, min_x: f32, max_x: f32, min_y: f32, max_y: f32) -> Self {
        let mut grid = Self::new(cell_size);
        grid.set_world_bounds(min_x, max_x, min_y, max_y);
        grid
    }

    /// Create grid with default cell size and MAX_WORLD_SIZE bounds.
    pub fn with_default_bounds() -> Self {
        Self::with_fixed_bounds(CELL_SIZE, -MAX_WORLD_SIZE, MAX_WORLD_SIZE, -MAX_WORLD_SIZE, MAX_WORLD_SIZE)
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
            self.width, self.height, total_cells,
            self.world_min_x, self.world_min_y, self.world_max_x, self.world_max_y,
            self.cell_size
        );
    }

    pub fn cell_size(&self) -> f32 {
        self.cell_size
    }

    #[inline(always)]
    pub fn world_to_cell(&self, x: f32, y: f32) -> (i32, i32) {
        (
            (x * self.inv_cell_size).floor() as i32,
            (y * self.inv_cell_size).floor() as i32,
        )
    }

    pub fn cell_to_world_min(&self, cell_x: i32, cell_y: i32) -> (f32, f32) {
        (cell_x as f32 * self.cell_size, cell_y as f32 * self.cell_size)
    }

    #[inline(always)]
    fn cell_index_unchecked(&self, x: f32, y: f32) -> usize {
        let (cx, cy) = self.world_to_cell(x, y);
        let lx = (cx - self.min_cell_x) as usize;
        let ly = (cy - self.min_cell_y) as usize;
        ly * self.width + lx
    }

    /// Rebuild grid using O(N) counting sort for cache-friendly layout.
    ///
    /// Phase 0: Collect entities and find bounds
    /// Phase 1: Count histogram (entities per cell)
    /// Phase 2: Prefix sum (compute offsets)
    /// Phase 3: Scatter (bin entities into contiguous buffer)
    pub fn rebuild(&mut self, entities: impl Iterator<Item = (Entity, f32, f32, f32)>) {
        // Phase 0: Collect into reusable scratch buffer (avoids allocation after first call)
        self.entity_scratch.clear();
        self.entity_scratch.extend(entities);

        if self.entity_scratch.is_empty() {
            self.proxies.clear();
            for cell in &mut self.cells {
                *cell = (0, 0);
            }
            return;
        }

        // Find min/max cell coordinates
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

        // Add 1-cell padding for queries at edges
        self.min_cell_x = min_cx - 1;
        self.min_cell_y = min_cy - 1;
        self.width = (max_cx - min_cx + 3) as usize;
        self.height = (max_cy - min_cy + 3) as usize;

        // Resize arrays
        let total_cells = self.width * self.height;
        self.cells.resize(total_cells, (0, 0));
        self.proxies.resize(self.entity_scratch.len(), PerceptionProxy::default());

        // Phase 1: Count histogram
        for cell in &mut self.cells {
            cell.1 = 0;
        }

        for (_, x, y, _) in &self.entity_scratch {
            let idx = self.cell_index_unchecked(*x, *y);
            self.cells[idx].1 += 1;
        }

        // Phase 2: Prefix sum (compute offsets)
        let mut offset = 0u32;
        for cell in &mut self.cells {
            cell.0 = offset;
            offset += cell.1;
            cell.1 = 0; // Reset count for scatter phase
        }

        // Phase 3: Scatter into contiguous buffer
        for &(entity, x, y, radius) in &self.entity_scratch {
            let idx = self.cell_index_unchecked(x, y);
            let (start, count) = &mut self.cells[idx];
            let write_pos = (*start + *count) as usize;
            self.proxies[write_pos] = PerceptionProxy { x, y, radius, entity };
            *count += 1;
        }
    }

    /// Parallel rebuild using Rayon.
    ///
    /// In fixed-bounds mode: Zero allocations per tick, uses pre-allocated buffers.
    /// In dynamic mode: Falls back to histogram-based approach (allocates per tick).
    pub fn rebuild_parallel(&mut self, entities: impl Iterator<Item = (Entity, f32, f32, f32)>) {
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

        if self.fixed_bounds {
            self.rebuild_parallel_fixed_bounds(n);
        } else {
            self.rebuild_parallel_dynamic_bounds(n);
        }
    }

    /// Fast path: Fixed bounds, no allocations per tick.
    /// Uses two-pass algorithm: count then scatter.
    fn rebuild_parallel_fixed_bounds(&mut self, n: usize) {
        let inv_cell_size = self.inv_cell_size;
        let min_cell_x = self.min_cell_x;
        let min_cell_y = self.min_cell_y;
        let width = self.width;

        // Resize proxies buffer (no realloc if capacity sufficient)
        self.proxies.resize(n, PerceptionProxy::default());

        // Reset cell counts to zero (reuse existing allocation)
        for cell in &mut self.cells {
            *cell = (0, 0);
        }

        // Phase 1: Count entities per cell (sequential, simple)
        for &(_, x, y, _) in &self.entity_scratch {
            let cx = (x * inv_cell_size).floor() as i32;
            let cy = (y * inv_cell_size).floor() as i32;
            let lx = (cx - min_cell_x) as usize;
            let ly = (cy - min_cell_y) as usize;
            if lx < width && ly < self.height {
                let idx = ly * width + lx;
                self.cells[idx].1 += 1;
            }
        }

        // Phase 2: Prefix sum (compute offsets)
        let mut offset = 0u32;
        for cell in &mut self.cells {
            cell.0 = offset;
            offset += cell.1;
        }

        // Reset atomic counters (reuse existing allocation)
        for counter in &self.atomic_counters {
            counter.store(0, Ordering::Relaxed);
        }

        // Phase 3: Parallel atomic scatter
        let proxies_ptr = SyncPtr(self.proxies.as_mut_ptr());
        let cells_ref = &self.cells;
        let atomic_counters = &self.atomic_counters;
        let height = self.height;

        // SAFETY: Each entity writes to a unique position via atomic increment
        self.entity_scratch.par_iter().for_each(|&(entity, x, y, radius)| {
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
                proxies_ptr.write_at(write_pos, PerceptionProxy { x, y, radius, entity });
            }
        });
    }

    /// Slow path: Dynamic bounds (legacy, allocates per tick).
    fn rebuild_parallel_dynamic_bounds(&mut self, n: usize) {
        let inv_cell_size = self.inv_cell_size;

        // Find bounds (parallel reduction)
        let (min_cx, max_cx, min_cy, max_cy) = self.entity_scratch
            .par_iter()
            .map(|(_, x, y, _)| {
                let (cx, cy) = (
                    (*x * inv_cell_size).floor() as i32,
                    (*y * inv_cell_size).floor() as i32,
                );
                (cx, cx, cy, cy)
            })
            .reduce(
                || (i32::MAX, i32::MIN, i32::MAX, i32::MIN),
                |(min_x1, max_x1, min_y1, max_y1), (min_x2, max_x2, min_y2, max_y2)| {
                    (min_x1.min(min_x2), max_x1.max(max_x2), min_y1.min(min_y2), max_y1.max(max_y2))
                },
            );

        // Add 1-cell padding
        self.min_cell_x = min_cx - 1;
        self.min_cell_y = min_cy - 1;
        self.width = (max_cx - min_cx + 3) as usize;
        self.height = (max_cy - min_cy + 3) as usize;

        let total_cells = self.width * self.height;
        self.cells.resize(total_cells, (0, 0));
        self.proxies.resize(n, PerceptionProxy::default());

        // Phase 1: Parallel histogram with thread-local counts
        const CHUNK_SIZE: usize = 4096;
        let min_cell_x = self.min_cell_x;
        let min_cell_y = self.min_cell_y;
        let width = self.width;

        let local_histograms: Vec<Vec<u32>> = self.entity_scratch
            .par_chunks(CHUNK_SIZE)
            .map(|chunk| {
                let mut local_counts = vec![0u32; total_cells];
                for (_, x, y, _) in chunk {
                    let cx = (*x * inv_cell_size).floor() as i32;
                    let cy = (*y * inv_cell_size).floor() as i32;
                    let idx = ((cy - min_cell_y) as usize) * width + ((cx - min_cell_x) as usize);
                    local_counts[idx] += 1;
                }
                local_counts
            })
            .collect();

        // Merge histograms (sequential but O(cells), not O(entities))
        for cell in &mut self.cells {
            cell.1 = 0;
        }
        for hist in &local_histograms {
            for (i, &count) in hist.iter().enumerate() {
                self.cells[i].1 += count;
            }
        }

        // Phase 2: Prefix sum (sequential, O(cells))
        let mut offset = 0u32;
        for cell in &mut self.cells {
            cell.0 = offset;
            offset += cell.1;
        }

        // Phase 3: Atomic scatter (allocates counters each time)
        let atomic_counters: Vec<AtomicU32> = self.cells
            .iter()
            .map(|_| AtomicU32::new(0))
            .collect();

        let proxies_ptr = SyncPtr(self.proxies.as_mut_ptr());
        let cells_ref = &self.cells;

        // SAFETY: Each entity writes to a unique position via atomic increment
        self.entity_scratch.par_iter().for_each(move |&(entity, x, y, radius)| {
            let cx = (x * inv_cell_size).floor() as i32;
            let cy = (y * inv_cell_size).floor() as i32;
            let idx = ((cy - min_cell_y) as usize) * width + ((cx - min_cell_x) as usize);

            let start = cells_ref[idx].0;
            let local_offset = atomic_counters[idx].fetch_add(1, Ordering::Relaxed);
            let write_pos = (start + local_offset) as usize;

            // SAFETY: Unique write position guaranteed by atomic increment
            unsafe {
                proxies_ptr.write_at(write_pos, PerceptionProxy { x, y, radius, entity });
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
            (min_qx..=max_qx).filter_map(move |cx| {
                // Cell center in world coordinates
                let cell_center_x = (cx as f32 * cell_size) + half_cell;
                let cell_center_y = (cy as f32 * cell_size) + half_cell;

                // Check if cell is behind creature (conservative: use cell_size threshold)
                let cell_dir_dot = (cell_center_x - x) * facing_x + (cell_center_y - y) * facing_y;
                if cell_dir_dot < -cell_size {
                    return None; // Entire cell is behind, skip
                }

                let idx = ((cy - self.min_cell_y) as usize) * self.width
                        + ((cx - self.min_cell_x) as usize);
                let (start, count) = unsafe { *self.cells.get_unchecked(idx) };
                if count == 0 {
                    None
                } else {
                    Some(unsafe {
                        self.proxies.get_unchecked(start as usize..(start + count) as usize)
                    })
                }
            }).flatten()
        })
    }

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

        let (center_cx, center_cy) = self.world_to_cell(x, y);
        let cells_radius = (radius * self.inv_cell_size).ceil() as i32;
        let cell_size = self.cell_size;
        let half_cell = cell_size * 0.5;

        let min_qx = (center_cx - cells_radius).max(self.min_cell_x);
        let max_qx = (center_cx + cells_radius).min(self.min_cell_x + self.width as i32 - 1);
        let min_qy = (center_cy - cells_radius).max(self.min_cell_y);
        let max_qy = (center_cy + cells_radius).min(self.min_cell_y + self.height as i32 - 1);

        for cy in min_qy..=max_qy {
            for cx in min_qx..=max_qx {
                let cell_center_x = (cx as f32 * cell_size) + half_cell;
                let cell_center_y = (cy as f32 * cell_size) + half_cell;

                // Adjacent cells (3x3 grid) - skip FOV check, always include, examine FIRST
                let is_adjacent = (cx - center_cx).abs() <= 1 && (cy - center_cy).abs() <= 1;

                if !is_adjacent {
                    // Only apply FOV culling for distant cells
                    let cell_dir_dot = (cell_center_x - x) * facing_x + (cell_center_y - y) * facing_y;
                    if cell_dir_dot < -cell_size {
                        continue;
                    }
                }

                let idx = ((cy - self.min_cell_y) as usize) * self.width
                        + ((cx - self.min_cell_x) as usize);
                let (_, count) = unsafe { *self.cells.get_unchecked(idx) };
                if count > 0 {
                    let dx = cell_center_x - x;
                    let dy = cell_center_y - y;
                    let dist_sq = dx * dx + dy * dy;

                    // Adjacent cells get raw distance, non-adjacent get offset so they sort after
                    let sort_key = if is_adjacent { dist_sq } else { dist_sq + NON_ADJACENT_OFFSET };
                    out.push((sort_key, idx));
                }
            }
        }

        // Sort by sort_key: adjacent cells first (by distance), then non-adjacent (by distance)
        out.sort_unstable_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal));
    }

    /// Get proxies for a cell by index. Use after collect_cells_sorted.
    #[inline(always)]
    pub fn get_cell_proxies(&self, cell_idx: usize) -> &[PerceptionProxy] {
        let (start, count) = unsafe { *self.cells.get_unchecked(cell_idx) };
        unsafe { self.proxies.get_unchecked(start as usize..(start + count) as usize) }
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

    /// Query entities within radius. Returns iterator over contiguous slices.
    #[inline(always)]
    pub fn query_radius(&self, x: f32, y: f32, radius: f32) -> impl Iterator<Item = &PerceptionProxy> {
        let (center_cx, center_cy) = self.world_to_cell(x, y);
        let cells_radius = (radius * self.inv_cell_size).ceil() as i32;

        // Pre-compute valid cell range (clamped to grid bounds)
        let min_qx = (center_cx - cells_radius).max(self.min_cell_x);
        let max_qx = (center_cx + cells_radius).min(self.min_cell_x + self.width as i32 - 1);
        let min_qy = (center_cy - cells_radius).max(self.min_cell_y);
        let max_qy = (center_cy + cells_radius).min(self.min_cell_y + self.height as i32 - 1);

        // Row-major iteration for cache locality
        // SAFETY: min/max are pre-clamped to valid cell bounds above
        (min_qy..=max_qy).flat_map(move |cy| {
            (min_qx..=max_qx).filter_map(move |cx| {
                let idx = ((cy - self.min_cell_y) as usize) * self.width
                        + ((cx - self.min_cell_x) as usize);
                let (start, count) = unsafe { *self.cells.get_unchecked(idx) };
                if count == 0 {
                    None
                } else {
                    Some(unsafe {
                        self.proxies.get_unchecked(start as usize..(start + count) as usize)
                    })
                }
            }).flatten()
        })
    }

    /// Query entities into a pre-allocated buffer (for Rayon thread-local buffers).
    #[inline(always)]
    pub fn query_radius_into(&self, x: f32, y: f32, radius: f32, results: &mut Vec<PerceptionProxy>) {
        results.clear();

        let (center_cx, center_cy) = self.world_to_cell(x, y);
        let cells_radius = (radius * self.inv_cell_size).ceil() as i32;

        let min_qx = (center_cx - cells_radius).max(self.min_cell_x);
        let max_qx = (center_cx + cells_radius).min(self.min_cell_x + self.width as i32 - 1);
        let min_qy = (center_cy - cells_radius).max(self.min_cell_y);
        let max_qy = (center_cy + cells_radius).min(self.min_cell_y + self.height as i32 - 1);

        // Pre-reserve for typical density (~3 entities per cell, capped at 64)
        let cell_count = ((max_qx - min_qx + 1) * (max_qy - min_qy + 1)) as usize;
        results.reserve((cell_count * 3).min(64));

        // SAFETY: min/max are pre-clamped to valid cell bounds above
        for cy in min_qy..=max_qy {
            for cx in min_qx..=max_qx {
                let idx = ((cy - self.min_cell_y) as usize) * self.width
                        + ((cx - self.min_cell_x) as usize);
                let (start, count) = unsafe { *self.cells.get_unchecked(idx) };
                if count > 0 {
                    let slice = unsafe {
                        self.proxies.get_unchecked(start as usize..(start + count) as usize)
                    };
                    results.extend_from_slice(slice);
                }
            }
        }
    }

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

    /// Get query cells with FOV culling - matches what query_radius_fov actually queries.
    /// Only returns cells that have creatures in them (non-empty cells).
    pub fn get_query_cells_fov(
        &self,
        x: f32,
        y: f32,
        radius: f32,
        facing_x: f32,
        facing_y: f32,
    ) -> Vec<(i32, i32)> {
        let (center_cx, center_cy) = self.world_to_cell(x, y);
        let cells_radius = (radius * self.inv_cell_size).ceil() as i32;
        let cell_size = self.cell_size;
        let half_cell = cell_size * 0.5;

        let min_qx = (center_cx - cells_radius).max(self.min_cell_x);
        let max_qx = (center_cx + cells_radius).min(self.min_cell_x + self.width as i32 - 1);
        let min_qy = (center_cy - cells_radius).max(self.min_cell_y);
        let max_qy = (center_cy + cells_radius).min(self.min_cell_y + self.height as i32 - 1);

        let capacity = ((max_qx - min_qx + 1) * (max_qy - min_qy + 1)) as usize;
        let mut cells = Vec::with_capacity(capacity.min(100));

        for cy in min_qy..=max_qy {
            for cx in min_qx..=max_qx {
                // Cell center in world coordinates
                let cell_center_x = (cx as f32 * cell_size) + half_cell;
                let cell_center_y = (cy as f32 * cell_size) + half_cell;

                // Check if cell is behind creature (same logic as query_radius_fov)
                let cell_dir_dot = (cell_center_x - x) * facing_x + (cell_center_y - y) * facing_y;
                if cell_dir_dot < -cell_size {
                    continue; // Cell is behind, skip
                }

                // Only include cells that have creatures in them
                let idx = ((cy - self.min_cell_y) as usize) * self.width
                        + ((cx - self.min_cell_x) as usize);
                if idx < self.cells.len() {
                    let (_, count) = self.cells[idx];
                    if count > 0 {
                        cells.push((cx, cy));
                    }
                }
            }
        }

        cells
    }

    pub fn entity_count(&self) -> usize {
        self.proxies.len()
    }

    pub fn cell_count(&self) -> usize {
        self.cells.iter().filter(|(_, count)| *count > 0).count()
    }

    pub fn allocated_cells(&self) -> usize {
        self.cells.len()
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

    #[inline]
    pub fn insert(&mut self, _entity: Entity, _x: f32, _y: f32, _radius: f32) {
        // Legacy API - not supported with counting sort approach
        // Use rebuild() instead
        panic!("insert() not supported - use rebuild() for DOD grid");
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
    pub fn with_fixed_bounds(cell_size: f32, min_x: f32, max_x: f32, min_y: f32, max_y: f32) -> Self {
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
        Self::with_fixed_bounds(CELL_SIZE, -MAX_WORLD_SIZE, MAX_WORLD_SIZE, -MAX_WORLD_SIZE, MAX_WORLD_SIZE)
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
            (entity1, 25.0, 25.0, 1.0),
            (entity2, 75.0, 25.0, 1.0),
            (entity3, 25.0, 75.0, 1.0),
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
            (Entity::from_raw(1), 25.0, 25.0, 1.0),
            (Entity::from_raw(2), 75.0, 25.0, 1.0),
        ];
        grid.rebuild(entities1.into_iter());
        assert_eq!(grid.entity_count(), 2);

        // Second rebuild with different entities
        let entities2 = vec![
            (Entity::from_raw(3), 125.0, 125.0, 1.0),
        ];
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
            (Entity::from_raw(1), 0.0, 0.0, 1.0),      // cell (0, 0)
            (Entity::from_raw(2), 100.0, 100.0, 1.0), // cell (2, 2)
            (Entity::from_raw(3), -50.0, -50.0, 1.0), // cell (-1, -1)
        ];

        grid.rebuild(entities.into_iter());

        assert!(grid.width >= 5);
        assert!(grid.height >= 5);
    }

    #[test]
    fn test_query_respects_bounds() {
        let mut grid = SpatialGrid::new(50.0);

        let entities = vec![
            (Entity::from_raw(1), 25.0, 25.0, 1.0),
        ];

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
        let entities = vec![
            (Entity::from_raw(1), 25.0, 25.0, 1.0),
        ];
        grid.rebuild(entities.into_iter());
        assert_eq!(grid.entity_count(), 1);

        // Then rebuild with empty
        grid.rebuild(std::iter::empty());
        assert_eq!(grid.entity_count(), 0);
    }

    #[test]
    fn test_perception_proxy_size() {
        // Entity requires 8-byte alignment, so struct is 24 bytes
        // (3 f32s = 12 bytes + Entity = 8 bytes + 4 bytes padding = 24)
        // Still 2.66 proxies per cache line - key benefit is contiguous buffer
        assert_eq!(std::mem::size_of::<PerceptionProxy>(), 24);
    }

    #[test]
    fn test_contiguous_buffer() {
        let mut grid = SpatialGrid::new(50.0);

        let entities = vec![
            (Entity::from_raw(1), 25.0, 25.0, 1.0),
            (Entity::from_raw(2), 26.0, 26.0, 1.0), // Same cell
            (Entity::from_raw(3), 27.0, 27.0, 1.0), // Same cell
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
            (Entity::from_raw(1), 100.0, 0.0, 1.0),  // In front (right)
            (Entity::from_raw(2), -100.0, 0.0, 1.0), // Behind (left)
            (Entity::from_raw(3), 0.0, 100.0, 1.0),  // To the side (up)
        ];

        grid.rebuild(entities.into_iter());

        // Query from origin, facing right (+X), large radius
        let facing_x = 1.0;
        let facing_y = 0.0;
        let results: Vec<_> = grid.query_radius_fov(0.0, 0.0, 150.0, facing_x, facing_y).collect();

        // Should find entity in front and to the side, but NOT the one behind
        let found_ids: Vec<u32> = results.iter().map(|p| p.entity.index()).collect();

        assert!(found_ids.contains(&1), "Should find entity in front");
        assert!(!found_ids.contains(&2), "Should NOT find entity behind (cell culled)");
        assert!(found_ids.contains(&3), "Should find entity to the side");
    }

    #[test]
    fn test_query_radius_fov_vs_regular_query() {
        let mut grid = SpatialGrid::new(50.0);

        let entities = vec![
            (Entity::from_raw(1), 75.0, 25.0, 1.0),   // Front-right
            (Entity::from_raw(2), -75.0, 25.0, 1.0),  // Back-left
            (Entity::from_raw(3), 75.0, -25.0, 1.0),  // Front-right
            (Entity::from_raw(4), -75.0, -25.0, 1.0), // Back-left
        ];

        grid.rebuild(entities.into_iter());

        // Regular query finds all 4
        let all_results: Vec<_> = grid.query_radius(0.0, 0.0, 100.0).collect();
        assert_eq!(all_results.len(), 4);

        // FOV query facing right should find only front entities
        let fov_results: Vec<_> = grid.query_radius_fov(0.0, 0.0, 100.0, 1.0, 0.0).collect();
        assert!(fov_results.len() < all_results.len(), "FOV query should return fewer results");

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
            (Entity::from_raw(1), -25.0, 25.0, 1.0),  // Adjacent cell behind (-1, 0)
            (Entity::from_raw(2), -125.0, 25.0, 1.0), // Far cell behind (-3, 0)
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
            (Entity::from_raw(1), -25.0, 25.0, 1.0),  // Adjacent behind (-1, 0), dist ~73
            (Entity::from_raw(2), 125.0, 25.0, 1.0),  // Non-adjacent in front (2, 0), dist ~77
            (Entity::from_raw(3), 75.0, 25.0, 1.0),   // Adjacent in front (1, 0), dist ~27
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

        assert!(adjacent_behind_pos.is_some(), "Adjacent cell behind should be included");
        assert!(adjacent_front_pos.is_some(), "Adjacent cell in front should be included");
        assert!(non_adjacent_pos.is_some(), "Non-adjacent cell in front should be included");

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
}
