
use bevy_ecs::prelude::*;
use bevy_reflect::Reflect;
use serde::{Deserialize, Serialize};

#[derive(Component, Clone, Copy, Debug, Serialize, Deserialize, Reflect)]
#[reflect(Component)]
pub struct Target {
    pub x: f32,
    pub y: f32,
}

impl Target {
    pub fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }
}

#[derive(Component, Clone, Copy, Debug, Default, Serialize, Deserialize, Reflect)]
#[reflect(Component)]
pub struct WanderState {
    pub wander_angle: f32,
    pub wander_radius: f32,   // TODO: from DNA
    pub wander_distance: f32, // TODO: from DNA
    pub angle_change: f32,    // TODO: from DNA
}

#[derive(Component, Clone, Copy, Debug, Serialize, Deserialize, Reflect)]
#[reflect(Component)]
pub struct FleeState {
    pub flee_speed_multiplier: f32, // TODO: from DNA
}

impl FleeState {
    pub fn new(_threat: Option<Entity>) -> Self {
        Self {
            flee_speed_multiplier: 1.5,
        }
    }
}
