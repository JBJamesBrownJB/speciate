use bevy_ecs::prelude::*;

#[cfg(test)]
use super::grid::SpatialGrid;
use super::hierarchical::HierarchicalGrid;
use crate::simulation::core::components::{BodySize, Position, Velocity};

#[cfg(feature = "dev-tools")]
use crate::instrumentation::SystemTimings;

/// Rebuild L0 spatial grid into the BACK buffer (double-buffered).
/// Uses parallel rebuild for ~3x speedup with Rayon.
pub fn rebuild_spatial_grid_system(
    mut grid: ResMut<HierarchicalGrid>,
    query: Query<(Entity, &Position, &Velocity, &BodySize)>,
    #[cfg(feature = "dev-tools")] timings: Res<SystemTimings>,
) {
    #[cfg(feature = "dev-tools")]
    crate::time_system!(timings, "spatial_grid_rebuild");

    // Write to L0 back buffer using parallel rebuild
    // Format: (entity, x, y, vx, vy, radius, conspicuousness)
    // conspicuousness is precomputed per-creature here (once) so the hot detection
    // loop never calls powf — see BodySize::conspicuousness.
    grid.l0.write_grid().rebuild_parallel(query.iter().map(|(e, pos, vel, size)| {
        (e, pos.x, pos.y, vel.vx, vel.vy, size.radius(), size.conspicuousness())
    }));
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
    use crate::Velocity;
    use bevy_ecs::world::World;

    #[test]
    fn test_rebuild_populates_grid() {
        let mut world = World::new();

        world.insert_resource(SpatialGrid::default());

        world.spawn((
            Position { x: 25.0, y: 25.0 },
            Velocity::default(),
            BodySize::new(1.0),
        ));
        world.spawn((
            Position { x: 75.0, y: 25.0 },
            Velocity::default(),
            BodySize::new(2.0),
        ));
        world.spawn((
            Position { x: 25.0, y: 75.0 },
            Velocity::default(),
            BodySize::new(1.5),
        ));

        // Collect to Vec first to avoid borrow conflicts
        let entities: Vec<_> = world
            .query::<(Entity, &Position, &Velocity, &BodySize)>()
            .iter(&world)
            .map(|(e, p, v, s)| (e, p.x, p.y, v.vx, v.vy, s.radius()))
            .collect();

        world
            .resource_mut::<SpatialGrid>()
            .rebuild(entities.into_iter());

        assert_eq!(world.resource::<SpatialGrid>().entity_count(), 3);
    }

    #[test]
    fn test_rebuild_clears_previous_entries() {
        let mut world = World::new();

        world.insert_resource(SpatialGrid::default());

        let entity = world
            .spawn((
                Position { x: 25.0, y: 25.0 },
                Velocity::default(),
                BodySize::new(1.0),
            ))
            .id();

        // First rebuild - collect to Vec first
        let entities: Vec<_> = world
            .query::<(Entity, &Position, &Velocity, &BodySize)>()
            .iter(&world)
            .map(|(e, p, v, s)| (e, p.x, p.y, v.vx, v.vy, s.radius()))
            .collect();
        world
            .resource_mut::<SpatialGrid>()
            .rebuild(entities.into_iter());

        assert_eq!(world.resource::<SpatialGrid>().entity_count(), 1);

        world.entity_mut(entity).despawn();

        // Rebuild after despawn
        let entities: Vec<_> = world
            .query::<(Entity, &Position, &Velocity, &BodySize)>()
            .iter(&world)
            .map(|(e, p, v, s)| (e, p.x, p.y, v.vx, v.vy, s.radius()))
            .collect();

        world
            .resource_mut::<SpatialGrid>()
            .rebuild(entities.into_iter());

        assert_eq!(world.resource::<SpatialGrid>().entity_count(), 0);
    }
}
