use bevy_ecs::prelude::*;

use crate::simulation::spatial::constants::CELL_HALF_DIAGONAL;

pub const MAX_PERCEIVED_OBSTACLES: usize = 4;

#[derive(Clone, Copy, Default, Debug)]
pub struct PerceivedObstacle {
    pub center_x: f32,
    pub center_y: f32,
    pub radius: f32,
}

impl PerceivedObstacle {
    pub fn new(center_x: f32, center_y: f32) -> Self {
        Self {
            center_x,
            center_y,
            radius: CELL_HALF_DIAGONAL,
        }
    }

    pub fn with_radius(center_x: f32, center_y: f32, radius: f32) -> Self {
        Self {
            center_x,
            center_y,
            radius,
        }
    }
}

#[derive(Component, Clone, Debug)]
pub struct ObstacleCache {
    pub obstacles: [PerceivedObstacle; MAX_PERCEIVED_OBSTACLES],
    pub count: u8,
    pub last_cell_x: u32,
    pub last_cell_y: u32,
}

impl ObstacleCache {
    pub fn new() -> Self {
        Self {
            obstacles: [PerceivedObstacle::default(); MAX_PERCEIVED_OBSTACLES],
            count: 0,
            // Use u32::MAX as sentinel for "never updated"
            last_cell_x: u32::MAX,
            last_cell_y: u32::MAX,
        }
    }

    pub fn clear(&mut self) {
        self.count = 0;
    }

    pub fn add(&mut self, obstacle: PerceivedObstacle) -> bool {
        if (self.count as usize) < MAX_PERCEIVED_OBSTACLES {
            self.obstacles[self.count as usize] = obstacle;
            self.count += 1;
            true
        } else {
            false
        }
    }

    pub fn set_last_cell(&mut self, cell_x: u32, cell_y: u32) {
        self.last_cell_x = cell_x;
        self.last_cell_y = cell_y;
    }

    pub fn is_same_cell(&self, cell_x: u32, cell_y: u32) -> bool {
        self.last_cell_x == cell_x && self.last_cell_y == cell_y
    }

    pub fn iter(&self) -> impl Iterator<Item = &PerceivedObstacle> {
        self.obstacles[..self.count as usize].iter()
    }

    pub fn is_empty(&self) -> bool {
        self.count == 0
    }

    pub fn len(&self) -> usize {
        self.count as usize
    }
}

impl Default for ObstacleCache {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_obstacle_cache_new() {
        let cache = ObstacleCache::new();
        assert_eq!(cache.count, 0);
        assert!(cache.is_empty());
        assert_eq!(cache.last_cell_x, u32::MAX);
        assert_eq!(cache.last_cell_y, u32::MAX);
    }

    #[test]
    fn test_obstacle_cache_add() {
        let mut cache = ObstacleCache::new();

        let added = cache.add(PerceivedObstacle::new(100.0, 200.0));
        assert!(added);
        assert_eq!(cache.len(), 1);
        assert!(!cache.is_empty());

        let obs = &cache.obstacles[0];
        assert_eq!(obs.center_x, 100.0);
        assert_eq!(obs.center_y, 200.0);
        assert_eq!(obs.radius, CELL_HALF_DIAGONAL);
    }

    #[test]
    fn test_obstacle_cache_add_max() {
        let mut cache = ObstacleCache::new();

        // Add up to max
        for i in 0..MAX_PERCEIVED_OBSTACLES {
            let added = cache.add(PerceivedObstacle::new(i as f32, 0.0));
            assert!(added);
        }
        assert_eq!(cache.len(), MAX_PERCEIVED_OBSTACLES);

        // Try to add one more - should fail
        let added = cache.add(PerceivedObstacle::new(999.0, 999.0));
        assert!(!added);
        assert_eq!(cache.len(), MAX_PERCEIVED_OBSTACLES);
    }

    #[test]
    fn test_obstacle_cache_clear() {
        let mut cache = ObstacleCache::new();

        cache.add(PerceivedObstacle::new(1.0, 2.0));
        cache.add(PerceivedObstacle::new(3.0, 4.0));
        assert_eq!(cache.len(), 2);

        cache.clear();
        assert!(cache.is_empty());
        assert_eq!(cache.len(), 0);
    }

    #[test]
    fn test_obstacle_cache_last_cell() {
        let mut cache = ObstacleCache::new();

        // Initially not in any cell
        assert!(!cache.is_same_cell(100, 100));

        cache.set_last_cell(100, 100);
        assert!(cache.is_same_cell(100, 100));
        assert!(!cache.is_same_cell(101, 100));
        assert!(!cache.is_same_cell(100, 101));
    }

    #[test]
    fn test_obstacle_cache_iter() {
        let mut cache = ObstacleCache::new();

        cache.add(PerceivedObstacle::new(10.0, 20.0));
        cache.add(PerceivedObstacle::new(30.0, 40.0));

        let positions: Vec<_> = cache.iter().map(|o| (o.center_x, o.center_y)).collect();
        assert_eq!(positions, vec![(10.0, 20.0), (30.0, 40.0)]);
    }

    #[test]
    fn test_perceived_obstacle_default_radius() {
        let obs = PerceivedObstacle::new(0.0, 0.0);
        assert_eq!(obs.radius, CELL_HALF_DIAGONAL);
    }

    #[test]
    fn test_perceived_obstacle_custom_radius() {
        let obs = PerceivedObstacle::with_radius(10.0, 20.0, 5.0);
        assert_eq!(obs.center_x, 10.0);
        assert_eq!(obs.center_y, 20.0);
        assert_eq!(obs.radius, 5.0);
    }
}
