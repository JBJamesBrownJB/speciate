use crate::simulation::core::components::{Acceleration, BodySize, Position, Rotation, Velocity};
use crate::simulation::creatures::components::{
    BehaviorMode, Brain, BrainMode, CanAvoidObstacles, CanFlee, CanSeek, CanWander, CreatureState,
    CritId, EntityTag, HomePosition, Target, WanderState,
};
use crate::simulation::creatures::constants::{ANGLE_CHANGE, WANDER_DISTANCE, WANDER_RADIUS};
use crate::simulation::creatures::dna::Dna;
use crate::simulation::perception::{L1Vision, NeighborCache, Perception};
use bevy_ecs::prelude::*;
use rand::Rng;

#[derive(Bundle)]
pub struct CritBundle {
    pub id: CritId,
    pub dna: Dna,
    pub position: Position,
    pub velocity: Velocity,
    pub acceleration: Acceleration,
    pub body_size: BodySize,
    pub rotation: Rotation,
    pub creature_state: CreatureState,
    pub brain: Brain,
    pub wander_state: WanderState,
    pub home_position: HomePosition,
    pub can_seek: CanSeek,
    pub can_flee: CanFlee,
    pub can_wander: CanWander,
    pub can_avoid_obstacles: CanAvoidObstacles,
    pub perception: Perception,
    pub neighbor_cache: NeighborCache,
    pub l1_vision: L1Vision,
    pub target: Target,
}

#[derive(Clone)]
pub struct CritBuilder {
    position: (f32, f32),
    velocity: (f32, f32),
    behavior: BehaviorMode,
    brain_mode: BrainMode,
    energy: f32,
    age: f32,
    target: Option<Target>,
    dna: Dna,
    size_override: Option<f32>,
    fov_override: Option<f32>,
    facing_override: Option<f32>,
    tag: Option<String>,
}

impl Default for CritBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl CritBuilder {
    pub fn new() -> Self {
        Self {
            position: (0.0, 0.0),
            velocity: (0.0, 0.0),
            behavior: BehaviorMode::Wandering,
            brain_mode: BrainMode::Normal,
            energy: 100.0,
            age: 0.0,
            target: None,
            dna: Dna::default(),
            size_override: None,
            fov_override: None,
            facing_override: None,
            tag: None,
        }
    }

    pub fn at(mut self, x: f32, y: f32) -> Self {
        self.position = (x, y);
        self
    }

    pub fn with_velocity(mut self, vx: f32, vy: f32) -> Self {
        self.velocity = (vx, vy);
        self
    }

    /// Set the creature's facing direction by angle in radians.
    /// 0 = facing +X, PI/2 = facing +Y, PI = facing -X, etc.
    pub fn facing(mut self, angle_radians: f32) -> Self {
        self.facing_override = Some(angle_radians);
        self
    }

    /// Set the creature's facing direction by direction vector.
    /// The vector will be normalized internally.
    pub fn facing_direction(mut self, dx: f32, dy: f32) -> Self {
        let angle = dy.atan2(dx);
        self.facing_override = Some(angle);
        self
    }

    /// Face towards a specific point from current position.
    pub fn facing_point(mut self, target_x: f32, target_y: f32) -> Self {
        let dx = target_x - self.position.0;
        let dy = target_y - self.position.1;
        let angle = dy.atan2(dx);
        self.facing_override = Some(angle);
        self
    }

    /// No-op for backward compatibility. All creatures have all capabilities.
    #[deprecated(note = "All creatures now have all capabilities by default")]
    pub fn with_seeking(self) -> Self {
        self
    }

    /// No-op for backward compatibility. All creatures have all capabilities.
    #[deprecated(note = "All creatures now have all capabilities by default")]
    pub fn with_fleeing(self) -> Self {
        self
    }

    /// No-op for backward compatibility. All creatures have all capabilities.
    #[deprecated(note = "All creatures now have all capabilities by default")]
    pub fn with_wandering(self) -> Self {
        self
    }

    /// No-op for backward compatibility. All creatures have all capabilities.
    #[deprecated(note = "All creatures now have all capabilities by default")]
    pub fn with_avoidance(self) -> Self {
        self
    }

    /// No-op for backward compatibility. All creatures have all capabilities.
    pub fn with_all_capabilities(self) -> Self {
        self
    }

    pub fn in_behavior(mut self, behavior: BehaviorMode) -> Self {
        self.behavior = behavior;
        self
    }

    pub fn with_brain_mode(mut self, mode: BrainMode) -> Self {
        self.brain_mode = mode;
        self
    }

    pub fn with_dormant_brain(mut self) -> Self {
        self.brain_mode = BrainMode::Dormant;
        self
    }

    pub fn with_energy(mut self, energy: f32) -> Self {
        self.energy = energy;
        self
    }

    pub fn with_age(mut self, age: f32) -> Self {
        self.age = age;
        self
    }

    /// Deprecated: max_speed is now derived from body size.
    #[deprecated(note = "max_speed is now derived from BodySize, this method is a no-op")]
    pub fn with_max_speed(self, _max_speed: f32) -> Self {
        self
    }

    pub fn with_size(mut self, size: f32) -> Self {
        self.size_override = Some(size);
        self
    }

    pub fn with_fov(mut self, fov_degrees: f32) -> Self {
        self.fov_override = Some(fov_degrees);
        self
    }

    pub fn with_dna(mut self, dna: Dna) -> Self {
        self.dna = dna;
        self
    }

    pub fn with_target(mut self, x: f32, y: f32) -> Self {
        self.target = Some(Target::at_point(x, y));
        self
    }

    pub fn with_tag(mut self, tag: impl Into<String>) -> Self {
        self.tag = Some(tag.into());
        self
    }

    /// Returns the tag if set, consuming it from the builder.
    /// Call this after build() to get the tag for separate insertion.
    pub fn take_tag(&mut self) -> Option<EntityTag> {
        self.tag.take().map(EntityTag)
    }

    pub fn as_seeker(mut self, target_x: f32, target_y: f32) -> Self {
        self.behavior = BehaviorMode::Seeking;
        self.target = Some(Target::at_point(target_x, target_y));
        self
    }

    pub fn as_wanderer(mut self, world_bounds: &crate::simulation::core::WorldBounds) -> Self {
        self.behavior = BehaviorMode::Wandering;

        let mut rng = rand::thread_rng();
        let random_angle = rng.gen_range(0.0..std::f32::consts::TAU);
        let random_distance = rng.gen_range(5.0..20.0);
        let initial_target_x = self.position.0 + random_angle.cos() * random_distance;
        let initial_target_y = self.position.1 + random_angle.sin() * random_distance;
        const EDGE_MARGIN: f32 = 10.0;
        let (clamped_x, clamped_y) =
            world_bounds.clamp_target(initial_target_x, initial_target_y, EDGE_MARGIN);

        self.target = Some(Target::at_point(clamped_x, clamped_y));
        self
    }

    pub fn build(self, id: u32) -> CritBundle {
        let mut rng = rand::thread_rng();

        let size = self
            .size_override
            .unwrap_or_else(|| self.dna.expressed_size());
        let fov_degrees = self
            .fov_override
            .unwrap_or_else(|| self.dna.expressed_fov());

        CritBundle {
            id: CritId(id),
            dna: self.dna,
            position: Position {
                x: self.position.0,
                y: self.position.1,
            },
            velocity: Velocity {
                vx: self.velocity.0,
                vy: self.velocity.1,
            },
            acceleration: Acceleration { ax: 0.0, ay: 0.0 },
            body_size: BodySize::new(size),
            rotation: Rotation::new(
                self.facing_override
                    .unwrap_or_else(|| rng.gen_range(0.0..std::f32::consts::TAU)),
            ),
            creature_state: CreatureState {
                behavior: self.behavior,
                energy: self.energy,
                age: self.age,
            },
            brain: Brain::with_mode(self.brain_mode),
            wander_state: WanderState {
                wander_angle: rng.gen_range(0.0..std::f32::consts::TAU),
                wander_radius: WANDER_RADIUS,
                wander_distance: WANDER_DISTANCE,
                angle_change: ANGLE_CHANGE,
            },
            home_position: HomePosition::new(self.position.0, self.position.1),
            can_seek: CanSeek,
            can_flee: CanFlee,
            can_wander: CanWander,
            can_avoid_obstacles: CanAvoidObstacles,
            perception: Perception::from_body_size_with_fov(size, fov_degrees),
            neighbor_cache: NeighborCache::new(),
            l1_vision: L1Vision::new(),
            target: self.target.unwrap_or(Target::at_point(0.0, 0.0)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_builder_defaults() {
        let builder = CritBuilder::new();
        assert_eq!(builder.position, (0.0, 0.0));
        assert_eq!(builder.velocity, (0.0, 0.0));
        assert_eq!(builder.energy, 100.0);
        assert_eq!(builder.behavior, BehaviorMode::Wandering);
    }

    #[test]
    #[allow(deprecated)]
    fn test_builder_fluent_api() {
        let builder = CritBuilder::new()
            .at(10.0, 20.0)
            .with_energy(50.0)
            .with_seeking()
            .in_behavior(BehaviorMode::Seeking);

        assert_eq!(builder.position, (10.0, 20.0));
        assert_eq!(builder.energy, 50.0);
        assert_eq!(builder.behavior, BehaviorMode::Seeking);
    }

    #[test]
    fn test_builder_as_seeker() {
        let builder = CritBuilder::new().as_seeker(100.0, 50.0);

        assert_eq!(builder.behavior, BehaviorMode::Seeking);
        assert!(builder.target.is_some());

        if let Some(target) = builder.target {
            assert_eq!(target.x, 100.0);
            assert_eq!(target.y, 50.0);
        }
    }

    #[test]
    fn test_builder_all_capabilities_is_noop() {
        // with_all_capabilities is now a no-op since all creatures have all capabilities
        let builder = CritBuilder::new().with_all_capabilities();
        assert_eq!(builder.position, (0.0, 0.0)); // Just verify it doesn't break the chain
    }

    #[test]
    fn test_builder_default_uses_dna_default() {
        let builder = CritBuilder::new();
        assert_eq!(builder.dna, Dna::default());
    }

    #[test]
    fn test_builder_default_produces_approximately_1m_creature() {
        let bundle = CritBuilder::new().build(0);
        assert!(
            (bundle.body_size.length - 1.0).abs() < 0.05,
            "Default creature should be ~1.0m, got {}",
            bundle.body_size.length
        );
    }

    #[test]
    fn test_builder_bundle_includes_dna_component() {
        let bundle = CritBuilder::new().build(0);
        assert_eq!(bundle.dna, Dna::default());
    }

    #[test]
    fn test_builder_with_dna_affects_size() {
        use crate::simulation::creatures::dna::SIZE_MAX;

        let dna = Dna::new(1.0, 0.5);
        let bundle = CritBuilder::new().with_dna(dna).build(0);

        assert_eq!(bundle.dna.size_gene, 1.0);
        assert_eq!(bundle.body_size.length, SIZE_MAX);
    }

    #[test]
    fn test_builder_with_dna_affects_fov() {
        use crate::simulation::creatures::constants::MAX_FOV_DEGREES;

        let dna = Dna::new(0.5, 1.0);
        let bundle = CritBuilder::new().with_dna(dna).build(0);

        assert_eq!(bundle.dna.fov_gene, 1.0);
        let expected_fov_rad = MAX_FOV_DEGREES.to_radians();
        assert!(
            (bundle.perception.fov_angle - expected_fov_rad).abs() < 0.01,
            "FOV should be {} rad, got {}",
            expected_fov_rad,
            bundle.perception.fov_angle
        );
    }

    #[test]
    fn test_builder_with_size_overrides_dna() {
        let dna = Dna::new(1.0, 0.5);
        let bundle = CritBuilder::new().with_dna(dna).with_size(2.0).build(0);

        assert_eq!(bundle.body_size.length, 2.0);
        assert_eq!(bundle.dna.size_gene, 1.0);
    }

    #[test]
    fn test_builder_with_fov_overrides_dna() {
        let dna = Dna::new(0.5, 1.0);
        let bundle = CritBuilder::new().with_dna(dna).with_fov(90.0).build(0);

        let expected_fov_rad = 90.0_f32.to_radians();
        assert!(
            (bundle.perception.fov_angle - expected_fov_rad).abs() < 0.01,
            "FOV should be {} rad, got {}",
            expected_fov_rad,
            bundle.perception.fov_angle
        );
        assert_eq!(bundle.dna.fov_gene, 1.0);
    }

    #[test]
    fn test_builder_small_creature_from_dna() {
        use crate::simulation::creatures::dna::SIZE_MIN;

        let dna = Dna::new(0.0, 0.5);
        let bundle = CritBuilder::new().with_dna(dna).build(0);

        assert_eq!(bundle.body_size.length, SIZE_MIN);
    }

    #[test]
    fn test_builder_large_creature_from_dna() {
        use crate::simulation::creatures::dna::SIZE_MAX;

        let dna = Dna::new(1.0, 0.5);
        let bundle = CritBuilder::new().with_dna(dna).build(0);

        assert_eq!(bundle.body_size.length, SIZE_MAX);
    }
}
