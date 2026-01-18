mod components;
mod grid;
mod systems;

pub use components::{ObstacleCache, PerceivedObstacle, MAX_PERCEIVED_OBSTACLES};
pub use grid::TerrainGrid;
pub use systems::update_obstacle_cache_system;

use crate::simulation::spatial::CELL_SIZE;

pub const TERRAIN_CELL_SIZE: f32 = CELL_SIZE; // 20m, matches L0

#[cfg(test)]
mod tests;
