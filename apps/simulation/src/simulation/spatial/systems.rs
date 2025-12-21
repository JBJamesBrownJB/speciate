use bevy_ecs::prelude::*;

use super::hierarchical::HierarchicalGrid;
#[cfg(test)]
use super::grid::SpatialGrid;
use crate::simulation::core::components::{BodySize, Position};

#[cfg(feature = "dev-tools")]
use crate::instrumentation::SystemTimings;

/// Rebuild L0 spatial grid into the BACK buffer (double-buffered).
/// Uses parallel rebuild for ~3x speedup with Rayon.
pub fn rebuild_spatial_grid_system(
    mut grid: ResMut<HierarchicalGrid>,
    query: Query<(Entity, &Position, &BodySize)>,
    #[cfg(feature = "dev-tools")] timings: Res<SystemTimings>,
) {
    #[cfg(feature = "dev-tools")]
    crate::time_system!(timings, "spatial_grid_rebuild");

    // Write to L0 back buffer using parallel rebuild
    grid.l0
        .write_grid()
        .rebuild_parallel(query.iter().map(|(e, pos, size)| (e, pos.x, pos.y, size.radius())));
}

/// Aggregate L0 grid data into L1 coarse grid.
///
/// Runs after L0 rebuild, before perception.
/// Delegates to HierarchicalGrid::aggregate_l1() for the actual work.
pub fn aggregate_l1_system(
    mut grid: ResMut<HierarchicalGrid>,
    #[cfg(feature = "dev-tools")] timings: Res<SystemTimings>,
) {
    #[cfg(feature = "dev-tools")]
    crate::time_system!(timings, "l1_aggregation");

    grid.aggregate_l1();
}

/// Swap L0 front/back buffers at end of tick.
/// After this, perception will see the newly rebuilt grid.
pub fn swap_spatial_grid_buffers_system(mut grid: ResMut<HierarchicalGrid>) {
    grid.l0.swap();
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy_ecs::world::World;

    #[test]
    fn test_rebuild_populates_grid() {
        let mut world = World::new();

        world.insert_resource(SpatialGrid::default());

        world.spawn((
            Position { x: 25.0, y: 25.0 },
            BodySize::new(1.0),
        ));
        world.spawn((
            Position { x: 75.0, y: 25.0 },
            BodySize::new(2.0),
        ));
        world.spawn((
            Position { x: 25.0, y: 75.0 },
            BodySize::new(1.5),
        ));

        // Collect to Vec first to avoid borrow conflicts
        let entities: Vec<_> = world
            .query::<(Entity, &Position, &BodySize)>()
            .iter(&world)
            .map(|(e, p, s)| (e, p.x, p.y, s.radius()))
            .collect();

        world.resource_mut::<SpatialGrid>().rebuild(entities.into_iter());

        assert_eq!(world.resource::<SpatialGrid>().entity_count(), 3);
    }

    #[test]
    fn test_rebuild_clears_previous_entries() {
        let mut world = World::new();

        world.insert_resource(SpatialGrid::default());

        let entity = world.spawn((
            Position { x: 25.0, y: 25.0 },
            BodySize::new(1.0),
        )).id();

        // First rebuild - collect to Vec first
        let entities: Vec<_> = world
            .query::<(Entity, &Position, &BodySize)>()
            .iter(&world)
            .map(|(e, p, s)| (e, p.x, p.y, s.radius()))
            .collect();
        world.resource_mut::<SpatialGrid>().rebuild(entities.into_iter());

        assert_eq!(world.resource::<SpatialGrid>().entity_count(), 1);

        world.entity_mut(entity).despawn();

        // Rebuild after despawn
        let entities: Vec<_> = world
            .query::<(Entity, &Position, &BodySize)>()
            .iter(&world)
            .map(|(e, p, s)| (e, p.x, p.y, s.radius()))
            .collect();

        world.resource_mut::<SpatialGrid>().rebuild(entities.into_iter());

        assert_eq!(world.resource::<SpatialGrid>().entity_count(), 0);
    }
}
