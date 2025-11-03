//! ECS Components for the Speciate simulation

use serde::{Deserialize, Serialize};

/// Position component representing an entity's location in 2D space
#[derive(Clone, Copy, Debug, Default, Serialize, Deserialize)]
pub struct Position {
    pub x: f32,
    pub y: f32,
}

impl Position {
    pub fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }
}

/// Velocity component representing an entity's speed and direction
#[derive(Clone, Copy, Debug, Default, Serialize, Deserialize)]
pub struct Velocity {
    pub vx: f32,
    pub vy: f32,
}

impl Velocity {
    pub fn new(vx: f32, vy: f32) -> Self {
        Self { vx, vy }
    }
}

/// Health component for entities that can take damage
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct Health {
    pub current: f32,
    pub max: f32,
}

impl Health {
    pub fn new(max: f32) -> Self {
        Self { current: max, max }
    }

    #[allow(dead_code)] // Will be used in Phase 2
    pub fn damage(&mut self, amount: f32) {
        self.current = (self.current - amount).max(0.0);
    }

    #[allow(dead_code)] // Will be used in Phase 2
    pub fn heal(&mut self, amount: f32) {
        self.current = (self.current + amount).min(self.max);
    }

    #[allow(dead_code)] // Will be used in Phase 2
    pub fn is_alive(&self) -> bool {
        self.current > 0.0
    }
}

/// ID component for unique entity identification
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct EntityId(pub u64);

impl EntityId {
    pub fn new(id: u64) -> Self {
        Self(id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_position_creation() {
        let pos = Position::new(10.0, 20.0);
        assert_eq!(pos.x, 10.0);
        assert_eq!(pos.y, 20.0);
    }

    #[test]
    fn test_velocity_creation() {
        let vel = Velocity::new(1.5, -2.5);
        assert_eq!(vel.vx, 1.5);
        assert_eq!(vel.vy, -2.5);
    }

    #[test]
    fn test_health_creation() {
        let health = Health::new(100.0);
        assert_eq!(health.current, 100.0);
        assert_eq!(health.max, 100.0);
        assert!(health.is_alive());
    }

    #[test]
    fn test_health_damage() {
        let mut health = Health::new(100.0);
        health.damage(25.0);
        assert_eq!(health.current, 75.0);
        assert!(health.is_alive());

        health.damage(100.0);
        assert_eq!(health.current, 0.0);
        assert!(!health.is_alive());
    }

    #[test]
    fn test_health_heal() {
        let mut health = Health::new(100.0);
        health.damage(50.0);
        health.heal(30.0);
        assert_eq!(health.current, 80.0);

        health.heal(50.0);
        assert_eq!(health.current, 100.0);
    }

    #[test]
    fn test_entity_id() {
        let id1 = EntityId::new(1);
        let id2 = EntityId::new(1);
        let id3 = EntityId::new(2);

        assert_eq!(id1, id2);
        assert_ne!(id1, id3);
    }
}
