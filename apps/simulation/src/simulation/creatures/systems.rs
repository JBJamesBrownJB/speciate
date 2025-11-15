
use super::events::SpawnCreatureEvent;
use bevy_ecs::prelude::*;
use bevy_ecs::system::Resource;
use log::info;
use std::collections::HashMap;

#[derive(Resource)]
pub struct NextCreatureId {
    pub next_id: u32,
}

impl Default for NextCreatureId {
    fn default() -> Self {
        Self { next_id: 1 }
    }
}

impl NextCreatureId {
    pub fn generate(&mut self) -> u32 {
        let id = self.next_id;
        self.next_id += 1;
        id
    }

    pub fn set_next(&mut self, next_id: u32) {
        self.next_id = next_id;
    }
}

#[derive(Resource, Default)]
pub struct EntityIdMap {
    map: HashMap<Entity, u32>,
}

impl EntityIdMap {
    pub fn insert(&mut self, entity: Entity, id: u32) {
        self.map.insert(entity, id);
    }

    pub fn get(&self, entity: &Entity) -> Option<&u32> {
        self.map.get(entity)
    }

    pub fn remove(&mut self, entity: &Entity) -> Option<u32> {
        self.map.remove(entity)
    }

    pub fn len(&self) -> usize {
        self.map.len()
    }

    pub fn is_empty(&self) -> bool {
        self.map.is_empty()
    }

    pub fn clear(&mut self) {
        self.map.clear();
    }

    pub fn iter(&self) -> impl Iterator<Item = (&Entity, &u32)> {
        self.map.iter()
    }
}

pub fn process_spawn_events(
    mut commands: Commands,
    mut events: EventReader<SpawnCreatureEvent>,
    mut next_id: ResMut<NextCreatureId>,
    mut entity_map: ResMut<EntityIdMap>,
) {
    for event in events.read() {

        let id = next_id.generate();


        let bundle = event.builder.clone().build(id);


        let entity = commands.spawn(bundle).id();


        entity_map.insert(entity, id);

        info!("[SPAWN] Creature #{} spawned via event system", id);
    }
}
