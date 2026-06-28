use bevy_ecs::system::Resource;

use super::coarse_grid::CoarseGrid;
use super::constants::CELL_SIZE;
use super::grid::DoubleBufferedSpatialGrid;
use crate::simulation::core::MAX_WORLD_SIZE;

/// Hierarchical spatial grid combining L0 and L1 grids.
///
/// - L0: 20m cells, stores entity IDs (PerceptionProxy)
/// - L1: 60m cells (3×3 L0), stores aggregated BioSignatures
///
/// L0 is double-buffered (perception reads front while rebuild writes back).
/// L1 is single-buffered (rebuilt from L0 each tick).
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
        // World size = 8 × L1_CELL_SIZE to guarantee reasonable grid allocation
        let half_world = L1_CELL_SIZE * 4.0;
        let grid = HierarchicalGrid::with_fixed_bounds(-half_world, half_world, -half_world, half_world);

        // Should have at least 6 L1 cells per side (8 × L1_CELL_SIZE / L1_CELL_SIZE = 8, minus edge effects)
        assert!(grid.l1.width() >= 6, "Expected L1 width >= 6, got {}", grid.l1.width());
        assert!(grid.l1.height() >= 6, "Expected L1 height >= 6, got {}", grid.l1.height());
    }

    /// RED TEST — proves that aggregate_l1 writes biomass to the WRONG L1 cell when
    /// the world origin is not aligned to a 3×L0-cell (60 m) boundary.
    ///
    /// With bounds (−80, 80) the SpatialGrid padding rule yields
    /// l0.min_cell_x = −5, and −5 % 3 = −2 ≠ 0, so `l0_to_l1_cell_index` maps
    /// the L0 array column to the wrong L1 column.  Biomass accumulates in the
    /// *adjacent* L1 cell rather than the cell that actually contains the creature.
    ///
    /// This test MUST FAIL until the fix lands.
    #[test]
    fn aggregate_l1_places_biomass_in_correct_l1_cell() {
        use bevy_ecs::prelude::Entity;

        // (−80, 80, −80, 80): SpatialGrid ±1 padding gives L0 min_cell_x = −5.
        // −5 % 3 = −2 ≠ 0  →  l0_to_l1_cell_index bug is live.
        let mut hgrid = HierarchicalGrid::with_fixed_bounds(-80.0, 80.0, -80.0, 80.0);

        // Populate the L0 *back* buffer (write_grid) with one creature at world (1.0, 1.0).
        // rebuild_parallel requires fixed_bounds = true, which with_fixed_bounds guarantees.
        hgrid.l0.write_grid().rebuild_parallel(std::iter::once(
            // (entity, x, y, vx, vy, radius, conspicuousness) — conspicuousness unused by L1 aggregation
            (Entity::from_raw(1), 1.0_f32, 1.0_f32, 0.0_f32, 0.0_f32, 2.0_f32, 2.0_f32),
        ));

        // aggregate_l1 reads the back buffer and maps L0 cells → L1 cells.
        // Due to the bug, the creature at L0 arr col 5 is aggregated into
        // L1 arr idx 5 (world cell (−1,−1)) instead of correct idx 10 (world cell (0,0)).
        hgrid.aggregate_l1();

        // The creature at world (1,1) belongs to L1 world cell (0,0).
        // If the bug is present, get_biosignature_at(1,1) reads L1 arr idx 10 which is empty.
        let sig = hgrid.l1.get_biosignature_at(1.0, 1.0);
        assert!(
            sig.creature_count > 0,
            "L1 cell at world (1,1) should contain the creature after aggregate_l1 \
             (creature_count={}). Bug: l0_to_l1_cell_index maps L0 arr col to the wrong \
             L1 cell when l0.min_cell_x is not divisible by 3.",
            sig.creature_count,
        );
    }
}
