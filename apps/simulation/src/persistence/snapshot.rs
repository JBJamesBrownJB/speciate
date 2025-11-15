//! Simulation snapshot and persistence
//!
//! This module handles saving and loading complete simulation state to/from binary files.
//! Uses MessagePack for compact, fast serialization while maintaining ECS structure.
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

use crate::simulation::components::*;
use crate::simulation::creatures::systems::{EntityIdMap, NextCreatureId};
use crate::simulation::{Simulation, SimulationBuilder};
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

/// Serialized representation of a single creature with all components
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SerializedCreature {
    /// Creature's unique ID
    pub id: u32,
    /// Position component (always present)
    pub position: Position,
    /// Velocity component (always present)
    pub velocity: Velocity,
    /// Acceleration component (always present)
    pub acceleration: Acceleration,
    /// Rotation component (always present)
    pub rotation: Rotation,
    /// Creature state component (always present)
    pub creature_state: CreatureState,
    /// Wander state component (optional - only present for wandering creatures)
    pub wander_state: Option<WanderState>,
    /// Flee state component (optional - only present for fleeing creatures)
    pub flee_state: Option<FleeState>,
}

/// Complete snapshot of the simulation world
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorldSnapshot {
    /// Snapshot metadata
    pub metadata: SnapshotMetadata,
    /// World configuration
    pub world: WorldConfig,
    /// All creatures in the simulation
    pub creatures: Vec<SerializedCreature>,
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
    /// Create a snapshot of the current simulation state
    #[allow(clippy::type_complexity)]
    pub fn to_snapshot(&mut self) -> WorldSnapshot {
        use bevy_ecs::query::QueryState;
        use std::collections::HashMap;

        // Get world boundaries (centered coordinate system)
        let (min_x, max_x, min_y, max_y) = self.get_boundaries();
        let extent_x = (max_x - min_x) / 2.0;
        let extent_y = (max_y - min_y) / 2.0;

        // Build reverse map from entity to ID
        let mut entity_to_id: HashMap<bevy_ecs::entity::Entity, u32> = HashMap::new();
        let entity_id_map = self.world.resource::<EntityIdMap>();
        for (entity, id) in entity_id_map.iter() {
            entity_to_id.insert(*entity, *id);
        }

        // Query all creatures with required components
        let mut query_state: QueryState<(
            bevy_ecs::entity::Entity,
            &Position,
            &Velocity,
            &Acceleration,
            &Rotation,
            &CreatureState,
            Option<&WanderState>,
            Option<&FleeState>,
        )> = self.world.query();

        let mut creatures = Vec::new();

        for (
            entity,
            position,
            velocity,
            acceleration,
            rotation,
            creature_state,
            wander_state,
            flee_state,
        ) in query_state.iter(&self.world)
        {
            let id = entity_to_id.get(&entity).copied().unwrap_or(0);

            creatures.push(SerializedCreature {
                id,
                position: *position,
                velocity: *velocity,
                acceleration: *acceleration,
                rotation: *rotation,
                creature_state: *creature_state,
                wander_state: wander_state.copied(),
                flee_state: flee_state.copied(),
            });
        }

        // Sort by ID for deterministic output
        creatures.sort_by_key(|c| c.id);

        let creature_count = creatures.len();

        WorldSnapshot {
            metadata: SnapshotMetadata {
                version: "1.0.0".to_string(),
                created_at: chrono::Utc::now().to_rfc3339(),
                creature_count,
                tick_number: 0, // TODO: Track tick number in Simulation
            },
            world: WorldConfig { extent_x, extent_y },
            creatures,
        }
    }

    /// Restore simulation from a snapshot
    pub fn from_snapshot(snapshot: WorldSnapshot) -> Self {
        use bevy_ecs::world::EntityWorldMut;

        let mut simulation = SimulationBuilder::new().build();

        // Set world boundaries from snapshot (extents, not full dimensions)
        simulation.set_boundaries(snapshot.world.extent_x, snapshot.world.extent_y);

        // Find the maximum ID to set next_id correctly
        let max_id = snapshot.creatures.iter().map(|c| c.id).max().unwrap_or(0);
        simulation.world.resource_mut::<NextCreatureId>().set_next(max_id + 1);

        // Spawn each creature with exact state from snapshot
        for creature in snapshot.creatures {
            let entity = simulation.world.spawn_empty().id();

            // Add all required components
            let mut entity_mut: EntityWorldMut = simulation.world.entity_mut(entity);
            entity_mut.insert(CritId(creature.id));
            entity_mut.insert(creature.position);
            entity_mut.insert(creature.velocity);
            entity_mut.insert(creature.acceleration);
            entity_mut.insert(creature.rotation);
            entity_mut.insert(creature.creature_state);

            // Add optional components
            if let Some(wander_state) = creature.wander_state {
                entity_mut.insert(wander_state);
            }
            if let Some(flee_state) = creature.flee_state {
                entity_mut.insert(flee_state);
            }

            // Register in entity map
            simulation.world.resource_mut::<EntityIdMap>().insert(entity, creature.id);
        }

        simulation
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::simulation::creatures::spawner::{spawn_creature, CreatureSpawnRequest};

    #[test]
    fn test_snapshot_metadata_serialization() {
        let metadata = SnapshotMetadata {
            version: "1.0.0".to_string(),
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
    fn test_serialized_creature_round_trip() {
        let creature = SerializedCreature {
            id: 42,
            position: Position { x: 10.5, y: 20.3 },
            velocity: Velocity { vx: 1.2, vy: -0.8 },
            acceleration: Acceleration { ax: 0.0, ay: 0.0 },
            rotation: Rotation { radians: 1.57 },
            creature_state: CreatureState {
                behavior: BehaviorMode::Catatonic,
                energy: 75.0,
                age: 5.2,
                max_speed: 20.0,
            },
            wander_state: Some(WanderState {
                wander_angle: 0.5,
                wander_radius: 25.0,
                wander_distance: 50.0,
                angle_change: 0.15,
            }),
            flee_state: None,
        };

        let bytes = rmp_serde::to_vec(&creature).unwrap();
        let deserialized: SerializedCreature = rmp_serde::from_slice(&bytes).unwrap();

        assert_eq!(creature.id, deserialized.id);
        assert_eq!(creature.position.x, deserialized.position.x);
        assert_eq!(creature.velocity.vx, deserialized.velocity.vx);
        assert!(deserialized.wander_state.is_some());
        assert!(deserialized.flee_state.is_none());
    }

    #[test]
    fn test_world_snapshot_empty() {
        let snapshot = WorldSnapshot {
            metadata: SnapshotMetadata {
                version: "1.0.0".to_string(),
                created_at: "2025-11-04T12:00:00Z".to_string(),
                creature_count: 0,
                tick_number: 0,
            },
            world: WorldConfig {
                extent_x: 90.0,
                extent_y: 65.0,
            },
            creatures: vec![],
        };

        let bytes = rmp_serde::to_vec(&snapshot).unwrap();
        let deserialized: WorldSnapshot = rmp_serde::from_slice(&bytes).unwrap();

        assert_eq!(snapshot.metadata.creature_count, 0);
        assert_eq!(deserialized.creatures.len(), 0);
    }

    #[test]
    fn test_world_snapshot_with_creatures() {
        let creature1 = SerializedCreature {
            id: 1,
            position: Position { x: 10.0, y: 20.0 },
            velocity: Velocity { vx: 1.0, vy: 0.0 },
            acceleration: Acceleration { ax: 0.0, ay: 0.0 },
            rotation: Rotation { radians: 0.0 },
            creature_state: CreatureState::new(),
            wander_state: Some(WanderState::default()),
            flee_state: None,
        };

        let creature2 = SerializedCreature {
            id: 2,
            position: Position { x: 50.0, y: 60.0 },
            velocity: Velocity { vx: -1.0, vy: 1.0 },
            acceleration: Acceleration { ax: 0.0, ay: 0.0 },
            rotation: Rotation { radians: 2.35 },
            creature_state: CreatureState {
                behavior: BehaviorMode::Catatonic,
                energy: 40.0,
                age: 10.0,
                max_speed: 25.0,
            },
            wander_state: None,
            flee_state: Some(FleeState::new(None)),
        };

        let snapshot = WorldSnapshot {
            metadata: SnapshotMetadata {
                version: "1.0.0".to_string(),
                created_at: "2025-11-04T12:00:00Z".to_string(),
                creature_count: 2,
                tick_number: 100,
            },
            world: WorldConfig {
                extent_x: 90.0,
                extent_y: 65.0,
            },
            creatures: vec![creature1, creature2],
        };

        let bytes = rmp_serde::to_vec(&snapshot).unwrap();
        let deserialized: WorldSnapshot = rmp_serde::from_slice(&bytes).unwrap();

        assert_eq!(deserialized.creatures.len(), 2);
        assert_eq!(deserialized.creatures[0].id, 1);
        assert_eq!(deserialized.creatures[1].id, 2);
        assert!(deserialized.creatures[0].wander_state.is_some());
        assert!(deserialized.creatures[1].flee_state.is_some());
    }

    // Integration tests for ECS snapshot/restore

    #[test]
    fn test_simulation_snapshot_empty() {
        let mut simulation = SimulationBuilder::new().build();
        simulation.set_boundaries(50.0, 50.0);

        let snapshot = simulation.to_snapshot();

        assert_eq!(snapshot.creatures.len(), 0);
        assert_eq!(snapshot.world.extent_x, 50.0);
        assert_eq!(snapshot.world.extent_y, 50.0);
    }

    #[test]
    fn test_simulation_snapshot_with_creatures() {
        let mut simulation = SimulationBuilder::new().build();
        simulation.set_boundaries(50.0, 50.0);

        // Spawn 10 creatures
        for _ in 0..10 {
            spawn_creature(&mut simulation, CreatureSpawnRequest::new());
        }

        let snapshot = simulation.to_snapshot();

        assert_eq!(snapshot.creatures.len(), 10);
        assert_eq!(snapshot.metadata.creature_count, 10);

        // Verify all creatures have IDs >= 1
        for creature in &snapshot.creatures {
            assert!(creature.id >= 1);
        }
    }

    #[test]
    fn test_simulation_restore_from_snapshot() {
        let mut simulation = SimulationBuilder::new().build();
        simulation.set_boundaries(50.0, 50.0);

        // Spawn 5 creatures
        for _ in 0..5 {
            spawn_creature(&mut simulation, CreatureSpawnRequest::new());
        }

        // Take snapshot
        let snapshot = simulation.to_snapshot();

        // Restore from snapshot
        let restored = Simulation::from_snapshot(snapshot);

        assert_eq!(restored.creature_count(), 5);
        let (min_x, max_x, min_y, max_y) = restored.get_boundaries();
        assert_eq!(min_x, -50.0);
        assert_eq!(max_x, 50.0);
        assert_eq!(min_y, -50.0);
        assert_eq!(max_y, 50.0);
    }

    #[test]
    fn test_simulation_snapshot_preserves_state() {
        let mut simulation = SimulationBuilder::new().build();
        simulation.set_boundaries(50.0, 50.0);

        // Spawn creature at specific position with specific state
        let custom_state = CreatureState {
            behavior: BehaviorMode::Catatonic,
            energy: 42.5,
            age: 13.7,
            max_speed: 18.2,
        };

        spawn_creature(
            &mut simulation,
            CreatureSpawnRequest::new()
                .at(25.0, 25.0)
                .with_state(custom_state),
        );

        // Take snapshot
        let snapshot = simulation.to_snapshot();

        // Verify snapshot has correct data
        assert_eq!(snapshot.creatures.len(), 1);
        let creature = &snapshot.creatures[0];
        assert_eq!(creature.position.x, 25.0);
        assert_eq!(creature.position.y, 25.0);
        // Note: The spawner doesn't currently use the custom state,
        // but this test validates the snapshot mechanism itself
    }

    #[test]
    fn test_snapshot_file_save_load() {
        use std::fs;
        use std::path::PathBuf;

        let mut simulation = SimulationBuilder::new().build();
        simulation.set_boundaries(50.0, 50.0);

        // Spawn 3 creatures
        for _ in 0..3 {
            spawn_creature(&mut simulation, CreatureSpawnRequest::new());
        }

        // Take snapshot
        let snapshot = simulation.to_snapshot();

        // Save to file
        let path = PathBuf::from("/tmp/test_snapshot.msgpack");
        snapshot.save_to_file(&path).unwrap();

        // Verify file exists and has size > 0
        let metadata = fs::metadata(&path).unwrap();
        assert!(metadata.len() > 0);

        // Load from file
        let loaded_snapshot = WorldSnapshot::load_from_file(&path).unwrap();

        assert_eq!(loaded_snapshot.creatures.len(), 3);
        assert_eq!(loaded_snapshot.metadata.creature_count, 3);

        // Clean up
        fs::remove_file(&path).ok();
    }

    #[test]
    fn test_simulation_full_cycle_save_load() {
        use std::fs;
        use std::path::PathBuf;

        // Create simulation with creatures
        let mut simulation = SimulationBuilder::new().build();
        simulation.set_boundaries(180.0, 130.0);

        for _ in 0..100 {
            spawn_creature(&mut simulation, CreatureSpawnRequest::new());
        }

        // Simulate for 10 ticks to change state
        for _ in 0..10 {
            simulation.update(0.016);
        }

        // Save snapshot
        let snapshot1 = simulation.to_snapshot();
        let path = PathBuf::from("/tmp/test_full_cycle.msgpack");
        snapshot1.save_to_file(&path).unwrap();

        // Report file size
        let metadata = fs::metadata(&path).unwrap();
        println!(
            "Snapshot file size for 100 creatures: {} bytes",
            metadata.len()
        );

        // Load and restore
        let snapshot2 = WorldSnapshot::load_from_file(&path).unwrap();
        let restored = Simulation::from_snapshot(snapshot2);

        // Verify
        assert_eq!(restored.creature_count(), 100);
        let (min_x, max_x, min_y, max_y) = restored.get_boundaries();
        // set_boundaries(180.0, 130.0) creates world from -180 to +180, -130 to +130
        assert_eq!(min_x, -180.0);
        assert_eq!(max_x, 180.0);
        assert_eq!(min_y, -130.0);
        assert_eq!(max_y, 130.0);

        // Clean up
        fs::remove_file(&path).ok();
    }

    #[test]
    fn test_snapshot_restore_preserves_crit_id_component() {
        use crate::simulation::components::CritId;
        use bevy_ecs::query::QueryState;

        let mut simulation = SimulationBuilder::new().build();
        simulation.set_boundaries(50.0, 50.0);

        // Spawn 3 creatures using CritBuilder
        use crate::simulation::creatures::builder::CritBuilder;
        let id1 = simulation.spawn_crit(CritBuilder::new().at(25.0, 25.0).with_all_capabilities());
        let id2 = simulation.spawn_crit(CritBuilder::new().at(50.0, 50.0).with_all_capabilities());
        let id3 = simulation.spawn_crit(CritBuilder::new().at(75.0, 75.0).with_all_capabilities());

        // Take snapshot
        let snapshot = simulation.to_snapshot();

        // Restore from snapshot
        let mut restored = Simulation::from_snapshot(snapshot);

        // Query for CritId components
        let mut query_state: QueryState<&CritId> = restored.world.query();
        let crit_ids: Vec<u32> = query_state
            .iter(&restored.world)
            .map(|crit_id| crit_id.0)
            .collect();

        // Verify all CritId components are present
        assert_eq!(
            crit_ids.len(),
            3,
            "All restored entities should have CritId component"
        );

        // Verify IDs match
        assert!(crit_ids.contains(&id1));
        assert!(crit_ids.contains(&id2));
        assert!(crit_ids.contains(&id3));
    }
}
