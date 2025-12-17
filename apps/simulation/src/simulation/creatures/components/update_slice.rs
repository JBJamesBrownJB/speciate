use bevy_ecs::prelude::*;
use bevy_reflect::Reflect;
use serde::{Deserialize, Serialize};

/// Update slice assignment for system skipping.
/// Creatures with the same slice_id update in the same tick.
/// Used to distribute expensive systems (perception, behavior) across ticks.
#[derive(Component, Clone, Copy, Debug, Serialize, Deserialize, Reflect)]
#[reflect(Component)]
pub struct UpdateSlice {
    pub id: u8,
}

impl UpdateSlice {
    pub fn new(id: u8) -> Self {
        Self { id }
    }
}
