use bevy_ecs::prelude::*;
use bevy_reflect::Reflect;
use serde::{Deserialize, Serialize};

pub const DEFAULT_ENERGY: f32 = 100.0;
pub const LOW_ENERGY_THRESHOLD: f32 = 30.0;
pub const EXHAUSTED_THRESHOLD: f32 = 10.0;

#[derive(Clone, Copy, Debug, PartialEq, Default, Serialize, Deserialize, Reflect)]
#[repr(u8)]
pub enum BehaviorMode {
    #[default]
    Catatonic = 0,
    Seeking = 1,
    Wandering = 2,
    Waiting = 3,
}

impl BehaviorMode {
    pub fn is_active(&self) -> bool {
        !matches!(self, BehaviorMode::Catatonic | BehaviorMode::Waiting)
    }
}

#[derive(Component, Clone, Copy, Debug, Serialize, Deserialize, Reflect)]
#[reflect(Component)]
pub struct CreatureState {
    pub behavior: BehaviorMode,
    pub energy: f32,
    pub age: f32,
    pub max_speed: f32,
}

impl Default for CreatureState {
    fn default() -> Self {
        Self {
            behavior: BehaviorMode::Catatonic,
            energy: DEFAULT_ENERGY,
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
        self.energy < LOW_ENERGY_THRESHOLD
    }

    pub fn is_exhausted(&self) -> bool {
        self.energy < EXHAUSTED_THRESHOLD
    }

    pub fn consume_energy(&mut self, amount: f32) {
        self.energy = (self.energy - amount).max(0.0);
    }

    pub fn restore_energy(&mut self, amount: f32) {
        self.energy = (self.energy + amount).min(100.0);
    }
}

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

    pub fn distance_from(&self, pos_x: f32, pos_y: f32) -> f32 {
        let dx = pos_x - self.x;
        let dy = pos_y - self.y;
        (dx * dx + dy * dy).sqrt()
    }
}
