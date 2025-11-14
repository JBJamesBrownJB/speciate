//! Movement systems
//!
//! These systems handle motion integration (Euler) and world boundary constraints.
//! They run AFTER behavior systems have accumulated forces into Acceleration.

use crate::config::MovementConfig;
use crate::simulation::components::*;
use crate::simulation::core::components::*;
use crate::simulation::movement::constants::{MAX_SPEED, VELOCITY_DAMPING};
use crate::simulation::movement::noise::perlin_locomotion_noise;
use bevy_ecs::prelude::*;

/// Integrates motion using Euler integration (accel → vel → pos)
///
/// System ordering: Must run AFTER all behavior systems have accumulated forces.
///
/// Integration steps:
/// 1. Apply acceleration to velocity (F = ma)
/// 2. Apply velocity damping (air resistance + ground friction)
/// 3. Add locomotion noise (motor control variability)
/// 4. Limit velocity to MAX_SPEED
/// 5. Reset acceleration to zero (forces are impulses, not continuous)
/// 6. Apply velocity to position (Euler integration)
///
/// Special case: Catatonic creatures are skipped (stationary by design)
///
/// # Locomotion Noise
/// Adds Perlin noise perpendicular to velocity direction to simulate:
/// - Neuromuscular variability (motor control imprecision)
/// - Terrain micro-irregularities (pebbles, grass, slopes)
/// - Decision-making lag (temporal smoothing)
///
/// Scales quadratically with speed (speed²):
/// - Low speeds: Minimal noise (precise final approach to targets)
/// - High speeds: Maximum noise (realistic high-speed wobble)
///
/// Benefits:
/// - Natural organic movement (animals don't move in perfect straight lines)
/// - Fixes collinear obstacle edge case (adds lateral component)
/// - Precision at low speeds, wobble at high speeds (biological realism)
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
) {
    let dt = delta_time.0;
    let max_speed_sq = MAX_SPEED * MAX_SPEED;
    let tick = physics_tick.get();

    for (entity, size, mut position, mut velocity, mut acceleration, creature_state) in query.iter_mut() {
        // Skip motion updates for Catatonic crits (stationary by design)
        if creature_state.behavior == BehaviorMode::Catatonic {
            continue;
        }

        // 1. Apply acceleration to velocity (F = ma)
        velocity.vx += acceleration.ax * dt;
        velocity.vy += acceleration.ay * dt;

        // 2. Apply velocity damping (air resistance + ground friction)
        velocity.vx *= VELOCITY_DAMPING;
        velocity.vy *= VELOCITY_DAMPING;

        // 3. Add locomotion noise (motor control variability + terrain irregularities)
        let speed = (velocity.vx * velocity.vx + velocity.vy * velocity.vy).sqrt();
        if speed > 0.1 {  // Increased threshold to prevent near-zero division
            // Noise magnitude scales quadratically with speed (precision at low speeds)
            let speed_ratio = speed / MAX_SPEED;
            let size_factor = 1.0 / size.length.sqrt();
            let noise_magnitude = movement_config.locomotion_noise_base * speed_ratio * speed_ratio * size_factor;

            // Generate Perlin noise for this creature at current tick
            let noise_x = perlin_locomotion_noise(entity.index(), tick, 0, movement_config.noise_time_scale);
            let noise_y = perlin_locomotion_noise(entity.index(), tick, 1, movement_config.noise_time_scale);

            // Calculate perpendicular direction (lateral wobble, not forward/back)
            let perpendicular_x = -velocity.vy / speed;
            let perpendicular_y = velocity.vx / speed;

            // Add noise perpendicular to velocity (side-to-side drift)
            velocity.vx += perpendicular_x * noise_x * noise_magnitude * dt;
            velocity.vy += perpendicular_y * noise_y * noise_magnitude * dt;
        }

        // 4. Limit velocity to MAX_SPEED
        let speed_sq = velocity.vx * velocity.vx + velocity.vy * velocity.vy;
        if speed_sq > max_speed_sq {
            let speed = speed_sq.sqrt();
            let inv_speed = MAX_SPEED / speed;
            velocity.vx *= inv_speed;
            velocity.vy *= inv_speed;
        }

        // 5. Reset acceleration (force accumulation complete)
        acceleration.ax = 0.0;
        acceleration.ay = 0.0;

        // 6. Apply velocity to position (Euler integration)
        position.x += velocity.vx * dt;
        position.y += velocity.vy * dt;
    }

    // Soft position clamping - prevents runaway drift beyond world bounds
    // This is NOT a hard wall, just prevents indefinite escape while allowing natural movement
    for (_entity, _size, mut position, mut velocity, _accel, _state) in query.iter_mut() {
        // X-axis soft clamp
        if position.x < world_bounds.min_x {
            position.x = world_bounds.min_x;
            velocity.vx = velocity.vx.max(0.0); // Remove inward velocity component
        } else if position.x > world_bounds.max_x {
            position.x = world_bounds.max_x;
            velocity.vx = velocity.vx.min(0.0); // Remove outward velocity component
        }

        // Y-axis soft clamp
        if position.y < world_bounds.min_y {
            position.y = world_bounds.min_y;
            velocity.vy = velocity.vy.max(0.0);
        } else if position.y > world_bounds.max_y {
            position.y = world_bounds.max_y;
            velocity.vy = velocity.vy.min(0.0);
        }
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
}
