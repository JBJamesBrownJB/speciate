//! Debug test to verify wander behavior (historical debugging test)

use speciate::simulation::components::*;
use speciate::simulation::creatures::builder::CritBuilder;
use speciate::SimulationBuilder;

#[test]
fn debug_single_wanderer_step_by_step() {
    // Single wanderer at origin - verify it moves and stays within territory
    let mut sim = SimulationBuilder::new()
        .set_boundaries(50.0, 50.0)
        .build();

    let builder = CritBuilder::new()
        .at(0.0, 0.0)
        .with_wandering()
        .in_behavior(BehaviorMode::Wandering);

    let crit_id = sim.spawn_crit(builder);

    // Run 200 updates
    for _ in 0..200 {
        sim.update(0.05);
    }

    // Verify creature is still within reasonable territory bounds
    let world = sim.world_mut();
    let mut query = world.query::<(&CritId, &Position, &HomePosition)>();

    for (id, pos, home) in query.iter(world) {
        if id.0 == crit_id {
            let dist_from_home = ((pos.x - home.x).powi(2) + (pos.y - home.y).powi(2)).sqrt();
            assert!(dist_from_home < 40.0, "Creature should stay within territory (got {:.2}m from home)", dist_from_home);
        }
    }
}
