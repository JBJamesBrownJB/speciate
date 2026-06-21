use bevy_ecs::prelude::*;
use bevy_reflect::Reflect;
use serde::{Deserialize, Serialize};

use crate::simulation::creatures::constants::{
    ACCEL_SIZE_EXPONENT, BASE_ACCELERATION, BASE_MAX_SPEED, DEFAULT_MASS, SPEED_SIZE_EXPONENT,
};
use crate::simulation::math::fast_atan2;

#[derive(Component, Clone, Copy, Debug, Default, Serialize, Deserialize, Reflect)]
#[reflect(Component)]
pub struct Position {
    pub x: f32,
    pub y: f32,
}

#[derive(Component, Clone, Copy, Debug, Default, Serialize, Deserialize, Reflect)]
#[reflect(Component)]
pub struct Velocity {
    pub vx: f32,
    pub vy: f32,
}

impl Velocity {
    pub fn magnitude(&self) -> f32 {
        (self.vx * self.vx + self.vy * self.vy).sqrt()
    }

    pub fn normalize(&mut self) {
        let mag = self.magnitude();
        if mag > 0.0 {
            self.vx /= mag;
            self.vy /= mag;
        }
    }

    pub fn limit(&mut self, max: f32) {
        let mag = self.magnitude();
        if mag > max {
            self.normalize();
            self.vx *= max;
            self.vy *= max;
        }
    }

    pub fn angle(&self) -> f32 {
        self.vy.atan2(self.vx)
    }
}

#[derive(Component, Clone, Copy, Debug, Default, Serialize, Deserialize, Reflect)]
#[reflect(Component)]
pub struct Acceleration {
    pub ax: f32,
    pub ay: f32,
}

#[derive(Component, Clone, Copy, Debug, Serialize, Deserialize, Reflect)]
#[reflect(Component)]
pub struct BodySize {
    pub length: f32,
    #[serde(skip, default = "default_inv_sqrt")]
    #[reflect(ignore)]
    pub inv_sqrt_length: f32,
}

fn default_inv_sqrt() -> f32 {
    1.0
}

impl BodySize {
    pub fn new(length: f32) -> Self {
        Self {
            length,
            inv_sqrt_length: 1.0 / length.sqrt(),
        }
    }

    pub fn radius(&self) -> f32 {
        self.length / 2.0
    }

    pub fn mass(&self) -> f32 {
        DEFAULT_MASS * self.length.powi(3)
    }

    /// Size-scaled maximum acceleration (m/s²).
    /// Smaller creatures accelerate faster: accel = BASE / size^0.5
    pub fn max_acceleration(&self) -> f32 {
        BASE_ACCELERATION / self.length.powf(ACCEL_SIZE_EXPONENT)
    }

    /// Size-scaled maximum speed (m/s).
    /// Larger creatures have higher top speed: speed = BASE × size^0.25
    pub fn max_speed(&self) -> f32 {
        BASE_MAX_SPEED * self.length.powf(SPEED_SIZE_EXPONENT)
    }

    pub fn max_force(&self) -> f32 {
        self.mass() * self.max_acceleration()
    }
}

impl Default for BodySize {
    fn default() -> Self {
        Self::new(1.0)
    }
}

#[derive(Resource, Clone, Copy, Debug)]
pub struct DeltaTime(pub f32);

impl Default for DeltaTime {
    fn default() -> Self {
        Self(0.05)
    }
}

#[derive(Resource, Clone, Copy, Debug, Default)]
pub struct PhysicsTick(pub u64);

impl PhysicsTick {
    pub fn increment(&mut self) {
        self.0 = self.0.wrapping_add(1);
    }

    pub fn get(&self) -> u64 {
        self.0
    }
}

#[derive(Resource, Clone, Copy, Debug)]
pub struct ActualTickRate(pub f32);

impl Default for ActualTickRate {
    fn default() -> Self {
        Self(-1.0)
    }
}

/// Runtime-adjustable frequency divisors for cognitive systems.
/// Divisor=1 means every tick (full rate), divisor=N means every Nth tick.
/// Default is 8 (standard benchmark throttle); clamp floor is 2 (no "off" option).
/// Uses entity-ID bucketing to distribute updates evenly across ticks.
#[derive(Resource, Clone, Copy, Debug)]
pub struct FreqConfig {
    pub perception_divisor: u8,
    pub behavior_divisor: u8,
    pub steering_divisor: u8,
}

impl Default for FreqConfig {
    fn default() -> Self {
        Self {
            perception_divisor: 8, // Default 8 (standard benchmark throttle); clamp floor is still 2
            behavior_divisor: 8,   // Default 8 (standard benchmark throttle); clamp floor is still 2
            steering_divisor: 1,   // Keep 1 (steering throttling removed)
        }
    }
}

impl FreqConfig {
    /// Clamp divisor to power-of-2: 2, 4, or 8 (minimum 2, no "off" option).
    ///
    /// Power-of-2 required for bitwise AND optimization in FrequencyThrottle:
    /// `entity.index() & (divisor-1)` is 1 CPU cycle vs 30 cycles for modulo.
    pub fn clamp_power_of_2(divisor: u8) -> u8 {
        match divisor {
            0..=2 => 2, // Minimum is 2
            3..=4 => 4,
            _ => 8,
        }
    }
}

#[derive(Resource, Clone, Copy, Debug)]
pub struct BoundaryConfig {
    pub min_x: f32,
    pub max_x: f32,
    pub min_y: f32,
    pub max_y: f32,
    pub margin: f32,
    pub max_force: f32,
}

impl Default for BoundaryConfig {
    fn default() -> Self {
        use super::world_bounds::MAX_WORLD_SIZE;
        Self {
            min_x: -MAX_WORLD_SIZE,
            max_x: MAX_WORLD_SIZE,
            min_y: -MAX_WORLD_SIZE,
            max_y: MAX_WORLD_SIZE,
            margin: MAX_WORLD_SIZE / 100.0,
            max_force: 1.0,
        }
    }
}

impl BoundaryConfig {
    pub fn center(&self) -> (f32, f32) {
        (
            (self.min_x + self.max_x) / 2.0,
            (self.min_y + self.max_y) / 2.0,
        )
    }

    pub fn dimensions(&self) -> (f32, f32) {
        (self.max_x - self.min_x, self.max_y - self.min_y)
    }
}

#[derive(Component, Clone, Copy, Debug, Serialize, Deserialize, Reflect)]
#[reflect(Component)]
pub struct Rotation {
    pub radians: f32,
    #[serde(skip, default = "default_cos")]
    #[reflect(ignore)]
    pub cos_radians: f32,
    #[serde(skip, default)]
    #[reflect(ignore)]
    pub sin_radians: f32,
}

fn default_cos() -> f32 {
    1.0
}

impl Default for Rotation {
    fn default() -> Self {
        Self {
            radians: 0.0,
            cos_radians: 1.0, // cos(0) = 1
            sin_radians: 0.0, // sin(0) = 0
        }
    }
}

impl Rotation {
    pub fn new(radians: f32) -> Self {
        Self {
            radians,
            cos_radians: radians.cos(),
            sin_radians: radians.sin(),
        }
    }

    /// Set rotation from normalized velocity direction.
    /// Avoids trig entirely - just uses the direction components directly.
    #[inline(always)]
    pub fn set_from_velocity(&mut self, vx: f32, vy: f32) {
        let mag_sq = vx * vx + vy * vy;
        if mag_sq > 0.0 {
            let inv_mag = 1.0 / mag_sq.sqrt();
            self.cos_radians = vx * inv_mag;
            self.sin_radians = vy * inv_mag;
            // Only compute radians when needed (rarely used after perception reads cached)
            self.radians = fast_atan2(vy, vx);
        }
    }

    /// Fast path: set cos/sin directly without computing radians (for high-frequency updates)
    #[inline(always)]
    pub fn set_direction(&mut self, vx: f32, vy: f32) {
        let mag_sq = vx * vx + vy * vy;
        if mag_sq > 0.0 {
            let inv_mag = 1.0 / mag_sq.sqrt();
            self.cos_radians = vx * inv_mag;
            self.sin_radians = vy * inv_mag;
            // Defer radians computation - only set when explicitly requested
        }
    }

    /// Update radians from cached cos/sin (call after set_direction if radians needed)
    #[inline(always)]
    pub fn sync_radians(&mut self) {
        self.radians = self.sin_radians.atan2(self.cos_radians);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_position_creation() {
        let pos = Position { x: 10.0, y: 20.0 };
        assert_eq!(pos.x, 10.0);
        assert_eq!(pos.y, 20.0);
    }

    #[test]
    fn test_velocity_creation() {
        let vel = Velocity { vx: 1.5, vy: -2.5 };
        assert_eq!(vel.vx, 1.5);
        assert_eq!(vel.vy, -2.5);
    }

    #[test]
    fn test_velocity_magnitude() {
        let vel = Velocity { vx: 3.0, vy: 4.0 };
        assert_eq!(vel.magnitude(), 5.0);

        let zero_vel = Velocity { vx: 0.0, vy: 0.0 };
        assert_eq!(zero_vel.magnitude(), 0.0);
    }

    #[test]
    fn test_velocity_normalize() {
        let mut vel = Velocity { vx: 3.0, vy: 4.0 };
        vel.normalize();

        assert!((vel.vx - 0.6).abs() < 0.001);
        assert!((vel.vy - 0.8).abs() < 0.001);
        assert!((vel.magnitude() - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_velocity_normalize_zero() {
        let mut vel = Velocity { vx: 0.0, vy: 0.0 };
        vel.normalize();

        assert_eq!(vel.vx, 0.0);
        assert_eq!(vel.vy, 0.0);
    }

    #[test]
    fn test_velocity_limit_under_max() {
        let mut vel = Velocity { vx: 3.0, vy: 4.0 };
        vel.limit(10.0);

        assert_eq!(vel.vx, 3.0);
        assert_eq!(vel.vy, 4.0);
    }

    #[test]
    fn test_velocity_limit_over_max() {
        let mut vel = Velocity { vx: 3.0, vy: 4.0 };
        vel.limit(2.5);

        assert!((vel.magnitude() - 2.5).abs() < 0.001);
        assert!((vel.vx - 1.5).abs() < 0.001);
        assert!((vel.vy - 2.0).abs() < 0.001);
    }

    #[test]
    fn test_velocity_limit_zero() {
        let mut vel = Velocity { vx: 0.0, vy: 0.0 };
        vel.limit(5.0);

        assert_eq!(vel.vx, 0.0);
        assert_eq!(vel.vy, 0.0);
    }

    #[test]
    fn test_velocity_angle_quadrants() {
        let vel1 = Velocity { vx: 1.0, vy: 1.0 };
        assert!((vel1.angle() - std::f32::consts::FRAC_PI_4).abs() < 0.001);

        let vel2 = Velocity { vx: -1.0, vy: 1.0 };
        assert!((vel2.angle() - 3.0 * std::f32::consts::FRAC_PI_4).abs() < 0.001);

        let vel4 = Velocity { vx: 1.0, vy: -1.0 };
        assert!((vel4.angle() + std::f32::consts::FRAC_PI_4).abs() < 0.001);
    }

    #[test]
    fn test_body_size_mass_scales_with_length_cubed() {
        let size_1m = BodySize::new(1.0);
        let size_2m = BodySize::new(2.0);

        let mass_1m = size_1m.mass();
        let mass_2m = size_2m.mass();

        assert_eq!(mass_1m, DEFAULT_MASS);
        assert_eq!(mass_2m, DEFAULT_MASS * 8.0); // 2^3 = 8
        assert_eq!(mass_2m / mass_1m, 8.0);
    }

    #[test]
    fn test_body_size_max_force_derives_from_mass_and_size() {
        let size = BodySize::new(1.0);

        let expected_mass = DEFAULT_MASS;
        let expected_max_force = expected_mass * size.max_acceleration();

        assert_eq!(size.mass(), expected_mass);
        assert_eq!(size.max_force(), expected_max_force);
    }

    #[test]
    fn test_larger_creatures_have_proportionally_more_force() {
        let small = BodySize::new(0.5);
        let medium = BodySize::new(1.0);
        let large = BodySize::new(2.0);

        assert!(small.max_force() < medium.max_force());
        assert!(medium.max_force() < large.max_force());

        // Force scales with length^2.5 (mass × size-scaled acceleration)
        // mass ∝ length³, max_accel ∝ 1/length^0.5
        // max_force ∝ length³ / length^0.5 = length^2.5
        let force_ratio = large.max_force() / small.max_force();
        let length_ratio = 2.0_f32 / 0.5; // = 4
        let expected_ratio = length_ratio.powf(2.5); // 4^2.5 = 32
        assert!((force_ratio - expected_ratio).abs() < 0.001);
    }

    #[test]
    fn test_clamp_power_of_2_minimum_is_2() {
        assert_eq!(FreqConfig::clamp_power_of_2(0), 2);
        assert_eq!(FreqConfig::clamp_power_of_2(1), 2);
        assert_eq!(FreqConfig::clamp_power_of_2(2), 2);
    }

    #[test]
    fn test_clamp_power_of_2_rounds_to_next_power() {
        assert_eq!(FreqConfig::clamp_power_of_2(3), 4);
        assert_eq!(FreqConfig::clamp_power_of_2(4), 4);
        assert_eq!(FreqConfig::clamp_power_of_2(5), 8);
        assert_eq!(FreqConfig::clamp_power_of_2(6), 8);
        assert_eq!(FreqConfig::clamp_power_of_2(7), 8);
        assert_eq!(FreqConfig::clamp_power_of_2(8), 8);
    }

    #[test]
    fn test_clamp_power_of_2_maximum_is_8() {
        assert_eq!(FreqConfig::clamp_power_of_2(9), 8);
        assert_eq!(FreqConfig::clamp_power_of_2(16), 8);
        assert_eq!(FreqConfig::clamp_power_of_2(100), 8);
        assert_eq!(FreqConfig::clamp_power_of_2(255), 8);
    }

    #[test]
    fn freq_config_default_is_8_for_cognitive_systems() {
        let cfg = FreqConfig::default();
        assert_eq!(cfg.perception_divisor, 8);
        assert_eq!(cfg.behavior_divisor, 8);
        assert_eq!(cfg.steering_divisor, 1);
    }
}
