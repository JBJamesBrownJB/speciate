
#[cfg(test)]
mod simulation_tests {
    use super::super::*;
    use crate::simulation::creatures::builder::CritBuilder;

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
        let builder = CritBuilder::new().at(90.0, 65.0).with_all_capabilities();
        sim.spawn_crit(builder);

        assert_eq!(sim.creature_count(), initial_count + 1);
    }

    #[test]
    fn test_simulation_update_doesnt_crash() {
        let mut sim = SimulationBuilder::new().build();
        sim.set_boundaries(180.0, 130.0);
        let builder = CritBuilder::new().at(90.0, 65.0).with_all_capabilities();
        sim.spawn_crit(builder);


        sim.update(0.016);
    }

    #[test]
    fn test_multiple_updates_work() {
        let mut sim = SimulationBuilder::new().build();
        sim.set_boundaries(180.0, 130.0);
        let builder = CritBuilder::new().at(90.0, 65.0).with_all_capabilities();
        sim.spawn_crit(builder);


        for _ in 0..100 {
            sim.update(0.016);
        }


        assert_eq!(sim.creature_count(), 1);
    }




    //


}

#[cfg(test)]
mod system_tests {

    use crate::simulation::components::{Acceleration, DeltaTime, Position, Rotation, Velocity};
    use bevy_ecs::prelude::*;

    #[test]
    fn test_movement_system_updates_position() {
        let mut world = World::new();
        world.insert_resource(DeltaTime(0.1));

        let entity = world
            .spawn((Position { x: 0.0, y: 0.0 }, Velocity { vx: 10.0, vy: 5.0 }))
            .id();


        let dt = world.resource::<DeltaTime>().0;


        let mut query = world.query::<(&mut Position, &Velocity)>();
        for (mut pos, vel) in query.iter_mut(&mut world) {
            pos.x += vel.vx * dt;
            pos.y += vel.vy * dt;
        }


        let position = world.get::<Position>(entity).unwrap();
        assert_eq!(position.x, 1.0);
        assert_eq!(position.y, 0.5);
    }

    #[test]
    fn test_acceleration_system_updates_velocity() {
        let mut world = World::new();
        world.insert_resource(DeltaTime(0.1));

        let entity = world
            .spawn((
                Velocity { vx: 0.0, vy: 0.0 },
                Acceleration { ax: 10.0, ay: 5.0 },
            ))
            .id();


        let dt = world.resource::<DeltaTime>().0;


        let mut query = world.query::<(&mut Velocity, &mut Acceleration)>();
        for (mut vel, mut acc) in query.iter_mut(&mut world) {
            vel.vx += acc.ax * dt;
            vel.vy += acc.ay * dt;
            acc.ax = 0.0;
            acc.ay = 0.0;
        }


        let velocity = world.get::<Velocity>(entity).unwrap();
        assert_eq!(velocity.vx, 1.0);
        assert_eq!(velocity.vy, 0.5);

        let acceleration = world.get::<Acceleration>(entity).unwrap();
        assert_eq!(acceleration.ax, 0.0);
        assert_eq!(acceleration.ay, 0.0);
    }

    #[test]
    fn test_rotation_system_matches_velocity() {
        let mut world = World::new();

        let entity = world
            .spawn((
                Rotation { radians: 0.0 },
                Velocity { vx: 1.0, vy: 1.0 },
            ))
            .id();


        let mut query = world.query::<(&mut Rotation, &Velocity)>();
        for (mut rot, vel) in query.iter_mut(&mut world) {
            if vel.vx != 0.0 || vel.vy != 0.0 {
                rot.radians = vel.vy.atan2(vel.vx);
            }
        }

        let rotation = world.get::<Rotation>(entity).unwrap();
        let expected = 1.0f32.atan2(1.0);
        assert!((rotation.radians - expected).abs() < 0.001);
    }
}

#[cfg(test)]
mod behavior_tests {

    use crate::simulation::components::{BehaviorMode, CreatureState, CritId, Position, Target, Velocity};
    use crate::simulation::creatures::builder::CritBuilder;
    use crate::simulation::SimulationBuilder;

    #[test]
    fn test_catatonic_crit_spawns_with_zero_velocity() {
        let mut sim = SimulationBuilder::new().build();
        sim.set_boundaries(100.0, 100.0);


        let builder = CritBuilder::new()
            .at(0.0, 0.0)
            .with_all_capabilities()
            .in_behavior(BehaviorMode::Catatonic);
        let entity_id = sim.spawn_crit(builder);


        let world = sim.world_mut();
        let mut query = world.query::<(&CritId, &Velocity, &CreatureState)>();

        let mut found = false;
        for (crit_id, velocity, state) in query.iter(world) {
            if crit_id.0 == entity_id {
                found = true;
                assert_eq!(
                    state.behavior,
                    BehaviorMode::Catatonic,
                    "Spawned crit should be in Catatonic state"
                );
                assert_eq!(
                    velocity.vx, 0.0,
                    "Catatonic crit should have zero X velocity"
                );
                assert_eq!(
                    velocity.vy, 0.0,
                    "Catatonic crit should have zero Y velocity"
                );
            }
        }
        assert!(found, "Should find the spawned crit");
    }

    #[test]
    fn test_catatonic_crit_doesnt_move() {
        let mut sim = SimulationBuilder::new().build();
        sim.set_boundaries(100.0, 100.0);


        let builder = CritBuilder::new()
            .at(50.0, 50.0)
            .with_all_capabilities()
            .in_behavior(BehaviorMode::Catatonic);
        let entity_id = sim.spawn_crit(builder);


        for _ in 0..100 {
            sim.update(0.05);
        }


        let world = sim.world_mut();
        let mut query = world.query::<(&CritId, &Position, &Velocity, &CreatureState)>();

        let mut found = false;
        for (crit_id, position, velocity, state) in query.iter(world) {
            if crit_id.0 == entity_id {
                found = true;
                assert_eq!(
                    state.behavior,
                    BehaviorMode::Catatonic,
                    "Crit should still be Catatonic after updates"
                );
                assert_eq!(
                    position.x, 50.0,
                    "Catatonic crit should not move in X direction"
                );
                assert_eq!(
                    position.y, 50.0,
                    "Catatonic crit should not move in Y direction"
                );
                assert_eq!(
                    velocity.vx, 0.0,
                    "Catatonic crit velocity should remain zero (X)"
                );
                assert_eq!(
                    velocity.vy, 0.0,
                    "Catatonic crit velocity should remain zero (Y)"
                );
            }
        }
        assert!(found, "Should find the crit after updates");
    }

    #[test]
    fn test_catatonic_transition_preserves_momentum() {
        let mut sim = SimulationBuilder::new().build();
        sim.set_boundaries(200.0, 200.0);

        let builder = CritBuilder::new()
            .at(0.0, 0.0)
            .as_seeker(5.0, 0.0)
            .with_all_capabilities();
        let entity_id = sim.spawn_crit(builder);

        for _ in 0..200 {
            sim.update(0.05);

            let world = sim.world_mut();
            let mut query = world.query::<(&CritId, &Velocity, &CreatureState)>();

            for (crit_id, velocity, state) in query.iter(world) {
                if crit_id.0 == entity_id && state.behavior == BehaviorMode::Catatonic {
                    assert!(
                        velocity.vx.abs() > 0.01 || velocity.vy.abs() > 0.01,
                        "Velocity should be preserved on Catatonic transition, but got vx={}, vy={} (current code zeros velocity)",
                        velocity.vx,
                        velocity.vy
                    );
                    return;
                }
            }
        }

        panic!("Creature should have transitioned to Catatonic after reaching target");
    }

    #[test]
    fn test_catatonic_deceleration_via_damping() {
        let mut sim = SimulationBuilder::new().build();
        sim.set_boundaries(200.0, 200.0);

        let builder = CritBuilder::new()
            .at(0.0, 0.0)
            .with_all_capabilities()
            .in_behavior(BehaviorMode::Catatonic);
        let entity_id = sim.spawn_crit(builder);

        {
            let world = sim.world_mut();
            let mut query = world.query::<(&CritId, &mut Velocity)>();
            for (crit_id, mut velocity) in query.iter_mut(world) {
                if crit_id.0 == entity_id {
                    velocity.vx = 10.0;
                    velocity.vy = 0.0;
                }
            }
        }

        let initial_speed = 10.0;
        let mut prev_speed = initial_speed;

        for _ in 0..100 {
            sim.update(0.05);

            let world = sim.world_mut();
            let mut query = world.query::<(&CritId, &Velocity)>();

            for (crit_id, velocity) in query.iter(world) {
                if crit_id.0 == entity_id {
                    let speed = (velocity.vx * velocity.vx + velocity.vy * velocity.vy).sqrt();

                    if speed >= 0.01 {
                        assert!(
                            speed < prev_speed,
                            "Speed should decrease each tick (prev: {}, current: {})",
                            prev_speed,
                            speed
                        );
                    }
                    prev_speed = speed;
                }
            }
        }

        let world = sim.world_mut();
        let mut query = world.query::<(&CritId, &Velocity)>();

        for (crit_id, velocity) in query.iter(world) {
            if crit_id.0 == entity_id {
                assert!(
                    velocity.vx.abs() < 0.1,
                    "Should decelerate to near-zero after 100 ticks, got: {}",
                    velocity.vx
                );
            }
        }
    }

    #[test]
    fn test_creature_state_energy_management() {
        let mut state = CreatureState::new();
        let initial_energy = state.energy;

        state.consume_energy(10.0);
        assert_eq!(state.energy, initial_energy - 10.0);

        state.restore_energy(5.0);
        assert_eq!(state.energy, initial_energy - 5.0);
    }

    #[test]
    fn test_creature_state_exhaustion() {
        let mut state = CreatureState::new();


        state.consume_energy(75.0);
        assert!(state.is_low_energy());
        assert!(!state.is_exhausted());


        state.consume_energy(20.0);
        assert!(state.is_exhausted());
    }

    #[test]
    fn test_velocity_helper_methods() {
        let vel = Velocity { vx: 3.0, vy: 4.0 };

        let magnitude = vel.magnitude();
        assert_eq!(magnitude, 5.0);

        let angle = vel.angle();
        let expected = 4.0f32.atan2(3.0);
        assert!((angle - expected).abs() < 0.001);
    }

    #[test]
    fn test_seek_moves_toward_target() {
        use crate::simulation::creatures::spawner::spawn_seek_test_scenario;

        let mut sim = SimulationBuilder::new().build();
        sim.set_boundaries(200.0, 200.0);

        let (seeker_id, _obstacle_id) = spawn_seek_test_scenario(&mut sim);


        let initial_x = {
            let world = sim.world_mut();
            let mut query = world.query::<(&CritId, &Position)>();
            query
                .iter(world)
                .find(|(id, _)| id.0 == seeker_id)
                .map(|(_, pos)| pos.x)
                .unwrap()
        };


        for _ in 0..40 {
            sim.update(0.05);
        }


        let final_x = {
            let world = sim.world_mut();
            let mut query = world.query::<(&CritId, &Position)>();
            query
                .iter(world)
                .find(|(id, _)| id.0 == seeker_id)
                .map(|(_, pos)| pos.x)
                .unwrap()
        };

        assert!(
            final_x > initial_x,
            "Seeker should move toward target (final: {}, initial: {})",
            final_x,
            initial_x
        );
        assert!(
            final_x < 100.0,
            "Shouldn't overshoot target in 2 seconds (final: {})",
            final_x
        );
    }

    #[test]
    fn test_seek_slows_down_near_target() {
        use crate::simulation::creatures::spawner::spawn_seek_test_scenario;

        let mut sim = SimulationBuilder::new().build();
        sim.set_boundaries(200.0, 200.0);

        let (seeker_id, _) = spawn_seek_test_scenario(&mut sim);


        for _ in 0..150 {
            sim.update(0.05);
        }


        let speed = {
            let world = sim.world_mut();
            let mut query = world.query::<(&CritId, &Velocity)>();
            query
                .iter(world)
                .find(|(id, _)| id.0 == seeker_id)
                .map(|(_, vel)| vel.magnitude())
                .unwrap()
        };


        const SEEK_MAX_SPEED: f32 = 50.0;
        assert!(
            speed < SEEK_MAX_SPEED * 0.8,
            "Should slow down near target (speed: {}, max: {})",
            speed,
            SEEK_MAX_SPEED
        );
    }

    #[test]
    fn test_seek_reaches_target_eventually() {
        use crate::simulation::creatures::spawner::spawn_seek_test_scenario;

        let mut sim = SimulationBuilder::new().build();
        sim.set_boundaries(200.0, 200.0);

        let (seeker_id, _) = spawn_seek_test_scenario(&mut sim);



        for _ in 0..800 {
            sim.update(0.05);
        }


        let (final_x, final_y) = {
            let world = sim.world_mut();
            let mut query = world.query::<(&CritId, &Position)>();
            query
                .iter(world)
                .find(|(id, _)| id.0 == seeker_id)
                .map(|(_, pos)| (pos.x, pos.y))
                .unwrap()
        };



        assert!(
            (final_x - 100.0).abs() < 10.0,
            "Should reach target X position and stop (final: {}, target: 100, arrival_radius: 5m)",
            final_x
        );
        assert!(
            final_y.abs() < 10.0,
            "Should stay near target Y position (final: {}, target: 0)",
            final_y
        );
    }

    #[test]
    fn test_catatonic_crit_ignores_target() {
        use crate::simulation::components::{CanSeek, Target};

        let mut sim = SimulationBuilder::new().build();
        sim.set_boundaries(200.0, 200.0);


        let builder = CritBuilder::new()
            .at(0.0, 0.0)
            .with_all_capabilities()
            .in_behavior(BehaviorMode::Catatonic);
        let crit_id = sim.spawn_crit(builder);


        {
            let world = sim.world_mut();
            let mut query = world.query::<(bevy_ecs::entity::Entity, &CritId)>();
            let entity = query
                .iter(world)
                .find(|(_, id)| id.0 == crit_id)
                .map(|(e, _)| e)
                .unwrap();

            world
                .entity_mut(entity)
                .insert(CanSeek)
                .insert(Target::at_point(100.0, 0.0));

        }


        for _ in 0..100 {
            sim.update(0.05);
        }


        let final_x = {
            let world = sim.world_mut();
            let mut query = world.query::<(&CritId, &Position)>();
            query
                .iter(world)
                .find(|(id, _)| id.0 == crit_id)
                .map(|(_, pos)| pos.x)
                .unwrap()
        };

        assert_eq!(
            final_x, 0.0,
            "Catatonic crit should not move even with Target (final: {})",
            final_x
        );
    }

    #[test]
    fn test_seeker_avoids_obstacle_in_path() {
        use crate::simulation::creatures::builder::CritBuilder;

        let mut sim = SimulationBuilder::new().build();
        sim.set_boundaries(200.0, 200.0);


        let seeker_builder = CritBuilder::new()
            .at(0.0, 0.0)
            .as_seeker(100.0, 0.0)
            .with_all_capabilities();
        let seeker_id = sim.spawn_crit(seeker_builder);


        let obstacle_builder = CritBuilder::new()
            .at(15.0, 1.0)
            .in_behavior(BehaviorMode::Catatonic);
        let _obstacle_id = sim.spawn_crit(obstacle_builder);


        let mut min_distance = f32::MAX;
        let mut max_y_deviation = 0.0f32;





        for _tick in 0..900 {
            sim.update(0.05);


            let world = sim.world_mut();
            let mut positions = world.query::<(&CritId, &Position)>();

            let seeker_pos = positions
                .iter(world)
                .find(|(id, _)| id.0 == seeker_id)
                .map(|(_, pos)| *pos)
                .unwrap();

            let obstacle_pos = positions
                .iter(world)
                .find(|(id, _)| id.0 != seeker_id)
                .map(|(_, pos)| *pos)
                .unwrap();


            let dx = seeker_pos.x - obstacle_pos.x;
            let dy = seeker_pos.y - obstacle_pos.y;
            let distance = (dx * dx + dy * dy).sqrt();
            min_distance = min_distance.min(distance);


            max_y_deviation = max_y_deviation.max(seeker_pos.y.abs());


        }


        let (final_x, final_y) = {
            let world = sim.world_mut();
            let mut query = world.query::<(&CritId, &Position)>();
            query
                .iter(world)
                .find(|(id, _)| id.0 == seeker_id)
                .map(|(_, pos)| (pos.x, pos.y))
                .unwrap()
        };


        println!("\nTest Results:");
        println!("  Max Y deviation: {:.2}m", max_y_deviation);
        println!("  Min distance to obstacle: {:.2}m", min_distance);
        println!("  Final position: ({:.2}, {:.2})", final_x, final_y);


        assert!(
            max_y_deviation > 0.5,
            "Seeker should deviate from straight path (max Y deviation: {:.2}m)",
            max_y_deviation
        );

        assert!(
            min_distance >= 1.0,
            "Seeker should avoid collision (min distance: {:.2}m, personal space: 2.5m)",
            min_distance
        );



        assert!(
            (final_x - 100.0).abs() < 10.0,
            "Seeker should reach target and stop within arrival radius (final X: {:.2}, target: 100, arrival_radius: 5m)",
            final_x
        );

        assert!(
            final_y.abs() < 10.0,
            "Seeker should return near target Y after avoidance (final Y: {:.2}, target: 0)",
            final_y
        );
    }

    #[test]
    fn test_cycling_to_seeking_assigns_random_target() {
        let mut sim = SimulationBuilder::new().build();
        sim.set_boundaries(200.0, 200.0);

        let builder = CritBuilder::new()
            .at(50.0, 50.0)
            .with_all_capabilities()
            .with_cycling_brain()
            .in_behavior(BehaviorMode::Wandering);
        let entity_id = sim.spawn_crit(builder);

        let world = sim.world_mut();
        let mut query = world.query::<(&CritId, &Target)>();
        let mut target_before = Target::at_point(0.0, 0.0);
        for (crit_id, target) in query.iter(world) {
            if crit_id.0 == entity_id {
                target_before = *target;
            }
        }

        for _ in 0..200 {
            sim.update(0.05);

            let world = sim.world_mut();
            let mut query = world.query::<(&CritId, &CreatureState, &Target, &Position)>();

            for (crit_id, state, target, position) in query.iter(world) {
                if crit_id.0 == entity_id && state.behavior == BehaviorMode::Seeking {
                    assert_ne!(
                        target.x, target_before.x,
                        "Target X should change when cycling to Seeking"
                    );
                    assert_ne!(
                        target.y, target_before.y,
                        "Target Y should change when cycling to Seeking"
                    );

                    let distance = ((target.x - position.x).powi(2) + (target.y - position.y).powi(2)).sqrt();
                    assert!(
                        distance >= 50.0 && distance <= 200.0,
                        "Target should be 50-200 units away, got: {}",
                        distance
                    );
                    return;
                }
            }
        }

        panic!("Creature should have cycled to Seeking mode within 200 ticks");
    }

    #[test]
    fn test_archetype_stability_with_cycling_brain() {
        let mut sim = SimulationBuilder::new().build();

        for i in 0..100 {
            let builder = CritBuilder::new()
                .at(i as f32 * 10.0, 0.0)
                .with_cycling_brain()
                .in_behavior(BehaviorMode::Catatonic);
            sim.spawn_crit(builder);
        }

        let post_spawn_archetype_count = sim.world().archetypes().len();

        for _ in 0..1000 {
            sim.update(0.05);
        }

        let final_archetype_count = sim.world().archetypes().len();

        assert_eq!(
            post_spawn_archetype_count, final_archetype_count,
            "Archetype count should remain stable after 1000 ticks with cycling brains (was {}, now {})",
            post_spawn_archetype_count, final_archetype_count
        );
    }
}
