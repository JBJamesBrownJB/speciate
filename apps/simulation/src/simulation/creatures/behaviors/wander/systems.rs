use super::constants::{
    BLEND_CENTER, COMFORT_RADIUS, HOMEWARD_FORCE, MAX_WANDER_DISTANCE, SIGMOID_STEEPNESS,
    WANDER_FORCE,
};
use crate::simulation::creatures::components::BehaviorMode;
use crate::simulation::math::{clamp_force, magnitude_sq, normalize};
use crate::simulation::queries::WanderQuery;
use rand::Rng;
use rayon::prelude::*;

pub fn territory_wandering_system(
    mut query: WanderQuery,
    #[cfg(feature = "dev-tools")] timings: bevy_ecs::system::Res<
        crate::instrumentation::SystemTimings,
    >,
) {
    #[cfg(feature = "dev-tools")]
    crate::time_system!(timings, "wander");

    // Collect entities for parallel processing
    let mut entities: Vec<_> = query.iter_mut().collect();

    entities.par_iter_mut().for_each(|(acceleration, wander_state, velocity, position, home, creature_state)| {
        if creature_state.behavior != BehaviorMode::Wandering {
            return;
        }

        // Thread-local RNG for parallel safety
        let mut rng = rand::thread_rng();

        let speed_sq = magnitude_sq(velocity.vx, velocity.vy);

        let (heading_x, heading_y) = if speed_sq < 0.0001 {
            let random_angle = rng.gen_range(0.0..std::f32::consts::TAU);
            let (sin_a, cos_a) = random_angle.sin_cos();
            (cos_a, sin_a)
        } else {
            normalize(velocity.vx, velocity.vy)
        };

        let circle_center_x = heading_x * wander_state.wander_distance;
        let circle_center_y = heading_y * wander_state.wander_distance;

        let angle_change = rng.gen_range(-wander_state.angle_change..wander_state.angle_change);
        wander_state.wander_angle += angle_change.to_radians();
        wander_state.wander_angle = wander_state.wander_angle.rem_euclid(std::f32::consts::TAU);

        let (sin_wander, cos_wander) = wander_state.wander_angle.sin_cos();
        let target_x = circle_center_x + wander_state.wander_radius * cos_wander;
        let target_y = circle_center_y + wander_state.wander_radius * sin_wander;

        let desired_vx = target_x;
        let desired_vy = target_y;

        let (norm_desired_x, norm_desired_y) = normalize(desired_vx, desired_vy);

        let max_speed = creature_state.max_speed;
        let scaled_desired_x = norm_desired_x * max_speed;
        let scaled_desired_y = norm_desired_y * max_speed;

        let steer_x = scaled_desired_x - velocity.vx;
        let steer_y = scaled_desired_y - velocity.vy;

        let wander_force = clamp_force(steer_x, steer_y, WANDER_FORCE);

        let distance_from_home = home.distance_from(position.x, position.y);

        let to_home_x = home.x - position.x;
        let to_home_y = home.y - position.y;

        let (norm_to_home_x, norm_to_home_y) = normalize(to_home_x, to_home_y);

        let urgency = (distance_from_home / MAX_WANDER_DISTANCE).min(1.0);

        let homeward_force_magnitude = HOMEWARD_FORCE * urgency;
        let homeward_force = (
            norm_to_home_x * homeward_force_magnitude,
            norm_to_home_y * homeward_force_magnitude,
        );

        let blend = calculate_territory_blend(distance_from_home, COMFORT_RADIUS, BLEND_CENTER);
        let final_force = blend_forces(wander_force, homeward_force, blend);

        if final_force.0.is_finite() && final_force.1.is_finite() {
            acceleration.ax += final_force.0;
            acceleration.ay += final_force.1;
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_territory_blend_near_home() {
        let blend = calculate_territory_blend(5.0, 10.0, 20.0);
        assert!(blend < 0.2, "Near home should favor wandering, got blend={}", blend);
    }

    #[test]
    fn test_territory_blend_at_center() {
        let blend = calculate_territory_blend(20.0, 10.0, 20.0);
        assert!(blend > 0.4 && blend < 0.6, "At blend center should be ~0.5, got {}", blend);
    }

    #[test]
    fn test_territory_blend_far_from_home() {
        let blend = calculate_territory_blend(35.0, 10.0, 20.0);
        assert!(blend > 0.8, "Far from home should favor seeking, got blend={}", blend);
    }

    #[test]
    fn test_territory_blend_nan_safety() {
        assert_eq!(calculate_territory_blend(f32::NAN, 10.0, 20.0), 0.5);
        assert_eq!(calculate_territory_blend(10.0, 0.0, 20.0), 0.5);
        assert_eq!(calculate_territory_blend(10.0, -5.0, 20.0), 0.5);
        assert_eq!(calculate_territory_blend(10.0, 10.0, 0.0), 0.5);
        assert_eq!(calculate_territory_blend(10.0, 10.0, -5.0), 0.5);
    }

    #[test]
    fn test_blend_forces_zero_blend() {
        let force_a = (10.0, 5.0);
        let force_b = (20.0, 15.0);
        let result = blend_forces(force_a, force_b, 0.0);
        assert_eq!(result, force_a);
    }

    #[test]
    fn test_blend_forces_full_blend() {
        let force_a = (10.0, 5.0);
        let force_b = (20.0, 15.0);
        let result = blend_forces(force_a, force_b, 1.0);
        assert_eq!(result, force_b);
    }

    #[test]
    fn test_blend_forces_half_blend() {
        let force_a = (10.0, 0.0);
        let force_b = (20.0, 10.0);
        let result = blend_forces(force_a, force_b, 0.5);
        assert_eq!(result, (15.0, 5.0));
    }

    #[test]
    fn test_blend_forces_nan_safety() {
        let result = blend_forces((f32::NAN, 5.0), (10.0, 10.0), 0.5);
        assert!(result.0.is_finite() && result.1.is_finite());
    }
}

pub fn calculate_territory_blend(
    distance_from_home: f32,
    comfort_radius: f32,
    blend_center: f32,
) -> f32 {
    if !distance_from_home.is_finite() || comfort_radius <= 0.0 || blend_center <= 0.0 {
        return 0.5;
    }

    // Sigmoid blend between wander (free roam) and homeward (territory bound)
    // Steepness controls how quickly the blend transitions
    let normalized = (distance_from_home - blend_center) / comfort_radius;
    let sigmoid = 1.0 / (1.0 + (-SIGMOID_STEEPNESS * normalized).exp());
    sigmoid.clamp(0.0, 1.0)
}

pub fn blend_forces(force_a: (f32, f32), force_b: (f32, f32), blend: f32) -> (f32, f32) {
    if !force_a.0.is_finite()
        || !force_a.1.is_finite()
        || !force_b.0.is_finite()
        || !force_b.1.is_finite()
        || !blend.is_finite()
    {
        return (0.0, 0.0);
    }

    let blend_clamped = blend.clamp(0.0, 1.0);
    let x = force_a.0 * (1.0 - blend_clamped) + force_b.0 * blend_clamped;
    let y = force_a.1 * (1.0 - blend_clamped) + force_b.1 * blend_clamped;

    (x, y)
}
