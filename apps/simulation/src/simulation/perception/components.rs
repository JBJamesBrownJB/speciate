use bevy_ecs::prelude::*;
use bevy_reflect::Reflect;
use serde::{Deserialize, Serialize};

use super::constants::{
    ENERGY_MODIFIER, MAX_PERCEIVED_NEIGHBORS, PANIC_THRESHOLD_RATIO, PERCEPTION_MULTIPLIER,
    PERSONAL_SPACE,
};
use crate::simulation::creatures::behaviors::avoidance::constants::AVOIDANCE_FORCE;

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

    pub fn from_body_size(body_length: f32) -> Self {
        Self::new(body_length * PERCEPTION_MULTIPLIER)
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

#[derive(Component, Debug, Clone, Copy, Reflect, Serialize, Deserialize)]
#[reflect(Component)]
pub struct AvoidanceBehavior {
    pub personal_space: f32,
    pub max_force: f32,
}

// Energy-driven personal space reduction (biological realism)
// Hungry creatures tolerate closer proximity to reach resources
// 100% energy = max_modifier space, 0% energy = min_modifier space (60% reduction, matches wolves/vultures)
// TODO(DNA): Replace hardcoded modifier range with energy_sensitivity gene
fn calculate_energy_modifier(energy_fraction: f32) -> f32 {
    let clamped = energy_fraction.clamp(0.0, 1.0);
    let range = ENERGY_MODIFIER.max_modifier - ENERGY_MODIFIER.min_modifier;
    ENERGY_MODIFIER.min_modifier + (range * clamped)
}

impl AvoidanceBehavior {
    pub fn new(personal_space: f32, max_force: f32) -> Self {
        Self {
            personal_space,
            max_force,
        }
    }

    pub fn from_body_size(body_length: f32) -> Self {
        let personal_space = body_length + PERSONAL_SPACE;
        Self::new(personal_space, AVOIDANCE_FORCE)
    }

    pub fn panic_threshold(&self) -> f32 {
        self.personal_space * PANIC_THRESHOLD_RATIO
    }

    pub fn effective_personal_space(&self, energy_fraction: f32) -> f32 {
        self.personal_space * calculate_energy_modifier(energy_fraction)
    }
}

impl Default for AvoidanceBehavior {
    fn default() -> Self {
        let personal_space = 1.0 + PERSONAL_SPACE;
        Self::new(personal_space, AVOIDANCE_FORCE)
    }
}

#[cfg(feature = "dev-tools")]
#[derive(Resource, Default)]
pub struct PerceptionDebugTarget(pub Option<Entity>);

#[cfg(feature = "dev-tools")]
impl PerceptionDebugTarget {
    pub fn set_by_crit_id(&mut self, crit_id: Option<u32>, lookup: impl Fn(u32) -> Option<Entity>) {
        self.0 = crit_id.and_then(lookup);
    }

    pub fn clear(&mut self) {
        self.0 = None;
    }

    pub fn get(&self) -> Option<Entity> {
        self.0
    }
}

#[cfg(feature = "dev-tools")]
#[derive(Clone, Debug, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct NeighborDebugInfo {
    pub id: u32,
    pub x: f32,
    pub y: f32,
}

#[cfg(feature = "dev-tools")]
#[derive(Resource, Default, Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PerceptionDebugSnapshot {
    pub entity_id: u32,
    pub x: f32,
    pub y: f32,
    pub perception_range: f32,
    pub neighbors: Vec<NeighborDebugInfo>,
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

    #[test]
    fn test_effective_personal_space_at_full_energy() {
        let avoidance = AvoidanceBehavior::new(10.0, 35.0);
        assert_eq!(avoidance.effective_personal_space(1.0), 10.0);
    }

    #[test]
    fn test_effective_personal_space_at_zero_energy() {
        let avoidance = AvoidanceBehavior::new(10.0, 35.0);
        assert_eq!(avoidance.effective_personal_space(0.0), 4.0);
    }

    #[test]
    fn test_effective_personal_space_at_half_energy() {
        let avoidance = AvoidanceBehavior::new(10.0, 35.0);
        let result = avoidance.effective_personal_space(0.5);
        assert!((result - 7.0).abs() < 0.001, "Expected ~7.0, got {}", result);
    }

    #[test]
    fn test_energy_fraction_clamped() {
        let avoidance = AvoidanceBehavior::new(10.0, 35.0);
        assert_eq!(avoidance.effective_personal_space(-1.0), 4.0);
        assert_eq!(avoidance.effective_personal_space(2.0), 10.0);
    }
}
