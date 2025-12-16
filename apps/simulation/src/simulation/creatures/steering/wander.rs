//! Wander steering behavior pure function.
//!
//! This module provides a testable pure function for Reynolds wander steering,
//! separate from the ECS system. This fixes the F=ma bug where steering (m/s)
//! was being treated as force (N).

use crate::simulation::math::{magnitude_sq, normalize, steering_to_acceleration, SteeringContext};

/// Parameters for wander behavior calculation.
#[derive(Debug, Clone, Copy)]
pub struct WanderParams {
    /// Current wander angle (radians)
    pub wander_angle: f32,
    /// Radius of the wander circle (meters)
    pub wander_radius: f32,
    /// Distance to project wander circle ahead (meters)
    pub wander_distance: f32,
    /// Force multiplier (0.0-1.0, fraction of max_force to use)
    pub force_multiplier: f32,
}

impl Default for WanderParams {
    fn default() -> Self {
        Self {
            wander_angle: 0.0,
            wander_radius: 10.0,
            wander_distance: 50.0,
            force_multiplier: 0.1, // 10% of max force for low-effort exploration
        }
    }
}

/// Result of wander calculation.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct WanderResult {
    /// Acceleration to apply (m/s²) - properly converted from force
    pub acceleration: (f32, f32),
    /// Updated wander angle after random adjustment (radians)
    pub new_wander_angle: f32,
}

/// Calculate wander steering acceleration using Reynolds wander behavior.
///
/// Algorithm:
/// 1. Get current heading from velocity (or wander angle if stopped)
/// 2. Project wander circle ahead of creature
/// 3. Apply random angle change to wander angle
/// 4. Pick target point on wander circle
/// 5. Calculate steering: desired_velocity - current_velocity
/// 6. Convert to acceleration with F=ma: clamp to (max_force × multiplier) / mass
///
/// CRITICAL: This function correctly converts the steering velocity difference
/// to acceleration, fixing the bug where velocity (m/s) was treated as force (N).
pub fn calculate_wander(
    ctx: &SteeringContext,
    wander: &WanderParams,
    angle_change_radians: f32,
) -> WanderResult {
    let (vx, vy) = ctx.velocity;

    // 1. Get current heading direction
    let speed_sq = magnitude_sq(vx, vy);
    let (heading_x, heading_y) = if speed_sq < 0.0001 {
        // Stationary: use wander angle as heading
        let (sin_a, cos_a) = wander.wander_angle.sin_cos();
        (cos_a, sin_a)
    } else {
        normalize(vx, vy)
    };

    // 2. Project wander circle ahead
    let circle_center_x = heading_x * wander.wander_distance;
    let circle_center_y = heading_y * wander.wander_distance;

    // 3. Apply random angle change
    let new_wander_angle =
        (wander.wander_angle + angle_change_radians).rem_euclid(std::f32::consts::TAU);

    // 4. Pick target point on wander circle
    let (sin_wander, cos_wander) = new_wander_angle.sin_cos();
    let target_x = circle_center_x + wander.wander_radius * cos_wander;
    let target_y = circle_center_y + wander.wander_radius * sin_wander;

    // 5. Calculate steering: desired_velocity - current_velocity
    let (norm_target_x, norm_target_y) = normalize(target_x, target_y);
    let desired_vx = norm_target_x * ctx.max_speed;
    let desired_vy = norm_target_y * ctx.max_speed;

    let steer_x = desired_vx - vx;
    let steer_y = desired_vy - vy;

    // 6. CORRECT F=ma CONVERSION
    // Create a scaled context with the wander force limit
    // max_accel = (max_force × multiplier) / mass
    let wander_max_force = ctx.max_force * wander.force_multiplier;
    let scaled_ctx = SteeringContext {
        max_force: wander_max_force,
        ..*ctx
    };

    let (ax, ay) = steering_to_acceleration((steer_x, steer_y), &scaled_ctx);

    // Validate output
    let (final_ax, final_ay) = if ax.is_finite() && ay.is_finite() {
        (ax, ay)
    } else {
        (0.0, 0.0)
    };

    WanderResult {
        acceleration: (final_ax, final_ay),
        new_wander_angle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Reference values for a typical creature
    const DEFAULT_MAX_FORCE: f32 = 390.0; // Newtons
    const DEFAULT_MASS: f32 = 65.0; // kg
    const DEFAULT_MAX_SPEED: f32 = 15.0; // m/s
    const WANDER_FORCE_MULT: f32 = 0.1; // 10% of max force

    fn default_context() -> SteeringContext {
        SteeringContext {
            velocity: (5.0, 0.0), // Moving right at 5 m/s
            max_speed: DEFAULT_MAX_SPEED,
            max_force: DEFAULT_MAX_FORCE,
            mass: DEFAULT_MASS,
        }
    }

    fn default_wander() -> WanderParams {
        WanderParams {
            wander_angle: 0.0,
            wander_radius: 10.0,
            wander_distance: 50.0,
            force_multiplier: WANDER_FORCE_MULT,
        }
    }

    // ============================================================
    // F=ma bug fix verification tests
    // ============================================================

    #[test]
    fn wander_returns_acceleration_not_force() {
        let ctx = default_context();
        let wander = default_wander();

        let result = calculate_wander(&ctx, &wander, 0.0);
        let accel_mag =
            (result.acceleration.0.powi(2) + result.acceleration.1.powi(2)).sqrt();

        // Maximum acceleration for wander = (max_force × 0.1) / mass
        // = (390 × 0.1) / 65 = 39 / 65 = 0.6 m/s²
        let max_wander_accel = (DEFAULT_MAX_FORCE * WANDER_FORCE_MULT) / DEFAULT_MASS;

        assert!(
            accel_mag <= max_wander_accel + 0.01,
            "Wander acceleration {} m/s² should be ≤ {} m/s² (max_force × mult / mass). \
             If this is ~39 m/s², the F=ma bug is present!",
            accel_mag,
            max_wander_accel
        );
    }

    #[test]
    fn wander_bug_would_produce_wrong_acceleration() {
        // This test documents what the bug produces vs correct behavior
        let ctx = default_context();

        // BUG: Treating steering as force directly
        // max_wander_force = 390 × 0.1 = 39 N
        // BUG treats this as 39 m/s² acceleration
        let buggy_max_accel = DEFAULT_MAX_FORCE * WANDER_FORCE_MULT;

        // CORRECT: max_accel = force / mass = 39 / 65 = 0.6 m/s²
        let correct_max_accel = (DEFAULT_MAX_FORCE * WANDER_FORCE_MULT) / DEFAULT_MASS;

        assert!(
            (buggy_max_accel / correct_max_accel - DEFAULT_MASS).abs() < 0.1,
            "Bug causes {}x overscaling (mass = {}kg)",
            buggy_max_accel / correct_max_accel,
            DEFAULT_MASS
        );

        // Verify our implementation uses the correct value
        let wander = default_wander();
        let result = calculate_wander(&ctx, &wander, 0.0);
        let accel_mag =
            (result.acceleration.0.powi(2) + result.acceleration.1.powi(2)).sqrt();

        assert!(
            accel_mag < buggy_max_accel,
            "Acceleration {} should be much less than buggy value {}",
            accel_mag,
            buggy_max_accel
        );
    }

    // ============================================================
    // Basic wander behavior tests
    // ============================================================

    #[test]
    fn wander_produces_steering() {
        let ctx = default_context();
        let wander = default_wander();

        let result = calculate_wander(&ctx, &wander, 0.0);
        let accel_mag =
            (result.acceleration.0.powi(2) + result.acceleration.1.powi(2)).sqrt();

        assert!(
            accel_mag > 0.0,
            "Wander should produce some steering acceleration"
        );
    }

    #[test]
    fn wander_angle_updates() {
        let wander = default_wander();
        let ctx = default_context();

        let angle_change = 0.1; // 0.1 radians
        let result = calculate_wander(&ctx, &wander, angle_change);

        let expected_angle = (wander.wander_angle + angle_change).rem_euclid(std::f32::consts::TAU);

        assert!(
            (result.new_wander_angle - expected_angle).abs() < 0.001,
            "Wander angle should update: {} vs expected {}",
            result.new_wander_angle,
            expected_angle
        );
    }

    #[test]
    fn wander_angle_wraps_around() {
        let wander = WanderParams {
            wander_angle: std::f32::consts::TAU - 0.1, // Near 360°
            ..default_wander()
        };
        let ctx = default_context();

        let result = calculate_wander(&ctx, &wander, 0.2);

        // Should wrap around to small positive angle
        assert!(
            result.new_wander_angle >= 0.0 && result.new_wander_angle < std::f32::consts::TAU,
            "Wander angle should wrap: {}",
            result.new_wander_angle
        );
    }

    // ============================================================
    // Stationary creature tests
    // ============================================================

    #[test]
    fn stationary_creature_uses_wander_angle_for_heading() {
        let ctx = SteeringContext {
            velocity: (0.0, 0.0), // Stationary
            max_speed: 15.0,
            max_force: 390.0,
            mass: 65.0,
        };
        let wander = WanderParams {
            wander_angle: std::f32::consts::PI / 2.0, // 90° (pointing up)
            ..default_wander()
        };

        let result = calculate_wander(&ctx, &wander, 0.0);

        // Stationary creature should start moving in wander direction
        // Since wander_angle is 90° and target is projected from that heading,
        // the acceleration should have a significant upward component
        let accel_mag =
            (result.acceleration.0.powi(2) + result.acceleration.1.powi(2)).sqrt();
        assert!(accel_mag > 0.0, "Stationary creature should still steer");
    }

    // ============================================================
    // Force multiplier tests
    // ============================================================

    #[test]
    fn higher_force_multiplier_allows_more_acceleration() {
        let ctx = default_context();

        let low_mult = WanderParams {
            force_multiplier: 0.1,
            ..default_wander()
        };
        let high_mult = WanderParams {
            force_multiplier: 0.5,
            ..default_wander()
        };

        let result_low = calculate_wander(&ctx, &low_mult, 0.0);
        let result_high = calculate_wander(&ctx, &high_mult, 0.0);

        let max_accel_low = (DEFAULT_MAX_FORCE * 0.1) / DEFAULT_MASS;
        let max_accel_high = (DEFAULT_MAX_FORCE * 0.5) / DEFAULT_MASS;

        let accel_low =
            (result_low.acceleration.0.powi(2) + result_low.acceleration.1.powi(2)).sqrt();
        let accel_high =
            (result_high.acceleration.0.powi(2) + result_high.acceleration.1.powi(2)).sqrt();

        assert!(
            accel_low <= max_accel_low + 0.01,
            "Low mult accel {} should be ≤ {}",
            accel_low,
            max_accel_low
        );
        assert!(
            accel_high <= max_accel_high + 0.01,
            "High mult accel {} should be ≤ {}",
            accel_high,
            max_accel_high
        );
    }

    // ============================================================
    // Edge case tests
    // ============================================================

    #[test]
    fn handles_zero_wander_distance() {
        let ctx = default_context();
        let wander = WanderParams {
            wander_distance: 0.0, // Degenerate case
            ..default_wander()
        };

        let result = calculate_wander(&ctx, &wander, 0.0);

        // Should not panic, acceleration should be finite
        assert!(
            result.acceleration.0.is_finite() && result.acceleration.1.is_finite(),
            "Should handle zero distance gracefully"
        );
    }

    #[test]
    fn handles_zero_wander_radius() {
        let ctx = default_context();
        let wander = WanderParams {
            wander_radius: 0.0, // Degenerate case
            ..default_wander()
        };

        let result = calculate_wander(&ctx, &wander, 0.0);

        // Should not panic, acceleration should be finite
        assert!(
            result.acceleration.0.is_finite() && result.acceleration.1.is_finite(),
            "Should handle zero radius gracefully"
        );
    }

    #[test]
    fn handles_zero_mass() {
        let ctx = SteeringContext {
            velocity: (5.0, 0.0),
            max_speed: 15.0,
            max_force: 390.0,
            mass: 0.0, // Zero mass
        };
        let wander = default_wander();

        let result = calculate_wander(&ctx, &wander, 0.0);

        // Zero mass should return zero acceleration (division protection)
        assert_eq!(
            result.acceleration,
            (0.0, 0.0),
            "Zero mass should return zero acceleration"
        );
    }

    // ============================================================
    // Integration test: Multi-frame simulation
    // ============================================================

    #[test]
    fn wander_produces_varied_directions_over_time() {
        let mut wander = default_wander();
        let ctx = default_context();

        let mut unique_angles = std::collections::HashSet::new();

        // Simulate 20 frames with random angle changes
        for i in 0..20 {
            let angle_change = (i as f32 * 0.1).sin() * 0.2; // Varied changes
            let result = calculate_wander(&ctx, &wander, angle_change);

            // Track unique acceleration directions
            let angle_bucket = (result.acceleration.1.atan2(result.acceleration.0) * 10.0) as i32;
            unique_angles.insert(angle_bucket);

            // Update wander state for next frame
            wander.wander_angle = result.new_wander_angle;
        }

        // Should produce varied directions (organic wandering)
        assert!(
            unique_angles.len() > 3,
            "Wander should produce varied directions over time, got {} unique",
            unique_angles.len()
        );
    }
}
