use bevy_ecs::system::Resource;
use rayon::prelude::*;

use super::coarse_grid::CoarseGrid;
use super::constants::CELL_SIZE;
use super::grid::DoubleBufferedSpatialGrid;
use crate::simulation::core::MAX_WORLD_SIZE;

#[inline]
fn l0_to_l1_idx(l0_cell_idx: usize, l0_width: usize, l1_width: usize, l1_height: usize) -> usize {
    let l0_cx = l0_cell_idx % l0_width;
    let l0_cy = l0_cell_idx / l0_width;
    let l1_cx = (l0_cx / 3).min(l1_width.saturating_sub(1));
    let l1_cy = (l0_cy / 3).min(l1_height.saturating_sub(1));
    l1_cy * l1_width + l1_cx
}

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
    /// Invariant: L0 must be immutable during aggregation. The `&self`-on-l0 /
    /// `&mut`-on-l1 split enforces this at the borrow-check level. Do not
    /// restructure this method in a way that allows concurrent L0 writes.
    pub fn aggregate_l1(&mut self) {
        self.aggregate_l1_parallel();
    }

    pub fn aggregate_l1_serial(&mut self) {
        use super::biosignature::BioSignature;

        self.l1.clear();

        let l0 = self.l0.write_grid();
        let l0_width = l0.width();

        for &l0_cell_idx in l0.non_empty_cells() {
            let l1_cell_idx = self.l1.l0_to_l1_cell_index(l0_cell_idx, l0_width);
            for proxy in l0.get_cell_proxies(l0_cell_idx) {
                let mass = BioSignature::mass_from_radius(proxy.radius);
                self.l1.add_to_cell(l1_cell_idx, mass, proxy.radius);
            }
        }
    }

    pub fn aggregate_l1_parallel(&mut self) {
        use super::biosignature::BioSignature;

        self.l1.clear();

        let l0 = self.l0.write_grid();
        let l0_width = l0.width();
        let l0_non_empty = l0.non_empty_cells();

        let l1_width = self.l1.width();
        let l1_height = self.l1.height();

        if l1_width == 0 || l1_height == 0 || l0_non_empty.is_empty() {
            return;
        }

        // Scatter into disjoint L1 row-bands. Each band owns a contiguous slice
        // of `l1.cells` (whole rows only) so writes never conflict.
        // Every band visits l0_non_empty in the SAME order, writing only cells
        // whose global l1_idx falls within [band_start, band_start+chunk).
        // Float addition order per L1 cell is therefore identical to serial → bit-identical.
        let chunk = l1_width; // one row per band keeps bands disjoint

        {
            let cells = self.l1.cells_mut();
            cells
                .par_chunks_mut(chunk)
                .enumerate()
                .for_each(|(band_index, band)| {
                    let band_start = band_index * chunk;
                    let band_end = band_start + band.len();

                    for &l0_cell_idx in l0_non_empty {
                        let l1_idx = l0_to_l1_idx(l0_cell_idx, l0_width, l1_width, l1_height);
                        if l1_idx < band_start || l1_idx >= band_end {
                            continue;
                        }
                        let local_idx = l1_idx - band_start;
                        for proxy in l0.get_cell_proxies(l0_cell_idx) {
                            let mass = BioSignature::mass_from_radius(proxy.radius);
                            band[local_idx].add(mass, proxy.radius);
                        }
                    }
                });
        }

        // Rebuild prev_non_empty with a full scan of the L1 cells slice.
        // ~28k cells max at 1M-creature world size; negligible vs creature count.
        let cells = self.l1.cells_mut();
        let mut non_empty: Vec<usize> = cells
            .iter()
            .enumerate()
            .filter_map(|(idx, cell)| if !cell.is_empty() { Some(idx) } else { None })
            .collect();
        let prev = self.l1.prev_non_empty_mut();
        prev.clear();
        prev.append(&mut non_empty);
    }

}

#[cfg(test)]
mod tests {
    use super::super::constants::L1_CELL_SIZE;
    use super::*;
    use bevy_ecs::entity::Entity;

    #[test]
    fn default_creates_empty_grids() {
        let grid = HierarchicalGrid::new();
        assert_eq!(grid.l0_cell_size(), CELL_SIZE);
        assert_eq!(grid.l1_cell_size(), L1_CELL_SIZE);
    }

    #[test]
    fn with_fixed_bounds_allocates_both_grids() {
        let half_world = L1_CELL_SIZE * 4.0;
        let grid = HierarchicalGrid::with_fixed_bounds(-half_world, half_world, -half_world, half_world);

        assert!(grid.l1.width() >= 6, "Expected L1 width >= 6, got {}", grid.l1.width());
        assert!(grid.l1.height() >= 6, "Expected L1 height >= 6, got {}", grid.l1.height());
    }

    fn make_entity(id: u32) -> Entity {
        Entity::from_raw(id)
    }

    fn populate_grid_for_parallel_test(grid: &mut HierarchicalGrid) {
        let entities: Vec<(Entity, f32, f32, f32, f32, f32)> = vec![
            (make_entity(0),  -200.0, -200.0, 0.0, 0.0, 1.0),
            (make_entity(1),  -200.0, -200.0, 0.0, 0.0, 2.0),
            (make_entity(2),   200.0, -200.0, 0.0, 0.0, 0.5),
            (make_entity(3),  -200.0,  200.0, 0.0, 0.0, 3.0),
            (make_entity(4),   200.0,  200.0, 0.0, 0.0, 1.5),
            (make_entity(5),     0.0,    0.0, 0.0, 0.0, 0.8),
            (make_entity(6),    10.0,   10.0, 0.0, 0.0, 1.2),
            (make_entity(7),  -300.0,  300.0, 0.0, 0.0, 2.5),
            (make_entity(8),   300.0, -300.0, 0.0, 0.0, 0.3),
            (make_entity(9),     5.0,    5.0, 0.0, 0.0, 0.9),
        ];
        grid.l0
            .write_grid()
            .rebuild(entities.into_iter());
    }

    #[test]
    fn parallel_aggregation_byte_identical_to_serial() {
        let world_half = L1_CELL_SIZE * 12.0;

        let mut serial_grid =
            HierarchicalGrid::with_fixed_bounds(-world_half, world_half, -world_half, world_half);
        populate_grid_for_parallel_test(&mut serial_grid);
        serial_grid.aggregate_l1_serial();

        let mut parallel_grid =
            HierarchicalGrid::with_fixed_bounds(-world_half, world_half, -world_half, world_half);
        populate_grid_for_parallel_test(&mut parallel_grid);
        parallel_grid.aggregate_l1_parallel();

        let l1_width = serial_grid.l1.width();
        let l1_height = serial_grid.l1.height();
        let total_l1 = l1_width * l1_height;

        assert_eq!(
            parallel_grid.l1.width(),
            l1_width,
            "L1 width mismatch"
        );
        assert_eq!(
            parallel_grid.l1.height(),
            l1_height,
            "L1 height mismatch"
        );

        let mut serial_non_empty: Vec<usize> =
            serial_grid.l1.non_empty_indices().to_vec();
        let mut parallel_non_empty: Vec<usize> =
            parallel_grid.l1.non_empty_indices().to_vec();
        serial_non_empty.sort_unstable();
        parallel_non_empty.sort_unstable();
        assert_eq!(
            serial_non_empty, parallel_non_empty,
            "Non-empty cell sets differ"
        );

        for idx in 0..total_l1 {
            let s = serial_grid.l1.get_biosignature(idx);
            let p = parallel_grid.l1.get_biosignature(idx);
            assert_eq!(
                s.total_mass.to_bits(),
                p.total_mass.to_bits(),
                "total_mass bit mismatch at L1 cell {idx}"
            );
            assert_eq!(
                s.max_size.to_bits(),
                p.max_size.to_bits(),
                "max_size bit mismatch at L1 cell {idx}"
            );
            assert_eq!(
                s.creature_count,
                p.creature_count,
                "creature_count mismatch at L1 cell {idx}"
            );
        }
    }

    #[test]
    fn parallel_aggregation_empty_input_no_panic() {
        let world_half = L1_CELL_SIZE * 4.0;
        let mut grid =
            HierarchicalGrid::with_fixed_bounds(-world_half, world_half, -world_half, world_half);
        grid.aggregate_l1_parallel();
        assert_eq!(grid.l1.non_empty_count(), 0);
    }

    #[test]
    fn parallel_aggregation_single_band_matches_serial() {
        let world_half = L1_CELL_SIZE * 2.0;

        let entities: Vec<(Entity, f32, f32, f32, f32, f32)> = vec![
            (make_entity(0), 0.0, 0.0, 0.0, 0.0, 1.0),
            (make_entity(1), 10.0, 10.0, 0.0, 0.0, 2.0),
        ];

        let mut serial_grid =
            HierarchicalGrid::with_fixed_bounds(-world_half, world_half, -world_half, world_half);
        serial_grid.l0.write_grid().rebuild(entities.clone().into_iter());
        serial_grid.aggregate_l1_serial();

        let mut parallel_grid =
            HierarchicalGrid::with_fixed_bounds(-world_half, world_half, -world_half, world_half);
        parallel_grid.l0.write_grid().rebuild(entities.into_iter());
        parallel_grid.aggregate_l1_parallel();

        let total_l1 = serial_grid.l1.width() * serial_grid.l1.height();
        for idx in 0..total_l1 {
            let s = serial_grid.l1.get_biosignature(idx);
            let p = parallel_grid.l1.get_biosignature(idx);
            assert_eq!(
                s.total_mass.to_bits(),
                p.total_mass.to_bits(),
                "total_mass bit mismatch at L1 cell {idx}"
            );
            assert_eq!(s.creature_count, p.creature_count);
        }
    }
}
