//! Wandering behavior system - Reynolds steering algorithm
//!
//! Implements smooth, organic wandering behavior using circle projection method
//! from "The Nature of Code" by Dan Shiffman and Reynolds steering behaviors.
//!
//! Follows the force accumulation pattern: ADDs to Acceleration.

use crate::simulation::components::*;
use crate::simulation::core::components::*;
use crate::simulation::movement::{STEERING, TERRITORY};
use bevy_ecs::prelude::*;
use rand::Rng;

/// Territory-based wandering with elastic tether to home
///
/// Hybrid force blending algorithm (elastic tether model):
/// 1. Calculate Reynolds wandering force (smooth random exploration)
/// 2. Calculate homeward seeking force (pull toward territory center)
/// 3. Blend forces using sigmoid curve based on distance from home
/// 4. ADD blended force to acceleration (force accumulation pattern)
///
/// Biological rationale (from movement ecology research - docs/biology/biology-notes.md):
/// - Animals don't wander randomly - they patrol territories with soft boundaries
/// - "Elastic tether" model: exploration freedom near home, urgency when far
/// - Sigmoid transition creates smooth behavioral shift (not hard threshold)
/// - Composite movement strategies are the norm in territorial species
///
/// Force blending strategy:
/// - Near home (0-10m): 90% wandering, 10% homeward (free exploration)
/// - Mid-range (10-20m): 50% wandering, 50% homeward (balanced patrol)
/// - Far from home (20-30m): 10% wandering, 90% homeward (emergency return)
///
/// Parameters (from zoologist consultation - docs/biology/biology-notes.md 2025-11-08):
/// - COMFORT_RADIUS: 10m (territory core, low home bias)
/// - BLEND_CENTER: 20m (50% blend point, patrol boundary)
/// - MAX_WANDER_DISTANCE: 30m (hard limit, maximum excursion)
/// - WANDER_FORCE_MAGNITUDE: 5.0 (gentle exploration)
/// - SEEK_FORCE_MAGNITUDE: 50.0 (strong homeward pull when needed)
///
/// TODO(DNA Future DNA system): Derive all parameters from DNA genes
/// - comfort_radius_multiplier (metabolic needs scale territory)
/// - exploration_bias (bold vs cautious personalities)
/// - stress_response (expand/contract under starvation, fleeing)
#[allow(clippy::type_complexity)]
pub fn territory_wandering_system(
    mut query: Query<
        (
            &mut Acceleration,
            &mut WanderState,
            &Velocity,
            &Position,
            &HomePosition,
            &CreatureState,
        ),
        With<CanWander>,
    >,
) {
    // Territory parameters from global constants (TODO: DNA Future DNA system)

    let mut rng = rand::thread_rng();

    for (mut acceleration, mut wander_state, velocity, position, home, creature_state) in
        query.iter_mut()
    {
        // Only apply when in Wandering behavior mode
        if creature_state.behavior != BehaviorMode::Wandering {
            continue;
        }

        // ===== PART 1: Calculate Reynolds Wandering Force =====

        // Get current heading from velocity
        let speed = (velocity.vx * velocity.vx + velocity.vy * velocity.vy).sqrt();

        // If stationary, use random heading to bootstrap movement
        let (heading_x, heading_y) = if speed < 0.01 {
            let random_angle = rng.gen_range(0.0..std::f32::consts::TAU);
            (random_angle.cos(), random_angle.sin())
        } else {
            // Normalize velocity to get heading direction
            (velocity.vx / speed, velocity.vy / speed)
        };

        // Project circle center ahead of creature
        let circle_center_x = heading_x * wander_state.wander_distance;
        let circle_center_y = heading_y * wander_state.wander_distance;

        // Randomly adjust wander angle (small displacement for smooth turning)
        let angle_change = rng.gen_range(-wander_state.angle_change..wander_state.angle_change);
        wander_state.wander_angle += angle_change.to_radians();
        wander_state.wander_angle = wander_state.wander_angle.rem_euclid(std::f32::consts::TAU);

        // Calculate point on circle perimeter
        let target_x =
            circle_center_x + wander_state.wander_radius * wander_state.wander_angle.cos();
        let target_y =
            circle_center_y + wander_state.wander_radius * wander_state.wander_angle.sin();

        // Calculate steering force toward wander point
        let desired_vx = target_x;
        let desired_vy = target_y;

        // Normalize desired velocity
        let desired_length = (desired_vx * desired_vx + desired_vy * desired_vy).sqrt();
        let (norm_desired_x, norm_desired_y) = if desired_length > 0.01 {
            (desired_vx / desired_length, desired_vy / desired_length)
        } else {
            (0.0, 0.0)
        };

        // Scale to max speed
        let max_speed = creature_state.max_speed;
        let scaled_desired_x = norm_desired_x * max_speed;
        let scaled_desired_y = norm_desired_y * max_speed;

        // Steering force = desired - current velocity
        let steer_x = scaled_desired_x - velocity.vx;
        let steer_y = scaled_desired_y - velocity.vy;

        // Limit steering force magnitude
        let steer_magnitude = (steer_x * steer_x + steer_y * steer_y).sqrt();
        let wander_force = if steer_magnitude > STEERING.wander_force {
            let scale = STEERING.wander_force / steer_magnitude;
            (steer_x * scale, steer_y * scale)
        } else {
            (steer_x, steer_y)
        };

        // ===== PART 2: Calculate Homeward Seeking Force =====

        let distance_from_home = home.distance_from(position.x, position.y);

        // Direction to home (normalized)
        let to_home_x = home.x - position.x;
        let to_home_y = home.y - position.y;
        let to_home_dist = (to_home_x * to_home_x + to_home_y * to_home_y).sqrt();

        let (norm_to_home_x, norm_to_home_y) = if to_home_dist > 0.01 {
            (to_home_x / to_home_dist, to_home_y / to_home_dist)
        } else {
            (0.0, 0.0) // Already at home
        };

        // Urgency factor: increases as creature approaches max wander distance
        let urgency = (distance_from_home / TERRITORY.max_wander_distance).min(1.0);

        // Homeward force magnitude scales with urgency
        let homeward_force_magnitude = TERRITORY.homeward_force * urgency;
        let homeward_force = (
            norm_to_home_x * homeward_force_magnitude,
            norm_to_home_y * homeward_force_magnitude,
        );

        // ===== PART 3: Blend Forces Based on Distance =====

        let blend = calculate_territory_blend(distance_from_home, TERRITORY.comfort_radius, TERRITORY.blend_center);
        let final_force = blend_forces(wander_force, homeward_force, blend);

        // ===== PART 4: Add to Acceleration (Force Accumulation) =====

        if final_force.0.is_finite() && final_force.1.is_finite() {
            acceleration.ax += final_force.0;
            acceleration.ay += final_force.1;
        }
    }
}

// REMOVED: wander_target_selection_system
//
// This system was removed as part of Sprint 6 territory wandering refactor.
// It selected random homeward-biased targets, but the wander_system ignored them
// (it only used Reynolds steering without reading the Target component).
//
// REPLACED BY: territory_wandering_system (hybrid force blending)
// The new system blends Reynolds wandering with direct homeward seeking forces,
// eliminating the need for explicit target selection. The elastic tether model
// provides smoother, more biologically realistic territory behavior.
//
// For details see:
// - docs/biology/biology-notes.md (2025-11-08 Territory-Based Wandering Refactor)
// - territory_wandering_system() function above

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_territory_blend_near_home() {
        // Inside comfort zone (0-10m) should have low blend (mostly wandering)
        let blend = calculate_territory_blend(5.0, 10.0, 20.0);
        assert!(blend < 0.2, "Near home should favor wandering, got blend={}", blend);
    }

    #[test]
    fn test_territory_blend_at_center() {
        // At blend center (20m) should be ~50% blended
        let blend = calculate_territory_blend(20.0, 10.0, 20.0);
        assert!(blend > 0.4 && blend < 0.6, "At blend center should be ~0.5, got {}", blend);
    }

    #[test]
    fn test_territory_blend_far_from_home() {
        // Beyond max wander distance (30m+) should have high blend (mostly seeking)
        let blend = calculate_territory_blend(35.0, 10.0, 20.0);
        assert!(blend > 0.8, "Far from home should favor seeking, got blend={}", blend);
    }

    #[test]
    fn test_territory_blend_nan_safety() {
        // Should return safe default (0.5) for invalid inputs
        assert_eq!(calculate_territory_blend(f32::NAN, 10.0, 20.0), 0.5);
        assert_eq!(calculate_territory_blend(10.0, 0.0, 20.0), 0.5);
        assert_eq!(calculate_territory_blend(10.0, -5.0, 20.0), 0.5);
        assert_eq!(calculate_territory_blend(10.0, 10.0, 0.0), 0.5);
        assert_eq!(calculate_territory_blend(10.0, 10.0, -5.0), 0.5);
    }

    #[test]
    fn test_blend_forces_zero_blend() {
        // 0% blend should return force_a
        let force_a = (10.0, 5.0);
        let force_b = (20.0, 15.0);
        let result = blend_forces(force_a, force_b, 0.0);
        assert_eq!(result, force_a);
    }

    #[test]
    fn test_blend_forces_full_blend() {
        // 100% blend should return force_b
        let force_a = (10.0, 5.0);
        let force_b = (20.0, 15.0);
        let result = blend_forces(force_a, force_b, 1.0);
        assert_eq!(result, force_b);
    }

    #[test]
    fn test_blend_forces_half_blend() {
        // 50% blend should return average
        let force_a = (10.0, 0.0);
        let force_b = (20.0, 10.0);
        let result = blend_forces(force_a, force_b, 0.5);
        assert_eq!(result, (15.0, 5.0));
    }

    #[test]
    fn test_blend_forces_nan_safety() {
        // Should handle NaN gracefully
        let result = blend_forces((f32::NAN, 5.0), (10.0, 10.0), 0.5);
        assert!(result.0.is_finite() && result.1.is_finite());
    }
}

/// Calculate blend factor between wandering and homeward seeking
///
/// Uses sigmoid curve for smooth transition:
/// - Near home (< comfort_radius): Low blend (mostly wandering)
/// - At blend_center: 50% blend
/// - Far from home (> blend_center): High blend (mostly seeking)
///
/// # Arguments
/// * `distance_from_home` - Current distance from home position (meters)
/// * `comfort_radius` - Territory comfort zone radius (meters)
/// * `blend_center` - Distance where blend reaches 50% (meters)
///
/// # Returns
/// Blend factor in range [0.0, 1.0]:
/// - 0.0 = 100% wandering, 0% seeking
/// - 0.5 = 50% wandering, 50% seeking
/// - 1.0 = 0% wandering, 100% seeking
///
/// # Safety
/// Returns 0.5 (neutral blend) for any invalid inputs (NaN, negative, zero)
pub fn calculate_territory_blend(
    distance_from_home: f32,
    comfort_radius: f32,
    blend_center: f32,
) -> f32 {
    // Guard against invalid inputs
    if !distance_from_home.is_finite() || comfort_radius <= 0.0 || blend_center <= 0.0 {
        return 0.5; // Neutral blend for invalid inputs
    }

    // Sigmoid parameters (from zoologist consultation - docs/biology/biology-notes.md)
    // Steepness determines transition sharpness:
    // - Low k (0.1-0.5): Gradual transition over wide range
    // - High k (1.0-3.0): Sharp transition near center
    // Using k=1.5 for biologically realistic "elastic tether" behavior (from TERRITORY.sigmoid_steepness)

    // Normalize distance relative to blend center and comfort zone
    let normalized = (distance_from_home - blend_center) / comfort_radius;

    // Sigmoid function: 1 / (1 + e^(-k*x))
    let sigmoid = 1.0 / (1.0 + (-TERRITORY.sigmoid_steepness * normalized).exp());

    // Clamp to [0, 1] for safety
    sigmoid.clamp(0.0, 1.0)
}

/// Blend two force vectors using linear interpolation
///
/// # Arguments
/// * `force_a` - First force vector (wander force)
/// * `force_b` - Second force vector (homeward force)
/// * `blend` - Blend factor [0.0, 1.0]
///
/// # Returns
/// Blended force: (1 - blend) * force_a + blend * force_b
///
/// # Safety
/// Returns zero force if any component is NaN
pub fn blend_forces(force_a: (f32, f32), force_b: (f32, f32), blend: f32) -> (f32, f32) {
    // Guard against NaN
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
