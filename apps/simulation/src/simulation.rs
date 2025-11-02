//! Core simulation engine with ECS systems

use crate::components::{EntityId, Health, Position, Velocity};
use std::collections::HashMap;

/// Simple entity data structure to hold component data
#[derive(Clone, Debug)]
pub struct Entity {
    pub id: EntityId,
    pub position: Position,
    pub velocity: Velocity,
    pub health: Health,
}

impl Entity {
    pub fn new(id: EntityId, position: Position, velocity: Velocity, health: Health) -> Self {
        Self {
            id,
            position,
            velocity,
            health,
        }
    }
}

/// Main simulation state manager
pub struct Simulation {
    entities: HashMap<u64, Entity>,
    next_entity_id: u64,
    tick: u64,
    timestep: f32, // seconds per tick (1/20 = 0.05 for 20Hz)
}

impl Simulation {
    /// Create a new simulation with 20Hz timestep
    pub fn new() -> Self {
        Self {
            entities: HashMap::new(),
            next_entity_id: 1,
            tick: 0,
            timestep: 1.0 / 20.0, // 20 Hz
        }
    }

    /// Spawn a new entity in the simulation
    pub fn spawn_entity(
        &mut self,
        position: Position,
        velocity: Velocity,
        health: Health,
    ) -> EntityId {
        let id = EntityId::new(self.next_entity_id);
        self.next_entity_id += 1;

        let entity = Entity::new(id, position, velocity, health);
        self.entities.insert(id.0, entity);

        id
    }

    /// Update the simulation by one tick
    pub fn update(&mut self) {
        // Update positions based on velocities
        for entity in self.entities.values_mut() {
            entity.position.x += entity.velocity.vx * self.timestep;
            entity.position.y += entity.velocity.vy * self.timestep;
        }

        self.tick += 1;
    }

    /// Get all entities in the simulation
    pub fn get_entities(&self) -> impl Iterator<Item = &Entity> {
        self.entities.values()
    }

    /// Get entity count
    pub fn entity_count(&self) -> usize {
        self.entities.len()
    }

    /// Get current simulation tick
    pub fn tick(&self) -> u64 {
        self.tick
    }

    /// Get simulation timestep in seconds
    pub fn timestep(&self) -> f32 {
        self.timestep
    }
}

impl Default for Simulation {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simulation_creation() {
        let sim = Simulation::new();
        assert_eq!(sim.tick(), 0);
        assert_eq!(sim.entity_count(), 0);
        assert!(sim.timestep > 0.04 && sim.timestep < 0.06); // ~0.05 for 20Hz
    }

    #[test]
    fn test_entity_spawning() {
        let mut sim = Simulation::new();
        let pos = Position::new(0.0, 0.0);
        let vel = Velocity::new(1.0, 0.0);
        let health = Health::new(100.0);

        let id = sim.spawn_entity(pos, vel, health);

        assert_eq!(sim.entity_count(), 1);
        assert_eq!(id.0, 1);
    }

    #[test]
    fn test_multiple_entity_spawning() {
        let mut sim = Simulation::new();

        for i in 0..10 {
            let pos = Position::new(i as f32, 0.0);
            let vel = Velocity::new(1.0, 0.0);
            let health = Health::new(100.0);
            sim.spawn_entity(pos, vel, health);
        }

        assert_eq!(sim.entity_count(), 10);
    }

    #[test]
    fn test_movement_update() {
        let mut sim = Simulation::new();
        let pos = Position::new(0.0, 0.0);
        let vel = Velocity::new(10.0, 5.0); // 10 units/sec in x, 5 units/sec in y
        let health = Health::new(100.0);

        sim.spawn_entity(pos, vel, health);
        sim.update();

        let entities: Vec<_> = sim.get_entities().collect();
        assert_eq!(entities.len(), 1);

        // After one tick (0.05s) at 10 units/sec, should move 0.5 units
        let moved_x = entities[0].position.x;
        let moved_y = entities[0].position.y;
        assert!((moved_x - 0.5).abs() < 0.0001);
        assert!((moved_y - 0.25).abs() < 0.0001);
    }

    #[test]
    fn test_tick_increment() {
        let mut sim = Simulation::new();
        assert_eq!(sim.tick(), 0);

        sim.update();
        assert_eq!(sim.tick(), 1);

        sim.update();
        assert_eq!(sim.tick(), 2);
    }

    #[test]
    fn test_entity_creation() {
        let id = EntityId::new(1);
        let pos = Position::new(5.0, 10.0);
        let vel = Velocity::new(1.0, 2.0);
        let health = Health::new(50.0);

        let entity = Entity::new(id, pos, vel, health);

        assert_eq!(entity.id, id);
        assert_eq!(entity.position.x, 5.0);
        assert_eq!(entity.position.y, 10.0);
        assert_eq!(entity.velocity.vx, 1.0);
        assert_eq!(entity.health.current, 50.0);
    }
}
