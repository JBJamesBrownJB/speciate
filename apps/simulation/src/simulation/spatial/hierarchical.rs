use bevy_ecs::system::Resource;

use super::coarse_grid::CoarseGrid;
use super::constants::CELL_SIZE;
use super::grid::DoubleBufferedSpatialGrid;
use crate::simulation::core::MAX_WORLD_SIZE;

/// Hierarchical spatial grid combining L0 (fine) and L1 (coarse) grids.
///
/// - L0: 10m cells, stores entity IDs (PerceptionProxy)
/// - L1: 30m cells (3×3 L0), stores aggregated BioSignatures
///
/// L0 is double-buffered (perception reads front while rebuild writes back).
/// L1 is single-buffered (rebuilt from L0 data each tick).
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
}

#[cfg(test)]
mod tests {
    use super::super::constants::L1_CELL_SIZE;
    use super::*;

    #[test]
    fn default_creates_empty_grids() {
        let grid = HierarchicalGrid::new();
        assert_eq!(grid.l0_cell_size(), CELL_SIZE);
        assert_eq!(grid.l1_cell_size(), L1_CELL_SIZE);
    }

    #[test]
    fn with_fixed_bounds_allocates_both_grids() {
        let grid = HierarchicalGrid::with_fixed_bounds(-100.0, 100.0, -100.0, 100.0);

        // L0: 200m / 10m = 20 cells per side
        // L1: 200m / 30m ≈ 7 cells per side
        assert!(grid.l1.width() >= 6);
        assert!(grid.l1.height() >= 6);
    }
}
