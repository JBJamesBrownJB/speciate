
use crate::config::MovementConfig;
#[cfg(feature = "dev-tools")]
use crate::instrumentation::SystemTimings;
use crate::simulation::components::*;
use crate::simulation::core::components::*;
use crate::simulation::math::normalize_angle;
use crate::simulation::movement::constants::{MAX_SPEED, MAX_TURN_RATE_RAD, STOPPED_THRESHOLD, VELOCITY_DAMPING};
use crate::simulation::movement::noise::perlin_locomotion_noise;
use bevy_ecs::prelude::*;
use rayon::prelude::*;
pub fn integrate_motion_system(
    mut query: Query<(
        Entity,
        &BodySize,
        &mut Position,
        &mut Velocity,
        &mut Acceleration,
        &CreatureState,
    )>,
    delta_time: Res<DeltaTime>,
    physics_tick: Res<PhysicsTick>,
    world_bounds: Res<crate::simulation::core::WorldBounds>,
    movement_config: Res<MovementConfig>,
    #[cfg(feature = "dev-tools")] timings: Res<SystemTimings>,
) {
    #[cfg(feature = "dev-tools")]
    crate::time_system!(timings, "movement");

    let dt = delta_time.0;
    let max_speed_sq = MAX_SPEED * MAX_SPEED;
    let tick = physics_tick.get();
    let noise_base = movement_config.locomotion_noise_base;
    let noise_time_scale = movement_config.noise_time_scale;

    // Collect entities into Vec for Rayon parallel processing
    let mut entities: Vec<_> = query.iter_mut().collect();

    let max_turn_rate_rad = MAX_TURN_RATE_RAD;
    let stopped_threshold_sq = STOPPED_THRESHOLD * STOPPED_THRESHOLD;

    // Parallel physics integration using Rayon
    entities.par_iter_mut().for_each(|(entity, size, position, velocity, acceleration, creature_state)| {
        if creature_state.behavior == BehaviorMode::Catatonic {
            acceleration.ax = 0.0;
            acceleration.ay = 0.0;

            let speed_sq = velocity.vx * velocity.vx + velocity.vy * velocity.vy;
            if speed_sq < stopped_threshold_sq {
                if velocity.vx != 0.0 || velocity.vy != 0.0 {
                    velocity.vx = 0.0;
                    velocity.vy = 0.0;
                }
                return;
            }

            velocity.vx *= VELOCITY_DAMPING;
            velocity.vy *= VELOCITY_DAMPING;

            position.x += velocity.vx * dt;
            position.y += velocity.vy * dt;

            return;
        }

        // Capture old heading before velocity changes
        let old_speed_sq = velocity.vx * velocity.vx + velocity.vy * velocity.vy;
        let old_angle = if old_speed_sq > stopped_threshold_sq {
            velocity.vy.atan2(velocity.vx)
        } else {
            f32::NAN
        };

        velocity.vx += acceleration.ax * dt;
        velocity.vy += acceleration.ay * dt;
        velocity.vx *= VELOCITY_DAMPING;
        velocity.vy *= VELOCITY_DAMPING;
        let speed_sq = velocity.vx * velocity.vx + velocity.vy * velocity.vy;
        if speed_sq > 0.01 {
            let speed = speed_sq.sqrt();
            let speed_ratio = speed / MAX_SPEED;
            let size_factor = size.inv_sqrt_length;
            let noise_magnitude = noise_base * speed_ratio * speed_ratio * size_factor;

            let noise_x = perlin_locomotion_noise(entity.index(), tick, 0, noise_time_scale);
            let noise_y = perlin_locomotion_noise(entity.index(), tick, 1, noise_time_scale);

            let perpendicular_x = -velocity.vy / speed;
            let perpendicular_y = velocity.vx / speed;

            velocity.vx += perpendicular_x * noise_x * noise_magnitude * dt;
            velocity.vy += perpendicular_y * noise_y * noise_magnitude * dt;
        }
        let speed_sq = velocity.vx * velocity.vx + velocity.vy * velocity.vy;
        if speed_sq > max_speed_sq {
            let scale = (max_speed_sq / speed_sq).sqrt();
            velocity.vx *= scale;
            velocity.vy *= scale;
        }

        // Turn rate limiting: clamp velocity direction change per frame
        let new_speed_sq = velocity.vx * velocity.vx + velocity.vy * velocity.vy;
        if old_angle.is_finite() && new_speed_sq > stopped_threshold_sq {
            let new_angle = velocity.vy.atan2(velocity.vx);
            let delta = normalize_angle(new_angle - old_angle);
            let max_delta = max_turn_rate_rad * dt;

            if delta.abs() > max_delta {
                let clamped_delta = delta.clamp(-max_delta, max_delta);
                let final_angle = old_angle + clamped_delta;
                let new_speed = new_speed_sq.sqrt();
                velocity.vx = new_speed * final_angle.cos();
                velocity.vy = new_speed * final_angle.sin();
            }
        }

        acceleration.ax = 0.0;
        acceleration.ay = 0.0;

        position.x += velocity.vx * dt;
        position.y += velocity.vy * dt;
    });

    // Parallel boundary enforcement (reuse collected entities)
    let min_x = world_bounds.min_x;
    let max_x = world_bounds.max_x;
    let min_y = world_bounds.min_y;
    let max_y = world_bounds.max_y;

    entities.par_iter_mut().for_each(|(_entity, _size, position, velocity, _accel, _state)| {
        if position.x < min_x {
            position.x = min_x;
            velocity.vx = velocity.vx.max(0.0);
        } else if position.x > max_x {
            position.x = max_x;
            velocity.vx = velocity.vx.min(0.0);
        }
        if position.y < min_y {
            position.y = min_y;
            velocity.vy = velocity.vy.max(0.0);
        } else if position.y > max_y {
            position.y = max_y;
            velocity.vy = velocity.vy.min(0.0);
        }
    });
}

pub fn update_body_size_cache(mut query: Query<&mut BodySize, Changed<BodySize>>) {
    for mut size in query.iter_mut() {
        size.inv_sqrt_length = 1.0 / size.length.sqrt();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
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
        world.insert_resource(crate::simulation::core::WorldBounds::new(-100.0, 100.0, -100.0, 100.0));
        world.insert_resource(MovementConfig::default());
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
                Velocity { vx: (i as f32 * 0.1).sin(), vy: (i as f32 * 0.1).cos() },
                Acceleration { ax: 0.0, ay: 0.0 },
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
        world2.insert_resource(crate::simulation::core::WorldBounds::new(-100.0, 100.0, -100.0, 100.0));
        world2.insert_resource(MovementConfig::default());
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
                Velocity { vx: (i as f32 * 0.1).sin(), vy: (i as f32 * 0.1).cos() },
                Acceleration { ax: 0.0, ay: 0.0 },
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
        for (i, ((x1, y1), (x2, y2))) in positions_run1.iter().zip(positions_run2.iter()).enumerate() {
            assert!(
                (x1 - x2).abs() < 0.0001,
                "Entity {} X position mismatch: {} vs {}",
                i, x1, x2
            );
            assert!(
                (y1 - y2).abs() < 0.0001,
                "Entity {} Y position mismatch: {} vs {}",
                i, y1, y2
            );
        }
    }

    #[test]
    fn test_all_creatures_processed_in_parallel() {
        let mut world = World::new();
        world.insert_resource(DeltaTime(0.016));
        world.insert_resource(PhysicsTick(0));
        world.insert_resource(crate::simulation::core::WorldBounds::new(-100.0, 100.0, -100.0, 100.0));
        world.insert_resource(MovementConfig::default());
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
            assert_eq!(accel.ax, 0.0, "Acceleration should be reset after integration");
            assert_eq!(accel.ay, 0.0, "Acceleration should be reset after integration");
            processed_count += 1;
        }

        assert_eq!(processed_count, 1000, "All 1000 entities should be processed");

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
        world.insert_resource(crate::simulation::core::WorldBounds::new(-100.0, 100.0, -100.0, 100.0));
        world.insert_resource(MovementConfig::default());
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
        assert!(velocities[0].vx <= 0.0, "Velocity at max_x boundary should be non-positive");

        // Entity 1: was beyond min_x, velocity.vx should be clamped to >= 0
        assert!(velocities[1].vx >= 0.0, "Velocity at min_x boundary should be non-negative");

        // Entity 2: was beyond max_y, velocity.vy should be clamped to <= 0
        assert!(velocities[2].vy <= 0.0, "Velocity at max_y boundary should be non-positive");

        // Entity 3: was beyond min_y, velocity.vy should be clamped to >= 0
        assert!(velocities[3].vy >= 0.0, "Velocity at min_y boundary should be non-negative");
    }

    #[test]
    fn test_turn_rate_limits_direction_change() {
        let mut world = World::new();
        world.insert_resource(DeltaTime(0.05));
        world.insert_resource(PhysicsTick(0));
        world.insert_resource(crate::simulation::core::WorldBounds::new(-1000.0, 1000.0, -1000.0, 1000.0));
        world.insert_resource(MovementConfig::default());
        #[cfg(feature = "dev-tools")]
        world.insert_resource(crate::instrumentation::SystemTimings::new());

        let mut state = CreatureState::default();
        state.behavior = crate::simulation::creatures::components::BehaviorMode::Wandering;

        let entity = world.spawn((
            BodySize::new(1.0),
            Position { x: 0.0, y: 0.0 },
            Velocity { vx: 10.0, vy: 0.0 },
            Acceleration { ax: 0.0, ay: 100.0 },
            state,
        )).id();

        use bevy_ecs::system::IntoSystem;
        let mut system = IntoSystem::into_system(integrate_motion_system);
        system.initialize(&mut world);
        system.run((), &mut world);

        let vel = world.get::<Velocity>(entity).unwrap();

        let initial_angle = 0.0_f32;
        let final_angle = vel.vy.atan2(vel.vx);
        let delta_degrees = (final_angle - initial_angle).to_degrees().abs();

        let max_expected = crate::simulation::movement::constants::MAX_TURN_RATE * 0.05 + 0.1;
        assert!(
            delta_degrees <= max_expected,
            "Turn rate should be limited to ~0.9 deg, got {} deg",
            delta_degrees
        );
        assert!(delta_degrees > 0.0, "Should have some turn, got {} deg", delta_degrees);
    }

    #[test]
    fn test_small_turns_not_affected() {
        let mut world = World::new();
        world.insert_resource(DeltaTime(0.05));
        world.insert_resource(PhysicsTick(0));
        world.insert_resource(crate::simulation::core::WorldBounds::new(-1000.0, 1000.0, -1000.0, 1000.0));
        world.insert_resource(MovementConfig::default());
        #[cfg(feature = "dev-tools")]
        world.insert_resource(crate::instrumentation::SystemTimings::new());

        let mut state = CreatureState::default();
        state.behavior = crate::simulation::creatures::components::BehaviorMode::Wandering;

        let entity = world.spawn((
            BodySize::new(1.0),
            Position { x: 0.0, y: 0.0 },
            Velocity { vx: 10.0, vy: 0.0 },
            Acceleration { ax: 1.0, ay: 0.1 },
            state,
        )).id();

        use bevy_ecs::system::IntoSystem;
        let mut system = IntoSystem::into_system(integrate_motion_system);
        system.initialize(&mut world);
        system.run((), &mut world);

        let vel = world.get::<Velocity>(entity).unwrap();
        assert!(vel.vy > 0.0, "Small upward component should be preserved");
    }

    #[test]
    fn test_stopped_creatures_can_turn_freely() {
        let mut world = World::new();
        world.insert_resource(DeltaTime(0.05));
        world.insert_resource(PhysicsTick(0));
        world.insert_resource(crate::simulation::core::WorldBounds::new(-1000.0, 1000.0, -1000.0, 1000.0));
        world.insert_resource(MovementConfig::default());
        #[cfg(feature = "dev-tools")]
        world.insert_resource(crate::instrumentation::SystemTimings::new());

        let mut state = CreatureState::default();
        state.behavior = crate::simulation::creatures::components::BehaviorMode::Wandering;

        let entity = world.spawn((
            BodySize::new(1.0),
            Position { x: 0.0, y: 0.0 },
            Velocity { vx: 0.0, vy: 0.0 },
            Acceleration { ax: 0.0, ay: 10.0 },
            state,
        )).id();

        use bevy_ecs::system::IntoSystem;
        let mut system = IntoSystem::into_system(integrate_motion_system);
        system.initialize(&mut world);
        system.run((), &mut world);

        let vel = world.get::<Velocity>(entity).unwrap();
        assert!(vel.vy > 0.0, "Should be moving up");
        assert!(vel.vx.abs() < 0.001, "Should not have horizontal component");
    }

    #[test]
    fn test_180_degree_reversal_is_gradual() {
        let mut world = World::new();
        world.insert_resource(DeltaTime(0.05));
        world.insert_resource(PhysicsTick(0));
        world.insert_resource(crate::simulation::core::WorldBounds::new(-1000.0, 1000.0, -1000.0, 1000.0));
        world.insert_resource(MovementConfig::default());
        #[cfg(feature = "dev-tools")]
        world.insert_resource(crate::instrumentation::SystemTimings::new());

        let mut state = CreatureState::default();
        state.behavior = crate::simulation::creatures::components::BehaviorMode::Wandering;

        let entity = world.spawn((
            BodySize::new(1.0),
            Position { x: 0.0, y: 0.0 },
            Velocity { vx: 10.0, vy: 0.0 },
            Acceleration { ax: -100.0, ay: 0.0 },
            state,
        )).id();

        use bevy_ecs::system::IntoSystem;
        let mut system = IntoSystem::into_system(integrate_motion_system);
        system.initialize(&mut world);
        system.run((), &mut world);

        let vel = world.get::<Velocity>(entity).unwrap();
        assert!(vel.vx > 0.0, "Should still be moving right after one tick, got vx={}", vel.vx);
    }
}
