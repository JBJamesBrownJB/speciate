use crate::simulation::creatures::components::CritId;
use crate::simulation::creatures::systems::{EntityIdMap, NextCreatureId};
use crate::simulation::{Simulation, SimulationBuilder};
use bevy_ecs::prelude::*;
use serde::{Deserialize, Serialize};
use std::fs;
use std::io;
use std::path::Path;

pub const SNAPSHOTS_DIR: &str = "snapshots";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SnapshotMetadata {
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
pub struct WorldSnapshot {
    pub metadata: SnapshotMetadata,
    pub world: WorldConfig,
    pub scene_ron: String,
    pub entity_id_map: Vec<(u32, u32)>,
}

#[derive(Debug)]
pub enum SnapshotError {
    IoError(io::Error),
    SerializationError(rmp_serde::encode::Error),
    DeserializationError(rmp_serde::decode::Error),
}

impl From<io::Error> for SnapshotError {
    fn from(err: io::Error) -> Self {
        SnapshotError::IoError(err)
    }
}

impl From<rmp_serde::encode::Error> for SnapshotError {
    fn from(err: rmp_serde::encode::Error) -> Self {
        SnapshotError::SerializationError(err)
    }
}

impl From<rmp_serde::decode::Error> for SnapshotError {
    fn from(err: rmp_serde::decode::Error) -> Self {
        SnapshotError::DeserializationError(err)
    }
}

impl std::fmt::Display for SnapshotError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SnapshotError::IoError(err) => write!(f, "IO error: {}", err),
            SnapshotError::SerializationError(err) => write!(f, "Serialization error: {}", err),
            SnapshotError::DeserializationError(err) => write!(f, "Deserialization error: {}", err),
        }
    }
}

impl std::error::Error for SnapshotError {}

impl WorldSnapshot {
    pub fn save_to_file(&self, path: &Path) -> Result<(), SnapshotError> {
        let bytes = rmp_serde::to_vec(self)?;
        fs::write(path, bytes)?;
        Ok(())
    }

    pub fn load_from_file(path: &Path) -> Result<Self, SnapshotError> {
        let bytes = fs::read(path)?;
        let snapshot = rmp_serde::from_slice(&bytes)?;
        Ok(snapshot)
    }
}

impl Simulation {
    pub fn to_snapshot(&mut self) -> WorldSnapshot {
        use bevy_scene::{DynamicSceneBuilder, serde::SceneSerializer};

        let (min_x, max_x, min_y, max_y) = self.get_boundaries();
        let extent_x = (max_x - min_x) / 2.0;
        let extent_y = (max_y - min_y) / 2.0;

        let mut query_state: QueryState<(Entity, &CritId)> = self.world.query();
        let creature_entities: Vec<Entity> = query_state.iter(&self.world)
            .map(|(entity, _)| entity)
            .collect();

        let creature_count = creature_entities.len();

        let entity_id_map: Vec<(u32, u32)> = query_state.iter(&self.world)
            .map(|(entity, crit_id)| (entity.index(), crit_id.0))
            .collect();

        let type_registry = self.world.resource::<AppTypeRegistry>();

        let scene = DynamicSceneBuilder::from_world(&self.world)
            .allow_all()
            .extract_entities(creature_entities.into_iter())
            .build();

        let type_registry_guard = type_registry.read();
        let scene_serializer = SceneSerializer::new(&scene, &type_registry_guard);
        let scene_ron = bevy_scene::ron::ser::to_string(&scene_serializer)
            .expect("Failed to serialize DynamicScene to RON");

        eprintln!("[DEBUG] RON scene (first 500 chars): {}", &scene_ron.chars().take(500).collect::<String>());
        eprintln!("[DEBUG] RON scene length: {} bytes", scene_ron.len());

        drop(type_registry_guard);

        WorldSnapshot {
            metadata: SnapshotMetadata {
                version: "2.0.0".to_string(),
                created_at: chrono::Utc::now().to_rfc3339(),
                creature_count,
                tick_number: 0,
            },
            world: WorldConfig { extent_x, extent_y },
            scene_ron,
            entity_id_map,
        }
    }

    pub fn from_snapshot(snapshot: WorldSnapshot) -> Self {
        use bevy_scene::serde::SceneDeserializer;

        let mut simulation = SimulationBuilder::new().build();

        simulation.set_boundaries(snapshot.world.extent_x, snapshot.world.extent_y);

        let max_id = snapshot.entity_id_map.iter()
            .map(|(_, crit_id)| *crit_id)
            .max()
            .unwrap_or(0);
        simulation.world.resource_mut::<NextCreatureId>().set_next(max_id + 1);

        let type_registry = simulation.world.resource::<AppTypeRegistry>();
        let type_registry_guard = type_registry.read();

        let mut ron_de = bevy_scene::ron::de::Deserializer::from_str(&snapshot.scene_ron)
            .expect("Failed to create RON deserializer");

        let scene_deserializer = SceneDeserializer {
            type_registry: &type_registry_guard,
        };

        use serde::de::DeserializeSeed;
        let scene = scene_deserializer.deserialize(&mut ron_de)
            .expect("Failed to deserialize DynamicScene from RON");

        drop(type_registry_guard);

        let mut entity_map = bevy_ecs::entity::EntityHashMap::default();
        scene.write_to_world(&mut simulation.world, &mut entity_map)
            .expect("Failed to restore DynamicScene to world");

        let mut query_state: QueryState<(Entity, &CritId)> = simulation.world.query();
        let mut entity_id_map = EntityIdMap::default();
        for (entity, crit_id) in query_state.iter(&simulation.world) {
            entity_id_map.insert(entity, crit_id.0);
        }
        simulation.world.insert_resource(entity_id_map);

        simulation
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::simulation::creatures::builder::CritBuilder;

    #[test]
    fn test_snapshot_metadata_serialization() {
        let metadata = SnapshotMetadata {
            version: "2.0.0".to_string(),
            created_at: "2025-11-04T12:00:00Z".to_string(),
            creature_count: 100,
            tick_number: 12345,
        };

        let bytes = rmp_serde::to_vec(&metadata).unwrap();
        let deserialized: SnapshotMetadata = rmp_serde::from_slice(&bytes).unwrap();

        assert_eq!(metadata.version, deserialized.version);
        assert_eq!(metadata.creature_count, deserialized.creature_count);
    }

    #[test]
    fn test_snapshot_empty_world() {
        let mut sim = SimulationBuilder::new()
            .set_boundaries(100.0, 75.0)
            .build();

        let snapshot = sim.to_snapshot();
        assert_eq!(snapshot.metadata.creature_count, 0);
        assert_eq!(snapshot.world.extent_x, 100.0);
        assert_eq!(snapshot.world.extent_y, 75.0);
    }

    #[test]
    fn test_snapshot_round_trip_preserves_all_components() {
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
        let id2 = sim.spawn_crit(builder2);

        let snapshot = sim.to_snapshot();
        assert_eq!(snapshot.metadata.creature_count, 2);

        let mut restored_sim = Simulation::from_snapshot(snapshot);

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
    fn test_snapshot_file_save_and_load() {
        use std::path::PathBuf;
        use tempfile::TempDir;

        let temp_dir = TempDir::new().unwrap();
        let snapshot_path = temp_dir.path().join("test_snapshot.msgpack");

        let mut sim = SimulationBuilder::new().build();
        let builder = CritBuilder::new().at(10.0, 20.0).with_all_capabilities();
        sim.spawn_crit(builder);

        let snapshot = sim.to_snapshot();
        snapshot.save_to_file(&snapshot_path).expect("Save should succeed");

        let loaded = WorldSnapshot::load_from_file(&snapshot_path).expect("Load should succeed");
        assert_eq!(loaded.metadata.creature_count, 1);

        let restored_sim = Simulation::from_snapshot(loaded);
        assert_eq!(restored_sim.creature_count(), 1);
    }

    #[test]
    fn test_snapshot_preserves_avoidance_components() {
        let mut sim = SimulationBuilder::new().build();

        let builder = CritBuilder::new()
            .at(0.0, 0.0)
            .with_avoidance();
        sim.spawn_crit(builder);

        let snapshot = sim.to_snapshot();

        let mut restored_sim = Simulation::from_snapshot(snapshot);

        use bevy_ecs::query::QueryState;
        use crate::simulation::perception::{Perception, AvoidanceBehavior};

        let mut query: QueryState<(&Perception, &AvoidanceBehavior)> = restored_sim.world_mut().query();
        let components: Vec<_> = query.iter(restored_sim.world()).collect();

        assert_eq!(components.len(), 1, "Restored creature should have Perception and AvoidanceBehavior");

        let (perception, avoidance) = components[0];
        assert!(perception.range > 0.0, "Perception range should be restored");
        assert!(avoidance.personal_space > 0.0, "Avoidance personal_space should be restored");
        assert!(avoidance.max_force > 0.0, "Avoidance max_force should be restored");
    }
}
