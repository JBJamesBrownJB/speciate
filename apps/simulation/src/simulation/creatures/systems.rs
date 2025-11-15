//! Creature spawning systems
//!
//! This module contains Bevy ECS systems for processing spawn-related events.

use super::events::SpawnCreatureEvent;
use bevy_ecs::prelude::*;
use bevy_ecs::system::Resource;
use log::info;
use std::collections::HashMap;

/// Resource tracking the next available creature ID
///
/// This is the authoritative source for creature IDs. All spawns (state loading,
/// dev commands, gameplay) increment this counter.
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
    /// Generate the next creature ID and increment the counter
    pub fn generate(&mut self) -> u32 {
        let id = self.next_id;
        self.next_id += 1;
        id
    }

    /// Set the next ID (used when restoring from snapshot)
    pub fn set_next(&mut self, next_id: u32) {
        self.next_id = next_id;
    }
}

/// Resource mapping Entity to creature ID
///
/// This allows looking up a creature's ID from its Entity, and is used by
/// creature_count() and other systems that need to track creatures.
#[derive(Resource, Default)]
pub struct EntityIdMap {
    map: HashMap<Entity, u32>,
}

impl EntityIdMap {
    /// Insert a mapping
    pub fn insert(&mut self, entity: Entity, id: u32) {
        self.map.insert(entity, id);
    }

    /// Get creature ID for an entity
    pub fn get(&self, entity: &Entity) -> Option<&u32> {
        self.map.get(entity)
    }

    /// Remove a mapping
    pub fn remove(&mut self, entity: &Entity) -> Option<u32> {
        self.map.remove(entity)
    }

    /// Get the number of creatures
    pub fn len(&self) -> usize {
        self.map.len()
    }

    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.map.is_empty()
    }

    /// Clear all mappings
    pub fn clear(&mut self) {
        self.map.clear();
    }

    /// Iterate over all entity-ID mappings
    pub fn iter(&self) -> impl Iterator<Item = (&Entity, &u32)> {
        self.map.iter()
    }
}

/// Process spawn creature events
///
/// This is the SINGLE AUTHORITATIVE HANDLER for all creature spawns.
/// All spawn requests flow through SpawnCreatureEvent → this system → ECS World.
///
/// The system:
/// 1. Reads spawn events from the event queue
/// 2. Assigns unique IDs from NextCreatureId
/// 3. Spawns entities using Commands
/// 4. Registers entities in EntityIdMap
///
/// # Integration
///
/// This system should run early in the schedule, before behavior systems,
/// so that newly spawned creatures are immediately available.
pub fn process_spawn_events(
    mut commands: Commands,
    mut events: EventReader<SpawnCreatureEvent>,
    mut next_id: ResMut<NextCreatureId>,
    mut entity_map: ResMut<EntityIdMap>,
) {
    for event in events.read() {
        // Assign unique ID
        let id = next_id.generate();

        // Build the creature bundle
        let bundle = event.builder.clone().build(id);

        // Spawn into ECS world
        let entity = commands.spawn(bundle).id();

        // Register in entity map
        entity_map.insert(entity, id);

        info!("[SPAWN] Creature #{} spawned via event system", id);
    }
}
