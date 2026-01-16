use super::biosignature::BioSignature;
use super::constants::{L1_CELL_SIZE, L2_CELL_SIZE};

/// Coarse spatial grid (L1) for aggregated bio-signatures, with L2 strategic layer.
///
/// L1: Each L1 cell covers a 3×3 block of L0 cells (60m × 60m).
/// L2: Each L2 cell covers a 3×3 block of L1 cells (180m × 180m).
///
/// Stores aggregated BioSignature data for efficient early-exit
/// optimization and size domination checks.
#[derive(Debug)]
pub struct CoarseGrid {
    // L1 grid
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

    // L2 strategic grid
    l2_cells: Vec<BioSignature>,
    l2_prev_non_empty: Vec<usize>,
    l2_width: usize,
    l2_height: usize,
    l2_cell_size: f32,
    l2_inv_cell_size: f32,
    l2_min_cell_x: i32,
    l2_min_cell_y: i32,
}

impl Default for CoarseGrid {
    fn default() -> Self {
        Self {
            // L1 grid
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
            // L2 grid
            l2_cells: Vec::new(),
            l2_prev_non_empty: Vec::new(),
            l2_width: 0,
            l2_height: 0,
            l2_cell_size: L2_CELL_SIZE,
            l2_inv_cell_size: 1.0 / L2_CELL_SIZE,
            l2_min_cell_x: 0,
            l2_min_cell_y: 0,
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

        // L1 grid allocation
        self.min_cell_x = (min_x * self.inv_cell_size).floor() as i32;
        self.min_cell_y = (min_y * self.inv_cell_size).floor() as i32;

        let max_cell_x = (max_x * self.inv_cell_size).ceil() as i32;
        let max_cell_y = (max_y * self.inv_cell_size).ceil() as i32;

        self.width = (max_cell_x - self.min_cell_x) as usize;
        self.height = (max_cell_y - self.min_cell_y) as usize;

        let total_cells = self.width * self.height;
        self.cells = vec![BioSignature::default(); total_cells];
        self.prev_non_empty = Vec::with_capacity(total_cells / 10); // Expect ~10% occupancy

        // L2 grid allocation
        self.l2_min_cell_x = (min_x * self.l2_inv_cell_size).floor() as i32;
        self.l2_min_cell_y = (min_y * self.l2_inv_cell_size).floor() as i32;

        let l2_max_cell_x = (max_x * self.l2_inv_cell_size).ceil() as i32;
        let l2_max_cell_y = (max_y * self.l2_inv_cell_size).ceil() as i32;

        self.l2_width = (l2_max_cell_x - self.l2_min_cell_x) as usize;
        self.l2_height = (l2_max_cell_y - self.l2_min_cell_y) as usize;

        let l2_total_cells = self.l2_width * self.l2_height;
        self.l2_cells = vec![BioSignature::default(); l2_total_cells];
        self.l2_prev_non_empty = Vec::with_capacity(l2_total_cells / 10);
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

    // ========== L2 Strategic Grid Methods ==========

    /// Clear only previously non-empty L2 cells.
    pub fn clear_l2(&mut self) {
        for &cell_idx in &self.l2_prev_non_empty {
            self.l2_cells[cell_idx].clear();
        }
        self.l2_prev_non_empty.clear();
    }

    /// Convert world position to L2 cell index.
    #[inline]
    pub fn position_to_l2_cell_index(&self, x: f32, y: f32) -> usize {
        let cx = (x * self.l2_inv_cell_size).floor() as i32 - self.l2_min_cell_x;
        let cy = (y * self.l2_inv_cell_size).floor() as i32 - self.l2_min_cell_y;

        let cx = cx.clamp(0, self.l2_width as i32 - 1) as usize;
        let cy = cy.clamp(0, self.l2_height as i32 - 1) as usize;

        cy * self.l2_width + cx
    }

    /// Convert L1 cell index to parent L2 cell index.
    /// L2 cells are 3×3 blocks of L1 cells.
    #[inline]
    pub fn l1_to_l2_cell_index(&self, l1_cell_idx: usize) -> usize {
        let l1_cx = l1_cell_idx % self.width;
        let l1_cy = l1_cell_idx / self.width;

        // L2 cell = L1 cell / 3
        let l2_cx = l1_cx / 3;
        let l2_cy = l1_cy / 3;

        // Clamp to valid L2 range
        let l2_cx = l2_cx.min(self.l2_width.saturating_sub(1));
        let l2_cy = l2_cy.min(self.l2_height.saturating_sub(1));

        l2_cy * self.l2_width + l2_cx
    }

    /// Get L2 biosignature for a cell index.
    #[inline]
    pub fn get_l2_biosignature(&self, cell_idx: usize) -> &BioSignature {
        &self.l2_cells[cell_idx]
    }

    /// Get L2 biosignature at world position.
    #[inline]
    pub fn get_l2_biosignature_at(&self, x: f32, y: f32) -> &BioSignature {
        let idx = self.position_to_l2_cell_index(x, y);
        &self.l2_cells[idx]
    }

    /// Add data directly to an L2 cell index.
    /// Tracks newly non-empty cells for efficient clearing.
    #[inline]
    pub fn add_to_l2_cell(&mut self, cell_idx: usize, mass: f32, size: f32) {
        let was_empty = self.l2_cells[cell_idx].is_empty();
        self.l2_cells[cell_idx].add(mass, size);
        if was_empty {
            self.l2_prev_non_empty.push(cell_idx);
        }
    }

    /// Merge an L1 BioSignature into an L2 cell (used for L1→L2 aggregation).
    /// Tracks newly non-empty cells for efficient clearing.
    #[inline]
    pub fn merge_to_l2_cell(&mut self, cell_idx: usize, biosig: &BioSignature) {
        let was_empty = self.l2_cells[cell_idx].is_empty();
        self.l2_cells[cell_idx].merge(biosig);
        if was_empty {
            self.l2_prev_non_empty.push(cell_idx);
        }
    }

    /// Get world center coordinates for an L2 cell by its index.
    #[inline]
    pub fn cell_center_from_l2_index(&self, cell_idx: usize) -> (f32, f32) {
        let cx = (cell_idx % self.l2_width) as i32 + self.l2_min_cell_x;
        let cy = (cell_idx / self.l2_width) as i32 + self.l2_min_cell_y;
        let center_x = (cx as f32 + 0.5) * self.l2_cell_size;
        let center_y = (cy as f32 + 0.5) * self.l2_cell_size;
        (center_x, center_y)
    }

    /// Get L2 grid width in cells.
    #[inline]
    pub fn l2_width(&self) -> usize {
        self.l2_width
    }

    /// Get L2 grid height in cells.
    #[inline]
    pub fn l2_height(&self) -> usize {
        self.l2_height
    }

    /// Get L2 cell size in world units.
    #[inline]
    pub fn l2_cell_size(&self) -> f32 {
        self.l2_cell_size
    }

    /// Get number of non-empty L2 cells (for telemetry).
    #[inline]
    pub fn l2_non_empty_count(&self) -> usize {
        self.l2_prev_non_empty.len()
    }

    /// Get all non-empty L2 cells with their coordinates and biosignature data.
    pub fn l2_non_empty_cells_with_data(&self) -> impl Iterator<Item = (i32, i32, &BioSignature)> {
        self.l2_prev_non_empty.iter().map(move |&cell_idx| {
            let cx = (cell_idx % self.l2_width) as i32 + self.l2_min_cell_x;
            let cy = (cell_idx / self.l2_width) as i32 + self.l2_min_cell_y;
            (cx, cy, &self.l2_cells[cell_idx])
        })
    }

    /// Get iterator over non-empty L1 cell indices.
    /// Used by aggregate_l2 to iterate only populated L1 cells.
    pub fn non_empty_l1_indices(&self) -> impl Iterator<Item = usize> + '_ {
        self.prev_non_empty.iter().copied()
    }

    /// Convert L1 cell index to cell coordinates.
    /// Returns (cx, cy) in grid coordinates (not world coordinates).
    #[inline]
    pub fn index_to_cell_coords(&self, cell_idx: usize) -> (i32, i32) {
        let cx = (cell_idx % self.width) as i32 + self.min_cell_x;
        let cy = (cell_idx / self.width) as i32 + self.min_cell_y;
        (cx, cy)
    }

    /// Convert cell coordinates to L1 cell index.
    /// Returns None if coordinates are outside the grid bounds.
    #[inline]
    pub fn get_cell_index_by_coords(&self, cx: i32, cy: i32) -> Option<usize> {
        let local_cx = cx - self.min_cell_x;
        let local_cy = cy - self.min_cell_y;

        if local_cx < 0
            || local_cy < 0
            || local_cx >= self.width as i32
            || local_cy >= self.height as i32
        {
            return None;
        }

        Some(local_cy as usize * self.width + local_cx as usize)
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

    // ========== L2 Tests ==========

    #[test]
    fn l2_cell_size_is_180m() {
        use super::super::constants::L2_CELL_SIZE;
        assert_eq!(L2_CELL_SIZE, 180.0);
    }

    #[test]
    fn set_world_bounds_allocates_l2_grid() {
        let mut grid = CoarseGrid::new();
        // World size = 540m to get at least 3×3 L2 cells at 180m
        grid.set_world_bounds(-270.0, 270.0, -270.0, 270.0);

        assert!(grid.l2_width >= 3, "Expected >= 3 L2 cells, got {}", grid.l2_width);
        assert!(grid.l2_height >= 3, "Expected >= 3 L2 cells, got {}", grid.l2_height);
        assert_eq!(grid.l2_cells.len(), grid.l2_width * grid.l2_height);
    }

    #[test]
    fn l1_to_l2_cell_index_maps_correctly() {
        let mut grid = CoarseGrid::new();
        // World: 540m × 540m → 9×9 L1 cells (60m each) → 3×3 L2 cells (180m each)
        grid.set_world_bounds(0.0, 540.0, 0.0, 540.0);

        // L1 cell (0,0) -> L2 cell (0,0)
        assert_eq!(grid.l1_to_l2_cell_index(0), 0);

        // L1 cell (2,2) -> L2 cell (0,0) (same block)
        assert_eq!(grid.l1_to_l2_cell_index(2 + 2 * grid.width), 0);

        // L1 cell (3,0) -> L2 cell (1,0)
        assert_eq!(grid.l1_to_l2_cell_index(3), 1);

        // L1 cell (0,3) -> L2 cell (0,1)
        assert_eq!(grid.l1_to_l2_cell_index(0 + 3 * grid.width), grid.l2_width);
    }

    #[test]
    fn add_to_l2_cell_accumulates() {
        let mut grid = CoarseGrid::new();
        grid.set_world_bounds(-270.0, 270.0, -270.0, 270.0);

        let l2_idx = grid.position_to_l2_cell_index(0.0, 0.0);

        // Add data to L2 cell
        grid.add_to_l2_cell(l2_idx, 10.0, 1.0);
        grid.add_to_l2_cell(l2_idx, 20.0, 2.0);

        let sig = grid.get_l2_biosignature(l2_idx);
        assert_eq!(sig.total_mass, 30.0);
        assert_eq!(sig.max_size, 2.0);
        assert_eq!(sig.creature_count, 2);
    }

    #[test]
    fn clear_l2_only_clears_non_empty() {
        let mut grid = CoarseGrid::new();
        grid.set_world_bounds(-270.0, 270.0, -270.0, 270.0);

        let l2_idx = grid.position_to_l2_cell_index(0.0, 0.0);
        grid.add_to_l2_cell(l2_idx, 10.0, 1.0);
        assert_eq!(grid.l2_non_empty_count(), 1);

        grid.clear_l2();
        assert_eq!(grid.l2_non_empty_count(), 0);

        let sig = grid.get_l2_biosignature(l2_idx);
        assert!(sig.is_empty());
    }

    #[test]
    fn cell_center_from_l2_index_works() {
        let mut grid = CoarseGrid::new();
        grid.set_world_bounds(0.0, 540.0, 0.0, 540.0);

        // First L2 cell center should be at (90, 90) - half of 180m
        let (cx, cy) = grid.cell_center_from_l2_index(0);
        assert!((cx - 90.0).abs() < 0.01, "Expected cx ~90, got {}", cx);
        assert!((cy - 90.0).abs() < 0.01, "Expected cy ~90, got {}", cy);
    }
}
