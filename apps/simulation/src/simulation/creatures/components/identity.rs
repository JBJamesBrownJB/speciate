//! Creature identity components

use bevy_ecs::prelude::*;
use bevy_reflect::Reflect;
use serde::{Deserialize, Serialize};

/// Stable, unique identifier for each crit
/// This ID is assigned at spawn time and never changes, even when the entity is despawned/respawned
#[derive(Component, Clone, Copy, Debug, Serialize, Deserialize, Reflect)]
#[reflect(Component)]
pub struct CritId(pub u32);
