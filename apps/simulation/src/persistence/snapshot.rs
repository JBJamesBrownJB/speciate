//! Simulation snapshot and persistence
//!
//! This module handles saving and loading complete simulation state to/from binary files.
//! Uses Bevy Reflection with DynamicSceneBuilder to automatically serialize ALL registered components.
//!
//! ## Architecture
//!
//! **Problem:** Manually listing components in queries hits Bevy's 16-element tuple limit and is error-prone.
//!
//! **Solution:** Use Bevy's reflection system (DynamicSceneBuilder) to automatically capture ALL components
//! that have been registered in the AppTypeRegistry. This ensures:
//! - No components are missed
//! - No query tuple limits
//! - Automatic schema evolution as new components are added
//!
//! **How it works:**
//! 1. All components have `#[derive(Reflect, Serialize, Deserialize)]`
//! 2. All components are registered in SimulationBuilder::new() via AppTypeRegistry
//! 3. DynamicSceneBuilder extracts all registered components from all entities with CritId
//! 4. DynamicScene is serialized to MessagePack for compact storage
//!
//! ## Recommended Directory Structure
//!
//! Snapshots should be stored in the `snapshots/` directory:
//!
//! ```text
//! snapshots/
//!   ├── simulation_2025-11-04_12-30-00.msgpack
//!   ├── simulation_2025-11-04_13-00-00.msgpack
//!   └── latest.msgpack
//! ```
//!
//! ## Usage Example
//!
//! ```no_run
//! use speciate::simulation::{Simulation, SimulationBuilder};
//! use speciate::persistence::WorldSnapshot;
//! use std::path::PathBuf;
//!
//! // Save snapshot
//! let mut simulation = SimulationBuilder::new().build();
//! let snapshot = simulation.to_snapshot();
//! snapshot.save_to_file(&PathBuf::from("snapshots/latest.msgpack")).unwrap();
//!
//! // Load snapshot
//! let loaded = WorldSnapshot::load_from_file(&PathBuf::from("snapshots/latest.msgpack")).unwrap();
//! let restored_sim = Simulation::from_snapshot(loaded);
//! ```

use crate::simulation::creatures::components::CritId;
use crate::simulation::creatures::systems::{EntityIdMap, NextCreatureId};
use crate::simulation::{Simulation, SimulationBuilder};
use bevy_ecs::prelude::*;
use serde::{Deserialize, Serialize};
use std::fs;
use std::io;
use std::path::Path;

/// Default directory for storing snapshots
pub const SNAPSHOTS_DIR: &str = "snapshots";

/// Metadata about a snapshot
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SnapshotMetadata {
    /// Snapshot format version (semantic versioning)
    pub version: String,
    /// ISO 8601 timestamp of snapshot creation
    pub created_at: String,
    /// Number of creatures in this snapshot
    pub creature_count: usize,
    /// Simulation tick number (for future time tracking)
    pub tick_number: u64,
}

/// World configuration at time of snapshot (centered coordinate system)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorldConfig {
    pub extent_x: f32, // Half-width (world spans -extent_x to +extent_x)
    pub extent_y: f32, // Half-height (world spans -extent_y to +extent_y)
}

/// Complete snapshot of the simulation world using Bevy Reflection
///
/// This uses DynamicScene to automatically capture ALL registered components
/// from all entities with a CritId component. No manual component listing required!
///
/// The scene is serialized to RON (Rusty Object Notation) format, which is Bevy's
/// native scene format. This ensures ALL registered components are preserved.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorldSnapshot {
    /// Snapshot metadata
    pub metadata: SnapshotMetadata,
    /// World configuration
    pub world: WorldConfig,
    /// Bevy DynamicScene serialized to RON format
    /// Contains all creature entities and their components
    /// This automatically includes ALL components registered in AppTypeRegistry
    pub scene_ron: String,
    /// Entity ID mapping (Entity → CritId) for reference
    pub entity_id_map: Vec<(u32, u32)>, // (entity_index, crit_id)
}

/// Errors that can occur during snapshot operations
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
    /// Save snapshot to a file using MessagePack binary format
    pub fn save_to_file(&self, path: &Path) -> Result<(), SnapshotError> {
        let bytes = rmp_serde::to_vec(self)?;
        fs::write(path, bytes)?;
        Ok(())
    }

    /// Load snapshot from a file
    pub fn load_from_file(path: &Path) -> Result<Self, SnapshotError> {
        let bytes = fs::read(path)?;
        let snapshot = rmp_serde::from_slice(&bytes)?;
        Ok(snapshot)
    }
}

impl Simulation {
    /// Create a snapshot of the current simulation state using Bevy Reflection
    ///
    /// Uses DynamicSceneBuilder to automatically serialize ALL registered components.
    /// This ensures no components are missed and works around Bevy's query tuple size limit.
    pub fn to_snapshot(&mut self) -> WorldSnapshot {
        use bevy_scene::{DynamicSceneBuilder, serde::SceneSerializer};

        // Get world boundaries (centered coordinate system)
        let (min_x, max_x, min_y, max_y) = self.get_boundaries();
        let extent_x = (max_x - min_x) / 2.0;
        let extent_y = (max_y - min_y) / 2.0;

        // Query all entities with CritId component (all creatures)
        let mut query_state: QueryState<(Entity, &CritId)> = self.world.query();
        let creature_entities: Vec<Entity> = query_state.iter(&self.world)
            .map(|(entity, _)| entity)
            .collect();

        let creature_count = creature_entities.len();

        // Build entity ID mapping for reference
        let entity_id_map: Vec<(u32, u32)> = query_state.iter(&self.world)
            .map(|(entity, crit_id)| (entity.index(), crit_id.0))
            .collect();

        // Use DynamicSceneBuilder to extract ALL registered components from creatures
        // This automatically includes Position, Velocity, Perception, AvoidanceBehavior,
        // capability markers, and any future components we add!
        let type_registry = self.world.resource::<AppTypeRegistry>();

        let scene = DynamicSceneBuilder::from_world(&self.world)
            .allow_all()  // CRITICAL: Allow all components to be extracted
            .extract_entities(creature_entities.into_iter())
            .build();

        // Serialize scene to RON format (Bevy's native scene format)
        let type_registry_guard = type_registry.read();
        let scene_serializer = SceneSerializer::new(&scene, &type_registry_guard);
        let scene_ron = bevy_scene::ron::ser::to_string(&scene_serializer)
            .expect("Failed to serialize DynamicScene to RON");

        // Debug: Print first 500 chars of RON to see what's being serialized
        eprintln!("[DEBUG] RON scene (first 500 chars): {}", &scene_ron.chars().take(500).collect::<String>());
        eprintln!("[DEBUG] RON scene length: {} bytes", scene_ron.len());

        drop(type_registry_guard);

        WorldSnapshot {
            metadata: SnapshotMetadata {
                version: "2.0.0".to_string(), // Bumped version for DynamicScene format
                created_at: chrono::Utc::now().to_rfc3339(),
                creature_count,
                tick_number: 0, // TODO: Track tick number in Simulation
            },
            world: WorldConfig { extent_x, extent_y },
            scene_ron,
            entity_id_map,
        }
    }

    /// Restore simulation from a snapshot using Bevy Reflection
    ///
    /// Uses DynamicScene::write_to_world() to automatically restore ALL components.
    /// This ensures perfect fidelity with the saved state.
    pub fn from_snapshot(snapshot: WorldSnapshot) -> Self {
        use bevy_scene::serde::SceneDeserializer;

        let mut simulation = SimulationBuilder::new().build();

        // Set world boundaries from snapshot (extents, not full dimensions)
        simulation.set_boundaries(snapshot.world.extent_x, snapshot.world.extent_y);

        // Find the maximum CritId to set next_id correctly
        let max_id = snapshot.entity_id_map.iter()
            .map(|(_, crit_id)| *crit_id)
            .max()
            .unwrap_or(0);
        simulation.world.resource_mut::<NextCreatureId>().set_next(max_id + 1);

        // Deserialize RON scene data
        let type_registry = simulation.world.resource::<AppTypeRegistry>();
        let type_registry_guard = type_registry.read();

        let mut ron_de = bevy_scene::ron::de::Deserializer::from_str(&snapshot.scene_ron)
            .expect("Failed to create RON deserializer");

        let scene_deserializer = SceneDeserializer {
            type_registry: &type_registry_guard,
        };

        // Use DeserializeSeed to deserialize with type registry context
        use serde::de::DeserializeSeed;
        let scene = scene_deserializer.deserialize(&mut ron_de)
            .expect("Failed to deserialize DynamicScene from RON");

        // Drop the type registry guard before mutably borrowing the world
        drop(type_registry_guard);

        // Write the DynamicScene to the world - this automatically restores ALL components!
        // The scene contains all creatures with all their components exactly as they were saved.
        let mut entity_map = bevy_ecs::entity::EntityHashMap::default();
        scene.write_to_world(&mut simulation.world, &mut entity_map)
            .expect("Failed to restore DynamicScene to world");

        // Rebuild the EntityIdMap from the restored entities
        // Query all entities with CritId and add them to the map
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
        // Create simulation with a creature that has ALL capabilities and components
        let mut sim = SimulationBuilder::new()
            .set_boundaries(200.0, 150.0)
            .build();

        // Spawn a seeker with avoidance (tests multiple components)
        let builder = CritBuilder::new()
            .at(50.0, 25.0)
            .as_seeker(100.0, 75.0)
            .with_avoidance();
        let id1 = sim.spawn_crit(builder);

        // Spawn a wanderer
        let builder2 = CritBuilder::new()
            .at(-30.0, -40.0)
            .with_wandering();
        let id2 = sim.spawn_crit(builder2);

        // Take snapshot
        let snapshot = sim.to_snapshot();
        assert_eq!(snapshot.metadata.creature_count, 2);

        // Restore from snapshot
        let mut restored_sim = Simulation::from_snapshot(snapshot);

        // Verify creature count
        assert_eq!(restored_sim.creature_count(), 2);

        // Verify world boundaries
        let (min_x, max_x, min_y, max_y) = restored_sim.get_boundaries();
        assert_eq!(min_x, -200.0);
        assert_eq!(max_x, 200.0);
        assert_eq!(min_y, -150.0);
        assert_eq!(max_y, 150.0);

        // CRITICAL TEST: Verify ALL components are preserved
        // This is the test that will catch missing Perception/AvoidanceBehavior components!
        use bevy_ecs::query::QueryState;
        use crate::simulation::core::components::Position;
        use crate::simulation::perception::{Perception, AvoidanceBehavior};
        use crate::simulation::creatures::components::perception::Target;
        use crate::simulation::creatures::components::capabilities::*;

        // Verify seeker has Target, Perception, and AvoidanceBehavior components
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

        // Verify position preserved
        assert_eq!(seeker_data.1.x, 50.0);
        assert_eq!(seeker_data.1.y, 25.0);

        // Verify Target component preserved (critical!)
        assert!(seeker_data.2.is_some(), "Target component should be preserved");
        let target = seeker_data.2.unwrap();
        assert_eq!(target.x, 100.0);
        assert_eq!(target.y, 75.0);

        // Verify Perception component preserved (CRITICAL - this is what was missing!)
        assert!(seeker_data.3.is_some(), "Perception component should be preserved");

        // Verify AvoidanceBehavior component preserved (CRITICAL - this is what was missing!)
        assert!(seeker_data.4.is_some(), "AvoidanceBehavior component should be preserved");

        // Verify capability markers preserved
        assert!(seeker_data.5.is_some(), "CanSeek capability should be preserved");
        assert!(seeker_data.6.is_some(), "CanAvoidObstacles capability should be preserved");
    }

    #[test]
    fn test_snapshot_file_save_and_load() {
        use std::path::PathBuf;
        use tempfile::TempDir;

        let temp_dir = TempDir::new().unwrap();
        let snapshot_path = temp_dir.path().join("test_snapshot.msgpack");

        // Create simulation and take snapshot
        let mut sim = SimulationBuilder::new().build();
        let builder = CritBuilder::new().at(10.0, 20.0).with_all_capabilities();
        sim.spawn_crit(builder);

        let snapshot = sim.to_snapshot();
        snapshot.save_to_file(&snapshot_path).expect("Save should succeed");

        // Load snapshot from file
        let loaded = WorldSnapshot::load_from_file(&snapshot_path).expect("Load should succeed");
        assert_eq!(loaded.metadata.creature_count, 1);

        // Restore simulation
        let restored_sim = Simulation::from_snapshot(loaded);
        assert_eq!(restored_sim.creature_count(), 1);
    }

    /// Regression test for Bug #3: Creatures walk through each other after snapshot reload
    ///
    /// This test verifies that Perception and AvoidanceBehavior components are preserved
    /// in snapshots, ensuring collision avoidance continues to work after reload.
    #[test]
    fn test_snapshot_preserves_avoidance_components() {
        let mut sim = SimulationBuilder::new().build();

        // Spawn creature with avoidance capability
        let builder = CritBuilder::new()
            .at(0.0, 0.0)
            .with_avoidance();
        sim.spawn_crit(builder);

        // Take snapshot
        let snapshot = sim.to_snapshot();

        // Restore from snapshot
        let mut restored_sim = Simulation::from_snapshot(snapshot);

        // Query for Perception and AvoidanceBehavior components
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
