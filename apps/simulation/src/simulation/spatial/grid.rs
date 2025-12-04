use bevy_ecs::prelude::*;

use super::constants::CELL_SIZE;

pub type EntityData = (Entity, f32, f32, f32);

/// Flat dense spatial grid with automatic bounds tracking.
///
/// Replaces FxHashMap with direct Vec indexing for cache-friendly access.
/// Only allocates cells for the active (populated) region of the world.
#[derive(Resource)]
pub struct SpatialGrid {
    // Flat 1D array indexed as 2D (row-major: y * width + x)
    cells: Vec<Vec<EntityData>>,

    // Active region bounds
    min_cell_x: i32,
    min_cell_y: i32,
    width: usize,
    height: usize,

    cell_size: f32,
    inv_cell_size: f32,
}

impl SpatialGrid {
    pub fn new(cell_size: f32) -> Self {
        Self {
            cells: Vec::new(),
            min_cell_x: 0,
            min_cell_y: 0,
            width: 0,
            height: 0,
            cell_size,
            inv_cell_size: 1.0 / cell_size,
        }
    }

    pub fn with_default_cell_size() -> Self {
        Self::new(CELL_SIZE)
    }

    pub fn cell_size(&self) -> f32 {
        self.cell_size
    }

    /// Convert world coordinates to cell coordinates
    #[inline]
    pub fn world_to_cell(&self, x: f32, y: f32) -> (i32, i32) {
        (
            (x * self.inv_cell_size).floor() as i32,
            (y * self.inv_cell_size).floor() as i32,
        )
    }

    pub fn cell_to_world_min(&self, cell_x: i32, cell_y: i32) -> (f32, f32) {
        (cell_x as f32 * self.cell_size, cell_y as f32 * self.cell_size)
    }

    /// Convert cell coordinates to flat array index (if within bounds)
    #[inline]
    fn cell_index(&self, cx: i32, cy: i32) -> Option<usize> {
        let lx = cx - self.min_cell_x;
        let ly = cy - self.min_cell_y;
        if lx >= 0 && ly >= 0 && (lx as usize) < self.width && (ly as usize) < self.height {
            Some((ly as usize) * self.width + (lx as usize))
        } else {
            None
        }
    }

    /// Rebuild grid with automatic bounds detection.
    /// Two-pass: first finds bounds, then inserts entities.
    pub fn rebuild(&mut self, entities: impl Iterator<Item = (Entity, f32, f32, f32)>) {
        // Pass 1: Collect entities and find bounds
        let all_entities: Vec<_> = entities.collect();

        if all_entities.is_empty() {
            // Clear everything for empty grid
            for cell in &mut self.cells {
                cell.clear();
            }
            return;
        }

        // Find min/max cell coordinates
        let mut min_cx = i32::MAX;
        let mut max_cx = i32::MIN;
        let mut min_cy = i32::MAX;
        let mut max_cy = i32::MIN;

        for (_, x, y, _) in &all_entities {
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

        // Resize cells array (preserve capacity of existing Vecs)
        let total_cells = self.width * self.height;
        self.cells.resize_with(total_cells, Vec::new);

        // Clear all cells
        for cell in &mut self.cells {
            cell.clear();
        }

        // Pass 2: Insert entities using direct indexing
        for (entity, x, y, radius) in all_entities {
            let (cx, cy) = self.world_to_cell(x, y);
            if let Some(idx) = self.cell_index(cx, cy) {
                self.cells[idx].push((entity, x, y, radius));
            }
        }
    }

    /// Query entities within radius using direct array indexing.
    /// Row-major iteration for cache locality.
    #[inline]
    pub fn query_radius(&self, x: f32, y: f32, radius: f32) -> impl Iterator<Item = &EntityData> {
        let (center_cx, center_cy) = self.world_to_cell(x, y);
        let cells_radius = (radius * self.inv_cell_size).ceil() as i32;

        // Pre-compute valid cell range (clamped to grid bounds)
        let min_qx = (center_cx - cells_radius).max(self.min_cell_x);
        let max_qx = (center_cx + cells_radius).min(self.min_cell_x + self.width as i32 - 1);
        let min_qy = (center_cy - cells_radius).max(self.min_cell_y);
        let max_qy = (center_cy + cells_radius).min(self.min_cell_y + self.height as i32 - 1);

        // Row-major iteration for cache locality
        (min_qy..=max_qy).flat_map(move |cy| {
            (min_qx..=max_qx).filter_map(move |cx| {
                self.cell_index(cx, cy).map(|idx| self.cells[idx].iter())
            }).flatten()
        })
    }

    /// Get list of cell coordinates that would be queried (for visualization)
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
        self.cells.iter().map(|v| v.len()).sum()
    }

    pub fn cell_count(&self) -> usize {
        self.cells.iter().filter(|v| !v.is_empty()).count()
    }

    /// Total allocated cells (including empty padding)
    pub fn allocated_cells(&self) -> usize {
        self.cells.len()
    }

    /// Grid dimensions
    pub fn dimensions(&self) -> (usize, usize) {
        (self.width, self.height)
    }

    /// Grid bounds (min cell coordinates)
    pub fn bounds(&self) -> (i32, i32) {
        (self.min_cell_x, self.min_cell_y)
    }

    // Legacy API compatibility - these call rebuild() internally

    /// Clear grid (legacy API - prefer rebuild())
    pub fn clear(&mut self) {
        for cell in &mut self.cells {
            cell.clear();
        }
    }

    /// Insert single entity (legacy API - prefer rebuild())
    /// Note: Only works if bounds are already set correctly!
    #[inline]
    pub fn insert(&mut self, entity: Entity, x: f32, y: f32, radius: f32) {
        let (cx, cy) = self.world_to_cell(x, y);
        if let Some(idx) = self.cell_index(cx, cy) {
            self.cells[idx].push((entity, x, y, radius));
        }
        // Silently ignore out-of-bounds inserts (bounds must be set via rebuild first)
    }
}

impl Default for SpatialGrid {
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
        assert!(nearby.iter().any(|(e, _, _, _)| *e == entity1));
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
}
