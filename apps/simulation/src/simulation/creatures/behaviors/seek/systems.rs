use crate::simulation::creatures::constants::{
    ARRIVAL_THRESHOLD, BRAKE_FORCE_MULT, MAX_SPEED, MIN_SLOW_ZONE_BODY_LENGTHS,
    POUNCE_SPEED, POUNCE_THRESHOLD, SEEK_FORCE_MULT, TTC_SLOW_THRESHOLD, TTC_STOP_THRESHOLD,
};
use crate::simulation::creatures::components::BehaviorMode;
use crate::simulation::math::{clamp_force, magnitude_sq};
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

    for (position, mut acceleration, velocity, size, target, mut creature_state) in query.iter_mut()
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

        let current_speed_sq = magnitude_sq(velocity.vx, velocity.vy);
        let current_speed = current_speed_sq.sqrt();

        // Pounce: Snap to target when very close AND moving slowly
        if edge_distance < POUNCE_THRESHOLD && current_speed < POUNCE_SPEED {
            creature_state.behavior = BehaviorMode::Catatonic;
            continue;
        }

        // Brake: Apply counter-force when approaching edge contact
        if edge_distance < ARRIVAL_THRESHOLD {
            let brake_force = size.max_force() * BRAKE_FORCE_MULT.get();
            acceleration.ax += -velocity.vx * brake_force;
            acceleration.ay += -velocity.vy * brake_force;
            continue;
        }

        let creature_max_speed = MAX_SPEED;

        // =================================================================
        // TIME-TO-CONTACT (TTC) DECELERATION
        // =================================================================
        // Biologically validated: animals compute τ = distance/closing_velocity
        // and modulate approach speed based on time remaining to contact.

        // Normalize direction to target
        let dir_x = to_target_x / center_distance;
        let dir_y = to_target_y / center_distance;

        // Closing velocity = velocity component toward target (dot product)
        let closing_velocity = velocity.vx * dir_x + velocity.vy * dir_y;

        // Compute time-to-contact (τ)
        let ttc = if closing_velocity > 0.1 {
            edge_distance / closing_velocity
        } else {
            // Not approaching or moving slowly - use distance-based fallback
            f32::INFINITY
        };

        // Minimum slow zone based on body size (fallback when TTC undefined)
        let min_slow_distance = size.length * MIN_SLOW_ZONE_BODY_LENGTHS;

        // Desired speed based on TTC
        let desired_speed = if ttc < TTC_STOP_THRESHOLD {
            // Very close - target zero velocity
            0.0
        } else if ttc < TTC_SLOW_THRESHOLD {
            // In deceleration zone - linear interpolation
            let t = (ttc - TTC_STOP_THRESHOLD) / (TTC_SLOW_THRESHOLD - TTC_STOP_THRESHOLD);
            creature_max_speed * t
        } else if edge_distance < min_slow_distance {
            // Fallback: within minimum slow zone but TTC says full speed
            // Use distance-based scaling for precision approaches
            let t = edge_distance / min_slow_distance;
            creature_max_speed * t.clamp(0.0, 1.0)
        } else {
            // Outside all slow zones - full speed
            creature_max_speed
        };

        let desired_vx = dir_x * desired_speed;
        let desired_vy = dir_y * desired_speed;
        let steer_x = desired_vx - velocity.vx;
        let steer_y = desired_vy - velocity.vy;

        let seek_max_force = size.max_force() * SEEK_FORCE_MULT.get();
        let (clamped_x, clamped_y) = clamp_force(steer_x, steer_y, seek_max_force);
        acceleration.ax += clamped_x;
        acceleration.ay += clamped_y;
    }
}
