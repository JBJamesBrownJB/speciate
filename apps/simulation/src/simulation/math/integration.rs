//! Physics integration pure functions.
//!
//! This module provides testable pure functions for integrating motion,
//! separate from the ECS system. This allows comprehensive unit testing
//! and scenario replay without the Bevy runtime.

use super::{fast_atan2, normalize_angle};

/// Parameters for motion integration.
/// All values use SI units (meters, seconds, radians).
#[derive(Debug, Clone, Copy)]
pub struct IntegrationParams {
    /// Current position (m)
    pub position: (f32, f32),
    /// Current velocity (m/s)
    pub velocity: (f32, f32),
    /// Acceleration to apply (m/s²)
    pub acceleration: (f32, f32),
    /// Time step (seconds)
    pub dt: f32,
    /// Drag coefficient for velocity damping
    pub drag_coefficient: f32,
    /// Maximum speed (m/s)
    pub max_speed: f32,
    /// Maximum turn rate (radians/second)
    pub max_turn_rate_rad: f32,
    /// Threshold below which creature is considered stopped (m/s)
    pub stopped_threshold: f32,
}

impl Default for IntegrationParams {
    fn default() -> Self {
        Self {
            position: (0.0, 0.0),
            velocity: (0.0, 0.0),
            acceleration: (0.0, 0.0),
            dt: 0.05, // 20Hz
            drag_coefficient: 2.0,
            max_speed: 15.0,
            max_turn_rate_rad: std::f32::consts::PI, // 180 deg/s
            stopped_threshold: 0.05,
        }
    }
}

/// Result of motion integration.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct IntegrationResult {
    /// New position (m)
    pub position: (f32, f32),
    /// New velocity (m/s)
    pub velocity: (f32, f32),
    /// Whether turn rate limiting was applied (for debugging visualization mismatch)
    pub turn_limited: bool,
    /// Whether speed clamping was applied
    pub speed_clamped: bool,
    /// Original velocity direction before turn limiting (radians)
    pub pre_limit_angle: f32,
    /// Final velocity direction after turn limiting (radians)
    pub post_limit_angle: f32,
}

/// Integrate motion using Euler integration with drag and turn rate limiting.
///
/// Physics model:
/// 1. Capture old heading (for turn rate limiting)
/// 2. Apply acceleration: v += a × dt
/// 3. Apply drag: v *= exp(-drag × dt)
/// 4. Clamp speed to max_speed
/// 5. Apply turn rate limiting (clamp heading change)
/// 6. Integrate position: p += v × dt
///
/// Returns the new position, velocity, and debug flags indicating whether
/// turn rate or speed limits were applied.
pub fn integrate_motion(params: &IntegrationParams) -> IntegrationResult {
    let (px, py) = params.position;
    let (mut vx, mut vy) = params.velocity;
    let (ax, ay) = params.acceleration;
    let dt = params.dt;
    let stopped_threshold_sq = params.stopped_threshold * params.stopped_threshold;

    // 1. Capture old heading (NaN if stopped)
    let old_speed_sq = vx * vx + vy * vy;
    let old_angle = if old_speed_sq > stopped_threshold_sq {
        fast_atan2(vy, vx)
    } else {
        f32::NAN
    };

    // 2. Apply acceleration (Euler integration)
    vx += ax * dt;
    vy += ay * dt;

    // 3. Apply drag: v *= exp(-drag × dt) (frame-rate independent)
    let drag_factor = (-params.drag_coefficient * dt).exp();
    vx *= drag_factor;
    vy *= drag_factor;

    // 4. Clamp speed to max_speed
    let speed_sq = vx * vx + vy * vy;
    let max_speed_sq = params.max_speed * params.max_speed;
    let speed_clamped = speed_sq > max_speed_sq;

    let current_speed = if speed_clamped {
        let speed = speed_sq.sqrt();
        let scale = params.max_speed / speed;
        vx *= scale;
        vy *= scale;
        params.max_speed
    } else {
        speed_sq.sqrt()
    };

    // 5. Turn rate limiting (only if moving and was previously moving)
    let mut turn_limited = false;
    let new_speed_sq = vx * vx + vy * vy;
    let new_angle = if new_speed_sq > stopped_threshold_sq {
        fast_atan2(vy, vx)
    } else {
        old_angle // Preserve old angle if now stopped
    };

    let (pre_limit_angle, post_limit_angle) = if old_angle.is_finite() && new_speed_sq > stopped_threshold_sq {
        let delta = normalize_angle(new_angle - old_angle);
        let max_delta = params.max_turn_rate_rad * dt;

        if delta.abs() > max_delta {
            turn_limited = true;
            let clamped_delta = delta.clamp(-max_delta, max_delta);
            let final_angle = old_angle + clamped_delta;

            // Reconstruct velocity with limited angle
            vx = current_speed * final_angle.cos();
            vy = current_speed * final_angle.sin();

            (new_angle, final_angle)
        } else {
            (new_angle, new_angle)
        }
    } else {
        (new_angle, new_angle)
    };

    // 6. Integrate position
    let new_px = px + vx * dt;
    let new_py = py + vy * dt;

    IntegrationResult {
        position: (new_px, new_py),
        velocity: (vx, vy),
        turn_limited,
        speed_clamped,
        pre_limit_angle,
        post_limit_angle,
    }
}

/// Simplified integration without turn rate limiting.
/// Use this when you want to test pure physics without turn constraints.
pub fn integrate_motion_no_turn_limit(params: &IntegrationParams) -> IntegrationResult {
    let (px, py) = params.position;
    let (mut vx, mut vy) = params.velocity;
    let (ax, ay) = params.acceleration;
    let dt = params.dt;

    // Apply acceleration
    vx += ax * dt;
    vy += ay * dt;

    // Apply drag
    let drag_factor = (-params.drag_coefficient * dt).exp();
    vx *= drag_factor;
    vy *= drag_factor;

    // Clamp speed
    let speed_sq = vx * vx + vy * vy;
    let max_speed_sq = params.max_speed * params.max_speed;
    let speed_clamped = speed_sq > max_speed_sq;

    if speed_clamped {
        let speed = speed_sq.sqrt();
        let scale = params.max_speed / speed;
        vx *= scale;
        vy *= scale;
    }

    // Integrate position
    let new_px = px + vx * dt;
    let new_py = py + vy * dt;

    let angle = fast_atan2(vy, vx);

    IntegrationResult {
        position: (new_px, new_py),
        velocity: (vx, vy),
        turn_limited: false,
        speed_clamped,
        pre_limit_angle: angle,
        post_limit_angle: angle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::f32::consts::PI;

    fn default_params() -> IntegrationParams {
        IntegrationParams {
            position: (0.0, 0.0),
            velocity: (10.0, 0.0), // Moving right at 10 m/s
            acceleration: (0.0, 0.0),
            dt: 0.05,
            drag_coefficient: 2.0,
            max_speed: 15.0,
            max_turn_rate_rad: PI, // 180 deg/s
            stopped_threshold: 0.05,
        }
    }

    // ============================================================
    // Basic integration tests
    // ============================================================

    #[test]
    fn position_integrates_with_velocity() {
        let params = IntegrationParams {
            position: (0.0, 0.0),
            velocity: (10.0, 0.0),
            acceleration: (0.0, 0.0),
            dt: 0.1,
            drag_coefficient: 0.0, // No drag for this test
            max_speed: 100.0,
            ..default_params()
        };

        let result = integrate_motion(&params);

        // Position should move by velocity × dt
        // With no drag/accel, 10 m/s × 0.1s = 1m
        assert!(
            (result.position.0 - 1.0).abs() < 0.01,
            "Position X should be ~1.0, got {}",
            result.position.0
        );
    }

    #[test]
    fn velocity_integrates_with_acceleration() {
        let params = IntegrationParams {
            position: (0.0, 0.0),
            velocity: (0.0, 0.0),
            acceleration: (10.0, 0.0), // 10 m/s² acceleration
            dt: 0.1,
            drag_coefficient: 0.0, // No drag for this test
            max_speed: 100.0,
            ..default_params()
        };

        let result = integrate_motion(&params);

        // Velocity should increase by acceleration × dt
        // 10 m/s² × 0.1s = 1 m/s
        assert!(
            (result.velocity.0 - 1.0).abs() < 0.01,
            "Velocity X should be ~1.0, got {}",
            result.velocity.0
        );
    }

    // ============================================================
    // Drag tests
    // ============================================================

    #[test]
    fn drag_reduces_velocity() {
        let params = IntegrationParams {
            position: (0.0, 0.0),
            velocity: (10.0, 0.0),
            acceleration: (0.0, 0.0),
            dt: 0.05,
            drag_coefficient: 2.0,
            max_speed: 100.0,
            ..default_params()
        };

        let result = integrate_motion(&params);

        // v *= exp(-2.0 × 0.05) = exp(-0.1) ≈ 0.9048
        let expected = 10.0 * (-2.0 * 0.05_f32).exp();
        assert!(
            (result.velocity.0 - expected).abs() < 0.01,
            "Velocity should be ~{:.2}, got {:.2}",
            expected,
            result.velocity.0
        );
    }

    #[test]
    fn drag_is_frame_rate_independent() {
        // Two approaches: 1×0.1s vs 2×0.05s should give same result

        let params_one_step = IntegrationParams {
            velocity: (10.0, 0.0),
            acceleration: (0.0, 0.0),
            dt: 0.1,
            drag_coefficient: 2.0,
            max_speed: 100.0,
            ..default_params()
        };

        let result_one = integrate_motion(&params_one_step);

        // Two steps
        let params_two_steps = IntegrationParams {
            velocity: (10.0, 0.0),
            acceleration: (0.0, 0.0),
            dt: 0.05,
            drag_coefficient: 2.0,
            max_speed: 100.0,
            ..default_params()
        };

        let result_step1 = integrate_motion(&params_two_steps);
        let params_step2 = IntegrationParams {
            position: result_step1.position,
            velocity: result_step1.velocity,
            ..params_two_steps
        };
        let result_two = integrate_motion(&params_step2);

        // Velocity after 0.1s should be similar
        // One step: v = 10 × exp(-0.2) ≈ 8.187
        // Two steps: v = 10 × exp(-0.1)² = 10 × exp(-0.2) ≈ 8.187
        assert!(
            (result_one.velocity.0 - result_two.velocity.0).abs() < 0.1,
            "Drag should be frame-rate independent: {:.2} vs {:.2}",
            result_one.velocity.0,
            result_two.velocity.0
        );
    }

    // ============================================================
    // Speed clamping tests
    // ============================================================

    #[test]
    fn speed_clamped_to_max() {
        let params = IntegrationParams {
            velocity: (0.0, 0.0),
            acceleration: (1000.0, 0.0), // Huge acceleration
            dt: 0.1,
            drag_coefficient: 0.0,
            max_speed: 15.0,
            ..default_params()
        };

        let result = integrate_motion(&params);
        let speed = (result.velocity.0.powi(2) + result.velocity.1.powi(2)).sqrt();

        assert!(
            speed <= 15.01,
            "Speed {} should be clamped to max 15.0",
            speed
        );
        assert!(result.speed_clamped, "Should report speed_clamped=true");
    }

    #[test]
    fn speed_not_clamped_when_under_max() {
        let params = IntegrationParams {
            velocity: (5.0, 0.0),
            acceleration: (0.0, 0.0),
            dt: 0.05,
            drag_coefficient: 0.0,
            max_speed: 15.0,
            ..default_params()
        };

        let result = integrate_motion(&params);

        assert!(!result.speed_clamped, "Should not clamp when under max");
    }

    // ============================================================
    // Turn rate limiting tests
    // ============================================================

    #[test]
    fn large_turn_is_limited() {
        let params = IntegrationParams {
            velocity: (10.0, 0.0), // Moving right
            acceleration: (0.0, 100.0), // Strong upward acceleration
            dt: 0.05,
            drag_coefficient: 0.0,
            max_speed: 100.0,
            max_turn_rate_rad: PI, // 180 deg/s = 9 deg per 0.05s
            stopped_threshold: 0.05,
            ..default_params()
        };

        let result = integrate_motion(&params);

        // Without limiting, 100 m/s² × 0.05s = 5 m/s upward
        // Angle would be atan2(5, 10) ≈ 26.6 degrees
        // But with 9 deg/frame limit, should be much smaller

        let angle_deg = result.post_limit_angle.to_degrees();
        let max_expected_deg = 180.0 * 0.05 + 0.5; // 9 deg + tolerance

        assert!(
            angle_deg.abs() <= max_expected_deg,
            "Angle {} deg should be limited to ~9 deg",
            angle_deg
        );
        assert!(result.turn_limited, "Should report turn_limited=true");
    }

    #[test]
    fn small_turn_not_limited() {
        let params = IntegrationParams {
            velocity: (10.0, 0.0),
            acceleration: (0.0, 0.5), // Small upward acceleration
            dt: 0.05,
            drag_coefficient: 0.0,
            max_speed: 100.0,
            max_turn_rate_rad: PI,
            ..default_params()
        };

        let result = integrate_motion(&params);

        assert!(!result.turn_limited, "Small turn should not be limited");
    }

    #[test]
    fn stopped_creature_can_turn_freely() {
        let params = IntegrationParams {
            velocity: (0.0, 0.0), // Stopped
            acceleration: (0.0, 10.0), // Accelerate upward
            dt: 0.05,
            drag_coefficient: 0.0,
            max_speed: 100.0,
            max_turn_rate_rad: PI,
            stopped_threshold: 0.05,
            ..default_params()
        };

        let result = integrate_motion(&params);

        // Stopped creature should accelerate in direction of acceleration
        assert!(result.velocity.1 > 0.0, "Should move upward");
        assert!(!result.turn_limited, "Stopped creature should not have turn limiting");
    }

    // ============================================================
    // Visualization mismatch detection tests
    // ============================================================

    #[test]
    fn turn_limiting_causes_angle_difference() {
        let params = IntegrationParams {
            velocity: (10.0, 0.0),
            acceleration: (0.0, 100.0), // Large perpendicular acceleration
            dt: 0.05,
            drag_coefficient: 0.0,
            max_speed: 100.0,
            max_turn_rate_rad: PI, // 180 deg/s
            ..default_params()
        };

        let result = integrate_motion(&params);

        if result.turn_limited {
            // pre_limit_angle is what the acceleration "wanted"
            // post_limit_angle is what the velocity actually is
            let angle_diff = (result.pre_limit_angle - result.post_limit_angle).abs();

            assert!(
                angle_diff > 0.01,
                "Turn limiting should cause angle difference, got {} rad",
                angle_diff
            );
        }
    }

    #[test]
    fn no_turn_limit_version_matches_angle() {
        let params = IntegrationParams {
            velocity: (10.0, 0.0),
            acceleration: (0.0, 100.0),
            dt: 0.05,
            drag_coefficient: 0.0,
            max_speed: 100.0,
            max_turn_rate_rad: PI,
            ..default_params()
        };

        let result = integrate_motion_no_turn_limit(&params);

        // Without turn limiting, pre and post angles should match
        assert!(
            (result.pre_limit_angle - result.post_limit_angle).abs() < 0.001,
            "No turn limit should have matching angles"
        );
        assert!(!result.turn_limited);
    }

    // ============================================================
    // Multi-frame oscillation test
    // ============================================================

    #[test]
    fn braking_does_not_reverse_velocity_direction() {
        // Simulate a creature moving right and braking hard
        let mut params = IntegrationParams {
            position: (0.0, 0.0),
            velocity: (10.0, 0.0), // Moving right
            acceleration: (-6.0, 0.0), // Max braking (typical max_accel)
            dt: 0.05,
            drag_coefficient: 2.0,
            max_speed: 15.0,
            max_turn_rate_rad: PI,
            stopped_threshold: 0.05,
        };

        let mut sign_changes = 0;
        let mut prev_vx_sign = params.velocity.0.signum();

        // Simulate 50 frames
        for _ in 0..50 {
            let result = integrate_motion(&params);

            // Track velocity sign changes
            let current_sign = result.velocity.0.signum();
            if current_sign != prev_vx_sign && result.velocity.0.abs() > 0.1 {
                sign_changes += 1;
            }
            prev_vx_sign = current_sign;

            // Update params for next frame
            params.position = result.position;
            params.velocity = result.velocity;

            // Stop braking once slow enough
            if result.velocity.0.abs() < 1.0 {
                params.acceleration = (0.0, 0.0);
            }
        }

        // Should decelerate smoothly to near-zero, at most 1 sign change
        assert!(
            sign_changes <= 1,
            "Velocity changed sign {} times - oscillation detected!",
            sign_changes
        );
    }

    // ============================================================
    // Edge cases
    // ============================================================

    #[test]
    fn zero_dt_returns_same_state() {
        let params = IntegrationParams {
            position: (5.0, 10.0),
            velocity: (3.0, 4.0),
            acceleration: (1.0, 2.0),
            dt: 0.0,
            drag_coefficient: 2.0,
            max_speed: 15.0,
            ..default_params()
        };

        let result = integrate_motion(&params);

        assert_eq!(result.position, params.position);
        // Velocity changes slightly due to drag_factor = exp(0) = 1
        // and accel × 0 = 0, so should be unchanged
        assert!((result.velocity.0 - params.velocity.0).abs() < 0.001);
        assert!((result.velocity.1 - params.velocity.1).abs() < 0.001);
    }

    #[test]
    fn handles_nan_velocity_gracefully() {
        let params = IntegrationParams {
            velocity: (f32::NAN, 0.0),
            ..default_params()
        };

        let result = integrate_motion(&params);

        // NaN propagates, but shouldn't panic
        // This is expected behavior - caller should sanitize input
        assert!(result.velocity.0.is_nan() || result.position.0.is_nan());
    }
}
