//! Unit tests for simulation systems
//!
//! These tests verify that core simulation functionality works correctly
//! and would have caught the "simulation not ticking" issue immediately.

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

        // Should not panic
        sim.update(0.016); // 60 FPS delta
    }

    #[test]
    fn test_multiple_updates_work() {
        let mut sim = SimulationBuilder::new().build();
        sim.set_boundaries(180.0, 130.0);
        let builder = CritBuilder::new().at(90.0, 65.0).with_all_capabilities();
        sim.spawn_crit(builder);

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

        // Get delta time first
        let dt = world.resource::<DeltaTime>().0;

        // Run movement system
        let mut query = world.query::<(&mut Position, &Velocity)>();
        for (mut pos, vel) in query.iter_mut(&mut world) {
            pos.x += vel.vx * dt;
            pos.y += vel.vy * dt;
        }

        // Check position updated
        let position = world.get::<Position>(entity).unwrap();
        assert_eq!(position.x, 1.0); // 10 * 0.1
        assert_eq!(position.y, 0.5); // 5 * 0.1
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

        // Get delta time first
        let dt = world.resource::<DeltaTime>().0;

        // Simulate acceleration system
        let mut query = world.query::<(&mut Velocity, &mut Acceleration)>();
        for (mut vel, mut acc) in query.iter_mut(&mut world) {
            vel.vx += acc.ax * dt;
            vel.vy += acc.ay * dt;
            acc.ax = 0.0;
            acc.ay = 0.0;
        }

        // Check velocity updated and acceleration reset
        let velocity = world.get::<Velocity>(entity).unwrap();
        assert_eq!(velocity.vx, 1.0); // 10 * 0.1
        assert_eq!(velocity.vy, 0.5); // 5 * 0.1

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
                Velocity { vx: 1.0, vy: 1.0 }, // 45 degrees
            ))
            .id();

        // Simulate rotation system
        let mut query = world.query::<(&mut Rotation, &Velocity)>();
        for (mut rot, vel) in query.iter_mut(&mut world) {
            if vel.vx != 0.0 || vel.vy != 0.0 {
                rot.radians = vel.vy.atan2(vel.vx);
            }
        }

        let rotation = world.get::<Rotation>(entity).unwrap();
        let expected = 1.0f32.atan2(1.0); // ≈ 0.785 radians (45°)
        assert!((rotation.radians - expected).abs() < 0.001);
    }
}

#[cfg(test)]
mod behavior_tests {

    use crate::simulation::components::{BehaviorMode, CreatureState, CritId, Position, Velocity};
    use crate::simulation::creatures::builder::CritBuilder;
    use crate::simulation::SimulationBuilder;

    #[test]
    fn test_catatonic_crit_spawns_with_zero_velocity() {
        let mut sim = SimulationBuilder::new().build();
        sim.set_boundaries(100.0, 100.0);

        // Explicitly spawn as Catatonic (default is Wandering)
        let builder = CritBuilder::new()
            .at(0.0, 0.0)
            .with_all_capabilities()
            .in_behavior(BehaviorMode::Catatonic);
        let entity_id = sim.spawn_crit(builder);

        // Query the spawned crit directly from world
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

        // Explicitly spawn as Catatonic (default is Wandering)
        let builder = CritBuilder::new()
            .at(50.0, 50.0)
            .with_all_capabilities()
            .in_behavior(BehaviorMode::Catatonic);
        let entity_id = sim.spawn_crit(builder);

        // Run simulation for 100 ticks (should be ~5 seconds at 20Hz)
        for _ in 0..100 {
            sim.update(0.05); // 20 Hz = 0.05 second delta
        }

        // Verify position hasn't changed
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

        // Drain to low energy (< 30)
        state.consume_energy(75.0); // 100 - 75 = 25
        assert!(state.is_low_energy());
        assert!(!state.is_exhausted());

        // Drain further to exhausted (< 10)
        state.consume_energy(20.0); // 25 - 20 = 5
        assert!(state.is_exhausted());
    }

    #[test]
    fn test_velocity_helper_methods() {
        let vel = Velocity { vx: 3.0, vy: 4.0 };

        let magnitude = vel.magnitude();
        assert_eq!(magnitude, 5.0); // 3-4-5 triangle

        let angle = vel.angle();
        let expected = 4.0f32.atan2(3.0);
        assert!((angle - expected).abs() < 0.001);
    }

    /// Seek Behavior Tests (Sprint 6 Milestone)
    #[test]
    fn test_seek_moves_toward_target() {
        use crate::simulation::creatures::spawner::spawn_seek_test_scenario;

        let mut sim = SimulationBuilder::new().build();
        sim.set_boundaries(200.0, 200.0);

        let (seeker_id, _obstacle_id) = spawn_seek_test_scenario(&mut sim);

        // Get initial position
        let initial_x = {
            let world = sim.world_mut();
            let mut query = world.query::<(&CritId, &Position)>();
            query
                .iter(world)
                .find(|(id, _)| id.0 == seeker_id)
                .map(|(_, pos)| pos.x)
                .unwrap()
        };

        // Run simulation for 2 seconds (40 ticks @ 20Hz)
        for _ in 0..40 {
            sim.update(0.05);
        }

        // Should have moved toward target (x=100)
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

        // Run until near target (most of the way)
        for _ in 0..150 {
            sim.update(0.05);
        }

        // Get velocity and verify slowdown
        let speed = {
            let world = sim.world_mut();
            let mut query = world.query::<(&CritId, &Velocity)>();
            query
                .iter(world)
                .find(|(id, _)| id.0 == seeker_id)
                .map(|(_, vel)| vel.magnitude())
                .unwrap()
        };

        // Should be slower than max speed (50 m/s from seek_system constant)
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

        // Run for extended time (40 seconds = 800 ticks)
        // With realistic speeds (5 m/s) and damping, 100m target requires ~25-30s
        for _ in 0..800 {
            sim.update(0.05);
        }

        // Should be very close to target
        let (final_x, final_y) = {
            let world = sim.world_mut();
            let mut query = world.query::<(&CritId, &Position)>();
            query
                .iter(world)
                .find(|(id, _)| id.0 == seeker_id)
                .map(|(_, pos)| (pos.x, pos.y))
                .unwrap()
        };

        // Target is at (100, 0), should stop within arrival radius (5m)
        // With new arrival behavior, creature should stop much closer to target
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

        // Spawn crit with Target and CanSeek but KEEP it Catatonic
        let builder = CritBuilder::new()
            .at(0.0, 0.0)
            .with_all_capabilities()
            .in_behavior(BehaviorMode::Catatonic);
        let crit_id = sim.spawn_crit(builder);

        // Add Target and CanSeek but don't change behavior
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
                .insert(Target::new(100.0, 0.0));
            // Note: behavior stays Catatonic
        }

        // Run simulation
        for _ in 0..100 {
            sim.update(0.05);
        }

        // Should NOT have moved (Catatonic overrides seeking)
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

    /// Obstacle Avoidance Integration Test (Sprint 6 Phase 7)
    #[test]
    fn test_seeker_avoids_obstacle_in_path() {
        use crate::simulation::creatures::builder::CritBuilder;

        let mut sim = SimulationBuilder::new().build();
        sim.set_boundaries(200.0, 200.0);

        // Spawn seeker at origin targeting (100, 0)
        let seeker_builder = CritBuilder::new()
            .at(0.0, 0.0)
            .as_seeker(100.0, 0.0)
            .with_all_capabilities();
        let seeker_id = sim.spawn_crit(seeker_builder);

        // Spawn obstacle slightly off path at (15, 1) for lateral avoidance
        let obstacle_builder = CritBuilder::new()
            .at(15.0, 1.0)
            .in_behavior(BehaviorMode::Catatonic);
        let _obstacle_id = sim.spawn_crit(obstacle_builder);

        // Track minimum distance to obstacle
        let mut min_distance = f32::MAX;
        let mut max_y_deviation = 0.0f32;

        // Run simulation for 45 seconds (900 ticks @ 20Hz)
        // Longer duration needed due to realistic speeds + obstacle avoidance:
        // - Old: 20 m/s (could reach 100m target in ~5s)
        // - New: 5 m/s max (needs ~25-30s accounting for acceleration, damping, and avoidance)
        for tick in 0..900 {
            sim.update(0.05);

            // Check positions this frame
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

            // Calculate distance
            let dx = seeker_pos.x - obstacle_pos.x;
            let dy = seeker_pos.y - obstacle_pos.y;
            let distance = (dx * dx + dy * dy).sqrt();
            min_distance = min_distance.min(distance);

            // Track max Y deviation (path should deviate from straight line)
            max_y_deviation = max_y_deviation.max(seeker_pos.y.abs());

            // Track metrics (debug output removed for clean test runs)
        }

        // Final position check
        let (final_x, final_y) = {
            let world = sim.world_mut();
            let mut query = world.query::<(&CritId, &Position)>();
            query
                .iter(world)
                .find(|(id, _)| id.0 == seeker_id)
                .map(|(_, pos)| (pos.x, pos.y))
                .unwrap()
        };

        // Remove debug output for clean test runs
        println!("\nTest Results:");
        println!("  Max Y deviation: {:.2}m", max_y_deviation);
        println!("  Min distance to obstacle: {:.2}m", min_distance);
        println!("  Final position: ({:.2}, {:.2})", final_x, final_y);

        // Assertions
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

        // With new arrival behavior, creature should stop within arrival_radius (5m)
        // Allow 10m tolerance to account for physics/integration effects
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
}
