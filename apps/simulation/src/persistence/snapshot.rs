use crate::simulation::creatures::components::CritId;
use crate::simulation::creatures::systems::NextCreatureId;
use crate::simulation::{Simulation, SimulationBuilder};
use bevy_ecs::prelude::*;
use serde::{Deserialize, Serialize};
use std::fs;
use std::io;
use std::path::Path;

pub const SNAPSHOTS_DIR: &str = "save-states";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SaveStateMetadata {
    pub version: String,
    pub created_at: String,
    pub creature_count: usize,
    pub tick_number: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorldConfig {
    pub extent_x: f32,
    pub extent_y: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorldSaveState {
    pub metadata: SaveStateMetadata,
    pub world: WorldConfig,
    pub scene_ron: String,
    pub entity_id_map: Vec<(u32, u32)>,
}

#[derive(Debug)]
pub enum SaveStateError {
    IoError(io::Error),
    SerializationError(rmp_serde::encode::Error),
    DeserializationError(rmp_serde::decode::Error),
    RonSerializationError(bevy_scene::ron::Error),
    RonDeserializationError(bevy_scene::ron::de::SpannedError),
    SceneWriteError(bevy_scene::SceneSpawnError),
    EmptyWorld,
}

impl From<io::Error> for SaveStateError {
    fn from(err: io::Error) -> Self {
        SaveStateError::IoError(err)
    }
}

impl From<rmp_serde::encode::Error> for SaveStateError {
    fn from(err: rmp_serde::encode::Error) -> Self {
        SaveStateError::SerializationError(err)
    }
}

impl From<rmp_serde::decode::Error> for SaveStateError {
    fn from(err: rmp_serde::decode::Error) -> Self {
        SaveStateError::DeserializationError(err)
    }
}

impl From<bevy_scene::ron::Error> for SaveStateError {
    fn from(err: bevy_scene::ron::Error) -> Self {
        SaveStateError::RonSerializationError(err)
    }
}

impl From<bevy_scene::ron::de::SpannedError> for SaveStateError {
    fn from(err: bevy_scene::ron::de::SpannedError) -> Self {
        SaveStateError::RonDeserializationError(err)
    }
}

impl From<bevy_scene::SceneSpawnError> for SaveStateError {
    fn from(err: bevy_scene::SceneSpawnError) -> Self {
        SaveStateError::SceneWriteError(err)
    }
}

impl std::fmt::Display for SaveStateError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SaveStateError::IoError(err) => write!(f, "IO error: {}", err),
            SaveStateError::SerializationError(err) => write!(f, "Serialization error: {}", err),
            SaveStateError::DeserializationError(err) => write!(f, "Deserialization error: {}", err),
            SaveStateError::RonSerializationError(err) => write!(f, "RON serialization error: {}", err),
            SaveStateError::RonDeserializationError(err) => write!(f, "RON deserialization error: {}", err),
            SaveStateError::SceneWriteError(err) => write!(f, "Scene write error: {}", err),
            SaveStateError::EmptyWorld => write!(f, "Cannot save empty world (no creatures)"),
        }
    }
}

impl std::error::Error for SaveStateError {}

impl WorldSaveState {
    pub fn save_to_file(&self, path: &Path) -> Result<(), SaveStateError> {
        use rmp_serde::encode::Serializer;
        use serde::Serialize;

        let mut buf = Vec::new();
        let mut serializer = Serializer::new(&mut buf)
            .with_struct_map(); // Use map format for better compatibility with large strings

        self.serialize(&mut serializer)?;

        fs::write(path, buf)?;
        Ok(())
    }

    pub fn load_from_file(path: &Path) -> Result<Self, SaveStateError> {
        use rmp_serde::decode::Deserializer;
        use serde::Deserialize;

        let bytes = fs::read(path)?;
        let mut deserializer = Deserializer::new(&bytes[..]);
        let save_state = WorldSaveState::deserialize(&mut deserializer)?;
        Ok(save_state)
    }
}

impl Simulation {
    pub fn to_save_state(&mut self) -> Result<WorldSaveState, SaveStateError> {
        use bevy_scene::{DynamicSceneBuilder, serde::SceneSerializer};

        let (min_x, max_x, min_y, max_y) = self.get_boundaries();
        let extent_x = (max_x - min_x) / 2.0;
        let extent_y = (max_y - min_y) / 2.0;

        let mut query_state: QueryState<(Entity, &CritId)> = self.world.query();
        let creature_entities: Vec<Entity> = query_state.iter(&self.world)
            .map(|(entity, _)| entity)
            .collect();

        let creature_count = creature_entities.len();

        // Don't save empty worlds (prevents corrupted saves)
        if creature_count == 0 {
            return Err(SaveStateError::EmptyWorld);
        }

        let entity_id_map: Vec<(u32, u32)> = query_state.iter(&self.world)
            .map(|(entity, crit_id)| (entity.index(), crit_id.0))
            .collect();

        let type_registry = self.world.resource::<AppTypeRegistry>();

        // Build scene with creature entities only (no .allow_all() to avoid bloat)
        let scene = DynamicSceneBuilder::from_world(&self.world)
            .extract_entities(creature_entities.into_iter())
            .build();

        let type_registry_guard = type_registry.read();
        let scene_serializer = SceneSerializer::new(&scene, &type_registry_guard);
        let scene_ron = bevy_scene::ron::ser::to_string(&scene_serializer)?;

        drop(type_registry_guard);

        Ok(WorldSaveState {
            metadata: SaveStateMetadata {
                version: "2.0.0".to_string(),
                created_at: chrono::Utc::now().to_rfc3339(),
                creature_count,
                tick_number: 0,
            },
            world: WorldConfig { extent_x, extent_y },
            scene_ron,
            entity_id_map,
        })
    }

    pub fn from_save_state(save_state: WorldSaveState) -> Result<Self, SaveStateError> {
        use bevy_scene::serde::SceneDeserializer;

        let mut simulation = SimulationBuilder::new().build();

        simulation.set_boundaries(save_state.world.extent_x, save_state.world.extent_y);

        let max_id = save_state.entity_id_map.iter()
            .map(|(_, crit_id)| *crit_id)
            .max()
            .unwrap_or(0);
        simulation.world.resource_mut::<NextCreatureId>().set_next(max_id + 1);

        let type_registry = simulation.world.resource::<AppTypeRegistry>();
        let type_registry_guard = type_registry.read();

        let mut ron_de = bevy_scene::ron::de::Deserializer::from_str(&save_state.scene_ron)?;

        let scene_deserializer = SceneDeserializer {
            type_registry: &type_registry_guard,
        };

        use serde::de::DeserializeSeed;
        let scene = scene_deserializer.deserialize(&mut ron_de)?;

        drop(type_registry_guard);

        let mut entity_map = bevy_ecs::entity::EntityHashMap::default();
        scene.write_to_world(&mut simulation.world, &mut entity_map)?;

        // Perception and NeighborCache are not serialized (fixed-array optimization).
        // Reconstruct from BodySize.
        use crate::simulation::core::components::BodySize;
        use crate::simulation::perception::{NeighborCache, Perception};
        let entities_needing_perception: Vec<(Entity, f32)> = simulation
            .world
            .query_filtered::<(Entity, &BodySize), Without<Perception>>()
            .iter(&simulation.world)
            .map(|(e, size)| (e, size.length))
            .collect();

        for (entity, body_length) in entities_needing_perception {
            simulation
                .world
                .entity_mut(entity)
                .insert(Perception::from_body_size(body_length));
        }

        // NeighborCache must also be reconstructed (contains Entity references + fixed arrays)
        let entities_needing_neighbor_cache: Vec<Entity> = simulation
            .world
            .query_filtered::<Entity, Without<NeighborCache>>()
            .iter(&simulation.world)
            .filter(|e| simulation.world.get::<CritId>(*e).is_some())
            .collect();

        for entity in entities_needing_neighbor_cache {
            simulation
                .world
                .entity_mut(entity)
                .insert(NeighborCache::new());
        }

        Ok(simulation)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::simulation::creatures::builder::CritBuilder;

    #[test]
    fn test_save_state_metadata_serialization() {
        let metadata = SaveStateMetadata {
            version: "2.0.0".to_string(),
            created_at: "2025-11-04T12:00:00Z".to_string(),
            creature_count: 100,
            tick_number: 12345,
        };

        let bytes = rmp_serde::to_vec(&metadata).unwrap();
        let deserialized: SaveStateMetadata = rmp_serde::from_slice(&bytes).unwrap();

        assert_eq!(metadata.version, deserialized.version);
        assert_eq!(metadata.creature_count, deserialized.creature_count);
    }

    #[test]
    fn test_save_state_empty_world() {
        let mut sim = SimulationBuilder::new()
            .set_boundaries(100.0, 75.0)
            .build();

        // Empty world should return EmptyWorld error
        let result = sim.to_save_state();
        assert!(result.is_err(), "Should not save empty world");
        match result {
            Err(SaveStateError::EmptyWorld) => {
                // Expected error
            }
            _ => panic!("Expected EmptyWorld error"),
        }
    }

    #[test]
    fn test_save_state_round_trip_preserves_all_components() {
        let mut sim = SimulationBuilder::new()
            .set_boundaries(200.0, 150.0)
            .build();

        let builder = CritBuilder::new()
            .at(50.0, 25.0)
            .as_seeker(100.0, 75.0)
            .with_avoidance();
        let id1 = sim.spawn_crit(builder);

        let builder2 = CritBuilder::new()
            .at(-30.0, -40.0)
            .with_wandering();
        let _id2 = sim.spawn_crit(builder2);

        let save_state = sim.to_save_state().expect("Failed to create save state");
        assert_eq!(save_state.metadata.creature_count, 2);

        let mut restored_sim = Simulation::from_save_state(save_state).expect("Failed to restore from save state");

        assert_eq!(restored_sim.creature_count(), 2);

        let (min_x, max_x, min_y, max_y) = restored_sim.get_boundaries();
        assert_eq!(min_x, -200.0);
        assert_eq!(max_x, 200.0);
        assert_eq!(min_y, -150.0);
        assert_eq!(max_y, 150.0);

        use bevy_ecs::query::QueryState;
        use crate::simulation::core::components::Position;
        use crate::simulation::perception::{Perception, AvoidanceBehavior};
        use crate::simulation::creatures::components::perception::Target;
        use crate::simulation::creatures::components::capabilities::*;

        let mut query: QueryState<(
            &CritId,
            &Position,
            Option<&Target>,
            Option<&Perception>,
            Option<&AvoidanceBehavior>,
            Option<&CanSeek>,
            Option<&CanAvoidObstacles>,
        )> = restored_sim.world_mut().query();

        let seeker_data = query.iter(restored_sim.world())
            .find(|(crit_id, _, _, _, _, _, _)| crit_id.0 == id1)
            .expect("Seeker creature should exist");

        assert_eq!(seeker_data.1.x, 50.0);
        assert_eq!(seeker_data.1.y, 25.0);

        assert!(seeker_data.2.is_some(), "Target component should be preserved");
        let target = seeker_data.2.unwrap();
        assert_eq!(target.x, 100.0);
        assert_eq!(target.y, 75.0);

        assert!(seeker_data.3.is_some(), "Perception component should be preserved");

        assert!(seeker_data.4.is_some(), "AvoidanceBehavior component should be preserved");

        assert!(seeker_data.5.is_some(), "CanSeek capability should be preserved");
        assert!(seeker_data.6.is_some(), "CanAvoidObstacles capability should be preserved");
    }

    #[test]
    fn test_save_state_file_save_and_load() {
        use tempfile::TempDir;

        let temp_dir = TempDir::new().unwrap();
        let save_state_path = temp_dir.path().join("test_save_state.msgpack");

        let mut sim = SimulationBuilder::new().build();
        let builder = CritBuilder::new().at(10.0, 20.0).with_all_capabilities();
        sim.spawn_crit(builder);

        let save_state = sim.to_save_state().expect("Failed to create save state");
        save_state.save_to_file(&save_state_path).expect("Save should succeed");

        let loaded = WorldSaveState::load_from_file(&save_state_path).expect("Load should succeed");
        assert_eq!(loaded.metadata.creature_count, 1);

        let restored_sim = Simulation::from_save_state(loaded).expect("Failed to restore from save state");
        assert_eq!(restored_sim.creature_count(), 1);
    }

    #[test]
    fn test_save_state_large_population() {
        use tempfile::TempDir;

        let temp_dir = TempDir::new().unwrap();
        let save_path = temp_dir.path().join("large_save.msgpack");

        let mut sim = SimulationBuilder::new()
            .set_boundaries(10000.0, 10000.0)
            .build();

        // Spawn 1000 creatures to stress-test MessagePack serialization
        for _ in 0..1000 {
            let builder = CritBuilder::new()
                .with_all_capabilities();
            sim.spawn_crit(builder);
        }

        assert_eq!(sim.creature_count(), 1000);

        // Save to file
        let save_state = sim.to_save_state().expect("Failed to create save state");
        save_state.save_to_file(&save_path).expect("Failed to save large state");

        // Verify file exists and is non-empty
        let file_size = std::fs::metadata(&save_path).unwrap().len();
        assert!(file_size > 100_000, "Save file should be large (got {} bytes)", file_size);

        // Load back
        let loaded = WorldSaveState::load_from_file(&save_path)
            .expect("Failed to load large save state");
        assert_eq!(loaded.metadata.creature_count, 1000);

        // Restore simulation
        let restored_sim = Simulation::from_save_state(loaded)
            .expect("Failed to restore from large save state");
        assert_eq!(restored_sim.creature_count(), 1000);
    }

    #[test]
    fn test_save_state_preserves_avoidance_components() {
        let mut sim = SimulationBuilder::new().build();

        let builder = CritBuilder::new()
            .at(0.0, 0.0)
            .with_avoidance();
        sim.spawn_crit(builder);

        let save_state = sim.to_save_state().expect("Failed to create save state");

        let mut restored_sim = Simulation::from_save_state(save_state).expect("Failed to restore from save state");

        use bevy_ecs::query::QueryState;
        use crate::simulation::perception::{Perception, AvoidanceBehavior};

        let mut query: QueryState<(&Perception, &AvoidanceBehavior)> = restored_sim.world_mut().query();
        let components: Vec<_> = query.iter(restored_sim.world()).collect();

        assert_eq!(components.len(), 1, "Restored creature should have Perception and AvoidanceBehavior");

        let (perception, avoidance) = components[0];
        assert!(perception.range > 0.0, "Perception range should be restored");
        assert!(avoidance.personal_space() > 0.0, "Avoidance personal_space should be restored");
    }

    #[test]
    fn test_save_state_reconstructs_neighbor_cache() {
        let mut sim = SimulationBuilder::new().build();

        let builder = CritBuilder::new().at(0.0, 0.0);
        sim.spawn_crit(builder);

        let save_state = sim.to_save_state().expect("Failed to create save state");

        let mut restored_sim = Simulation::from_save_state(save_state).expect("Failed to restore from save state");

        use bevy_ecs::query::QueryState;
        use crate::simulation::perception::NeighborCache;

        let mut query: QueryState<&NeighborCache> = restored_sim.world_mut().query();
        let neighbor_caches: Vec<_> = query.iter(restored_sim.world()).collect();

        assert_eq!(neighbor_caches.len(), 1, "Restored creature should have NeighborCache");
        assert!(!neighbor_caches[0].has_neighbors(), "Fresh NeighborCache should have no neighbors");
    }
}
