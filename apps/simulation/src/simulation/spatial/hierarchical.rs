use bevy_ecs::system::Resource;

use super::coarse_grid::CoarseGrid;
use super::constants::CELL_SIZE;
use super::grid::DoubleBufferedSpatialGrid;
use crate::simulation::core::MAX_WORLD_SIZE;

/// Hierarchical spatial grid combining L0, L1, and L2 grids.
///
/// - L0: 20m cells, stores entity IDs (PerceptionProxy)
/// - L1: 60m cells (3×3 L0), stores aggregated BioSignatures
/// - L2: 180m cells (3×3 L1), stores strategic BioSignatures
///
/// L0 is double-buffered (perception reads front while rebuild writes back).
/// L1 and L2 are single-buffered (rebuilt from lower level each tick).
#[derive(Resource)]
pub struct HierarchicalGrid {
    pub l0: DoubleBufferedSpatialGrid,
    pub l1: CoarseGrid,
}

impl Default for HierarchicalGrid {
    fn default() -> Self {
        Self::new()
    }
}

impl HierarchicalGrid {
    pub fn new() -> Self {
        Self {
            l0: DoubleBufferedSpatialGrid::new(CELL_SIZE),
            l1: CoarseGrid::new(),
        }
    }

    /// Create with fixed world bounds (pre-allocated, no per-tick allocations).
    pub fn with_fixed_bounds(min_x: f32, max_x: f32, min_y: f32, max_y: f32) -> Self {
        let mut grid = Self {
            l0: DoubleBufferedSpatialGrid::with_fixed_bounds(CELL_SIZE, min_x, max_x, min_y, max_y),
            l1: CoarseGrid::new(),
        };
        grid.l1.set_world_bounds(min_x, max_x, min_y, max_y);
        grid
    }

    /// Create with default world bounds (MAX_WORLD_SIZE).
    pub fn with_default_bounds() -> Self {
        Self::with_fixed_bounds(
            -MAX_WORLD_SIZE,
            MAX_WORLD_SIZE,
            -MAX_WORLD_SIZE,
            MAX_WORLD_SIZE,
        )
    }

    /// Set world bounds on both L0 and L1 grids. Call once at startup.
    pub fn set_world_bounds(&mut self, min_x: f32, max_x: f32, min_y: f32, max_y: f32) {
        self.l0.set_world_bounds(min_x, max_x, min_y, max_y);
        self.l1.set_world_bounds(min_x, max_x, min_y, max_y);
    }

    /// Get L0 cell size.
    #[inline]
    pub fn l0_cell_size(&self) -> f32 {
        self.l0.cell_size()
    }

    /// Get L1 cell size.
    #[inline]
    pub fn l1_cell_size(&self) -> f32 {
        self.l1.cell_size()
    }

    /// Aggregate L0 back buffer data into L1 coarse grid.
    ///
    /// Call after L0 rebuild, before buffer swap.
    /// Reduces L0 proxies into L1 BioSignatures by:
    /// - Iterating only non-empty L0 cells
    /// - Deriving mass from radius
    /// - Accumulating total_mass, max_size, creature_count per L1 cell
    pub fn aggregate_l1(&mut self) {
        use super::biosignature::BioSignature;

        // Clear L1 (only previously non-empty cells)
        self.l1.clear();

        // Get L0 back buffer (just rebuilt, not yet swapped)
        let l0 = self.l0.write_grid();
        let l0_width = l0.width();

        // Iterate only populated L0 cells
        for &l0_cell_idx in l0.non_empty_cells() {
            let l1_cell_idx = self.l1.l0_to_l1_cell_index(l0_cell_idx, l0_width);

            // Aggregate all proxies in this L0 cell
            for proxy in l0.get_cell_proxies(l0_cell_idx) {
                let mass = BioSignature::mass_from_radius(proxy.radius);
                self.l1.add_to_cell(l1_cell_idx, mass, proxy.radius);
            }
        }
    }

    /// Aggregate L1 data into L2 strategic grid.
    ///
    /// Call after aggregate_l1(), before perception systems.
    /// Reduces L1 BioSignatures into L2 BioSignatures by:
    /// - Iterating only non-empty L1 cells
    /// - Merging total_mass, max_size, creature_count per L2 cell
    pub fn aggregate_l2(&mut self) {
        // Clear L2 (only previously non-empty cells)
        self.l1.clear_l2();

        // Collect non-empty L1 indices first to avoid borrowing issues
        let non_empty_l1: Vec<usize> = self.l1.non_empty_l1_indices().collect();

        // Aggregate non-empty L1 cells into L2
        for l1_idx in non_empty_l1 {
            let l2_idx = self.l1.l1_to_l2_cell_index(l1_idx);
            let biosig = *self.l1.get_biosignature(l1_idx);
            self.l1.merge_to_l2_cell(l2_idx, &biosig);
        }
    }

    /// Get L2 cell size.
    #[inline]
    pub fn l2_cell_size(&self) -> f32 {
        self.l1.l2_cell_size()
    }
}

#[cfg(test)]
mod tests {
    use super::super::constants::{L1_CELL_SIZE, L2_CELL_SIZE};
    use super::*;

    #[test]
    fn default_creates_empty_grids() {
        let grid = HierarchicalGrid::new();
        assert_eq!(grid.l0_cell_size(), CELL_SIZE);
        assert_eq!(grid.l1_cell_size(), L1_CELL_SIZE);
        assert_eq!(grid.l2_cell_size(), L2_CELL_SIZE);
    }

    #[test]
    fn with_fixed_bounds_allocates_both_grids() {
        // World size = 8 × L1_CELL_SIZE to guarantee reasonable grid allocation
        let half_world = L1_CELL_SIZE * 4.0;
        let grid = HierarchicalGrid::with_fixed_bounds(-half_world, half_world, -half_world, half_world);

        // Should have at least 6 L1 cells per side (8 × L1_CELL_SIZE / L1_CELL_SIZE = 8, minus edge effects)
        assert!(grid.l1.width() >= 6, "Expected L1 width >= 6, got {}", grid.l1.width());
        assert!(grid.l1.height() >= 6, "Expected L1 height >= 6, got {}", grid.l1.height());
    }

    #[test]
    fn aggregate_l2_sums_l1_biosignatures() {
        // World: 540m × 540m → enough for 9×9 L1 cells → 3×3 L2 cells
        let mut grid = HierarchicalGrid::with_fixed_bounds(0.0, 540.0, 0.0, 540.0);

        // Add creatures to L1 cells manually (bypassing L0 for direct test)
        // These should all map to L2 cell (0,0)
        grid.l1.add_to_cell(0, 10.0, 1.0); // L1 cell (0,0) → L2 (0,0), 1 creature
        grid.l1.add_to_cell(1, 20.0, 2.0); // L1 cell (1,0) → L2 (0,0), 1 creature
        grid.l1.add_to_cell(2, 30.0, 3.0); // L1 cell (2,0) → L2 (0,0), 1 creature

        // Run L2 aggregation
        grid.aggregate_l2();

        // Check L2 cell (0,0) has aggregated data from 3 L1 cells
        let l2_biosig = grid.l1.get_l2_biosignature(0);
        assert_eq!(l2_biosig.total_mass, 60.0); // 10 + 20 + 30
        assert_eq!(l2_biosig.max_size, 3.0); // max(1, 2, 3)
        assert_eq!(l2_biosig.creature_count, 3); // 3 creatures total
    }

    #[test]
    fn aggregate_l2_only_aggregates_non_empty_l1() {
        let mut grid = HierarchicalGrid::with_fixed_bounds(0.0, 540.0, 0.0, 540.0);

        // Add creatures to only 2 L1 cells
        grid.l1.add_to_cell(0, 10.0, 1.0);
        grid.l1.add_to_cell(grid.l1.width() * 3, 20.0, 2.0); // Different L2 cell (0,1)

        grid.aggregate_l2();

        // Should have exactly 2 non-empty L2 cells
        assert_eq!(grid.l1.l2_non_empty_count(), 2);
    }

    #[test]
    fn aggregate_l2_clears_previous_data() {
        let mut grid = HierarchicalGrid::with_fixed_bounds(0.0, 540.0, 0.0, 540.0);

        // First tick: add to L1, aggregate
        grid.l1.add_to_cell(0, 100.0, 5.0);
        grid.aggregate_l2();
        assert_eq!(grid.l1.get_l2_biosignature(0).total_mass, 100.0);

        // Clear L1 (simulating new tick)
        grid.l1.clear();

        // Second tick: add different data to L1
        grid.l1.add_to_cell(0, 50.0, 2.0);
        grid.aggregate_l2();

        // L2 should have new data, not accumulated
        assert_eq!(grid.l1.get_l2_biosignature(0).total_mass, 50.0);
        assert_eq!(grid.l1.get_l2_biosignature(0).max_size, 2.0);
    }
}
