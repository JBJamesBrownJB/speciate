//! Test to verify MAX_WANDER_DISTANCE emergency return works

use speciate::simulation::components::*;
use speciate::simulation::creatures::builder::CritBuilder;
use speciate::SimulationBuilder;

#[test]
fn test_max_wander_distance_triggers_return() {
    // Spawn creature at origin, manually push it far away (35m), verify homeward force pulls it back
    let mut sim = SimulationBuilder::new()
        .set_boundaries(50.0, 50.0)
        .build();

    let builder = CritBuilder::new()
        .at(0.0, 0.0)
        .with_wandering()
        .in_behavior(BehaviorMode::Wandering);

    let crit_id = sim.spawn_crit(builder);

    // Manually move creature 35m from home (beyond MAX_WANDER_DISTANCE=30m)
    {
        let world = sim.world_mut();
        let mut query = world.query::<(&CritId, &mut Position)>();

        for (id, mut pos) in query.iter_mut(world) {
            if id.0 == crit_id {
                // Move creature to 35m from home
                pos.x = 35.0;
                pos.y = 0.0;
            }
        }
    }

    // Run several updates to build up velocity - hybrid system should apply strong homeward force
    for _ in 0..10 {
        sim.update(0.05);
    }

    // Verify creature is moving toward home
    {
        let world = sim.world_mut();
        let mut query = world.query::<(&CritId, &Position, &HomePosition)>();

        for (id, pos, home) in query.iter(world) {
            if id.0 == crit_id {
                let dist_from_home = ((pos.x - home.x).powi(2) + (pos.y - home.y).powi(2)).sqrt();

                // After 10 ticks of strong homeward force, creature should have moved closer to home
                assert!(
                    pos.x < 35.0,
                    "Creature should be moving homeward (X should be < 35), got x={:.2}",
                    pos.x
                );

                // Should still be beyond comfort radius but heading home
                assert!(
                    dist_from_home > 10.0,
                    "Creature should still be beyond comfort radius, got {:.2}m",
                    dist_from_home
                );
            }
        }
    }
}
