//! Physics calculations using correct F=ma unit conversions.
//!
//! This module provides pure functions for steering behaviors that correctly
//! convert forces (Newtons) to acceleration (m/s²) using F=ma → a=F/m.
//!
//! CRITICAL: Avoidance and arrival systems must use these functions to avoid
//! treating force values as acceleration values (which causes 65x overscaling
//! for a 65kg creature: max_force=390N but max_accel=6m/s²).

use super::magnitude_sq;

/// Parameters for avoidance force calculation.
#[derive(Debug, Clone, Copy)]
pub struct AvoidanceParams {
    /// Direction away from obstacle (normalized)
    pub away_dir: (f32, f32),
    /// Edge-to-edge distance (meters, ≥0)
    pub safe_distance: f32,
    /// Effective personal space (meters)
    pub effective_space: f32,
    /// Maximum force creature can exert (Newtons)
    pub max_force: f32,
    /// Creature mass (kg)
    pub mass: f32,
    /// Emergency brake distance threshold (meters)
    pub emergency_distance: f32,
}

/// Result of avoidance calculation.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AvoidanceResult {
    /// Acceleration to apply (m/s²)
    pub acceleration: (f32, f32),
}

/// Calculate avoidance acceleration with correct F=ma physics.
///
/// Uses inverse-square urgency scaling with emergency brake zone:
/// - Inside emergency distance: max acceleration (max_force / mass)
/// - Outside: inverse-square falloff based on distance/personal_space ratio
///
/// Returns acceleration in m/s², NOT force in Newtons.
pub fn calculate_avoidance(params: &AvoidanceParams) -> AvoidanceResult {
    let (dir_x, dir_y) = params.away_dir;
    let safe_distance = params.safe_distance;
    let effective_space = params.effective_space;
    let max_force = params.max_force;
    let mass = params.mass;
    let emergency_distance = params.emergency_distance;

    // Correct F=ma conversion: max_accel = max_force / mass
    let max_accel = max_force / mass;

    // Calculate urgency using inverse-square scaling
    let urgency = if effective_space > 0.001 {
        let ratio = effective_space / safe_distance;
        ratio * ratio
    } else {
        1.0
    };

    // Emergency brake: max acceleration when very close
    let accel_magnitude = if safe_distance < emergency_distance {
        max_accel
    } else {
        (max_accel * urgency).min(max_accel)
    };

    // Apply acceleration in away direction
    let ax = dir_x * accel_magnitude;
    let ay = dir_y * accel_magnitude;

    AvoidanceResult {
        acceleration: (ax, ay),
    }
}

/// Clamp steering vector to maximum acceleration (not force!).
///
/// This is the corrected version of clamp_force that takes mass into account.
/// steering is in m/s² units (already acceleration), max_force is in Newtons.
#[inline]
pub fn clamp_steering_to_max_accel(
    steer_x: f32,
    steer_y: f32,
    max_force: f32,
    mass: f32,
) -> (f32, f32) {
    let max_accel = max_force / mass;
    let max_accel_sq = max_accel * max_accel;
    let steer_mag_sq = magnitude_sq(steer_x, steer_y);

    if steer_mag_sq > max_accel_sq {
        let steer_mag = steer_mag_sq.sqrt();
        let scale = max_accel / steer_mag;
        (steer_x * scale, steer_y * scale)
    } else {
        (steer_x, steer_y)
    }
}

/// Convert force vector to acceleration vector using F=ma.
///
/// This is the fundamental physics conversion that was missing from the
/// original avoidance system (which treated force as acceleration directly).
#[inline]
pub fn force_to_acceleration(force_x: f32, force_y: f32, mass: f32) -> (f32, f32) {
    if mass > 0.001 {
        (force_x / mass, force_y / mass)
    } else {
        (0.0, 0.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Reference values for a typical creature
    const DEFAULT_MAX_FORCE: f32 = 390.0; // Newtons
    const DEFAULT_MASS: f32 = 65.0; // kg
    const EXPECTED_MAX_ACCEL: f32 = 6.0; // 390/65 = 6 m/s²

    // Test 1: Prove the F=ma bug - force ≠ acceleration
    #[test]
    fn force_and_acceleration_are_different_units() {
        // This test documents the bug we're fixing
        // If max_force (390N) were used as acceleration, a 65kg creature would
        // experience 390 m/s² instead of 6 m/s² - that's 65x too high!

        let max_force = DEFAULT_MAX_FORCE;
        let mass = DEFAULT_MASS;

        // WRONG: Using force as acceleration (the bug)
        let wrong_accel = max_force; // 390 "m/s²" - WRONG!

        // CORRECT: F=ma → a=F/m
        let correct_accel = max_force / mass; // 6 m/s²

        assert!(
            (correct_accel - EXPECTED_MAX_ACCEL).abs() < 0.01,
            "Max accel should be {} m/s², got {}",
            EXPECTED_MAX_ACCEL,
            correct_accel
        );

        assert!(
            (wrong_accel / correct_accel - 65.0).abs() < 0.1,
            "Bug causes {}x overscaling (mass={}kg)",
            wrong_accel / correct_accel,
            mass
        );
    }

    // Test 2: Avoidance calculation uses correct physics
    #[test]
    fn avoidance_returns_acceleration_not_force() {
        let params = AvoidanceParams {
            away_dir: (1.0, 0.0), // Unit vector pointing away
            safe_distance: 0.1, // Very close (emergency zone)
            effective_space: 2.5,
            max_force: DEFAULT_MAX_FORCE,
            mass: DEFAULT_MASS,
            emergency_distance: 0.5,
        };

        let result = calculate_avoidance(&params);
        let accel_mag = (result.acceleration.0.powi(2) + result.acceleration.1.powi(2)).sqrt();

        // Should be limited to max_accel (6 m/s²), NOT max_force (390N)
        assert!(
            accel_mag <= EXPECTED_MAX_ACCEL + 0.01,
            "Acceleration magnitude should be ≤{} m/s², got {} m/s². \
             This indicates the F=ma bug - force is being used as acceleration!",
            EXPECTED_MAX_ACCEL,
            accel_mag
        );
    }

    // Test 3: Clamp steering respects mass
    #[test]
    fn clamp_steering_divides_by_mass() {
        // Large steering vector that exceeds max_force
        let steer_x = 500.0;
        let steer_y = 0.0;

        let (ax, ay) = clamp_steering_to_max_accel(steer_x, steer_y, DEFAULT_MAX_FORCE, DEFAULT_MASS);

        let accel_mag = (ax * ax + ay * ay).sqrt();

        assert!(
            (accel_mag - EXPECTED_MAX_ACCEL).abs() < 0.01,
            "Clamped acceleration should be {} m/s², got {}",
            EXPECTED_MAX_ACCEL,
            accel_mag
        );
    }

    // Test 4: Force to acceleration conversion
    #[test]
    fn force_to_acceleration_uses_correct_formula() {
        let force_x = 390.0;
        let force_y = 0.0;
        let mass = 65.0;

        let (ax, ay) = force_to_acceleration(force_x, force_y, mass);

        assert!(
            (ax - 6.0).abs() < 0.01,
            "390N / 65kg should be 6 m/s², got {}",
            ax
        );
        assert_eq!(ay, 0.0);
    }

    // Test 5: Emergency zone uses max_accel, not max_force
    #[test]
    fn emergency_zone_uses_max_accel() {
        let params = AvoidanceParams {
            away_dir: (1.0, 0.0),
            safe_distance: 0.01, // Extremely close
            effective_space: 2.5,
            max_force: DEFAULT_MAX_FORCE,
            mass: DEFAULT_MASS,
            emergency_distance: 0.5,
        };

        let result = calculate_avoidance(&params);

        // In emergency zone, should use max_accel = 6 m/s²
        assert!(
            (result.acceleration.0 - EXPECTED_MAX_ACCEL).abs() < 0.01,
            "Emergency zone acceleration should be {} m/s², got {}",
            EXPECTED_MAX_ACCEL,
            result.acceleration.0
        );
    }

    // Test 6: Outside emergency zone scales by urgency
    #[test]
    fn outside_emergency_zone_scales_by_urgency() {
        // At safe_distance = effective_space, urgency = 1.0
        let params_at_boundary = AvoidanceParams {
            away_dir: (1.0, 0.0),
            safe_distance: 2.5, // At personal space boundary
            effective_space: 2.5,
            max_force: DEFAULT_MAX_FORCE,
            mass: DEFAULT_MASS,
            emergency_distance: 0.5,
        };

        // At safe_distance = 2 × effective_space, urgency = 0.25
        let params_far = AvoidanceParams {
            away_dir: (1.0, 0.0),
            safe_distance: 5.0, // 2× personal space
            effective_space: 2.5,
            max_force: DEFAULT_MAX_FORCE,
            mass: DEFAULT_MASS,
            emergency_distance: 0.5,
        };

        let result_boundary = calculate_avoidance(&params_at_boundary);
        let result_far = calculate_avoidance(&params_far);

        // At boundary: urgency = 1.0, accel = max_accel × 1.0 = 6
        assert!(
            (result_boundary.acceleration.0 - EXPECTED_MAX_ACCEL).abs() < 0.01,
            "At boundary: expected {} m/s², got {}",
            EXPECTED_MAX_ACCEL,
            result_boundary.acceleration.0
        );

        // Far away: urgency = 0.25, accel = max_accel × 0.25 = 1.5
        let expected_far = EXPECTED_MAX_ACCEL * 0.25;
        assert!(
            (result_far.acceleration.0 - expected_far).abs() < 0.01,
            "Far away: expected {} m/s², got {}",
            expected_far,
            result_far.acceleration.0
        );
    }

    // Test 7: Zero mass doesn't cause division by zero
    #[test]
    fn zero_mass_returns_zero_acceleration() {
        let (ax, ay) = force_to_acceleration(100.0, 100.0, 0.0);
        assert_eq!((ax, ay), (0.0, 0.0), "Zero mass should return zero acceleration");
    }

    // Test 8: Simulation demonstrates the bug impact
    #[test]
    fn simulation_shows_bug_causes_wild_velocities() {
        // Simulate one frame with the BUG (force as acceleration)
        let dt = 0.05; // 50ms frame
        let force = DEFAULT_MAX_FORCE; // 390N

        // BUG: velocity += force × dt (treating 390N as 390 m/s²)
        let buggy_velocity_change = force * dt; // 19.5 m/s per frame!

        // CORRECT: velocity += (force / mass) × dt
        let correct_velocity_change = (force / DEFAULT_MASS) * dt; // 0.3 m/s per frame

        assert!(
            buggy_velocity_change > 19.0,
            "Bug causes {} m/s velocity change per frame",
            buggy_velocity_change
        );

        assert!(
            correct_velocity_change < 0.5,
            "Correct physics: {} m/s velocity change per frame",
            correct_velocity_change
        );

        // The bug causes 65× too much velocity change per frame
        let ratio = buggy_velocity_change / correct_velocity_change;
        assert!(
            (ratio - 65.0).abs() < 0.1,
            "Bug causes {}x velocity overscaling",
            ratio
        );
    }
}
