use super::constants::{
    ARRIVAL_THRESHOLD, BRAKE_FORCE, MAX_FORCE, POUNCE_SPEED, POUNCE_THRESHOLD, SLOW_ZONE_DECAY,
    SLOW_ZONE_DECAY_EXP,
};
use crate::simulation::creatures::components::BehaviorMode;
use crate::simulation::math::{clamp_force, magnitude_sq};
use crate::simulation::movement::constants::{MAX_SPEED, SLOW_ZONE_MULTIPLIER};
use crate::simulation::queries::SeekQuery;
#[cfg(feature = "dev-tools")]
use crate::instrumentation::SystemTimings;
#[cfg(feature = "dev-tools")]
use bevy_ecs::system::Res;

pub fn seek_system(
    mut query: SeekQuery,
    #[cfg(feature = "dev-tools")] timings: Res<SystemTimings>,
) {
    #[cfg(feature = "dev-tools")]
    crate::time_system!(timings, "seek");

    for (position, mut acceleration, velocity, size, target, mut creature_state) in
        query.iter_mut()
    {
        if creature_state.behavior != BehaviorMode::Seeking {
            continue;
        }

        let to_target_x = target.x - position.x;
        let to_target_y = target.y - position.y;
        let center_distance_sq = magnitude_sq(to_target_x, to_target_y);

        if center_distance_sq < 0.000001 {
            creature_state.behavior = BehaviorMode::Catatonic;
            continue;
        }

        let self_radius = size.radius();
        let target_radius = target.radius.get();
        let center_distance = center_distance_sq.sqrt();

        // Edge-to-edge distance (how far apart the surfaces are)
        let edge_distance = center_distance - self_radius - target_radius;

        let slow_zone = (ARRIVAL_THRESHOLD + self_radius + target_radius) * SLOW_ZONE_MULTIPLIER;

        let current_speed_sq = magnitude_sq(velocity.vx, velocity.vy);

        // Pounce: Snap to target when very close AND moving slowly
        if edge_distance < POUNCE_THRESHOLD
            && current_speed_sq < POUNCE_SPEED * POUNCE_SPEED
        {
            creature_state.behavior = BehaviorMode::Catatonic;
            continue;
        }

        // Brake: Apply counter-force when approaching edge contact
        if edge_distance < ARRIVAL_THRESHOLD {
            acceleration.ax += -velocity.vx * BRAKE_FORCE;
            acceleration.ay += -velocity.vy * BRAKE_FORCE;
            continue;
        }

        let creature_max_speed = MAX_SPEED;

        // Calculate desired speed based on distance (decelerate in slow zone)
        let arrival_radius = ARRIVAL_THRESHOLD + self_radius + target_radius;
        let desired_speed = if center_distance > slow_zone {
            creature_max_speed
        } else {
            let slow_zone_distance = slow_zone - arrival_radius;
            let distance_into_zone = center_distance - arrival_radius;
            let ratio = distance_into_zone / slow_zone_distance;
            // Exponential decay in slow zone: speed decreases smoothly as we approach target
            creature_max_speed * (SLOW_ZONE_DECAY * ratio).exp() / SLOW_ZONE_DECAY_EXP
        };

        let desired_vx = (to_target_x / center_distance) * desired_speed;
        let desired_vy = (to_target_y / center_distance) * desired_speed;
        let steer_x = desired_vx - velocity.vx;
        let steer_y = desired_vy - velocity.vy;

        let (clamped_x, clamped_y) = clamp_force(steer_x, steer_y, MAX_FORCE);
        acceleration.ax += clamped_x;
        acceleration.ay += clamped_y;
    }
}
