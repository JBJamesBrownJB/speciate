use crate::simulation::components::*;
use crate::simulation::core::components::BodySize;
use crate::simulation::movement::constants::MAX_SPEED;
use crate::simulation::perception::*;
use bevy_ecs::prelude::*;
use rand::Rng;

#[derive(Bundle)]
pub struct CritBundle {
    pub id: CritId,
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
    pub avoidance_behavior: AvoidanceBehavior,
    pub target: Target,
}

#[derive(Clone, Copy, Debug)]
#[derive(Default)]
pub struct CritCapabilities {
    pub can_seek: bool,
    pub can_flee: bool,
    pub can_wander: bool,
    pub can_avoid: bool,
}


impl CritCapabilities {
    pub fn all() -> Self {
        Self {
            can_seek: true,
            can_flee: true,
            can_wander: true,
            can_avoid: true,
        }
    }
}

#[derive(Clone)]
pub struct CritBuilder {
    position: (f32, f32),
    velocity: (f32, f32),
    capabilities: CritCapabilities,
    behavior: BehaviorMode,
    brain_mode: BrainMode,
    energy: f32,
    age: f32,
    max_speed: f32,
    target: Option<Target>,
    size: f32,
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
            capabilities: CritCapabilities::default(),
            behavior: BehaviorMode::Wandering,
            brain_mode: BrainMode::Normal,
            energy: 100.0,
            age: 0.0,
            max_speed: MAX_SPEED,
            target: None,
            size: 1.0,
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

    pub fn with_seeking(mut self) -> Self {
        self.capabilities.can_seek = true;
        self
    }

    pub fn with_fleeing(mut self) -> Self {
        self.capabilities.can_flee = true;
        self
    }

    pub fn with_wandering(mut self) -> Self {
        self.capabilities.can_wander = true;
        self
    }

    pub fn with_avoidance(mut self) -> Self {
        self.capabilities.can_avoid = true;
        self
    }

    pub fn with_all_capabilities(mut self) -> Self {
        self.capabilities = CritCapabilities::all();
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

    pub fn with_cycling_brain(mut self) -> Self {
        self.brain_mode = BrainMode::Cycling;
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

    pub fn with_max_speed(mut self, max_speed: f32) -> Self {
        self.max_speed = max_speed;
        self
    }

    pub fn with_size(mut self, size: f32) -> Self {
        self.size = size;
        self
    }

    pub fn with_target(mut self, x: f32, y: f32) -> Self {
        self.target = Some(Target::new(x, y));
        self
    }

    pub fn as_seeker(mut self, target_x: f32, target_y: f32) -> Self {
        self.capabilities.can_seek = true;
        self.behavior = BehaviorMode::Seeking;
        self.target = Some(Target::new(target_x, target_y));
        self
    }

    pub fn as_wanderer(mut self, world_bounds: &crate::simulation::core::WorldBounds) -> Self {
        self.capabilities.can_wander = true;
        self.behavior = BehaviorMode::Wandering;

        let mut rng = rand::thread_rng();
        let random_angle = rng.gen_range(0.0..std::f32::consts::TAU);
        let random_distance = rng.gen_range(5.0..20.0);
        let initial_target_x = self.position.0 + random_angle.cos() * random_distance;
        let initial_target_y = self.position.1 + random_angle.sin() * random_distance;
        const EDGE_MARGIN: f32 = 10.0;
        let (clamped_x, clamped_y) =
            world_bounds.clamp_target(initial_target_x, initial_target_y, EDGE_MARGIN);

        self.target = Some(Target::new(clamped_x, clamped_y));
        self
    }

    pub fn build(self, id: u32) -> CritBundle {
        let mut rng = rand::thread_rng();

        CritBundle {
            id: CritId(id),
            position: Position {
                x: self.position.0,
                y: self.position.1,
            },
            velocity: Velocity {
                vx: self.velocity.0,
                vy: self.velocity.1,
            },
            acceleration: Acceleration { ax: 0.0, ay: 0.0 },
            body_size: BodySize::new(self.size),
            rotation: Rotation {
                radians: rng.gen_range(0.0..std::f32::consts::TAU),
            },
            creature_state: CreatureState {
                behavior: self.behavior,
                energy: self.energy,
                age: self.age,
                max_speed: self.max_speed,
            },
            brain: Brain::with_mode(self.brain_mode),
            wander_state: WanderState {
                wander_angle: rng.gen_range(0.0..std::f32::consts::TAU),
                wander_radius: 5.0,
                wander_distance: 3.0,
                angle_change: 4.5,
            },
            home_position: HomePosition::new(self.position.0, self.position.1),
            can_seek: CanSeek,
            can_flee: CanFlee,
            can_wander: CanWander,
            can_avoid_obstacles: CanAvoidObstacles,
            perception: Perception::from_body_size(self.size),
            avoidance_behavior: AvoidanceBehavior::from_body_size(self.size),
            target: self.target.unwrap_or(Target::new(0.0, 0.0)),
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
    fn test_builder_fluent_api() {
        let builder = CritBuilder::new()
            .at(10.0, 20.0)
            .with_energy(50.0)
            .with_seeking()
            .in_behavior(BehaviorMode::Seeking);

        assert_eq!(builder.position, (10.0, 20.0));
        assert_eq!(builder.energy, 50.0);
        assert!(builder.capabilities.can_seek);
        assert_eq!(builder.behavior, BehaviorMode::Seeking);
    }

    #[test]
    fn test_builder_as_seeker() {
        let builder = CritBuilder::new().as_seeker(100.0, 50.0);

        assert!(builder.capabilities.can_seek);
        assert_eq!(builder.behavior, BehaviorMode::Seeking);
        assert!(builder.target.is_some());

        if let Some(target) = builder.target {
            assert_eq!(target.x, 100.0);
            assert_eq!(target.y, 50.0);
        }
    }

    #[test]
    fn test_builder_all_capabilities() {
        let builder = CritBuilder::new().with_all_capabilities();

        assert!(builder.capabilities.can_seek);
        assert!(builder.capabilities.can_flee);
        assert!(builder.capabilities.can_wander);
        assert!(builder.capabilities.can_avoid);
    }
}
