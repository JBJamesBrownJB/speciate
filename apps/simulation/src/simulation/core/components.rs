use bevy_ecs::prelude::*;
use bevy_reflect::Reflect;
use serde::{Deserialize, Serialize};

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

#[derive(Component, Clone, Copy, Debug, Default, Serialize, Deserialize, Reflect)]
#[reflect(Component)]
pub struct Rotation {
    pub radians: f32,
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
}
