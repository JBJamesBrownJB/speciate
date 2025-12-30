use super::biosignature::BioSignature;
use super::constants::L1_CELL_SIZE;

/// Coarse spatial grid (L1) for aggregated bio-signatures.
///
/// Each L1 cell covers a 3×3 block of L0 cells (30m × 30m).
/// Stores aggregated BioSignature data for efficient early-exit
/// optimization and size domination checks.
#[derive(Debug)]
pub struct CoarseGrid {
    cells: Vec<BioSignature>,
    prev_non_empty: Vec<usize>,
    width: usize,
    height: usize,
    cell_size: f32,
    inv_cell_size: f32,
    world_min_x: f32,
    world_min_y: f32,
    min_cell_x: i32,
    min_cell_y: i32,
}

impl Default for CoarseGrid {
    fn default() -> Self {
        Self {
            cells: Vec::new(),
            prev_non_empty: Vec::new(),
            width: 0,
            height: 0,
            cell_size: L1_CELL_SIZE,
            inv_cell_size: 1.0 / L1_CELL_SIZE,
            world_min_x: 0.0,
            world_min_y: 0.0,
            min_cell_x: 0,
            min_cell_y: 0,
        }
    }
}

impl CoarseGrid {
    pub fn new() -> Self {
        Self::default()
    }

    /// Pre-allocate grid for fixed world bounds.
    /// Call once at startup, not per-tick.
    pub fn set_world_bounds(&mut self, min_x: f32, max_x: f32, min_y: f32, max_y: f32) {
        self.world_min_x = min_x;
        self.world_min_y = min_y;

        self.min_cell_x = (min_x * self.inv_cell_size).floor() as i32;
        self.min_cell_y = (min_y * self.inv_cell_size).floor() as i32;

        let max_cell_x = (max_x * self.inv_cell_size).ceil() as i32;
        let max_cell_y = (max_y * self.inv_cell_size).ceil() as i32;

        self.width = (max_cell_x - self.min_cell_x) as usize;
        self.height = (max_cell_y - self.min_cell_y) as usize;

        let total_cells = self.width * self.height;
        self.cells = vec![BioSignature::default(); total_cells];
        self.prev_non_empty = Vec::with_capacity(total_cells / 10); // Expect ~10% occupancy
    }

    /// Clear only previously non-empty cells (O(non-empty) not O(total)).
    pub fn clear(&mut self) {
        for &cell_idx in &self.prev_non_empty {
            self.cells[cell_idx].clear();
        }
        self.prev_non_empty.clear();
    }

    /// Convert world position to cell index.
    /// Uses same formula as L0 SpatialGrid: world coords → cell coords → array index.
    #[inline]
    pub fn position_to_cell_index(&self, x: f32, y: f32) -> usize {
        // World position to cell coordinate, then offset by min_cell to get array index
        let cx = (x * self.inv_cell_size).floor() as i32 - self.min_cell_x;
        let cy = (y * self.inv_cell_size).floor() as i32 - self.min_cell_y;

        // Clamp to valid range
        let cx = cx.clamp(0, self.width as i32 - 1) as usize;
        let cy = cy.clamp(0, self.height as i32 - 1) as usize;

        cy * self.width + cx
    }

    /// Get biosignature for a cell index.
    #[inline]
    pub fn get_biosignature(&self, cell_idx: usize) -> &BioSignature {
        &self.cells[cell_idx]
    }

    /// Get biosignature at world position.
    #[inline]
    pub fn get_biosignature_at(&self, x: f32, y: f32) -> &BioSignature {
        let idx = self.position_to_cell_index(x, y);
        &self.cells[idx]
    }

    /// Add creature data to the cell at given position.
    /// Tracks newly non-empty cells for efficient clearing.
    #[inline]
    pub fn add_creature(&mut self, x: f32, y: f32, mass: f32, size: f32) {
        let cell_idx = self.position_to_cell_index(x, y);
        self.add_to_cell(cell_idx, mass, size);
    }

    /// Add creature data directly to a cell index.
    /// Used when iterating L0 cells and computing L1 cell index externally.
    #[inline]
    pub fn add_to_cell(&mut self, cell_idx: usize, mass: f32, size: f32) {
        let was_empty = self.cells[cell_idx].is_empty();
        self.cells[cell_idx].add(mass, size);
        if was_empty {
            self.prev_non_empty.push(cell_idx);
        }
    }

    /// Convert L0 cell index to parent L1 cell index.
    /// L1 cells are 3×3 blocks of L0 cells.
    #[inline]
    pub fn l0_to_l1_cell_index(&self, l0_cell_idx: usize, l0_width: usize) -> usize {
        let l0_cx = l0_cell_idx % l0_width;
        let l0_cy = l0_cell_idx / l0_width;

        // L1 cell = L0 cell / 3 (hardcoded: fov_patterns.rs lookup tables assume 3×3)
        let l1_cx = l0_cx / 3;
        let l1_cy = l0_cy / 3;

        // Clamp to valid L1 range
        let l1_cx = l1_cx.min(self.width.saturating_sub(1));
        let l1_cy = l1_cy.min(self.height.saturating_sub(1));

        l1_cy * self.width + l1_cx
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

    /// Get cell size in world units.
    #[inline]
    pub fn cell_size(&self) -> f32 {
        self.cell_size
    }

    /// Get number of non-empty cells (for telemetry).
    #[inline]
    pub fn non_empty_count(&self) -> usize {
        self.prev_non_empty.len()
    }

    /// Get all non-empty cells with their coordinates and biosignature data.
    /// Returns (cell_x, cell_y, biosignature) for each non-empty cell.
    pub fn non_empty_cells_with_data(&self) -> impl Iterator<Item = (i32, i32, &BioSignature)> {
        self.prev_non_empty.iter().map(move |&cell_idx| {
            let cx = (cell_idx % self.width) as i32 + self.min_cell_x;
            let cy = (cell_idx / self.width) as i32 + self.min_cell_y;
            (cx, cy, &self.cells[cell_idx])
        })
    }

    /// Get world center coordinates for a cell by its index.
    /// Returns (center_x, center_y) in world coordinates.
    #[inline]
    pub fn cell_center_from_index(&self, cell_idx: usize) -> (f32, f32) {
        let cx = (cell_idx % self.width) as i32 + self.min_cell_x;
        let cy = (cell_idx / self.width) as i32 + self.min_cell_y;
        let center_x = (cx as f32 + 0.5) * self.cell_size;
        let center_y = (cy as f32 + 0.5) * self.cell_size;
        (center_x, center_y)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn set_world_bounds_allocates_grid() {
        use super::super::constants::L1_CELL_SIZE;

        let mut grid = CoarseGrid::new();
        // World size = 4 × L1_CELL_SIZE to guarantee at least 4×4 cells
        let half_world = L1_CELL_SIZE * 2.0;
        grid.set_world_bounds(-half_world, half_world, -half_world, half_world);

        assert!(grid.width >= 3, "Expected >= 3 cells, got {}", grid.width);
        assert!(grid.height >= 3, "Expected >= 3 cells, got {}", grid.height);
        assert_eq!(grid.cells.len(), grid.width * grid.height);
    }

    #[test]
    fn position_to_cell_index_works() {
        let mut grid = CoarseGrid::new();
        grid.set_world_bounds(-150.0, 150.0, -150.0, 150.0);

        // Center should map to a middle cell
        let center_idx = grid.position_to_cell_index(0.0, 0.0);
        assert!(center_idx < grid.cells.len());

        // Corner positions should be valid
        let corner_idx = grid.position_to_cell_index(-149.0, -149.0);
        assert!(corner_idx < grid.cells.len());
    }

    #[test]
    fn add_creature_accumulates() {
        let mut grid = CoarseGrid::new();
        grid.set_world_bounds(-150.0, 150.0, -150.0, 150.0);

        // Add creatures at same position
        grid.add_creature(0.0, 0.0, 10.0, 1.0);
        grid.add_creature(1.0, 1.0, 20.0, 2.0); // Same cell

        let sig = grid.get_biosignature_at(0.0, 0.0);
        assert_eq!(sig.total_mass, 30.0);
        assert_eq!(sig.max_size, 2.0);
        assert_eq!(sig.creature_count, 2);
    }

    #[test]
    fn clear_only_clears_non_empty() {
        let mut grid = CoarseGrid::new();
        grid.set_world_bounds(-150.0, 150.0, -150.0, 150.0);

        // Add to one cell
        grid.add_creature(0.0, 0.0, 10.0, 1.0);
        assert_eq!(grid.non_empty_count(), 1);

        // Clear
        grid.clear();
        assert_eq!(grid.non_empty_count(), 0);

        let sig = grid.get_biosignature_at(0.0, 0.0);
        assert!(sig.is_empty());
    }

    #[test]
    fn l0_to_l1_cell_index_maps_correctly() {
        let mut grid = CoarseGrid::new();
        // Set up so L1 has known dimensions
        grid.set_world_bounds(0.0, 90.0, 0.0, 90.0); // 3×3 L1 cells at 30m

        // L0 width = 90m / 10m = 9 cells
        let l0_width = 9;

        // L0 cell (0,0) -> L1 cell (0,0)
        assert_eq!(grid.l0_to_l1_cell_index(0, l0_width), 0);

        // L0 cell (2,2) -> L1 cell (0,0) (same block)
        assert_eq!(grid.l0_to_l1_cell_index(2 + 2 * l0_width, l0_width), 0);

        // L0 cell (3,0) -> L1 cell (1,0)
        assert_eq!(grid.l0_to_l1_cell_index(3, l0_width), 1);

        // L0 cell (0,3) -> L1 cell (0,1)
        assert_eq!(
            grid.l0_to_l1_cell_index(0 + 3 * l0_width, l0_width),
            grid.width
        );
    }
}
