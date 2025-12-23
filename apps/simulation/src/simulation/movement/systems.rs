use crate::config::MovementConfig;
#[cfg(feature = "dev-tools")]
use crate::instrumentation::SystemTimings;
use crate::simulation::core::components::{
    Acceleration, BodySize, DeltaTime, PhysicsTick, Position, Rotation, Velocity,
};
use crate::simulation::creatures::components::{BehaviorMode, CreatureState};
use crate::simulation::creatures::constants::{
    DRAG_COEFFICIENT, MAX_TURN_RATE, MAX_TURN_RATE_DEG, MIN_TURN_RATE_DEG,
    NOISE_SPEED_THRESHOLD_SQ, STOPPED_THRESHOLD, TURN_RATE_SIZE_EXPONENT, TURN_RATE_SPEED_PENALTY,
};
use crate::simulation::math::{fast_atan2, normalize_angle};
use crate::simulation::movement::noise::NoiseTable;
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
        &mut Rotation,
    )>,
    delta_time: Res<DeltaTime>,
    physics_tick: Res<PhysicsTick>,
    world_bounds: Res<crate::simulation::core::WorldBounds>,
    movement_config: Res<MovementConfig>,
    noise_table: Res<NoiseTable>,
    #[cfg(feature = "dev-tools")] timings: Res<SystemTimings>,
) {
    #[cfg(feature = "dev-tools")]
    crate::time_system!(timings, "movement");

    let dt = delta_time.0;
    let tick = physics_tick.get();

    // Time-based drag: v *= exp(-drag * dt) is frame-rate independent
    let drag_factor = (-DRAG_COEFFICIENT * dt).exp();
    let noise_base = movement_config.locomotion_noise_base;
    let noise_time_scale = movement_config.noise_time_scale;

    // Capture bounds for parallel access
    let min_x = world_bounds.min_x;
    let max_x = world_bounds.max_x;
    let min_y = world_bounds.min_y;
    let max_y = world_bounds.max_y;

    // Get reference to noise table for parallel access
    let noise_ref = &*noise_table;

    // Collect entities into Vec for Rayon parallel processing
    let mut entities: Vec<_> = query.iter_mut().collect();

    let stopped_threshold_sq = STOPPED_THRESHOLD * STOPPED_THRESHOLD;

    // Parallel physics integration + boundary enforcement + rotation (merged into single loop)
    entities.par_iter_mut().for_each(
        |(entity, size, position, velocity, acceleration, creature_state, rotation)| {
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

                velocity.vx *= drag_factor;
                velocity.vy *= drag_factor;

                position.x += velocity.vx * dt;
                position.y += velocity.vy * dt;

                // Boundary enforcement for coasting catatonic creatures
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

                return;
            }

            // Capture old heading before velocity changes (fast_atan2: ~5x faster)
            let old_speed_sq = velocity.vx * velocity.vx + velocity.vy * velocity.vy;
            let old_angle = if old_speed_sq > stopped_threshold_sq {
                fast_atan2(velocity.vy, velocity.vx)
            } else {
                f32::NAN
            };

            velocity.vx += acceleration.ax * dt;
            velocity.vy += acceleration.ay * dt;
            velocity.vx *= drag_factor;
            velocity.vy *= drag_factor;

            // Size-based speed limit for this creature
            let max_speed = size.max_speed();
            let max_speed_sq = max_speed * max_speed;

            // Track speed for reuse (avoid redundant sqrt)
            let mut speed_sq = velocity.vx * velocity.vx + velocity.vy * velocity.vy;
            let mut current_speed = 0.0_f32;
            let mut speed_computed = false;

            if speed_sq > NOISE_SPEED_THRESHOLD_SQ {
                current_speed = speed_sq.sqrt();
                let speed_ratio = current_speed / max_speed;
                let size_factor = size.inv_sqrt_length;
                let noise_magnitude = noise_base * speed_ratio * speed_ratio * size_factor;

                let noise_x = noise_ref.get(entity.index(), tick, 0, noise_time_scale);
                let noise_y = noise_ref.get(entity.index(), tick, 1, noise_time_scale);

                let inv_speed = 1.0 / current_speed;
                let perpendicular_x = -velocity.vy * inv_speed;
                let perpendicular_y = velocity.vx * inv_speed;

                velocity.vx += perpendicular_x * noise_x * noise_magnitude * dt;
                velocity.vy += perpendicular_y * noise_y * noise_magnitude * dt;

                // Recalculate after noise modification
                speed_sq = velocity.vx * velocity.vx + velocity.vy * velocity.vy;
                speed_computed = false; // Speed changed, need fresh sqrt if used
            }

            // Speed clamping
            let was_clamped = speed_sq > max_speed_sq;
            if was_clamped {
                if !speed_computed {
                    current_speed = speed_sq.sqrt();
                }
                let scale = max_speed / current_speed;
                velocity.vx *= scale;
                velocity.vy *= scale;
                current_speed = max_speed; // After clamping, speed is exactly max_speed
                speed_sq = max_speed_sq;
                // Note: speed_computed not set - if turn rate limiting needs speed,
                // sqrt(max_speed_sq) = max_speed = current_speed, so result is same
            }

            // Size-dependent turn rate limiting
            // Biological basis: turn_rate ∝ 1/size^1.33 (moment of inertia vs muscle torque)
            if old_angle.is_finite() && speed_sq > stopped_threshold_sq {
                // Calculate size-dependent base turn rate (deg/s)
                let base_turn_rate_deg = (MAX_TURN_RATE
                    / size.length.powf(TURN_RATE_SIZE_EXPONENT))
                .clamp(MIN_TURN_RATE_DEG, MAX_TURN_RATE_DEG);

                // Apply speed penalty: faster movement = less agile turning
                // At max speed, creatures retain (1 - PENALTY) of their turn ability
                let current_speed_for_penalty = if speed_computed || was_clamped {
                    current_speed
                } else {
                    speed_sq.sqrt()
                };
                let normalized_speed = (current_speed_for_penalty / max_speed).min(1.0);
                let speed_factor =
                    1.0 - TURN_RATE_SPEED_PENALTY * normalized_speed * normalized_speed;
                let effective_turn_rate_deg = base_turn_rate_deg * speed_factor;

                // Convert to radians and apply dt
                let max_delta = effective_turn_rate_deg.to_radians() * dt;

                let new_angle = fast_atan2(velocity.vy, velocity.vx);
                let delta = normalize_angle(new_angle - old_angle);

                if delta.abs() > max_delta {
                    let clamped_delta = delta.clamp(-max_delta, max_delta);
                    let final_angle = old_angle + clamped_delta;
                    // Reuse speed from penalty calculation
                    let new_speed = current_speed_for_penalty;
                    velocity.vx = new_speed * final_angle.cos();
                    velocity.vy = new_speed * final_angle.sin();
                }
            }

            acceleration.ax = 0.0;
            acceleration.ay = 0.0;

            position.x += velocity.vx * dt;
            position.y += velocity.vy * dt;

            // Boundary enforcement (merged into main loop)
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

            // Rotation update (fused for parallelization - vx/vy already in cache)
            rotation.set_from_velocity(velocity.vx, velocity.vy);
        },
    );
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
    fn test_stopped_creatures_can_turn_freely() {
        let mut world = World::new();
        world.insert_resource(DeltaTime(0.05));
        world.insert_resource(PhysicsTick(0));
        world.insert_resource(crate::simulation::core::WorldBounds::new(
            -1000.0, 1000.0, -1000.0, 1000.0,
        ));
        // Disable noise so it doesn't interfere with turn test
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
                Velocity { vx: 0.0, vy: 0.0 },
                Acceleration { ax: 0.0, ay: 10.0 },
                Rotation::default(),
                state,
            ))
            .id();

        use bevy_ecs::system::IntoSystem;
        let mut system = IntoSystem::into_system(integrate_motion_system);
        system.initialize(&mut world);
        system.run((), &mut world);

        let vel = world.get::<Velocity>(entity).unwrap();
        assert!(vel.vy > 0.0, "Should be moving up");
        assert!(vel.vx.abs() < 0.001, "Should not have horizontal component");
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
}
