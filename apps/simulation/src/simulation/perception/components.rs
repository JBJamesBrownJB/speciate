use bevy_ecs::prelude::*;
use bevy_reflect::Reflect;
use serde::{Deserialize, Serialize};

pub const MAX_PERCEIVED_NEIGHBORS: usize = 40;

#[derive(Resource, Default)]
pub struct PerceptionScratchBuffer {
    pub positions: Vec<(Entity, f32, f32, f32)>,
}

#[derive(Component, Debug, Clone)]
pub struct Perception {
    pub range: f32,
    neighbor_count: u8,
    neighbors: [Entity; MAX_PERCEIVED_NEIGHBORS],
    pub obstacles: Vec<Entity>, // Placeholder for future obstacle tracking
}

impl Perception {
    pub fn new(range: f32) -> Self {
        Self {
            range,
            neighbor_count: 0,
            neighbors: [Entity::PLACEHOLDER; MAX_PERCEIVED_NEIGHBORS],
            obstacles: Vec::new(),
        }
    }

    pub fn default_range() -> Self {
        Self::new(10.0)
    }

    pub fn from_body_size(body_length: f32) -> Self {
        use crate::simulation::movement::PERCEPTION;
        Self::new(body_length * PERCEPTION.perception_multiplier)
    }

    pub fn has_neighbors(&self) -> bool {
        self.neighbor_count > 0
    }

    pub fn neighbor_count(&self) -> usize {
        self.neighbor_count as usize
    }

    pub fn clear(&mut self) {
        self.neighbor_count = 0;
    }

    pub fn add_neighbor(&mut self, entity: Entity) {
        if (self.neighbor_count as usize) < MAX_PERCEIVED_NEIGHBORS {
            self.neighbors[self.neighbor_count as usize] = entity;
            self.neighbor_count += 1;
        }
    }

    pub fn iter_neighbors(&self) -> impl Iterator<Item = Entity> + '_ {
        self.neighbors[..self.neighbor_count as usize].iter().copied()
    }

    pub fn contains(&self, entity: Entity) -> bool {
        self.neighbors[..self.neighbor_count as usize].contains(&entity)
    }

    pub fn is_full(&self) -> bool {
        self.neighbor_count as usize >= MAX_PERCEIVED_NEIGHBORS
    }
}

impl Default for Perception {
    fn default() -> Self {
        Self::default_range()
    }
}

#[derive(Component, Debug, Clone, Copy, Reflect, Serialize, Deserialize)]
#[reflect(Component)]
pub struct AvoidanceBehavior {
    pub personal_space: f32,
    pub max_force: f32,
}

impl AvoidanceBehavior {
    pub fn new(personal_space: f32, max_force: f32) -> Self {
        Self {
            personal_space,
            max_force,
        }
    }

    pub fn default_params() -> Self {
        use crate::simulation::movement::{PERCEPTION, STEERING};
        let personal_space = 1.0 + PERCEPTION.personal_space;
        Self::new(personal_space, STEERING.avoidance_force)
    }

    pub fn from_body_size(body_length: f32) -> Self {
        use crate::simulation::movement::{PERCEPTION, STEERING};
        let personal_space = body_length + PERCEPTION.personal_space;
        Self::new(personal_space, STEERING.avoidance_force)
    }

    pub fn panic_threshold(&self) -> f32 {
        use crate::simulation::movement::PERCEPTION;
        self.personal_space * PERCEPTION.panic_threshold_ratio
    }
}

impl Default for AvoidanceBehavior {
    fn default() -> Self {
        Self::default_params()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_perception_scaling_with_body_size() {
        let small_perception = Perception::from_body_size(0.5);
        assert_eq!(small_perception.range, 5.0);

        let standard_perception = Perception::from_body_size(1.0);
        assert_eq!(standard_perception.range, 10.0);

        let large_perception = Perception::from_body_size(2.0);
        assert_eq!(large_perception.range, 20.0);
    }

    #[test]
    fn test_avoidance_scaling_with_body_size() {
        let small_avoid = AvoidanceBehavior::from_body_size(0.5);
        assert_eq!(small_avoid.personal_space, 2.0);

        let standard_avoid = AvoidanceBehavior::from_body_size(1.0);
        assert_eq!(standard_avoid.personal_space, 2.5);

        let large_avoid = AvoidanceBehavior::from_body_size(2.0);
        assert_eq!(large_avoid.personal_space, 3.5);
    }

    #[test]
    fn test_panic_threshold() {
        let avoidance = AvoidanceBehavior::new(2.5, 15.0);
        let panic = avoidance.panic_threshold();

        assert_eq!(panic, 1.25);
        assert!(panic < avoidance.personal_space);
    }

    #[test]
    fn test_perception_neighbor_tracking() {
        let mut perception = Perception::new(10.0);

        assert!(!perception.has_neighbors());
        assert_eq!(perception.neighbor_count(), 0);

        perception.add_neighbor(Entity::PLACEHOLDER);
        perception.add_neighbor(Entity::PLACEHOLDER);

        assert!(perception.has_neighbors());
        assert_eq!(perception.neighbor_count(), 2);

        perception.clear();
        assert!(!perception.has_neighbors());
        assert_eq!(perception.neighbor_count(), 0);
    }
}
