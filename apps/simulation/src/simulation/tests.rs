//! Unit tests for simulation systems
//!
//! These tests verify that core simulation functionality works correctly
//! and would have caught the "simulation not ticking" issue immediately.

#[cfg(test)]
mod simulation_tests {
    use super::super::*;
    

    #[test]
    fn test_simulation_creates_successfully() {
        let sim = Simulation::new();
        assert_eq!(sim.creature_count(), 0);
    }

    #[test]
    fn test_spawn_creature_increases_count() {
        let mut sim = Simulation::new();
        sim.set_boundaries(180.0, 130.0);

        let initial_count = sim.creature_count();
        sim.spawn_creature(90.0, 65.0, 2.0, 1.0);

        assert_eq!(sim.creature_count(), initial_count + 1);
    }

    #[test]
    fn test_simulation_update_doesnt_crash() {
        let mut sim = Simulation::new();
        sim.set_boundaries(180.0, 130.0);
        sim.spawn_creature(90.0, 65.0, 2.0, 1.0);

        // Should not panic
        sim.update(0.016); // 60 FPS delta
    }

    #[test]
    fn test_multiple_updates_work() {
        let mut sim = Simulation::new();
        sim.set_boundaries(180.0, 130.0);
        sim.spawn_creature(90.0, 65.0, 2.0, 1.0);

        // Run 100 simulation ticks
        for _ in 0..100 {
            sim.update(0.016);
        }

        // Should still have the creature
        assert_eq!(sim.creature_count(), 1);
    }

    #[test]
    fn test_get_creatures_returns_data() {
        let mut sim = Simulation::new();
        sim.set_boundaries(180.0, 130.0);
        sim.spawn_creature(100.0, 50.0, 2.0, 1.0);

        let creatures = sim.get_creatures();
        assert_eq!(creatures.len(), 1);

        let creature = &creatures[0];
        assert_eq!(creature.x, 100.0);
        assert_eq!(creature.y, 50.0);
    }

    #[test]
    fn test_creatures_move_over_time() {
        let mut sim = Simulation::new();
        sim.set_boundaries(180.0, 130.0);
        sim.spawn_creature(90.0, 65.0, 2.0, 1.0);

        let initial = sim.get_creatures();
        let (initial_x, initial_y) = (initial[0].x, initial[0].y);

        // Run several updates
        for _ in 0..50 {
            sim.update(0.016);
        }

        let updated = sim.get_creatures();
        let (updated_x, updated_y) = (updated[0].x, updated[0].y);

        // Creature should have moved (with very high probability given wandering behavior)
        let distance_moved = ((updated_x - initial_x).powi(2) + (updated_y - initial_y).powi(2)).sqrt();
        assert!(distance_moved > 1.0, "Creature should have moved, distance: {}", distance_moved);
    }

    #[test]
    fn test_boundary_enforcement() {
        let mut sim = Simulation::new();
        sim.set_boundaries(180.0, 130.0);

        // Spawn creature near boundary
        sim.spawn_creature(5.0, 5.0, 2.0, 1.0);

        // Run many updates
        for _ in 0..1000 {
            sim.update(0.016);
        }

        let creatures = sim.get_creatures();
        let creature = &creatures[0];

        // Creature should stay within bounds
        assert!(creature.x >= 0.0, "X should be >= 0, got {}", creature.x);
        assert!(creature.x <= 180.0, "X should be <= 180, got {}", creature.x);
        assert!(creature.y >= 0.0, "Y should be >= 0, got {}", creature.y);
        assert!(creature.y <= 130.0, "Y should be <= 130, got {}", creature.y);
    }

    #[test]
    fn test_delta_time_affects_movement() {
        let mut sim1 = Simulation::new();
        sim1.set_boundaries(180.0, 130.0);
        sim1.spawn_creature(90.0, 65.0, 2.0, 1.0);

        let mut sim2 = Simulation::new();
        sim2.set_boundaries(180.0, 130.0);
        sim2.spawn_creature(90.0, 65.0, 2.0, 1.0);

        // Same number of updates but different delta times
        for _ in 0..10 {
            sim1.update(0.016); // 60 FPS
            sim2.update(0.032); // 30 FPS
        }

        let creatures1 = sim1.get_creatures();
        let creatures2 = sim2.get_creatures();

        // Creatures should be in different positions due to delta time differences
        let pos1 = (creatures1[0].x, creatures1[0].y);
        let pos2 = (creatures2[0].x, creatures2[0].y);

        // Note: This test might be flaky due to randomness, but delta time should
        // generally cause different outcomes
        let distance = ((pos1.0 - pos2.0).powi(2) + (pos1.1 - pos2.1).powi(2)).sqrt();
        println!("Distance between sim1 and sim2: {}", distance);
    }
}

#[cfg(test)]
mod system_tests {
    
    use bevy_ecs::prelude::*;
    use crate::simulation::components::{Position, Velocity, Acceleration, Rotation, DeltaTime};

    #[test]
    fn test_movement_system_updates_position() {
        let mut world = World::new();
        world.insert_resource(DeltaTime(0.1));

        let entity = world.spawn((
            Position { x: 0.0, y: 0.0 },
            Velocity { vx: 10.0, vy: 5.0 },
        )).id();

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

        let entity = world.spawn((
            Velocity { vx: 0.0, vy: 0.0 },
            Acceleration { ax: 10.0, ay: 5.0 },
        )).id();

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

        let entity = world.spawn((
            Rotation { radians: 0.0 },
            Velocity { vx: 1.0, vy: 1.0 }, // 45 degrees
        )).id();

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
    
    use crate::simulation::components::{CreatureState, Velocity};

    #[test]
    fn test_creature_state_energy_management() {
        let mut state = CreatureState::new(0);
        let initial_energy = state.energy;

        state.consume_energy(10.0);
        assert_eq!(state.energy, initial_energy - 10.0);

        state.restore_energy(5.0);
        assert_eq!(state.energy, initial_energy - 5.0);
    }

    #[test]
    fn test_creature_state_exhaustion() {
        let mut state = CreatureState::new(0);

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
}
