use super::builder::CritBuilder;
use bevy_ecs::event::Event;

#[derive(Event, Clone)]
pub struct SpawnCreatureEvent {
    pub builder: CritBuilder,
}

impl SpawnCreatureEvent {
    pub fn new(builder: CritBuilder) -> Self {
        Self { builder }
    }
}
