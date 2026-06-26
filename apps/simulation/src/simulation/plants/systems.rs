use bevy_ecs::prelude::*;
use super::grid::PlantGrid;
#[cfg(feature = "dev-tools")]
use crate::instrumentation::SystemTimings;

/// Runs every tick — owns all plant lifecycle logic.
///
/// Takes `Option<ResMut<PlantGrid>>` so the system compiles and runs harmlessly in
/// test simulations that don't insert the resource. Future additions: density growth,
/// seed dispersal, cell death when density reaches zero, and depletion signals from
/// creature feeding.
pub fn update_plants(
    _grid: Option<ResMut<PlantGrid>>,
    #[cfg(feature = "dev-tools")] timings: Res<SystemTimings>,
) {
    #[cfg(feature = "dev-tools")]
    crate::time_system!(timings, "plants");
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy_ecs::prelude::*;
    use crate::simulation::core::components::BoundaryConfig;
    use crate::simulation::plants::PlantGrid;

    #[test]
    fn update_plants_runs_without_panicking() {
        let mut world = World::new();
        let bounds = BoundaryConfig {
            min_x: -100.0,
            max_x: 100.0,
            min_y: -100.0,
            max_y: 100.0,
            margin: 10.0,
            max_force: 1.0,
        };
        let grid = PlantGrid::from_bounds(&bounds);
        world.insert_resource(grid);

        let mut schedule = Schedule::default();
        schedule.add_systems(update_plants);
        schedule.run(&mut world);

        let grid = world.resource::<PlantGrid>();
        assert_eq!(grid.live_count(), 0);
    }
}
