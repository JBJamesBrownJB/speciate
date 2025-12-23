//! Arrival behavior: smooth deceleration to target using correct F=ma physics.
//!
//! Implements Reynolds steering arrival with:
//! - Dynamic slowing radius based on kinematic braking distance
//! - Snap threshold to eliminate oscillation at close range
//! - Proper unit conversion (force / mass = acceleration)

/// Parameters for arrival deceleration calculation.
/// Uses primitives (not ECS components) for testability.
#[derive(Debug, Clone, Copy)]
pub struct ArrivalParams {
    /// Current velocity (m/s)
    pub velocity: (f32, f32),
    /// Vector from creature center to target center
    pub to_target: (f32, f32),
    /// Creature radius (m)
    pub self_radius: f32,
    /// Target radius (m)
    pub target_radius: f32,
    /// Maximum speed (m/s)
    pub max_speed: f32,
    /// Maximum force (Newtons)
    pub max_force: f32,
    /// Creature mass (kg)
    pub mass: f32,
}

/// Result of arrival calculation.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ArrivalResult {
    /// Acceleration to apply (m/s²)
    pub acceleration: (f32, f32),
    /// Should transition to Catatonic (arrived at target)
    pub arrived: bool,
}

/// Snap threshold for edge-to-edge distance (meters)
/// Creature "arrives" when its edge touches the target's edge
const SNAP_EDGE_THRESHOLD: f32 = 0.1;

/// Maximum speed at which snap-to-target can occur (m/s)
/// With drag 0.5, coast distance = speed / drag, so SNAP_MAX_SPEED of 2.0 gives ~4m coast
const SNAP_MAX_SPEED: f32 = 2.0;

/// Calculate arrival/braking acceleration using correct F=ma physics.
///
/// Uses Reynolds steering arrival behavior:
/// 1. Snap to target when edges nearly touching AND moving slowly
/// 2. Dynamic slowing radius based on kinematic braking distance: d = v²/(2a)
/// 3. Linear speed ramp within slowing radius
/// 4. Steering force = desired_velocity - current_velocity
/// 5. Clamp acceleration to max_force / mass
pub fn calculate_arrival(params: &ArrivalParams) -> ArrivalResult {
    let (vx, vy) = params.velocity;
    let (dx, dy) = params.to_target;

    // Calculate distances
    let center_distance_sq = dx * dx + dy * dy;
    let center_distance = center_distance_sq.sqrt();

    // Edge-to-edge distance (0 = edges touching, negative = overlapping)
    let edge_distance = (center_distance - params.self_radius - params.target_radius).max(0.0);

    // Calculate current speed
    let speed_sq = vx * vx + vy * vy;
    let speed = speed_sq.sqrt();

    // 1. SNAP THRESHOLD - eliminates oscillation at close range
    // Snap when edges are nearly touching (edge_distance < threshold)
    if edge_distance < SNAP_EDGE_THRESHOLD && speed < SNAP_MAX_SPEED {
        return ArrivalResult {
            acceleration: (0.0, 0.0),
            arrived: true,
        };
    }

    // Direction to target (handle zero distance)
    let (dir_x, dir_y) = if center_distance > 0.001 {
        (dx / center_distance, dy / center_distance)
    } else {
        return ArrivalResult {
            acceleration: (0.0, 0.0),
            arrived: true,
        };
    };

    // 2. DYNAMIC SLOWING RADIUS based on kinematic braking distance
    // d = v² / (2 × a_max), where a_max = F_max / m (correct F=ma conversion!)
    let max_decel = params.max_force / params.mass;
    let kinematic_brake_distance = if max_decel > 0.001 {
        speed_sq / (2.0 * max_decel)
    } else {
        0.0
    };

    // Slowing radius = braking distance + small margin
    let slowing_radius = kinematic_brake_distance + 0.5;

    // 3. LINEAR SPEED RAMP within slowing radius (Reynolds arrival)
    // Use edge_distance - creature wants to reach the TARGET'S EDGE, not center
    let desired_speed = if edge_distance < slowing_radius {
        // Linear interpolation: 0 at edge contact, max_speed at slowing_radius
        let t = edge_distance / slowing_radius;
        params.max_speed * t.clamp(0.0, 1.0)
    } else {
        // Outside slowing radius - full speed
        params.max_speed
    };

    // 4. STEERING = desired_velocity - current_velocity (Reynolds formula)
    let desired_vx = dir_x * desired_speed;
    let desired_vy = dir_y * desired_speed;
    let steer_x = desired_vx - vx;
    let steer_y = desired_vy - vy;

    // 5. CLAMP steering force to max_force, then convert to acceleration (F=ma → a=F/m)
    let steer_mag_sq = steer_x * steer_x + steer_y * steer_y;
    let max_accel = params.max_force / params.mass; // Correct unit conversion!
    let max_accel_sq = max_accel * max_accel;

    let (ax, ay) = if steer_mag_sq > max_accel_sq {
        let steer_mag = steer_mag_sq.sqrt();
        let scale = max_accel / steer_mag;
        (steer_x * scale, steer_y * scale)
    } else {
        (steer_x, steer_y)
    };

    ArrivalResult {
        acceleration: (ax, ay),
        arrived: false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Test 1: Core bug reproduction - high speed approach should NOT reverse velocity
    #[test]
    fn high_speed_approach_does_not_reverse_velocity() {
        let params = ArrivalParams {
            velocity: (10.0, 0.0),
            to_target: (0.15, 0.0),
            self_radius: 0.5,
            target_radius: 0.5,
            max_speed: 15.0,
            max_force: 390.0,
            mass: 65.0,
        };

        let result = calculate_arrival(&params);

        // After applying acceleration for one frame (dt=0.05), velocity should still be positive
        let dt = 0.05;
        let new_vx = params.velocity.0 + result.acceleration.0 * dt;

        assert!(
            new_vx > 0.0,
            "Velocity should remain positive after braking, got {}. \
             This indicates the oscillation bug - velocity reversed!",
            new_vx
        );
    }

    // Test 2: Smooth deceleration over multiple frames (no oscillation)
    #[test]
    fn deceleration_is_smooth_over_multiple_frames() {
        let mut vx: f32 = 10.0;
        let mut vy: f32 = 0.0;
        let mut px: f32 = -50.0;
        let py: f32 = 0.0;
        let target_x: f32 = 0.0;
        let target_y: f32 = 0.0;
        let dt: f32 = 0.05;

        let mut sign_changes = 0;
        let mut prev_vx_sign = vx.signum();

        // Simulate 100 frames (5 seconds)
        for _ in 0..100 {
            let params = ArrivalParams {
                velocity: (vx, vy),
                to_target: (target_x - px, target_y - py),
                self_radius: 0.5,
                target_radius: 0.5,
                max_speed: 15.0,
                max_force: 390.0,
                mass: 65.0,
            };

            let result = calculate_arrival(&params);

            if result.arrived {
                break;
            }

            // Apply acceleration
            vx += result.acceleration.0 * dt;
            vy += result.acceleration.1 * dt;

            // Integrate position
            px += vx * dt;

            // Track sign changes (oscillation detection)
            let current_sign = vx.signum();
            if current_sign != prev_vx_sign && vx.abs() > 0.1 {
                sign_changes += 1;
            }
            prev_vx_sign = current_sign;
        }

        assert!(
            sign_changes <= 1,
            "Velocity changed sign {} times - oscillation detected!",
            sign_changes
        );
    }

    // Test 3: Snap to target when very close and slow
    #[test]
    fn snap_to_target_when_very_close() {
        let params = ArrivalParams {
            velocity: (0.5, 0.0),
            to_target: (0.05, 0.0),
            self_radius: 0.5,
            target_radius: 0.5,
            max_speed: 15.0,
            max_force: 390.0,
            mass: 65.0,
        };

        let result = calculate_arrival(&params);

        assert!(
            result.arrived,
            "Should snap to target when edge_distance < snap_threshold and speed < 1.0"
        );
    }

    // Test 4: Acceleration respects max_accel (max_force / mass) limit
    #[test]
    fn acceleration_respects_max_accel_limit() {
        let params = ArrivalParams {
            velocity: (15.0, 0.0),
            to_target: (1.0, 0.0),
            self_radius: 0.5,
            target_radius: 0.5,
            max_speed: 15.0,
            max_force: 390.0,
            mass: 65.0,
        };

        let result = calculate_arrival(&params);

        // Acceleration is clamped to max_accel = max_force / mass (F=ma)
        let max_accel = params.max_force / params.mass; // 390/65 = 6 m/s²
        let accel_magnitude =
            (result.acceleration.0.powi(2) + result.acceleration.1.powi(2)).sqrt();

        assert!(
            accel_magnitude <= max_accel + 0.001,
            "Acceleration magnitude {} m/s² exceeds max_accel {} m/s² (max_force {} / mass {})",
            accel_magnitude,
            max_accel,
            params.max_force,
            params.mass
        );
    }

    // Test 5: Inside slowing radius reduces desired speed
    #[test]
    fn inside_slowing_radius_reduces_desired_speed() {
        // Far approach at low speed - should accelerate toward max speed
        let far_params = ArrivalParams {
            velocity: (2.0, 0.0),
            to_target: (100.0, 0.0),
            self_radius: 0.5,
            target_radius: 0.5,
            max_speed: 15.0,
            max_force: 390.0,
            mass: 65.0,
        };

        // Close approach at high speed - should brake (negative acceleration)
        // At 10 m/s, slowing_radius = 100/(2*6) + 0.1 = 8.4m
        // Edge distance = 2m - 0.5 - 0.5 = 1m, which is inside slowing radius
        let close_params = ArrivalParams {
            velocity: (10.0, 0.0),
            to_target: (2.0, 0.0),
            self_radius: 0.5,
            target_radius: 0.5,
            max_speed: 15.0,
            max_force: 390.0,
            mass: 65.0,
        };

        let far_result = calculate_arrival(&far_params);
        let close_result = calculate_arrival(&close_params);

        // Far away at low speed: should accelerate (positive)
        assert!(
            far_result.acceleration.0 > 0.0,
            "Far approach should have positive acceleration. Got: {}",
            far_result.acceleration.0
        );

        // Close at high speed: should brake (negative or much less positive)
        assert!(
            far_result.acceleration.0 > close_result.acceleration.0,
            "Close high-speed approach should have less forward acceleration than far slow approach. \
             Far: {}, Close: {}",
            far_result.acceleration.0,
            close_result.acceleration.0
        );
    }

    // Test 6: Outside slowing radius uses max speed
    #[test]
    fn outside_slowing_radius_uses_max_speed() {
        let params = ArrivalParams {
            velocity: (5.0, 0.0),
            to_target: (100.0, 0.0),
            self_radius: 0.5,
            target_radius: 0.5,
            max_speed: 15.0,
            max_force: 390.0,
            mass: 65.0,
        };

        let result = calculate_arrival(&params);

        // Should accelerate toward max_speed (15 m/s)
        // Current speed is 5, so steering should be positive
        assert!(
            result.acceleration.0 > 0.0,
            "Should accelerate toward target when far away. Got: {}",
            result.acceleration.0
        );
    }

    // Test 7: Stationary at target returns arrived
    #[test]
    fn stationary_at_target_returns_arrived() {
        let params = ArrivalParams {
            velocity: (0.0, 0.0),
            to_target: (0.02, 0.0),
            self_radius: 0.5,
            target_radius: 0.5,
            max_speed: 15.0,
            max_force: 390.0,
            mass: 65.0,
        };

        let result = calculate_arrival(&params);

        assert!(
            result.arrived,
            "Should be arrived when stationary and very close to target"
        );
    }

    // Test 8: 2D diagonal approach works correctly
    #[test]
    fn diagonal_approach_works() {
        let params = ArrivalParams {
            velocity: (7.07, 7.07),
            to_target: (10.0, 10.0),
            self_radius: 0.5,
            target_radius: 0.5,
            max_speed: 15.0,
            max_force: 390.0,
            mass: 65.0,
        };

        let result = calculate_arrival(&params);

        // Should have acceleration in both dimensions
        // Direction should be roughly toward target
        let accel_mag = (result.acceleration.0.powi(2) + result.acceleration.1.powi(2)).sqrt();

        assert!(
            accel_mag > 0.0 || result.arrived,
            "Should produce non-zero acceleration or arrive"
        );
    }
}
