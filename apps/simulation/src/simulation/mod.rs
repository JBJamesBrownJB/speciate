pub mod components;
pub mod agent_systems;
pub mod sim;
pub mod resources;
pub mod timing;

pub use sim::{Simulation, SimulationBuilder};
// resources is for internal use only

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simulation_creates_successfully() {
        let sim = SimulationBuilder::new().build();
        assert_eq!(sim.creature_count(), 0);
    }

    #[test]
    fn test_spawn_creature_increases_count() {
        let mut sim = SimulationBuilder::new().build();
        sim.set_boundaries(180.0, 130.0);

        let initial_count = sim.creature_count();
        sim.spawn_creature(90.0, 65.0, 2.0, 1.0);

        assert_eq!(sim.creature_count(), initial_count + 1);
    }

    #[test]
    fn test_simulation_update_doesnt_crash() {
        let mut sim = SimulationBuilder::new().build();
        sim.set_boundaries(180.0, 130.0);
        sim.spawn_creature(90.0, 65.0, 2.0, 1.0);

        // Should not panic
        sim.update(0.016); // 60 FPS delta
    }

    #[test]
    fn test_multiple_updates_work() {
        let mut sim = SimulationBuilder::new().build();
        sim.set_boundaries(180.0, 130.0);
        sim.spawn_creature(90.0, 65.0, 2.0, 1.0);

        // Run 100 simulation ticks
        for _ in 0..100 {
            sim.update(0.016);
        }

        // Should still have the creature
        assert_eq!(sim.creature_count(), 1);
    }

    // NOTE: Tests using get_creatures() removed since we stripped out
    // serialization/network functionality. The simulation is now console-only
    // and doesn't expose creature data for inspection.
    //
    // For testing creature behavior, use ECS queries directly in integration tests
    // or observe console output during manual testing.
}
