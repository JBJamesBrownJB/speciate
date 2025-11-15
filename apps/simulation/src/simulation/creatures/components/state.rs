//! Creature state and behavior components
//!
//! Contains the BehaviorMode enum (state machine) and CreatureState (mutable state).
//! Following the hybrid ECS pattern: BehaviorMode determines which systems execute.

use bevy_ecs::prelude::*;
use bevy_reflect::Reflect;
use serde::{Deserialize, Serialize};

/// Behavior modes for creatures (A-Life state machine)
///
/// This enum represents mutually exclusive high-level behavioral modes.
/// Systems check this to determine if they should execute for a given creature.
#[derive(Clone, Copy, Debug, PartialEq, Default, Serialize, Deserialize, Reflect)]
pub enum BehaviorMode {
    /// Stationary, no movement or behavior
    #[default]
    Catatonic,
    /// Actively seeking a target position
    Seeking,
    /// Wandering/patrolling around home position
    Wandering,
    // Future modes (uncomment when implemented):
    // Fleeing,
    // Feeding,
    // Resting,
    // Mating,
    // Hunting,
    // Exploring,
}

/// General creature state component (required for all creatures)
///
/// Contains mutable state that changes during simulation.
/// TODO: Migrate max_speed and other parameters to DNA system (DNA system (in progress))
#[derive(Component, Clone, Copy, Debug, Serialize, Deserialize, Reflect)]
#[reflect(Component)]
pub struct CreatureState {
    pub behavior: BehaviorMode,
    pub energy: f32,
    pub age: f32,
    pub max_speed: f32, // TODO: from DNA
}

impl Default for CreatureState {
    fn default() -> Self {
        Self {
            behavior: BehaviorMode::Catatonic,
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

/// Home position component for wandering creatures
///
/// Represents the "home" or "territory center" that wandering creatures
/// prefer to stay near. Typically set to the spawn position, but could
/// be updated for migration, denning, etc.
///
/// The farther a creature wanders from home, the more likely it will
/// select new targets biased toward returning home (probability-based,
/// not a hard boundary).
///
/// TODO: In future, home position could be:
/// - Updated when finding a den/nest
/// - Abandoned when starving (expand search)
/// - Expanded during mating season
/// - Inherited by offspring (territorial behavior)
#[derive(Component, Clone, Copy, Debug, Serialize, Deserialize, Reflect)]
#[reflect(Component)]
pub struct HomePosition {
    pub x: f32,
    pub y: f32,
}

impl HomePosition {
    pub fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }

    /// Calculate distance from current position to home
    pub fn distance_from(&self, pos_x: f32, pos_y: f32) -> f32 {
        let dx = pos_x - self.x;
        let dy = pos_y - self.y;
        (dx * dx + dy * dy).sqrt()
    }
}
