use std::f32::consts::{PI, TAU};

const EPSILON: f32 = 0.0001;

#[inline]
pub fn magnitude_sq(x: f32, y: f32) -> f32 {
    x * x + y * y
}

#[inline]
pub fn magnitude(x: f32, y: f32) -> f32 {
    magnitude_sq(x, y).sqrt()
}

#[inline]
pub fn normalize(x: f32, y: f32) -> (f32, f32) {
    let mag_sq = magnitude_sq(x, y);
    if mag_sq < EPSILON {
        return (0.0, 0.0);
    }
    let mag = mag_sq.sqrt();
    (x / mag, y / mag)
}

#[inline]
pub fn clamp_force(x: f32, y: f32, max_force: f32) -> (f32, f32) {
    let mag_sq = magnitude_sq(x, y);
    let max_sq = max_force * max_force;
    if mag_sq > max_sq {
        let scale = max_force / mag_sq.sqrt();
        (x * scale, y * scale)
    } else {
        (x, y)
    }
}

#[inline]
pub fn normalize_angle(angle: f32) -> f32 {
    let mut a = angle.rem_euclid(TAU);
    if a > PI {
        a -= TAU;
    }
    a
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_magnitude_sq_zero() {
        assert_eq!(magnitude_sq(0.0, 0.0), 0.0);
    }

    #[test]
    fn test_magnitude_sq_unit_x() {
        assert_eq!(magnitude_sq(1.0, 0.0), 1.0);
    }

    #[test]
    fn test_magnitude_sq_unit_y() {
        assert_eq!(magnitude_sq(0.0, 1.0), 1.0);
    }

    #[test]
    fn test_magnitude_sq_3_4_5() {
        assert_eq!(magnitude_sq(3.0, 4.0), 25.0);
    }

    #[test]
    fn test_magnitude_zero() {
        assert_eq!(magnitude(0.0, 0.0), 0.0);
    }

    #[test]
    fn test_magnitude_unit_x() {
        assert_eq!(magnitude(1.0, 0.0), 1.0);
    }

    #[test]
    fn test_magnitude_3_4_5() {
        assert_eq!(magnitude(3.0, 4.0), 5.0);
    }

    #[test]
    fn test_normalize_zero_returns_zero() {
        assert_eq!(normalize(0.0, 0.0), (0.0, 0.0));
    }

    #[test]
    fn test_normalize_near_zero_returns_zero() {
        assert_eq!(normalize(0.00001, 0.0), (0.0, 0.0));
    }

    #[test]
    fn test_normalize_unit_x() {
        let (x, y) = normalize(5.0, 0.0);
        assert!((x - 1.0).abs() < 0.0001);
        assert!(y.abs() < 0.0001);
    }

    #[test]
    fn test_normalize_unit_y() {
        let (x, y) = normalize(0.0, 5.0);
        assert!(x.abs() < 0.0001);
        assert!((y - 1.0).abs() < 0.0001);
    }

    #[test]
    fn test_normalize_diagonal() {
        let (x, y) = normalize(3.0, 4.0);
        assert!((x - 0.6).abs() < 0.0001);
        assert!((y - 0.8).abs() < 0.0001);
    }

    #[test]
    fn test_normalize_negative() {
        let (x, y) = normalize(-3.0, -4.0);
        assert!((x - (-0.6)).abs() < 0.0001);
        assert!((y - (-0.8)).abs() < 0.0001);
    }

    #[test]
    fn test_clamp_force_under_limit() {
        let (x, y) = clamp_force(3.0, 4.0, 10.0);
        assert_eq!((x, y), (3.0, 4.0));
    }

    #[test]
    fn test_clamp_force_at_limit() {
        let (x, y) = clamp_force(3.0, 4.0, 5.0);
        assert!((x - 3.0).abs() < 0.0001);
        assert!((y - 4.0).abs() < 0.0001);
    }

    #[test]
    fn test_clamp_force_over_limit() {
        let (x, y) = clamp_force(6.0, 8.0, 5.0);
        let mag = magnitude(x, y);
        assert!((mag - 5.0).abs() < 0.0001, "Magnitude should be 5.0, got {}", mag);
        assert!((x - 3.0).abs() < 0.0001, "X should be 3.0, got {}", x);
        assert!((y - 4.0).abs() < 0.0001, "Y should be 4.0, got {}", y);
    }

    #[test]
    fn test_clamp_force_preserves_direction() {
        let (x, y) = clamp_force(-6.0, -8.0, 5.0);
        assert!((x - (-3.0)).abs() < 0.0001);
        assert!((y - (-4.0)).abs() < 0.0001);
    }

    #[test]
    fn test_clamp_force_zero_input() {
        let (x, y) = clamp_force(0.0, 0.0, 10.0);
        assert_eq!((x, y), (0.0, 0.0));
    }

    #[test]
    fn test_normalize_angle_zero() {
        assert!((normalize_angle(0.0) - 0.0).abs() < 0.0001);
    }

    #[test]
    fn test_normalize_angle_positive() {
        assert!((normalize_angle(PI / 2.0) - PI / 2.0).abs() < 0.0001);
    }

    #[test]
    fn test_normalize_angle_negative() {
        assert!((normalize_angle(-PI / 2.0) - (-PI / 2.0)).abs() < 0.0001);
    }

    #[test]
    fn test_normalize_angle_over_pi() {
        let result = normalize_angle(3.0 * PI / 2.0);
        assert!((result - (-PI / 2.0)).abs() < 0.0001, "Expected -PI/2, got {}", result);
    }

    #[test]
    fn test_normalize_angle_over_2pi() {
        let result = normalize_angle(2.5 * PI);
        assert!((result - PI / 2.0).abs() < 0.0001, "Expected PI/2, got {}", result);
    }

    #[test]
    fn test_normalize_angle_negative_wrap() {
        let result = normalize_angle(-3.0 * PI / 2.0);
        assert!((result - PI / 2.0).abs() < 0.0001, "Expected PI/2, got {}", result);
    }
}
