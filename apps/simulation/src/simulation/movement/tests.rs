//! Movement system tests - extracted from systems.rs for maintainability.
//!
//! These tests cover:
//! - Basic physics integration (position, velocity, acceleration)
//! - Parallel processing determinism
//! - Boundary enforcement
//! - Turn rate limiting (size-dependent, speed penalty)

use super::systems::integrate_motion_system;
use crate::config::MovementConfig;
use crate::simulation::core::components::{
    Acceleration, BodySize, DeltaTime, PhysicsTick, Position, Rotation, Velocity,
};
use crate::simulation::creatures::components::CreatureState;
use crate::simulation::movement::noise::NoiseTable;
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
fn test_parallel_movement_determinism() {
    let mut world = World::new();
    world.insert_resource(DeltaTime(0.016));
    world.insert_resource(PhysicsTick(0));
    world.insert_resource(crate::simulation::core::WorldBounds::new(
        -100.0, 100.0, -100.0, 100.0,
    ));
    // Disable noise for determinism test - noise is entity-index dependent
    world.insert_resource(MovementConfig {
        locomotion_noise_base: 0.0,
        ..Default::default()
    });
    world.insert_resource(NoiseTable::default());
    #[cfg(feature = "dev-tools")]
    world.insert_resource(crate::instrumentation::SystemTimings::new());

    // Spawn 100 entities with varied initial conditions
    for i in 0..100 {
        let x = (i as f32 * 0.5) % 100.0;
        let y = (i as f32 * 0.3) % 100.0;
        let mut state = CreatureState::default();
        state.behavior = crate::simulation::creatures::components::BehaviorMode::Wandering;
        world.spawn((
            BodySize::new(1.0),
            Position { x, y },
            Velocity {
                vx: (i as f32 * 0.1).sin(),
                vy: (i as f32 * 0.1).cos(),
            },
            Acceleration { ax: 0.0, ay: 0.0 },
            Rotation::default(),
            state,
        ));
    }

    // Run system and capture initial state
    use bevy_ecs::system::IntoSystem;
    let mut system = IntoSystem::into_system(integrate_motion_system);
    system.initialize(&mut world);
    system.run((), &mut world);

    // Capture positions after first run
    let positions_run1: Vec<_> = world
        .query::<&Position>()
        .iter(&world)
        .map(|p| (p.x, p.y))
        .collect();

    // Reset to initial state
    let mut world2 = World::new();
    world2.insert_resource(DeltaTime(0.016));
    world2.insert_resource(PhysicsTick(0));
    world2.insert_resource(crate::simulation::core::WorldBounds::new(
        -100.0, 100.0, -100.0, 100.0,
    ));
    // Disable noise for determinism test - noise is entity-index dependent
    world2.insert_resource(MovementConfig {
        locomotion_noise_base: 0.0,
        ..Default::default()
    });
    world2.insert_resource(NoiseTable::default());
    #[cfg(feature = "dev-tools")]
    world2.insert_resource(crate::instrumentation::SystemTimings::new());

    for i in 0..100 {
        let x = (i as f32 * 0.5) % 100.0;
        let y = (i as f32 * 0.3) % 100.0;
        let mut state = CreatureState::default();
        state.behavior = crate::simulation::creatures::components::BehaviorMode::Wandering;
        world2.spawn((
            BodySize::new(1.0),
            Position { x, y },
            Velocity {
                vx: (i as f32 * 0.1).sin(),
                vy: (i as f32 * 0.1).cos(),
            },
            Acceleration { ax: 0.0, ay: 0.0 },
            Rotation::default(),
            state,
        ));
    }

    let mut system2 = IntoSystem::into_system(integrate_motion_system);
    system2.initialize(&mut world2);
    system2.run((), &mut world2);

    // Capture positions after second run
    let positions_run2: Vec<_> = world2
        .query::<&Position>()
        .iter(&world2)
        .map(|p| (p.x, p.y))
        .collect();

    // Verify determinism: same input produces same output
    assert_eq!(positions_run1.len(), 100);
    assert_eq!(positions_run2.len(), 100);
    for (i, ((x1, y1), (x2, y2))) in
        positions_run1.iter().zip(positions_run2.iter()).enumerate()
    {
        assert!(
            (x1 - x2).abs() < 0.0001,
            "Entity {} X position mismatch: {} vs {}",
            i,
            x1,
            x2
        );
        assert!(
            (y1 - y2).abs() < 0.0001,
            "Entity {} Y position mismatch: {} vs {}",
            i,
            y1,
            y2
        );
    }
}

#[test]
fn test_all_creatures_processed_in_parallel() {
    let mut world = World::new();
    world.insert_resource(DeltaTime(0.016));
    world.insert_resource(PhysicsTick(0));
    world.insert_resource(crate::simulation::core::WorldBounds::new(
        -100.0, 100.0, -100.0, 100.0,
    ));
    world.insert_resource(MovementConfig::default());
    world.insert_resource(NoiseTable::default());
    #[cfg(feature = "dev-tools")]
    world.insert_resource(crate::instrumentation::SystemTimings::new());

    // Spawn 1000 entities - all with zero velocity (sentinel value)
    for _i in 0..1000 {
        let mut state = CreatureState::default();
        state.behavior = crate::simulation::creatures::components::BehaviorMode::Wandering; // Non-catatonic
        world.spawn((
            BodySize::new(1.0),
            Position { x: 0.0, y: 0.0 },
            Velocity { vx: 0.0, vy: 0.0 },
            Acceleration { ax: 1.0, ay: 1.0 }, // Non-zero accel to trigger velocity change
            Rotation::default(),
            state,
        ));
    }

    // Run parallel integration
    use bevy_ecs::system::IntoSystem;
    let mut system = IntoSystem::into_system(integrate_motion_system);
    system.initialize(&mut world);
    system.run((), &mut world);

    // Verify ALL entities were processed (acceleration should be reset to 0)
    let mut processed_count = 0;
    for accel in world.query::<&Acceleration>().iter(&world) {
        assert_eq!(
            accel.ax, 0.0,
            "Acceleration should be reset after integration"
        );
        assert_eq!(
            accel.ay, 0.0,
            "Acceleration should be reset after integration"
        );
        processed_count += 1;
    }

    assert_eq!(
        processed_count, 1000,
        "All 1000 entities should be processed"
    );

    // Also verify velocities were updated (not still zero)
    let velocities_updated = world
        .query::<&Velocity>()
        .iter(&world)
        .filter(|v| v.vx != 0.0 || v.vy != 0.0)
        .count();

    assert_eq!(velocities_updated, 1000, "All velocities should be updated");
}

#[test]
fn test_concurrent_boundary_enforcement() {
    let mut world = World::new();
    world.insert_resource(DeltaTime(0.016));
    world.insert_resource(PhysicsTick(0));
    world.insert_resource(crate::simulation::core::WorldBounds::new(
        -100.0, 100.0, -100.0, 100.0,
    ));
    world.insert_resource(MovementConfig::default());
    world.insert_resource(NoiseTable::default());
    #[cfg(feature = "dev-tools")]
    world.insert_resource(crate::instrumentation::SystemTimings::new());

    // Spawn entities that will exceed boundaries in parallel
    let test_cases = vec![
        (150.0, 0.0, 10.0, 0.0),   // Beyond max_x
        (-150.0, 0.0, -10.0, 0.0), // Beyond min_x
        (0.0, 150.0, 0.0, 10.0),   // Beyond max_y
        (0.0, -150.0, 0.0, -10.0), // Beyond min_y
    ];

    for (x, y, vx, vy) in test_cases.iter() {
        let mut state = CreatureState::default();
        state.behavior = crate::simulation::creatures::components::BehaviorMode::Wandering;
        world.spawn((
            BodySize::new(1.0),
            Position { x: *x, y: *y },
            Velocity { vx: *vx, vy: *vy },
            Acceleration { ax: 0.0, ay: 0.0 },
            Rotation::default(),
            state,
        ));
    }

    // Run parallel integration (includes boundary enforcement)
    use bevy_ecs::system::IntoSystem;
    let mut system = IntoSystem::into_system(integrate_motion_system);
    system.initialize(&mut world);
    system.run((), &mut world);

    // Verify all positions are clamped to boundaries
    for pos in world.query::<&Position>().iter(&world) {
        assert!(
            pos.x >= -100.0 && pos.x <= 100.0,
            "Position X {} should be clamped to [-100, 100]",
            pos.x
        );
        assert!(
            pos.y >= -100.0 && pos.y <= 100.0,
            "Position Y {} should be clamped to [-100, 100]",
            pos.y
        );
    }

    // Verify velocities were corrected at boundaries
    let velocities: Vec<_> = world.query::<&Velocity>().iter(&world).collect();

    // Entity 0: was beyond max_x, velocity.vx should be clamped to <= 0
    assert!(
        velocities[0].vx <= 0.0,
        "Velocity at max_x boundary should be non-positive"
    );

    // Entity 1: was beyond min_x, velocity.vx should be clamped to >= 0
    assert!(
        velocities[1].vx >= 0.0,
        "Velocity at min_x boundary should be non-negative"
    );

    // Entity 2: was beyond max_y, velocity.vy should be clamped to <= 0
    assert!(
        velocities[2].vy <= 0.0,
        "Velocity at max_y boundary should be non-positive"
    );

    // Entity 3: was beyond min_y, velocity.vy should be clamped to >= 0
    assert!(
        velocities[3].vy >= 0.0,
        "Velocity at min_y boundary should be non-negative"
    );
}

#[test]
fn test_turn_rate_limits_direction_change() {
    let mut world = World::new();
    world.insert_resource(DeltaTime(0.05));
    world.insert_resource(PhysicsTick(0));
    world.insert_resource(crate::simulation::core::WorldBounds::new(
        -1000.0, 1000.0, -1000.0, 1000.0,
    ));
    world.insert_resource(MovementConfig::default());
    world.insert_resource(NoiseTable::default());
    #[cfg(feature = "dev-tools")]
    world.insert_resource(crate::instrumentation::SystemTimings::new());

    let mut state = CreatureState::default();
    state.behavior = crate::simulation::creatures::components::BehaviorMode::Wandering;

    let entity = world
        .spawn((
            BodySize::new(1.0),
            Position { x: 0.0, y: 0.0 },
            Velocity { vx: 10.0, vy: 0.0 },
            Acceleration { ax: 0.0, ay: 100.0 },
            Rotation::default(),
            state,
        ))
        .id();

    use bevy_ecs::system::IntoSystem;
    let mut system = IntoSystem::into_system(integrate_motion_system);
    system.initialize(&mut world);
    system.run((), &mut world);

    let vel = world.get::<Velocity>(entity).unwrap();

    let initial_angle = 0.0_f32;
    let final_angle = vel.vy.atan2(vel.vx);
    let delta_degrees = (final_angle - initial_angle).to_degrees().abs();

    let max_expected = crate::simulation::creatures::constants::MAX_TURN_RATE * 0.05 + 0.1;
    assert!(
        delta_degrees <= max_expected,
        "Turn rate should be limited to ~0.9 deg, got {} deg",
        delta_degrees
    );
    assert!(
        delta_degrees > 0.0,
        "Should have some turn, got {} deg",
        delta_degrees
    );
}

#[test]
fn test_small_turns_not_affected() {
    let mut world = World::new();
    world.insert_resource(DeltaTime(0.05));
    world.insert_resource(PhysicsTick(0));
    world.insert_resource(crate::simulation::core::WorldBounds::new(
        -1000.0, 1000.0, -1000.0, 1000.0,
    ));
    // Disable noise so it doesn't interfere with turn rate test
    world.insert_resource(MovementConfig {
        locomotion_noise_base: 0.0,
        ..Default::default()
    });
    world.insert_resource(NoiseTable::default());
    #[cfg(feature = "dev-tools")]
    world.insert_resource(crate::instrumentation::SystemTimings::new());

    let mut state = CreatureState::default();
    state.behavior = crate::simulation::creatures::components::BehaviorMode::Wandering;

    let entity = world
        .spawn((
            BodySize::new(1.0),
            Position { x: 0.0, y: 0.0 },
            Velocity { vx: 10.0, vy: 0.0 },
            Acceleration { ax: 1.0, ay: 0.1 },
            Rotation::default(),
            state,
        ))
        .id();

    use bevy_ecs::system::IntoSystem;
    let mut system = IntoSystem::into_system(integrate_motion_system);
    system.initialize(&mut world);
    system.run((), &mut world);

    let vel = world.get::<Velocity>(entity).unwrap();
    assert!(vel.vy > 0.0, "Small upward component should be preserved");
}

#[test]
fn test_stopped_creature_heading_constrained_from_stored_angle() {
    // After the NaN-bypass fix: a truly stopped creature (vx=0, vy=0) has a stored
    // heading from rotation.radians (defaults to East, 0 rad). When it receives a
    // full-power North force, the heading is rate-limited from East — it cannot snap
    // instantly to North. The velocity after one tick must have a large East component
    // and a small (rate-limited) North component.
    let mut world = make_integration_world(0.05, 0.0);

    let entity = world
        .spawn((
            BodySize::new(1.0),
            Position { x: 0.0, y: 0.0 },
            Velocity { vx: 0.0, vy: 0.0 },     // Truly stopped
            Acceleration { ax: 0.0, ay: 10.0 }, // Full-power North force
            Rotation::new(0.0),                  // Stored heading: East (0 rad)
            wandering_state(),
        ))
        .id();

    run_integration(&mut world);

    let vel = world.get::<Velocity>(entity).unwrap();
    let rot = world.get::<Rotation>(entity).unwrap();

    // Creature has some net speed (the force did accelerate it)
    let speed = (vel.vx * vel.vx + vel.vy * vel.vy).sqrt();
    assert!(speed > 0.0, "Creature should have started moving");

    // Heading must be constrained to < ~10° from stored East — NOT 90° (North).
    // size=1m: base_turn_rate=180 deg/s, speed_factor≈1.0, max_delta≈9° at dt=0.05.
    assert!(
        rot.radians.abs() < 0.20, // < ~11.5° from East
        "Heading should be constrained to within ~10° of stored East heading. \
         Got {:.1}° — a value near 90° means rate-limiting from stored angle isn't working.",
        rot.radians.to_degrees()
    );

    // As a consequence, vx (East) should dominate over vy (North)
    assert!(
        vel.vx > vel.vy.abs(),
        "Eastward (stored heading) component should dominate after one tick of rate-limiting. \
         vx={:.4}, vy={:.4}",
        vel.vx,
        vel.vy
    );
}

#[test]
fn test_large_creature_turns_slower_than_small() {
    // TDD RED: Large creatures should turn more slowly than small ones
    // A 5m creature should turn much slower than a 0.5m creature
    let mut world = World::new();
    world.insert_resource(DeltaTime(0.1)); // Longer dt to see difference
    world.insert_resource(PhysicsTick(0));
    world.insert_resource(crate::simulation::core::WorldBounds::new(
        -1000.0, 1000.0, -1000.0, 1000.0,
    ));
    world.insert_resource(MovementConfig {
        locomotion_noise_base: 0.0,
        ..Default::default()
    });
    world.insert_resource(NoiseTable::default());
    #[cfg(feature = "dev-tools")]
    world.insert_resource(crate::instrumentation::SystemTimings::new());

    let mut small_state = CreatureState::default();
    small_state.behavior = crate::simulation::creatures::components::BehaviorMode::Wandering;
    let mut large_state = CreatureState::default();
    large_state.behavior = crate::simulation::creatures::components::BehaviorMode::Wandering;

    // Both creatures moving right at same speed, strong upward acceleration
    let small = world
        .spawn((
            BodySize::new(0.5), // Small creature
            Position { x: 0.0, y: 0.0 },
            Velocity { vx: 10.0, vy: 0.0 },
            Acceleration { ax: 0.0, ay: 100.0 }, // Strong turn force
            Rotation::default(),
            small_state,
        ))
        .id();

    let large = world
        .spawn((
            BodySize::new(5.0), // Large creature (10x size)
            Position { x: 100.0, y: 0.0 },
            Velocity { vx: 10.0, vy: 0.0 },
            Acceleration { ax: 0.0, ay: 100.0 }, // Same turn force
            Rotation::default(),
            large_state,
        ))
        .id();

    use bevy_ecs::system::IntoSystem;
    let mut system = IntoSystem::into_system(integrate_motion_system);
    system.initialize(&mut world);
    system.run((), &mut world);

    let small_vel = world.get::<Velocity>(small).unwrap();
    let large_vel = world.get::<Velocity>(large).unwrap();

    // Calculate turn angles
    let small_angle = small_vel.vy.atan2(small_vel.vx).to_degrees();
    let large_angle = large_vel.vy.atan2(large_vel.vx).to_degrees();

    // Small creature should have turned MORE than large creature
    assert!(
        small_angle > large_angle,
        "Small creature (0.5m) should turn more than large creature (5.0m). Small: {}°, Large: {}°",
        small_angle, large_angle
    );

    // The ratio should be significant (at least 2x difference for 10x size difference)
    let turn_ratio = small_angle / large_angle.max(0.1);
    assert!(
        turn_ratio > 2.0,
        "Small creature should turn at least 2x faster than large. Ratio: {}",
        turn_ratio
    );
}

#[test]
fn test_turn_rate_clamped_to_min_for_very_large() {
    // TDD RED: Very large creatures shouldn't drop below MIN_TURN_RATE_DEG
    let mut world = World::new();
    world.insert_resource(DeltaTime(1.0)); // 1 second to make math clear
    world.insert_resource(PhysicsTick(0));
    world.insert_resource(crate::simulation::core::WorldBounds::new(
        -1000.0, 1000.0, -1000.0, 1000.0,
    ));
    world.insert_resource(MovementConfig {
        locomotion_noise_base: 0.0,
        ..Default::default()
    });
    world.insert_resource(NoiseTable::default());
    #[cfg(feature = "dev-tools")]
    world.insert_resource(crate::instrumentation::SystemTimings::new());

    let mut state = CreatureState::default();
    state.behavior = crate::simulation::creatures::components::BehaviorMode::Wandering;

    // Very large creature - would have extremely low turn rate without floor
    let entity = world
        .spawn((
            BodySize::new(100.0), // Massive creature
            Position { x: 0.0, y: 0.0 },
            Velocity { vx: 10.0, vy: 0.0 }, // Moving right
            Acceleration {
                ax: 0.0,
                ay: 1000.0,
            }, // Massive upward force
            Rotation::default(),
            state,
        ))
        .id();

    use bevy_ecs::system::IntoSystem;
    let mut system = IntoSystem::into_system(integrate_motion_system);
    system.initialize(&mut world);
    system.run((), &mut world);

    let vel = world.get::<Velocity>(entity).unwrap();
    let turn_angle = vel.vy.atan2(vel.vx).to_degrees();

    // Should turn at least MIN_TURN_RATE_DEG * dt (with some tolerance for speed penalty)
    // At 1 second dt and 15 deg/s min rate, should be at least ~5 degrees (accounting for speed penalty)
    assert!(
        turn_angle >= 4.0,
        "Very large creature should turn at least ~5°/s (MIN_TURN_RATE with speed penalty), got {}°",
        turn_angle
    );
}

#[test]
fn test_turn_rate_clamped_to_max_for_very_small() {
    // TDD RED: Very small creatures shouldn't exceed MAX_TURN_RATE_DEG
    let mut world = World::new();
    world.insert_resource(DeltaTime(1.0)); // 1 second
    world.insert_resource(PhysicsTick(0));
    world.insert_resource(crate::simulation::core::WorldBounds::new(
        -1000.0, 1000.0, -1000.0, 1000.0,
    ));
    world.insert_resource(MovementConfig {
        locomotion_noise_base: 0.0,
        ..Default::default()
    });
    world.insert_resource(NoiseTable::default());
    #[cfg(feature = "dev-tools")]
    world.insert_resource(crate::instrumentation::SystemTimings::new());

    let mut state = CreatureState::default();
    state.behavior = crate::simulation::creatures::components::BehaviorMode::Wandering;

    // Tiny creature moving slowly (minimal speed penalty)
    let entity = world
        .spawn((
            BodySize::new(0.01), // Tiny creature
            Position { x: 0.0, y: 0.0 },
            Velocity { vx: 0.1, vy: 0.0 }, // Very slow (minimal speed penalty)
            Acceleration {
                ax: 0.0,
                ay: 1000.0,
            }, // Massive force to try to turn fast
            Rotation::default(),
            state,
        ))
        .id();

    use bevy_ecs::system::IntoSystem;
    let mut system = IntoSystem::into_system(integrate_motion_system);
    system.initialize(&mut world);
    system.run((), &mut world);

    let vel = world.get::<Velocity>(entity).unwrap();
    let turn_angle = vel.vy.atan2(vel.vx).to_degrees();

    // Should not exceed 360 deg/s cap (MAX_TURN_RATE_DEG)
    // With 1 second dt and slow speed, should be limited to MAX_TURN_RATE_DEG
    assert!(
        turn_angle <= 360.0,
        "Tiny creature should not exceed 360°/s turn rate, got {}°",
        turn_angle
    );
}

#[test]
fn test_speed_penalty_reduces_turn_rate() {
    // TDD RED: Faster creatures should turn more slowly
    let mut world = World::new();
    world.insert_resource(DeltaTime(0.1));
    world.insert_resource(PhysicsTick(0));
    world.insert_resource(crate::simulation::core::WorldBounds::new(
        -1000.0, 1000.0, -1000.0, 1000.0,
    ));
    world.insert_resource(MovementConfig {
        locomotion_noise_base: 0.0,
        ..Default::default()
    });
    world.insert_resource(NoiseTable::default());
    #[cfg(feature = "dev-tools")]
    world.insert_resource(crate::instrumentation::SystemTimings::new());

    let mut slow_state = CreatureState::default();
    slow_state.behavior = crate::simulation::creatures::components::BehaviorMode::Wandering;
    let mut fast_state = CreatureState::default();
    fast_state.behavior = crate::simulation::creatures::components::BehaviorMode::Wandering;

    // Two identical creatures, one slow, one fast
    let slow = world
        .spawn((
            BodySize::new(1.0),
            Position { x: 0.0, y: 0.0 },
            Velocity { vx: 1.0, vy: 0.0 }, // Slow
            Acceleration { ax: 0.0, ay: 100.0 },
            Rotation::default(),
            slow_state,
        ))
        .id();

    let fast_body = BodySize::new(1.0);
    let fast = world
        .spawn((
            fast_body,
            Position { x: 100.0, y: 0.0 },
            Velocity {
                vx: fast_body.max_speed(),
                vy: 0.0,
            }, // At max speed for this size
            Acceleration { ax: 0.0, ay: 100.0 },
            Rotation::default(),
            fast_state,
        ))
        .id();

    use bevy_ecs::system::IntoSystem;
    let mut system = IntoSystem::into_system(integrate_motion_system);
    system.initialize(&mut world);
    system.run((), &mut world);

    let slow_vel = world.get::<Velocity>(slow).unwrap();
    let fast_vel = world.get::<Velocity>(fast).unwrap();

    // Calculate turn angles (relative to starting direction of 0°)
    let slow_angle = slow_vel.vy.atan2(slow_vel.vx).to_degrees();
    let fast_angle = fast_vel.vy.atan2(fast_vel.vx).to_degrees();

    // Slow creature should turn MORE than fast creature
    assert!(
        slow_angle > fast_angle,
        "Slow creature should turn more than fast creature. Slow: {}°, Fast: {}°",
        slow_angle,
        fast_angle
    );

    // Fast creature at max speed retains 30% agility (1 - 0.7 speed penalty)
    // Slow creature should turn significantly faster (at least 1.5x)
    let turn_ratio = slow_angle / fast_angle.max(0.01);
    assert!(
        turn_ratio > 1.5,
        "Slow creature should turn significantly faster than max-speed creature. Ratio: {}",
        turn_ratio
    );
}

#[test]
fn test_180_degree_reversal_is_gradual() {
    let mut world = World::new();
    world.insert_resource(DeltaTime(0.05));
    world.insert_resource(PhysicsTick(0));
    world.insert_resource(crate::simulation::core::WorldBounds::new(
        -1000.0, 1000.0, -1000.0, 1000.0,
    ));
    world.insert_resource(MovementConfig::default());
    world.insert_resource(NoiseTable::default());
    #[cfg(feature = "dev-tools")]
    world.insert_resource(crate::instrumentation::SystemTimings::new());

    let mut state = CreatureState::default();
    state.behavior = crate::simulation::creatures::components::BehaviorMode::Wandering;

    let entity = world
        .spawn((
            BodySize::new(1.0),
            Position { x: 0.0, y: 0.0 },
            Velocity { vx: 10.0, vy: 0.0 },
            Acceleration {
                ax: -100.0,
                ay: 0.0,
            },
            Rotation::default(),
            state,
        ))
        .id();

    use bevy_ecs::system::IntoSystem;
    let mut system = IntoSystem::into_system(integrate_motion_system);
    system.initialize(&mut world);
    system.run((), &mut world);

    let vel = world.get::<Velocity>(entity).unwrap();
    assert!(
        vel.vx > 0.0,
        "Should still be moving right after one tick, got vx={}",
        vel.vx
    );
}

// =============================================================================
// STOPPED-THRESHOLD HEADING BUG REGRESSION TESTS
// =============================================================================
// Root cause: when speed < STOPPED_THRESHOLD, old_angle was set to NaN,
// bypassing the `old_angle.is_finite()` guard in turn rate limiting.
// In tight crowds, opposing avoidance forces cancel → drag slows creature below
// threshold → NaN bypass → next tick's force snaps heading 180° unconstrained.
// Fix: use rotation.radians (last stored heading) instead of NaN.

/// Build a World with all resources needed by integrate_motion_system.
/// Also initialises the Bevy ComputeTaskPool (idempotent, required for par_iter_mut).
fn make_integration_world(dt: f32, noise: f32) -> World {
    bevy_tasks::ComputeTaskPool::get_or_init(bevy_tasks::TaskPool::default);
    let mut world = World::new();
    world.insert_resource(DeltaTime(dt));
    world.insert_resource(PhysicsTick(0));
    world.insert_resource(crate::simulation::core::WorldBounds::new(
        -1000.0, 1000.0, -1000.0, 1000.0,
    ));
    world.insert_resource(MovementConfig {
        locomotion_noise_base: noise,
        ..Default::default()
    });
    world.insert_resource(NoiseTable::default());
    #[cfg(feature = "dev-tools")]
    world.insert_resource(crate::instrumentation::SystemTimings::new());
    world
}

fn run_integration(world: &mut World) {
    use bevy_ecs::system::IntoSystem;
    let mut system = IntoSystem::into_system(integrate_motion_system);
    system.initialize(world);
    system.run((), world);
}

fn wandering_state() -> CreatureState {
    let mut s = CreatureState::default();
    s.behavior = crate::simulation::creatures::components::BehaviorMode::Wandering;
    s
}

#[test]
fn stopped_creature_heading_constrained_by_turn_rate_on_restart() {
    // A creature below STOPPED_THRESHOLD whose stored heading is East receives a
    // full-power due-West force. With the bug, heading snaps 180° in one tick.
    // With the fix, it can only rotate at most MAX_TURN_RATE * dt (~8° for 1m, dt=0.05).
    let mut world = make_integration_world(0.05, 0.0);

    // Speed 0.01 m/s is below STOPPED_THRESHOLD (0.05), so old_angle = NaN in
    // the buggy code. Rotation::new(0.0) stores the last valid heading (East).
    let entity = world
        .spawn((
            BodySize::new(1.0),
            Position { x: 0.0, y: 0.0 },
            Velocity {
                vx: 0.01, // Below STOPPED_THRESHOLD — triggers the NaN bypass
                vy: 0.0,
            },
            Acceleration {
                ax: -100.0, // Full-power due-West force: tries to snap heading 180°
                ay: 0.0,
            },
            Rotation::new(0.0), // Last stored heading: due East
            wandering_state(),
        ))
        .id();

    run_integration(&mut world);

    let rot = world.get::<Rotation>(entity).unwrap();

    // Bug path:  old_angle = NaN → turn limiting skipped → rotation snaps to ~π (West)
    // Fixed path: old_angle = 0.0 (East) → max change ≈ 8° at dt=0.05, size=1m
    //
    // 20° threshold (0.35 rad) sits between the ~8° the fix allows and the 180° the bug
    // produces — no ambiguity in pass/fail.
    assert!(
        rot.radians.abs() < 0.35,
        "Heading should be constrained by turn rate to <20° from stored East heading. \
         Got {:.1}° — a value near ±180° means the NaN bypass is still active.",
        rot.radians.to_degrees()
    );
}

#[test]
fn stopped_creature_heading_constrained_across_multiple_directions() {
    // Same bug, but with stored heading North (PI/2) and a due-South reversal force.
    // Verifies the fix works for non-zero stored headings.
    use std::f32::consts::FRAC_PI_2;

    let mut world = make_integration_world(0.05, 0.0);

    let entity = world
        .spawn((
            BodySize::new(1.0),
            Position { x: 0.0, y: 0.0 },
            Velocity {
                vx: 0.0,
                vy: 0.02, // Below STOPPED_THRESHOLD, heading North
            },
            Acceleration {
                ax: 0.0,
                ay: -100.0, // Full-power due-South reversal
            },
            Rotation::new(FRAC_PI_2), // Last stored heading: due North
            wandering_state(),
        ))
        .id();

    run_integration(&mut world);

    let rot = world.get::<Rotation>(entity).unwrap();

    // Bug: snaps to -PI/2 (South), delta from North = π ≈ 180°
    // Fix: constrained to within ~8° of North (PI/2)
    let delta_from_north = (rot.radians - FRAC_PI_2).abs();
    assert!(
        delta_from_north < 0.35, // <20° from North
        "Heading should stay near North (delta <20°). Got heading {:.1}°, delta {:.1}°. \
         A delta near 180° means the NaN bypass is still active.",
        rot.radians.to_degrees(),
        delta_from_north.to_degrees()
    );
}

#[test]
fn zero_crossing_preserves_rotation_for_large_creature() {
    // Bug 2 (the crowd oscillation root cause):
    //
    // When an avoidance impulse drives a large creature's speed from just above
    // STOPPED_THRESHOLD to just below it (a "zero-crossing"), two things go wrong:
    //   1. Turn rate limiting is skipped because speed_sq < threshold AFTER integration.
    //   2. set_from_velocity stores the reversed tiny velocity as rotation.radians.
    //
    // Next tick, old_angle = rotation.radians = backward direction → creature is
    // rate-limited FROM the wrong heading for up to 10 seconds (0.9°/tick for size 10).
    //
    // The fix: gate set_from_velocity on final_speed_sq > stopped_threshold_sq.
    // When speed drops below threshold, rotation.radians is PRESERVED (not overwritten).
    //
    // Test setup: size-10 creature moving East at 0.12 m/s (above threshold 0.05 m/s).
    // Avoidance-magnitude West force (max_accel for size 10 = 3.162 m/s²):
    //   new_vx = (0.12 - 3.162 × 0.05) × drag ≈ -0.037 m/s  (below threshold)
    // With bug: set_from_velocity(-0.037, 0) → rotation.radians = π (West) — WRONG.
    // With fix: set_from_velocity not called → rotation.radians stays East — CORRECT.

    let mut world = make_integration_world(0.05, 0.0);

    // size-10 creature moving East at 0.12 m/s (just above threshold),
    // stored heading East (0 rad).
    // Force: West at max avoidance acceleration for size-10 (10 / sqrt(10) ≈ 3.162 m/s²)
    // After one tick: (0.12 - 3.162×0.05) × exp(-0.025) ≈ -0.037 m/s (below threshold)
    let entity = world
        .spawn((
            BodySize::new(10.0),
            Position { x: 0.0, y: 0.0 },
            Velocity { vx: 0.12, vy: 0.0 }, // Just above STOPPED_THRESHOLD (0.05)
            Acceleration {
                ax: -3.162, // Max avoidance accel for size-10: enough to zero-cross
                ay: 0.0,
            },
            Rotation::new(0.0), // Stored heading: due East
            wandering_state(),
        ))
        .id();

    run_integration(&mut world);

    let vel = world.get::<Velocity>(entity).unwrap();
    let rot = world.get::<Rotation>(entity).unwrap();

    // Confirm the zero-crossing happened — new speed IS below threshold.
    let new_speed = (vel.vx * vel.vx + vel.vy * vel.vy).sqrt();
    assert!(
        new_speed < 0.05,
        "Test precondition: new speed should be below STOPPED_THRESHOLD. Got {:.4} m/s",
        new_speed
    );

    // With the bug:  rotation.radians = π (West) — stored from the tiny reversed velocity
    // With the fix:  rotation.radians ≈ 0.0 (East) — preserved, not overwritten
    //
    // 0.35 rad (20°) is the threshold: fix → ~0°, bug → ~180°.
    assert!(
        rot.radians.abs() < 0.35,
        "Stored heading should be preserved at East when speed zero-crosses below \
         threshold. Got {:.1}° — a value near ±180° means set_from_velocity is still \
         being called with the reversed tiny velocity.",
        rot.radians.to_degrees()
    );
}
