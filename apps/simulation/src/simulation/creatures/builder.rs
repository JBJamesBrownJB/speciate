//! Crit builder for constructing creatures with specific capabilities and behaviors
//!
//! This module provides a clean separation between creature construction (what it is)
//! and spawning (putting it in the world). The builder pattern makes it easy to create
//! crits with different configurations.

use crate::simulation::components::*;
use crate::simulation::core::components::BodySize;
use crate::simulation::movement::constants::MAX_SPEED;
use crate::simulation::perception::*;
use bevy_ecs::prelude::*;
use rand::Rng;

/// Complete component bundle for a crit
///
/// Note: This bundle includes ALL capability markers unconditionally.
/// Systems filter by BehaviorState to determine which capabilities are active.
///
/// TODO(Future DNA system): When DNA is implemented, consider making capabilities
/// conditional based on genetic traits. For now, all creatures have all
/// capabilities (zero-sized types, no memory cost).
#[derive(Bundle)]
pub struct CritBundle {
    pub id: CritId,
    pub position: Position,
    pub velocity: Velocity,
    pub acceleration: Acceleration,
    pub body_size: BodySize,
    pub rotation: Rotation,
    pub creature_state: CreatureState,
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

/// Capability flags for what a crit can do
/// Following the hybrid ECS pattern: capabilities are permanent, behavior state is mutable
#[derive(Clone, Copy, Debug)]
pub struct CritCapabilities {
    pub can_seek: bool,
    pub can_flee: bool,
    pub can_wander: bool,
    pub can_avoid: bool,
}

impl Default for CritCapabilities {
    fn default() -> Self {
        Self {
            can_seek: false,
            can_flee: false,
            can_wander: false,
            can_avoid: false,
        }
    }
}

impl CritCapabilities {
    /// All capabilities enabled (useful for testing)
    pub fn all() -> Self {
        Self {
            can_seek: true,
            can_flee: true,
            can_wander: true,
            can_avoid: true,
        }
    }
}

/// Builder for constructing crits with specific capabilities and behaviors
///
/// # Example
/// ```no_run
/// use speciate::simulation::creatures::builder::CritBuilder;
/// use speciate::BehaviorMode;
///
/// let builder = CritBuilder::new()
///     .at(0.0, 0.0)
///     .with_seeking()
///     .in_behavior(BehaviorMode::Seeking);
/// ```
#[derive(Clone)]
pub struct CritBuilder {
    position: (f32, f32),
    velocity: (f32, f32),
    capabilities: CritCapabilities,
    behavior: BehaviorMode,
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
    /// Create a new crit builder with sensible defaults
    ///
    /// Default crit:
    /// - Position: (0, 0)
    /// - Velocity: (0, 0)
    /// - No capabilities enabled
    /// - Behavior: Wandering (exploratory default)
    /// - Energy: 100.0
    /// - Age: 0.0
    /// - Max speed: 30.0 m/s (biologically realistic from constants)
    pub fn new() -> Self {
        Self {
            position: (0.0, 0.0),
            velocity: (0.0, 0.0),
            capabilities: CritCapabilities::default(),
            behavior: BehaviorMode::Wandering,
            energy: 100.0,
            age: 0.0,
            max_speed: MAX_SPEED,
            target: None,
            size: 1.0, // 1m body length (wolf-sized)
        }
    }

    // ========== Position & Velocity ==========

    /// Set spawn position
    pub fn at(mut self, x: f32, y: f32) -> Self {
        self.position = (x, y);
        self
    }

    /// Set initial velocity (usually leave at zero)
    pub fn with_velocity(mut self, vx: f32, vy: f32) -> Self {
        self.velocity = (vx, vy);
        self
    }

    // ========== Capabilities ==========

    /// Enable seeking capability
    pub fn with_seeking(mut self) -> Self {
        self.capabilities.can_seek = true;
        self
    }

    /// Enable fleeing capability
    pub fn with_fleeing(mut self) -> Self {
        self.capabilities.can_flee = true;
        self
    }

    /// Enable wandering capability
    pub fn with_wandering(mut self) -> Self {
        self.capabilities.can_wander = true;
        self
    }

    /// Enable obstacle avoidance capability
    pub fn with_avoidance(mut self) -> Self {
        self.capabilities.can_avoid = true;
        self
    }

    /// Enable all capabilities (useful for testing or "smart" crits)
    pub fn with_all_capabilities(mut self) -> Self {
        self.capabilities = CritCapabilities::all();
        self
    }

    // ========== Behavior State ==========

    /// Set the initial behavior mode
    pub fn in_behavior(mut self, behavior: BehaviorMode) -> Self {
        self.behavior = behavior;
        self
    }

    // ========== State ==========

    /// Set initial energy
    pub fn with_energy(mut self, energy: f32) -> Self {
        self.energy = energy;
        self
    }

    /// Set initial age
    pub fn with_age(mut self, age: f32) -> Self {
        self.age = age;
        self
    }

    /// Set max speed
    pub fn with_max_speed(mut self, max_speed: f32) -> Self {
        self.max_speed = max_speed;
        self
    }

    /// Set body size (length in meters)
    pub fn with_size(mut self, size: f32) -> Self {
        self.size = size;
        self
    }

    /// Set target position (for seeking behavior)
    pub fn with_target(mut self, x: f32, y: f32) -> Self {
        self.target = Some(Target::new(x, y));
        self
    }

    // ========== Presets (Convenience) ==========

    /// Configure as a seeker aimed at a target
    ///
    /// Enables seeking capability, sets behavior to Seeking, and sets target position
    pub fn as_seeker(mut self, target_x: f32, target_y: f32) -> Self {
        self.capabilities.can_seek = true;
        self.behavior = BehaviorMode::Seeking;
        self.target = Some(Target::new(target_x, target_y));
        self
    }

    /// Configure as a wanderer
    ///
    /// Enables wandering capability, sets behavior to Wandering, and sets home to current position.
    /// The creature will wander with a bias toward staying near its home position.
    ///
    /// # Arguments
    /// * `world_bounds` - World boundaries used to clamp the initial wander target
    pub fn as_wanderer(mut self, world_bounds: &crate::simulation::core::WorldBounds) -> Self {
        self.capabilities.can_wander = true;
        self.behavior = BehaviorMode::Wandering;

        // Initial target is a random nearby point, clamped to world bounds
        let mut rng = rand::thread_rng();
        let random_angle = rng.gen_range(0.0..std::f32::consts::TAU);
        let random_distance = rng.gen_range(5.0..20.0);
        let initial_target_x = self.position.0 + random_angle.cos() * random_distance;
        let initial_target_y = self.position.1 + random_angle.sin() * random_distance;
        const EDGE_MARGIN: f32 = 10.0; // Same margin as wander_target_selection_system
        let (clamped_x, clamped_y) =
            world_bounds.clamp_target(initial_target_x, initial_target_y, EDGE_MARGIN);

        self.target = Some(Target::new(clamped_x, clamped_y));
        self
    }

    // ========== Build ==========

    /// Build the component bundle for this crit
    ///
    /// This method consumes the builder and returns a tuple of components
    /// that can be spawned directly into the Bevy world.
    ///
    /// # Arguments
    /// * `id` - The unique CritId to assign
    ///
    /// # Returns
    /// A tuple of components representing the complete crit entity
    ///
    /// # Note
    /// All capability markers are always added (they're zero-sized types).
    /// Perception and AvoidanceBehavior use default parameters (will be DNA-driven in Future DNA system).
    /// Target component is always added but will have default (0, 0) if not set.
    /// Systems check BehaviorState to determine which behaviors to execute.
    pub fn build(self, id: u32) -> CritBundle {
        let mut rng = rand::thread_rng();

        // Following our hybrid ECS architecture:
        // - All capabilities are always present (zero-sized markers)
        // - BehaviorState determines which systems actually run
        // - Perception and AvoidanceBehavior scale with body size
        // - Target is always present (will be (0,0) if not used)
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
            wander_state: WanderState {
                wander_angle: rng.gen_range(0.0..std::f32::consts::TAU),
                // TODO(DNA Future DNA system): Derive from DNA genes
                wander_radius: 5.0, // TODO(DNA): perception_range * 0.3 * size.sqrt()
                wander_distance: 3.0, // TODO(DNA): body_size * 3.0 (planning horizon)
                angle_change: 4.5,  // TODO(DNA): 50% of max turn rate from biomechanics
            },
            // Home position - defaults to spawn position (territory center)
            home_position: HomePosition::new(self.position.0, self.position.1),
            // Capability markers - always add (zero-sized types)
            can_seek: CanSeek,
            can_flee: CanFlee,
            can_wander: CanWander,
            can_avoid_obstacles: CanAvoidObstacles,
            // Perception and avoidance scale with body size
            perception: Perception::from_body_size(self.size),
            avoidance_behavior: AvoidanceBehavior::from_body_size(self.size),
            // Target - use provided or default to (0, 0)
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
