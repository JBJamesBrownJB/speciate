use bevy_ecs::prelude::Resource;

use crate::simulation::core::components::BoundaryConfig;

/// P0 plant grid resolution: 4m cells.
///
/// Chosen so each L0 cell (20m) covers exactly 5×5 = 25 plant cells — clean aggregation
/// into FoodScore. Fine enough that creature feeding depletes a tangible area without
/// requiring per-blade simulation.
pub const P0_CELL_SIZE: f32 = 4.0;

/// Floats per live cell in the sparse IPC buffer: world_x, world_y, density, plant_type.
pub const FLOATS_PER_PLANT_CELL: usize = 4;

/// A single cell in the P0 plant grid.
#[derive(Clone, Copy, Debug, Default)]
pub struct PlantCell {
    /// Vegetation density, 0.0 = bare ground, 1.0 = fully vegetated.
    pub density: f32,
    /// Index into the PlantSpecies table. 0 = empty/bare ground.
    pub plant_type: u8,
}

impl PlantCell {
    pub fn is_live(&self) -> bool {
        self.density > 0.0 && self.plant_type != 0
    }
}

/// Flat 2-D plant density grid at P0 resolution (4m cells).
///
/// Indexed by `row * cols + col` where row/col derive from world position.
/// Held as a Bevy Resource; the NAPI thread reads a pre-serialised sparse snapshot
/// via a separate `Arc<Mutex<Vec<f32>>>` rather than accessing the grid directly.
#[derive(Debug, Resource)]
pub struct PlantGrid {
    cells: Vec<PlantCell>,
    pub rows: u32,
    pub cols: u32,
    pub min_x: f32,
    pub min_y: f32,
}

impl PlantGrid {
    /// Build a PlantGrid covering the given world bounds at P0_CELL_SIZE resolution.
    pub fn from_bounds(bounds: &BoundaryConfig) -> Self {
        let width = bounds.max_x - bounds.min_x;
        let height = bounds.max_y - bounds.min_y;
        let cols = (width / P0_CELL_SIZE).ceil() as u32;
        let rows = (height / P0_CELL_SIZE).ceil() as u32;
        Self {
            cells: vec![PlantCell::default(); (rows * cols) as usize],
            rows,
            cols,
            min_x: bounds.min_x,
            min_y: bounds.min_y,
        }
    }

    /// Map world position to a cell index, returning `None` if out-of-bounds.
    pub fn cell_idx(&self, world_x: f32, world_y: f32) -> Option<usize> {
        let col = ((world_x - self.min_x) / P0_CELL_SIZE) as i32;
        let row = ((world_y - self.min_y) / P0_CELL_SIZE) as i32;
        if col < 0 || row < 0 || col >= self.cols as i32 || row >= self.rows as i32 {
            return None;
        }
        Some(row as usize * self.cols as usize + col as usize)
    }

    /// World-space centre of a cell given its flat index.
    pub fn cell_centre(&self, idx: usize) -> (f32, f32) {
        let col = (idx % self.cols as usize) as f32;
        let row = (idx / self.cols as usize) as f32;
        let x = self.min_x + col * P0_CELL_SIZE + P0_CELL_SIZE * 0.5;
        let y = self.min_y + row * P0_CELL_SIZE + P0_CELL_SIZE * 0.5;
        (x, y)
    }

    /// Place a plant at a world position. Overwrites any existing cell.
    pub fn set_plant(&mut self, world_x: f32, world_y: f32, density: f32, plant_type: u8) {
        if let Some(idx) = self.cell_idx(world_x, world_y) {
            self.cells[idx] = PlantCell { density, plant_type };
        }
    }

    /// Seed `count` plants of the given type at evenly-scattered positions across the grid.
    ///
    /// Uses a deterministic grid-jitter pattern so tests are reproducible without an rng
    /// dependency. Spacing = sqrt(total_area / count).
    pub fn seed_scattered(
        &mut self,
        count: u32,
        plant_type: u8,
        density: f32,
        seed: u64,
    ) {
        if count == 0 {
            return;
        }
        let total_cells = (self.rows * self.cols) as u64;
        let step = (total_cells / count as u64).max(1);
        let offset = seed % step;
        let mut placed = 0u32;
        let mut i = offset;
        while placed < count && i < total_cells {
            let idx = i as usize;
            if idx < self.cells.len() {
                self.cells[idx] = PlantCell { density, plant_type };
                placed += 1;
            }
            i += step;
        }
    }

    /// Decrement plant density at a world position by `amount`. Returns energy yield
    /// (density consumed × `biomass_yield`). Clamps density at 0.
    pub fn consume(&mut self, world_x: f32, world_y: f32, amount: f32, biomass_yield: f32) -> f32 {
        if let Some(idx) = self.cell_idx(world_x, world_y) {
            let cell = &mut self.cells[idx];
            let consumed = amount.min(cell.density);
            cell.density -= consumed;
            if cell.density <= 0.0 {
                cell.density = 0.0;
                cell.plant_type = 0;
            }
            return consumed * biomass_yield;
        }
        0.0
    }

    /// Write live cells into a pre-allocated f32 buffer in sparse format:
    /// `[live_count, x₀, y₀, density₀, type₀_as_f32, x₁, ...]`
    ///
    /// Returns the number of f32s written (always 1 + live_count * FLOATS_PER_PLANT_CELL).
    /// The caller must ensure `buf.len() >= 1 + live_cells * FLOATS_PER_PLANT_CELL`.
    pub fn write_sparse(&self, buf: &mut Vec<f32>) {
        buf.clear();
        buf.push(0.0); // placeholder for count, filled at end

        let mut count = 0u32;
        for (idx, cell) in self.cells.iter().enumerate() {
            if !cell.is_live() {
                continue;
            }
            let (cx, cy) = self.cell_centre(idx);
            buf.push(cx);
            buf.push(cy);
            buf.push(cell.density);
            buf.push(cell.plant_type as f32);
            count += 1;
        }
        buf[0] = count as f32;
    }

    /// Return all live cells as world-space tuples `(x, y, density, plant_type)`.
    pub fn live_cells_world(&self) -> Vec<(f32, f32, f32, u8)> {
        self.cells
            .iter()
            .enumerate()
            .filter(|(_, c)| c.is_live())
            .map(|(idx, c)| {
                let (x, y) = self.cell_centre(idx);
                (x, y, c.density, c.plant_type)
            })
            .collect()
    }

    pub fn live_count(&self) -> usize {
        self.cells.iter().filter(|c| c.is_live()).count()
    }

    pub fn total_cells(&self) -> usize {
        self.cells.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_bounds() -> BoundaryConfig {
        BoundaryConfig {
            min_x: -100.0,
            max_x: 100.0,
            min_y: -100.0,
            max_y: 100.0,
            margin: 10.0,
            max_force: 1.0,
        }
    }

    #[test]
    fn grid_dimensions_match_bounds() {
        let grid = PlantGrid::from_bounds(&test_bounds());
        // 200m / 4m = 50 cells per axis
        assert_eq!(grid.cols, 50);
        assert_eq!(grid.rows, 50);
        assert_eq!(grid.total_cells(), 2500);
    }

    #[test]
    fn cell_idx_in_bounds() {
        let grid = PlantGrid::from_bounds(&test_bounds());
        assert!(grid.cell_idx(-100.0, -100.0).is_some());
        assert!(grid.cell_idx(0.0, 0.0).is_some());
        // Out of range
        assert!(grid.cell_idx(200.0, 0.0).is_none());
        assert!(grid.cell_idx(0.0, -200.0).is_none());
    }

    #[test]
    fn set_and_read_plant() {
        let mut grid = PlantGrid::from_bounds(&test_bounds());
        grid.set_plant(10.0, 20.0, 0.8, 1);
        let idx = grid.cell_idx(10.0, 20.0).unwrap();
        assert!((grid.cells[idx].density - 0.8).abs() < 1e-5);
        assert_eq!(grid.cells[idx].plant_type, 1);
        assert!(grid.cells[idx].is_live());
    }

    #[test]
    fn seed_scattered_places_requested_count() {
        let mut grid = PlantGrid::from_bounds(&test_bounds());
        grid.seed_scattered(50, 1, 1.0, 42);
        assert_eq!(grid.live_count(), 50);
    }

    #[test]
    fn seed_scattered_is_within_bounds() {
        let mut grid = PlantGrid::from_bounds(&test_bounds());
        grid.seed_scattered(100, 1, 1.0, 7);
        // Every live cell centre must be within world bounds
        for (idx, cell) in grid.cells.iter().enumerate() {
            if cell.is_live() {
                let (cx, cy) = grid.cell_centre(idx);
                assert!(
                    cx >= -100.0 && cx <= 100.0 && cy >= -100.0 && cy <= 100.0,
                    "cell centre ({}, {}) out of bounds",
                    cx,
                    cy
                );
            }
        }
    }

    #[test]
    fn write_sparse_count_header_matches_live_cells() {
        let mut grid = PlantGrid::from_bounds(&test_bounds());
        grid.seed_scattered(20, 1, 0.9, 3);
        let mut buf = Vec::new();
        grid.write_sparse(&mut buf);
        let count = buf[0] as usize;
        assert_eq!(count, 20);
        assert_eq!(buf.len(), 1 + count * FLOATS_PER_PLANT_CELL);
    }

    #[test]
    fn write_sparse_positions_are_world_space() {
        let mut grid = PlantGrid::from_bounds(&test_bounds());
        // Place one plant at known position
        grid.set_plant(0.0, 0.0, 1.0, 1);
        let mut buf = Vec::new();
        grid.write_sparse(&mut buf);
        assert_eq!(buf[0] as usize, 1);
        // x and y should be near world origin (within half a cell)
        assert!((buf[1]).abs() < P0_CELL_SIZE);
        assert!((buf[2]).abs() < P0_CELL_SIZE);
    }

    #[test]
    fn consume_returns_energy_and_decrements_density() {
        let mut grid = PlantGrid::from_bounds(&test_bounds());
        grid.set_plant(5.0, 5.0, 1.0, 1);
        let energy = grid.consume(5.0, 5.0, 0.3, 10.0);
        assert!((energy - 3.0).abs() < 1e-5, "energy={energy}");
        let idx = grid.cell_idx(5.0, 5.0).unwrap();
        assert!((grid.cells[idx].density - 0.7).abs() < 1e-4);
    }

    #[test]
    fn consume_clears_cell_when_fully_eaten() {
        let mut grid = PlantGrid::from_bounds(&test_bounds());
        grid.set_plant(5.0, 5.0, 0.1, 1);
        grid.consume(5.0, 5.0, 1.0, 1.0); // eat more than available
        let idx = grid.cell_idx(5.0, 5.0).unwrap();
        assert!(!grid.cells[idx].is_live());
    }
}
