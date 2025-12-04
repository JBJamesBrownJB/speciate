use bevy_ecs::prelude::*;

use super::constants::CELL_SIZE;

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
#[derive(Resource)]
pub struct SpatialGrid {
    // Single contiguous buffer of all proxies
    proxies: Vec<PerceptionProxy>,

    // Cell -> slice mapping: (start_index, count)
    // Index = (cy - min_cell_y) * width + (cx - min_cell_x)
    cells: Vec<(u32, u32)>,

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
            proxies: Vec::new(),
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

    #[inline]
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
        // Phase 0: Collect and find bounds
        let all_entities: Vec<_> = entities.collect();

        if all_entities.is_empty() {
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

        // Resize arrays
        let total_cells = self.width * self.height;
        self.cells.resize(total_cells, (0, 0));
        self.proxies.resize(all_entities.len(), PerceptionProxy::default());

        // Phase 1: Count histogram
        for cell in &mut self.cells {
            cell.1 = 0;
        }

        for (_, x, y, _) in &all_entities {
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
        for (entity, x, y, radius) in all_entities {
            let idx = self.cell_index_unchecked(x, y);
            let (start, count) = &mut self.cells[idx];
            let write_pos = (*start + *count) as usize;
            self.proxies[write_pos] = PerceptionProxy { x, y, radius, entity };
            *count += 1;
        }
    }

    /// Query entities within radius. Returns iterator over contiguous slices.
    #[inline]
    pub fn query_radius(&self, x: f32, y: f32, radius: f32) -> impl Iterator<Item = &PerceptionProxy> {
        let (center_cx, center_cy) = self.world_to_cell(x, y);
        let cells_radius = (radius * self.inv_cell_size).ceil() as i32;

        // Pre-compute valid cell range (clamped to grid bounds)
        let min_qx = (center_cx - cells_radius).max(self.min_cell_x);
        let max_qx = (center_cx + cells_radius).min(self.min_cell_x + self.width as i32 - 1);
        let min_qy = (center_cy - cells_radius).max(self.min_cell_y);
        let max_qy = (center_cy + cells_radius).min(self.min_cell_y + self.height as i32 - 1);

        // Row-major iteration for cache locality
        (min_qy..=max_qy).flat_map(move |cy| {
            (min_qx..=max_qx).flat_map(move |cx| {
                let idx = ((cy - self.min_cell_y) as usize) * self.width
                        + ((cx - self.min_cell_x) as usize);
                let (start, count) = self.cells[idx];
                // Return slice of contiguous memory
                &self.proxies[start as usize..(start + count) as usize]
            })
        })
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
}
