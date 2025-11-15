//! Seeking behavior system
//!
//! Reynolds steering with exponential arrival behavior.
//! Follows force accumulation pattern: ADDs to Acceleration.

use crate::simulation::components::*;
use crate::simulation::core::components::*;
use crate::simulation::movement::constants::{MAX_SPEED, SEEKING, SLOW_ZONE_MULTIPLIER};
use bevy_ecs::prelude::*;

/// Seeking behavior: steer toward target with smooth arrival
///
/// Algorithm:
/// 1. Calculate distance to target
/// 2. Exponential deceleration in slow zone (gentle far out, sharp near target)
/// 3. Pounce when close and slow (snap to target, prevent creeping)
/// 4. Emergency brake if within arrival radius
/// 5. Calculate steering force and ADD to acceleration
///
/// Arrival zones (SLOW_ZONE_MULTIPLIER = 35.0):
/// - Slow zone: 17.5m (begin exponential deceleration)
/// - Pounce: 0.5m @ speed < 3.5 m/s (snap to target)
/// - Emergency brake: < 0.5m (hard counter-force)
///
/// Exponential deceleration gives "land on a dime" behavior - maintains speed for
/// max reaction time, then brakes hard. Only overshoots if too fast with insufficient distance.
///
/// TODO: Migrate to DNA-driven parameters
#[allow(clippy::type_complexity)]
pub fn seek_system(
    mut query: Query<
        (
            &mut Position,
            &mut Acceleration,
            &mut Velocity,
            &BodySize,
            &Target,
            &mut CreatureState,
        ),
        With<CanSeek>,
    >,
) {
    // Seeking parameters from global constants (TODO: DNA Future DNA system)

    for (position, mut acceleration, mut velocity, size, target, mut creature_state) in
        query.iter_mut()
    {
        if creature_state.behavior != BehaviorMode::Seeking {
            continue;
        }

        // Calculate center-to-center distance to target
        let to_target_x = target.x - position.x;
        let to_target_y = target.y - position.y;
        let center_distance = (to_target_x * to_target_x + to_target_y * to_target_y).sqrt();

        // Guard against division by zero (already at exact target position)
        if center_distance < 0.001 {
            velocity.vx = 0.0;
            velocity.vy = 0.0;
            creature_state.behavior = BehaviorMode::Catatonic;
            continue;
        }

        // Arrival threshold accounts for body radius (stop when edge reaches target)
        let body_radius = size.radius() / 2.0;
        let arrival_radius = SEEKING.arrival_tolerance + body_radius;
        let slow_zone = arrival_radius * SLOW_ZONE_MULTIPLIER;
        let current_speed = (velocity.vx * velocity.vx + velocity.vy * velocity.vy).sqrt();

        // Pounce: stop when close and slow (prevents creeping)
        // Check center distance against pounce threshold + body radius
        if center_distance < (SEEKING.pounce_distance + body_radius)
            && current_speed < SEEKING.pounce_speed
        {
            velocity.vx = 0.0;
            velocity.vy = 0.0;
            creature_state.behavior = BehaviorMode::Catatonic;
            continue;
        }

        // Emergency brake: hard counter-force if within arrival radius
        if center_distance < arrival_radius {
            acceleration.ax += -velocity.vx * SEEKING.brake_force;
            acceleration.ay += -velocity.vy * SEEKING.brake_force;
            continue;
        }

        let creature_max_speed = MAX_SPEED;

        // Exponential deceleration in slow zone (gentle far out, sharp near target)
        let desired_speed = if center_distance > slow_zone {
            creature_max_speed
        } else {
            let slow_zone_distance = slow_zone - arrival_radius;
            let distance_into_zone = center_distance - arrival_radius;
            let ratio = distance_into_zone / slow_zone_distance;
            creature_max_speed * (SEEKING.slow_zone_decay * ratio).exp() / SEEKING.slow_zone_decay.exp()
        };

        let desired_vx = (to_target_x / center_distance) * desired_speed;
        let desired_vy = (to_target_y / center_distance) * desired_speed;
        let steer_x = desired_vx - velocity.vx;
        let steer_y = desired_vy - velocity.vy;

        // Limit and accumulate steering force
        let steer_mag_sq = steer_x * steer_x + steer_y * steer_y;
        if steer_mag_sq > SEEKING.max_force * SEEKING.max_force {
            let scale = SEEKING.max_force / steer_mag_sq.sqrt();
            acceleration.ax += steer_x * scale;
            acceleration.ay += steer_y * scale;
        } else {
            acceleration.ax += steer_x;
            acceleration.ay += steer_y;
        }
    }
}
