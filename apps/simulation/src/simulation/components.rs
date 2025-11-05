//! ECS Components for the Speciate simulation

use bevy_ecs::prelude::*;
use serde::{Deserialize, Serialize};

/// Stable, unique identifier for each agent
/// This ID is assigned at spawn time and never changes, even when the entity is despawned/respawned
#[derive(Component, Clone, Copy, Debug, Serialize, Deserialize)]
pub struct AgentId(pub u32);

/// Position component representing an entity's location in 2D space
#[derive(Component, Clone, Copy, Debug, Default, Serialize, Deserialize)]
pub struct Position {
    pub x: f32,
    pub y: f32,
}

impl Position {
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

/// Acceleration component for steering forces (Nature of Code)
#[derive(Component, Clone, Copy, Debug, Default, Serialize, Deserialize)]
pub struct Acceleration {
    pub ax: f32,
    pub ay: f32,
}


/// Rotation component for creature orientation
#[derive(Component, Clone, Copy, Debug, Default, Serialize, Deserialize)]
pub struct Rotation {
    pub radians: f32,
}

impl Rotation {
}

/// Behavior modes for creatures (A-Life state machine)
#[derive(Clone, Copy, Debug, PartialEq, Default, Serialize, Deserialize)]
pub enum BehaviorMode {
    #[default]
    Wandering,
    Fleeing,
    Feeding,
    Resting,
    // Future: Mating, Hunting, Exploring, etc.
}

/// General creature state component (required for all creatures)
#[derive(Component, Clone, Copy, Debug, Serialize, Deserialize)]
pub struct CreatureState {
    pub behavior: BehaviorMode,
    pub energy: f32,
    pub age: f32,
    pub max_speed: f32,
}

impl Default for CreatureState {
    fn default() -> Self {
        Self {
            behavior: BehaviorMode::Wandering,
            energy: 100.0,
            age: 0.0,
            max_speed: 20.0,
        }
    }
}

impl CreatureState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn is_low_energy(&self) -> bool {
        self.energy < 30.0
    }

    pub fn is_exhausted(&self) -> bool {
        self.energy < 10.0
    }

    pub fn consume_energy(&mut self, amount: f32) {
        self.energy = (self.energy - amount).max(0.0);
    }

    pub fn restore_energy(&mut self, amount: f32) {
        self.energy = (self.energy + amount).min(100.0);
    }
}

/// Wander state for autonomous movement behavior (behavior-specific, optional)
#[derive(Component, Clone, Copy, Debug, Default, Serialize, Deserialize)]
pub struct WanderState {
    pub wander_angle: f32,
    pub wander_radius: f32,
    pub wander_distance: f32,
    pub angle_change: f32,
}

/// Flee state for escaping danger (behavior-specific, optional)
#[derive(Component, Clone, Copy, Debug, Serialize, Deserialize)]
pub struct FleeState {
    pub flee_speed_multiplier: f32,
}

impl FleeState {
    pub fn new(_threat: Option<Entity>) -> Self {
        Self {
            flee_speed_multiplier: 1.5,
        }
    }
}

/// Delta time resource for frame-independent physics
#[derive(Resource, Clone, Copy, Debug)]
pub struct DeltaTime(pub f32);

impl Default for DeltaTime {
    fn default() -> Self {
        Self(0.016) // Default to ~60 FPS
    }
}

/// Boundary configuration resource for world limits
#[derive(Resource, Clone, Copy, Debug)]
pub struct BoundaryConfig {
    pub width: f32,
    pub height: f32,
    pub margin: f32,
    pub max_force: f32,
}

impl Default for BoundaryConfig {
    fn default() -> Self {
        Self {
            width: 180.0,
            height: 130.0,
            margin: 20.0,
            max_force: 1.0,
        }
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
    fn test_creature_state_energy_management() {
        let mut state = CreatureState::new();
        let initial_energy = state.energy;

        state.consume_energy(10.0);
        assert_eq!(state.energy, initial_energy - 10.0);

        state.restore_energy(5.0);
        assert_eq!(state.energy, initial_energy - 5.0);
    }

    #[test]
    fn test_creature_state_exhaustion() {
        let mut state = CreatureState::new();

        // Drain to low energy (< 30)
        state.consume_energy(75.0); // 100 - 75 = 25
        assert!(state.is_low_energy());
        assert!(!state.is_exhausted());

        // Drain further to exhausted (< 10)
        state.consume_energy(20.0); // 25 - 20 = 5
        assert!(state.is_exhausted());
    }

    #[test]
    fn test_velocity_helper_methods() {
        let vel = Velocity { vx: 3.0, vy: 4.0 };

        let magnitude = vel.magnitude();
        assert_eq!(magnitude, 5.0); // 3-4-5 triangle

        let angle = vel.angle();
        let expected = 4.0f32.atan2(3.0);
        assert!((angle - expected).abs() < 0.001);
    }

    /// Velocity Method Tests
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
