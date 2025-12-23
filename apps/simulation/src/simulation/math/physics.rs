//! Physics calculations using correct F=ma unit conversions.
//!
//! This module provides pure functions for steering behaviors that correctly
//! convert forces (Newtons) to acceleration (m/s²) using F=ma → a=F/m.

use super::magnitude_sq;

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

    const DEFAULT_MAX_FORCE: f32 = 390.0;
    const DEFAULT_MASS: f32 = 65.0;
    const EXPECTED_MAX_ACCEL: f32 = 6.0;

    #[test]
    fn force_and_acceleration_are_different_units() {
        let max_force = DEFAULT_MAX_FORCE;
        let mass = DEFAULT_MASS;
        let correct_accel = max_force / mass;

        assert!(
            (correct_accel - EXPECTED_MAX_ACCEL).abs() < 0.01,
            "Max accel should be {} m/s², got {}",
            EXPECTED_MAX_ACCEL,
            correct_accel
        );
    }

    #[test]
    fn clamp_steering_divides_by_mass() {
        let steer_x = 500.0;
        let steer_y = 0.0;

        let (ax, ay) =
            clamp_steering_to_max_accel(steer_x, steer_y, DEFAULT_MAX_FORCE, DEFAULT_MASS);

        let accel_mag = (ax * ax + ay * ay).sqrt();

        assert!(
            (accel_mag - EXPECTED_MAX_ACCEL).abs() < 0.01,
            "Clamped acceleration should be {} m/s², got {}",
            EXPECTED_MAX_ACCEL,
            accel_mag
        );
    }

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

    #[test]
    fn zero_mass_returns_zero_acceleration() {
        let (ax, ay) = force_to_acceleration(100.0, 100.0, 0.0);
        assert_eq!(
            (ax, ay),
            (0.0, 0.0),
            "Zero mass should return zero acceleration"
        );
    }
}
