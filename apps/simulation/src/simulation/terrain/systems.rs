use bevy_ecs::prelude::*;

use super::components::{ObstacleCache, PerceivedObstacle};
use super::grid::TerrainGrid;
use crate::simulation::core::components::Position;

pub fn update_obstacle_cache_system(
    terrain: Res<TerrainGrid>,
    mut query: Query<(&Position, &mut ObstacleCache)>,
) {
    for (pos, mut cache) in query.iter_mut() {
        let (cell_x, cell_y) = terrain.world_to_cell(pos.x, pos.y);

        // Early exit: still in same cell, cache is valid
        if cache.is_same_cell(cell_x, cell_y) {
            continue;
        }

        // Update last cell and rebuild cache
        cache.set_last_cell(cell_x, cell_y);
        cache.clear();

        // Scan 3x3 neighborhood for blocked cells
        for dy in -1i32..=1 {
            for dx in -1i32..=1 {
                // Skip self cell
                if dx == 0 && dy == 0 {
                    continue;
                }

                let check_x = cell_x as i32 + dx;
                let check_y = cell_y as i32 + dy;

                if terrain.is_blocked_cell_signed(check_x, check_y) {
                    let (center_x, center_y) =
                        terrain.cell_to_world_center(check_x as u32, check_y as u32);
                    cache.add(PerceivedObstacle::new(center_x, center_y));
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::simulation::terrain::TerrainGrid;
    use bevy_ecs::world::World;

    fn setup_world_with_terrain() -> World {
        let mut world = World::new();
        world.insert_resource(TerrainGrid::new());
        world
    }

    #[test]
    fn test_obstacle_cache_updates_on_cell_change() {
        let mut world = setup_world_with_terrain();

        // Block a cell near origin
        {
            let mut terrain = world.resource_mut::<TerrainGrid>();
            // Block cell at (126, 125) which is adjacent to origin cell (125, 125)
            terrain.set_blocked_cell(126, 125, true);
        }

        // Spawn creature at origin
        let entity = world
            .spawn((Position { x: 0.0, y: 0.0 }, ObstacleCache::new()))
            .id();

        // Run the system
        let mut schedule = Schedule::default();
        schedule.add_systems(update_obstacle_cache_system);
        schedule.run(&mut world);

        // Check cache was populated
        let cache = world.get::<ObstacleCache>(entity).unwrap();
        assert_eq!(cache.len(), 1, "Should detect one blocked cell");
        assert!(cache.is_same_cell(125, 125), "Should record current cell");
    }

    #[test]
    fn test_obstacle_cache_skips_same_cell() {
        let mut world = setup_world_with_terrain();

        // Block adjacent cell
        {
            let mut terrain = world.resource_mut::<TerrainGrid>();
            terrain.set_blocked_cell(126, 125, true);
        }

        // Spawn creature already "in" cell (125, 125)
        let mut cache = ObstacleCache::new();
        cache.set_last_cell(125, 125); // Pretend already updated
        cache.add(PerceivedObstacle::new(999.0, 999.0)); // Dummy data

        let entity = world.spawn((Position { x: 0.0, y: 0.0 }, cache)).id();

        // Run the system
        let mut schedule = Schedule::default();
        schedule.add_systems(update_obstacle_cache_system);
        schedule.run(&mut world);

        // Cache should NOT have been rebuilt (early exit)
        let cache = world.get::<ObstacleCache>(entity).unwrap();
        assert_eq!(cache.obstacles[0].center_x, 999.0, "Cache should be unchanged");
    }

    #[test]
    fn test_obstacle_cache_detects_multiple_obstacles() {
        let mut world = setup_world_with_terrain();

        // Block multiple adjacent cells (forming a corner)
        {
            let mut terrain = world.resource_mut::<TerrainGrid>();
            terrain.set_blocked_cell(126, 125, true); // Right
            terrain.set_blocked_cell(125, 126, true); // Above
            terrain.set_blocked_cell(126, 126, true); // Diagonal
        }

        let entity = world
            .spawn((Position { x: 0.0, y: 0.0 }, ObstacleCache::new()))
            .id();

        let mut schedule = Schedule::default();
        schedule.add_systems(update_obstacle_cache_system);
        schedule.run(&mut world);

        let cache = world.get::<ObstacleCache>(entity).unwrap();
        assert_eq!(cache.len(), 3, "Should detect three blocked cells");
    }

    #[test]
    fn test_obstacle_cache_no_obstacles() {
        let mut world = setup_world_with_terrain();

        // No blocked cells
        let entity = world
            .spawn((Position { x: 0.0, y: 0.0 }, ObstacleCache::new()))
            .id();

        let mut schedule = Schedule::default();
        schedule.add_systems(update_obstacle_cache_system);
        schedule.run(&mut world);

        let cache = world.get::<ObstacleCache>(entity).unwrap();
        assert!(cache.is_empty(), "Should have no obstacles");
        assert!(cache.is_same_cell(125, 125), "Should still record cell");
    }

    #[test]
    fn test_obstacle_cache_rebuilds_on_move() {
        let mut world = setup_world_with_terrain();

        // Block cell adjacent to starting position
        {
            let mut terrain = world.resource_mut::<TerrainGrid>();
            terrain.set_blocked_cell(126, 125, true);
        }

        // Creature starts at origin
        let entity = world
            .spawn((Position { x: 0.0, y: 0.0 }, ObstacleCache::new()))
            .id();

        let mut schedule = Schedule::default();
        schedule.add_systems(update_obstacle_cache_system);

        // First run - should detect obstacle
        schedule.run(&mut world);
        {
            let cache = world.get::<ObstacleCache>(entity).unwrap();
            assert_eq!(cache.len(), 1);
        }

        // Move creature to a different cell (far from the blocked cell)
        {
            let mut pos = world.get_mut::<Position>(entity).unwrap();
            pos.x = -500.0; // Different cell
            pos.y = -500.0;
        }

        // Second run - should rebuild with no obstacles
        schedule.run(&mut world);
        {
            let cache = world.get::<ObstacleCache>(entity).unwrap();
            assert!(cache.is_empty(), "Should have no obstacles after moving away");
        }
    }

    #[test]
    fn test_obstacle_cache_at_world_edge() {
        let mut world = setup_world_with_terrain();

        // Creature at corner of world
        let entity = world
            .spawn((
                Position {
                    x: -2490.0,
                    y: -2490.0,
                },
                ObstacleCache::new(),
            ))
            .id();

        let mut schedule = Schedule::default();
        schedule.add_systems(update_obstacle_cache_system);
        schedule.run(&mut world);

        // Out of bounds cells should be treated as blocked
        let cache = world.get::<ObstacleCache>(entity).unwrap();
        // At corner (0,0), cells at (-1,*) and (*,-1) are out of bounds = blocked
        assert!(cache.len() >= 3, "Should detect out-of-bounds as blocked");
    }
}
