use crate::simulation::components::*;
use crate::simulation::core::components::*;
use crate::simulation::movement::constants::{MAX_SPEED, SEEKING, SLOW_ZONE_MULTIPLIER};
use bevy_ecs::prelude::*;

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
    for (position, mut acceleration, mut velocity, size, target, mut creature_state) in
        query.iter_mut()
    {
        if creature_state.behavior != BehaviorMode::Seeking {
            continue;
        }

        let to_target_x = target.x - position.x;
        let to_target_y = target.y - position.y;
        let center_distance = (to_target_x * to_target_x + to_target_y * to_target_y).sqrt();

        if center_distance < 0.001 {
            velocity.vx = 0.0;
            velocity.vy = 0.0;
            creature_state.behavior = BehaviorMode::Catatonic;
            continue;
        }

        let body_radius = size.radius() / 2.0;
        let arrival_radius = SEEKING.arrival_tolerance + body_radius;
        let slow_zone = arrival_radius * SLOW_ZONE_MULTIPLIER;
        let current_speed = (velocity.vx * velocity.vx + velocity.vy * velocity.vy).sqrt();

        if center_distance < (SEEKING.pounce_distance + body_radius)
            && current_speed < SEEKING.pounce_speed
        {
            velocity.vx = 0.0;
            velocity.vy = 0.0;
            creature_state.behavior = BehaviorMode::Catatonic;
            continue;
        }

        if center_distance < arrival_radius {
            acceleration.ax += -velocity.vx * SEEKING.brake_force;
            acceleration.ay += -velocity.vy * SEEKING.brake_force;
            continue;
        }

        let creature_max_speed = MAX_SPEED;

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
