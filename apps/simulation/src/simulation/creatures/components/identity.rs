
use bevy_ecs::prelude::*;
use bevy_reflect::Reflect;
use serde::{Deserialize, Serialize};

#[derive(Component, Clone, Copy, Debug, Serialize, Deserialize, Reflect)]
#[reflect(Component)]
pub struct CritId(pub u32);

/// Tag component for identifying creatures in spec assertions.
/// Used to track specific creatures (e.g., "west-seeker") across ticks.
#[derive(Component, Clone, Debug)]
pub struct EntityTag(pub String);
