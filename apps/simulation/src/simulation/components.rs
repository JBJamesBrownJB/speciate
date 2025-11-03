//! ECS Components for the Speciate simulation

use bevy_ecs::prelude::*;
use serde::{Deserialize, Serialize};

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

/// Size component for creature dimensions
#[derive(Component, Clone, Copy, Debug, Serialize, Deserialize)]
pub struct Size {
    pub width: f32,
    pub height: f32,
}

impl Size {
}

/// Behavior modes for creatures (A-Life state machine)
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize, Default)]
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
    pub species_id: u32,
    pub max_speed: f32,
    pub perception_radius: f32,
}

impl CreatureState {
    pub fn new(species_id: u32) -> Self {
        Self {
            behavior: BehaviorMode::Wandering,
            energy: 100.0,
            age: 0.0,
            species_id,
            max_speed: 20.0,
            perception_radius: 50.0,
        }
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
#[derive(Component, Clone, Copy, Debug, Default)]
pub struct WanderState {
    pub wander_angle: f32,
    pub wander_radius: f32,
    pub wander_distance: f32,
    pub angle_change: f32,
}

/// Flee state for escaping danger (behavior-specific, optional)
#[derive(Component, Clone, Copy, Debug)]
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

/// Creature data for network serialization (not a component)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreatureData {
    pub id: u32,
    pub x: f32,
    pub y: f32,
    pub rotation: f32,
    pub width: f32,
    pub height: f32,
    pub behavior: BehaviorMode,
    pub energy: f32,
    pub species_id: u32,
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
}
