//! Core spatial and physics components
//!
//! These components are shared across all domains and form the foundation
//! of the ECS simulation. They represent pure physical state with no
//! behavior-specific logic.

use bevy_ecs::prelude::*;
use serde::{Deserialize, Serialize};

/// Position component representing an entity's location in 2D space
#[derive(Component, Clone, Copy, Debug, Default, Serialize, Deserialize)]
pub struct Position {
    pub x: f32,
    pub y: f32,
}

/// Velocity component representing an entity's speed and direction
#[derive(Component, Clone, Copy, Debug, Default, Serialize, Deserialize)]
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

/// Acceleration component for steering forces (Nature of Code pattern)
/// Systems ADD to acceleration, physics integrates it into velocity
#[derive(Component, Clone, Copy, Debug, Default, Serialize, Deserialize)]
pub struct Acceleration {
    pub ax: f32,
    pub ay: f32,
}

/// Body size component (volumetric physics)
/// Creatures occupy space on grid: 1m body = 1m × 1m area
#[derive(Component, Clone, Copy, Debug, Serialize, Deserialize)]
pub struct BodySize {
    pub length: f32, // Body length in meters
}

impl BodySize {
    pub fn new(length: f32) -> Self {
        Self { length }
    }

    /// Body radius (half the length) for edge-to-edge distance calculations
    pub fn radius(&self) -> f32 {
        self.length / 2.0
    }
}

impl Default for BodySize {
    fn default() -> Self {
        Self { length: 1.0 } // 1m default (wolf-sized)
    }
}

/// Delta time resource for frame-independent physics
#[derive(Resource, Clone, Copy, Debug)]
pub struct DeltaTime(pub f32);

impl Default for DeltaTime {
    fn default() -> Self {
        Self(0.05) // Default to 20 Hz simulation rate
    }
}

/// Physics tick counter for temporal variation
///
/// Increments every simulation update. Used for:
/// - Perlin noise seed (temporal variation in locomotion noise)
/// - Time-based events
/// - Deterministic randomness that varies over time
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

/// Measured tick rate resource (actual Hz, not target)
///
/// Updated by main loop after each tick based on rolling average.
/// Used for frontend HUD display to show actual simulation performance.
#[derive(Resource, Clone, Copy, Debug)]
pub struct ActualTickRate(pub f32);

impl Default for ActualTickRate {
    fn default() -> Self {
        Self(-1.0) // Sentinel: -1.0 means not yet measured
    }
}

/// Boundary configuration resource for world limits
/// Uses centered coordinate system: world extends from (min_x, min_y) to (max_x, max_y)
/// with (0, 0) at the center
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
        // World size: 2,000,000m × 2,000,000m (2,000 km × 2,000 km)
        // Centered at (0, 0) with ±1,000,000m extent
        const EXTENT: f32 = 1_000_000.0; // ±1 million meters
        Self {
            min_x: -EXTENT,
            max_x: EXTENT,
            min_y: -EXTENT,
            max_y: EXTENT,
            margin: 10_000.0, // 10km margin for soft boundary forces
            max_force: 1.0,
        }
    }
}

impl BoundaryConfig {
    /// Get world center coordinates (always 0, 0 for centered system)
    pub fn center(&self) -> (f32, f32) {
        (
            (self.min_x + self.max_x) / 2.0,
            (self.min_y + self.max_y) / 2.0,
        )
    }

    /// Get world dimensions (width, height)
    pub fn dimensions(&self) -> (f32, f32) {
        (self.max_x - self.min_x, self.max_y - self.min_y)
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

        // Should remain zero when normalizing zero velocity
        assert_eq!(vel.vx, 0.0);
        assert_eq!(vel.vy, 0.0);
    }

    #[test]
    fn test_velocity_limit_under_max() {
        let mut vel = Velocity { vx: 3.0, vy: 4.0 };
        vel.limit(10.0);

        // Should remain unchanged (magnitude 5.0 < 10.0)
        assert_eq!(vel.vx, 3.0);
        assert_eq!(vel.vy, 4.0);
    }

    #[test]
    fn test_velocity_limit_over_max() {
        let mut vel = Velocity { vx: 3.0, vy: 4.0 };
        vel.limit(2.5);

        // Should be limited to magnitude 2.5 but keep direction
        assert!((vel.magnitude() - 2.5).abs() < 0.001);
        assert!((vel.vx - 1.5).abs() < 0.001); // 3/5 * 2.5
        assert!((vel.vy - 2.0).abs() < 0.001); // 4/5 * 2.5
    }

    #[test]
    fn test_velocity_limit_zero() {
        let mut vel = Velocity { vx: 0.0, vy: 0.0 };
        vel.limit(5.0);

        // Should remain zero
        assert_eq!(vel.vx, 0.0);
        assert_eq!(vel.vy, 0.0);
    }

    #[test]
    fn test_velocity_angle_quadrants() {
        // Quadrant 1: positive x, positive y
        let vel1 = Velocity { vx: 1.0, vy: 1.0 };
        assert!((vel1.angle() - std::f32::consts::FRAC_PI_4).abs() < 0.001);

        // Quadrant 2: negative x, positive y
        let vel2 = Velocity { vx: -1.0, vy: 1.0 };
        assert!((vel2.angle() - 3.0 * std::f32::consts::FRAC_PI_4).abs() < 0.001);

        // Quadrant 4: positive x, negative y
        let vel4 = Velocity { vx: 1.0, vy: -1.0 };
        assert!((vel4.angle() + std::f32::consts::FRAC_PI_4).abs() < 0.001);
    }
}
