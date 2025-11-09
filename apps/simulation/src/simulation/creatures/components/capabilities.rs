//! Creature capability marker components
//!
//! Following the hybrid ECS pattern: These are zero-sized types (ZST) that mark
//! permanent creature capabilities. They are added at spawn and never removed
//! (archetype stability).

use bevy_ecs::prelude::*;
use serde::{Deserialize, Serialize};

/// Capability marker: Entity can perform seeking behavior
/// Added at spawn, never removed (archetype stability)
#[derive(Component, Clone, Copy, Debug, Default, Serialize, Deserialize)]
pub struct CanSeek;

/// Capability marker: Entity can perform fleeing behavior
/// Added at spawn, never removed (archetype stability)
#[derive(Component, Clone, Copy, Debug, Default, Serialize, Deserialize)]
pub struct CanFlee;

/// Capability marker: Entity can perform wandering behavior
/// Added at spawn, never removed (archetype stability)
#[derive(Component, Clone, Copy, Debug, Default, Serialize, Deserialize)]
pub struct CanWander;

/// Capability marker: Entity can avoid obstacles
/// Added at spawn, never removed (archetype stability)
#[derive(Component, Clone, Copy, Debug, Default, Serialize, Deserialize)]
pub struct CanAvoidObstacles;
