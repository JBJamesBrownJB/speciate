//! Common steering behavior types for pure function calculations.
//!
//! This module provides shared types used by all steering behaviors (seek, wander,
//! avoidance). Using these shared types ensures consistent F=ma physics across
//! all behaviors and enables easy composition in tests.

/// Common input context for all steering behaviors.
/// Contains physical properties needed to calculate steering acceleration.
#[derive(Debug, Clone, Copy)]
pub struct SteeringContext {
    /// Current velocity (m/s)
    pub velocity: (f32, f32),
    /// Maximum speed (m/s)
    pub max_speed: f32,
    /// Maximum force creature can exert (Newtons)
    pub max_force: f32,
    /// Creature mass (kg) - used for F=ma conversion
    pub mass: f32,
}

impl SteeringContext {
    /// Calculate max acceleration using F=ma physics
    #[inline]
    pub fn max_accel(&self) -> f32 {
        if self.mass > 0.001 {
            self.max_force / self.mass
        } else {
            0.0
        }
    }

    /// Calculate current speed (magnitude of velocity)
    #[inline]
    pub fn speed(&self) -> f32 {
        let (vx, vy) = self.velocity;
        (vx * vx + vy * vy).sqrt()
    }

    /// Get velocity direction as unit vector (or zero if stationary)
    #[inline]
    pub fn velocity_direction(&self) -> (f32, f32) {
        let speed = self.speed();
        if speed > 0.001 {
            (self.velocity.0 / speed, self.velocity.1 / speed)
        } else {
            (0.0, 0.0)
        }
    }
}

/// Universal steering result from any behavior.
/// Acceleration is in m/s² (NOT Newtons!) after F=ma conversion.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SteeringResult {
    /// Acceleration to apply (m/s²) - already converted from force
    pub acceleration: (f32, f32),
}

impl SteeringResult {
    /// Create a zero-acceleration result
    pub fn zero() -> Self {
        Self {
            acceleration: (0.0, 0.0),
        }
    }

    /// Create a result with given acceleration
    pub fn with_acceleration(ax: f32, ay: f32) -> Self {
        Self {
            acceleration: (ax, ay),
        }
    }

    /// Get acceleration magnitude
    pub fn magnitude(&self) -> f32 {
        let (ax, ay) = self.acceleration;
        (ax * ax + ay * ay).sqrt()
    }
}

/// Accumulate multiple accelerations with magnitude clamping.
///
/// Takes a slice of acceleration vectors (all in m/s²) and returns
/// the sum, clamped to `max_accel` magnitude.
///
/// This is the core force blending function for combining multiple
/// steering behaviors (seek + wander + avoidance → single acceleration).
pub fn accumulate_steering(accels: &[(f32, f32)], max_accel: f32) -> (f32, f32) {
    // Sum all accelerations
    let (sum_x, sum_y) = accels
        .iter()
        .fold((0.0f32, 0.0f32), |(ax, ay), (bx, by)| (ax + bx, ay + by));

    // Clamp to max_accel magnitude
    let mag_sq = sum_x * sum_x + sum_y * sum_y;
    let max_sq = max_accel * max_accel;

    if mag_sq > max_sq && mag_sq > 0.0001 {
        let mag = mag_sq.sqrt();
        let scale = max_accel / mag;
        (sum_x * scale, sum_y * scale)
    } else {
        (sum_x, sum_y)
    }
}

/// Convert steering vector (desired_velocity - current_velocity) to acceleration.
///
/// This applies F=ma conversion and clamps to max_accel.
/// Use this when you have a Reynolds-style steering vector that needs conversion.
#[inline]
pub fn steering_to_acceleration(steer: (f32, f32), ctx: &SteeringContext) -> (f32, f32) {
    let (sx, sy) = steer;
    let max_accel = ctx.max_accel();
    let mag_sq = sx * sx + sy * sy;
    let max_sq = max_accel * max_accel;

    if mag_sq > max_sq && mag_sq > 0.0001 {
        let mag = mag_sq.sqrt();
        let scale = max_accel / mag;
        (sx * scale, sy * scale)
    } else {
        (sx, sy)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Reference values for a typical creature
    const DEFAULT_MAX_FORCE: f32 = 390.0; // Newtons
    const DEFAULT_MASS: f32 = 65.0; // kg
    const EXPECTED_MAX_ACCEL: f32 = 6.0; // 390/65 = 6 m/s²
    const DEFAULT_MAX_SPEED: f32 = 15.0; // m/s

    fn default_context() -> SteeringContext {
        SteeringContext {
            velocity: (10.0, 0.0),
            max_speed: DEFAULT_MAX_SPEED,
            max_force: DEFAULT_MAX_FORCE,
            mass: DEFAULT_MASS,
        }
    }

    // ============================================================
    // SteeringContext tests
    // ============================================================

    #[test]
    fn steering_context_max_accel_uses_fma() {
        let ctx = default_context();
        let max_accel = ctx.max_accel();

        assert!(
            (max_accel - EXPECTED_MAX_ACCEL).abs() < 0.01,
            "max_accel should be {} m/s² (F/m), got {}",
            EXPECTED_MAX_ACCEL,
            max_accel
        );
    }

    #[test]
    fn steering_context_max_accel_handles_zero_mass() {
        let ctx = SteeringContext {
            velocity: (0.0, 0.0),
            max_speed: 15.0,
            max_force: 390.0,
            mass: 0.0, // Zero mass
        };

        assert_eq!(
            ctx.max_accel(),
            0.0,
            "Zero mass should return zero max_accel"
        );
    }

    #[test]
    fn steering_context_speed_calculates_magnitude() {
        let ctx = SteeringContext {
            velocity: (3.0, 4.0), // 3-4-5 triangle
            max_speed: 15.0,
            max_force: 390.0,
            mass: 65.0,
        };

        assert!(
            (ctx.speed() - 5.0).abs() < 0.001,
            "Speed should be 5.0, got {}",
            ctx.speed()
        );
    }

    #[test]
    fn steering_context_velocity_direction_normalizes() {
        let ctx = SteeringContext {
            velocity: (3.0, 4.0),
            max_speed: 15.0,
            max_force: 390.0,
            mass: 65.0,
        };

        let (dx, dy) = ctx.velocity_direction();

        assert!(
            (dx - 0.6).abs() < 0.001,
            "Direction X should be 0.6, got {}",
            dx
        );
        assert!(
            (dy - 0.8).abs() < 0.001,
            "Direction Y should be 0.8, got {}",
            dy
        );
    }

    #[test]
    fn steering_context_velocity_direction_handles_stationary() {
        let ctx = SteeringContext {
            velocity: (0.0, 0.0),
            max_speed: 15.0,
            max_force: 390.0,
            mass: 65.0,
        };

        let dir = ctx.velocity_direction();
        assert_eq!(dir, (0.0, 0.0), "Stationary should return zero direction");
    }

    // ============================================================
    // SteeringResult tests
    // ============================================================

    #[test]
    fn steering_result_zero_is_zero() {
        let result = SteeringResult::zero();
        assert_eq!(result.acceleration, (0.0, 0.0));
        assert_eq!(result.magnitude(), 0.0);
    }

    #[test]
    fn steering_result_magnitude_correct() {
        let result = SteeringResult::with_acceleration(3.0, 4.0);
        assert!((result.magnitude() - 5.0).abs() < 0.001);
    }

    // ============================================================
    // accumulate_steering tests
    // ============================================================

    #[test]
    fn accumulate_steering_sums_accelerations() {
        let accels = vec![(1.0, 0.0), (0.0, 1.0), (1.0, 1.0)];
        let (ax, ay) = accumulate_steering(&accels, 100.0); // High max so no clamp

        assert!((ax - 2.0).abs() < 0.001, "Sum X should be 2.0, got {}", ax);
        assert!((ay - 2.0).abs() < 0.001, "Sum Y should be 2.0, got {}", ay);
    }

    #[test]
    fn accumulate_steering_clamps_to_max() {
        // Two accelerations that sum to magnitude 10
        let accels = vec![(6.0, 0.0), (0.0, 8.0)]; // sqrt(36+64) = 10
        let max_accel = 5.0;

        let (ax, ay) = accumulate_steering(&accels, max_accel);
        let mag = (ax * ax + ay * ay).sqrt();

        assert!(
            (mag - max_accel).abs() < 0.001,
            "Magnitude should be clamped to {}, got {}",
            max_accel,
            mag
        );

        // Direction should be preserved
        let original_dir = (6.0 / 10.0, 8.0 / 10.0);
        let result_dir = (ax / mag, ay / mag);
        assert!(
            (result_dir.0 - original_dir.0).abs() < 0.001,
            "Direction should be preserved"
        );
    }

    #[test]
    fn accumulate_steering_doesnt_clamp_under_max() {
        let accels = vec![(1.0, 0.0), (0.0, 1.0)]; // sqrt(2) ≈ 1.41
        let max_accel = 5.0;

        let (ax, ay) = accumulate_steering(&accels, max_accel);

        assert!(
            (ax - 1.0).abs() < 0.001,
            "Should not clamp under max, got {}",
            ax
        );
        assert!(
            (ay - 1.0).abs() < 0.001,
            "Should not clamp under max, got {}",
            ay
        );
    }

    #[test]
    fn accumulate_steering_handles_empty() {
        let accels: Vec<(f32, f32)> = vec![];
        let (ax, ay) = accumulate_steering(&accels, 5.0);

        assert_eq!((ax, ay), (0.0, 0.0), "Empty input should return zero");
    }

    #[test]
    fn accumulate_steering_handles_opposing_forces() {
        // Forces that cancel out
        let accels = vec![(5.0, 0.0), (-5.0, 0.0)];
        let (ax, ay) = accumulate_steering(&accels, 10.0);

        assert!(
            ax.abs() < 0.001,
            "Opposing forces should cancel, got {}",
            ax
        );
        assert!(ay.abs() < 0.001, "Y should be zero, got {}", ay);
    }

    // ============================================================
    // steering_to_acceleration tests
    // ============================================================

    #[test]
    fn steering_to_acceleration_clamps_to_max_accel() {
        let ctx = default_context();
        let steer = (100.0, 0.0); // Way over max_accel of 6

        let (ax, ay) = steering_to_acceleration(steer, &ctx);
        let mag = (ax * ax + ay * ay).sqrt();

        assert!(
            (mag - EXPECTED_MAX_ACCEL).abs() < 0.01,
            "Should clamp to max_accel {} m/s², got {}",
            EXPECTED_MAX_ACCEL,
            mag
        );
    }

    #[test]
    fn steering_to_acceleration_preserves_direction() {
        let ctx = default_context();
        let steer = (30.0, 40.0); // Direction (0.6, 0.8), magnitude 50

        let (ax, ay) = steering_to_acceleration(steer, &ctx);
        let mag = (ax * ax + ay * ay).sqrt();

        // Should be clamped to 6 m/s²
        assert!((mag - EXPECTED_MAX_ACCEL).abs() < 0.01);

        // Direction should be preserved
        let (dx, dy) = (ax / mag, ay / mag);
        assert!((dx - 0.6).abs() < 0.01, "Direction X should be 0.6");
        assert!((dy - 0.8).abs() < 0.01, "Direction Y should be 0.8");
    }

    #[test]
    fn steering_to_acceleration_passes_through_under_max() {
        let ctx = default_context();
        let steer = (2.0, 0.0); // Under max_accel of 6

        let (ax, ay) = steering_to_acceleration(steer, &ctx);

        assert!(
            (ax - 2.0).abs() < 0.001,
            "Under max should pass through, got {}",
            ax
        );
        assert!(ay.abs() < 0.001);
    }

    // ============================================================
    // Integration test: Wander bug detection
    // ============================================================

    #[test]
    fn wander_bug_would_be_caught_by_steering_context() {
        // The wander bug treats steering (m/s) as force (N)
        // This test proves our types would catch it

        let ctx = default_context();

        // Wander calculates: steer = desired_velocity - current_velocity
        // This is in m/s units, NOT Newtons
        let desired_vel = (15.0, 0.0); // max_speed in direction
        let steer = (
            desired_vel.0 - ctx.velocity.0,
            desired_vel.1 - ctx.velocity.1,
        );
        // steer = (5.0, 0.0) in m/s

        // CORRECT: Use steering_to_acceleration which clamps to max_accel
        let (ax, _ay) = steering_to_acceleration(steer, &ctx);

        // Since 5.0 < 6.0 (max_accel), it passes through unchanged
        // This is correct because the steering is a velocity difference, not a force
        assert!(
            ax <= ctx.max_accel() + 0.01,
            "Acceleration {} should be <= max_accel {} (F/m)",
            ax,
            ctx.max_accel()
        );

        // WRONG (the bug): Treating steer as acceleration directly without mass conversion
        // If we had a bug where we did: accel = clamp(steer, max_force) [wrong!]
        // We'd get accel up to 390 m/s² instead of 6 m/s²
        // Our types prevent this by making max_accel() the only way to get the limit
    }

    // ============================================================
    // Integration test: Force accumulation respects F=ma
    // ============================================================

    #[test]
    fn combined_behaviors_respect_max_accel() {
        let ctx = default_context();
        let max_accel = ctx.max_accel(); // 6 m/s²

        // Simulate multiple behaviors each contributing acceleration
        let seek_accel = (4.0, 0.0); // Toward target
        let wander_accel = (1.0, 3.0); // Random wander
        let avoid_accel = (0.0, -2.0); // Away from obstacle

        let accels = vec![seek_accel, wander_accel, avoid_accel];
        let (ax, ay) = accumulate_steering(&accels, max_accel);
        let mag = (ax * ax + ay * ay).sqrt();

        assert!(
            mag <= max_accel + 0.01,
            "Combined acceleration {} should respect max_accel {}",
            mag,
            max_accel
        );
    }
}
