//! Bevy App wrapper for NAPI integration
//!
//! This module bridges the existing Bevy simulation with the NAPI custom run loop.
//! It handles:
//! - Command processing from JavaScript (spawn, kill_all, load_trial)
//! - Position export to DoubleBuffer (SoA layout)
//! - Telemetry collection and reporting

use crate::simulation::core::{Simulation, SimulationBuilder, MAX_WORLD_SIZE};
use crate::simulation::core::components::{Position, Rotation};
use crate::simulation::creatures::components::{BehaviorMode, CritId};
use crate::simulation::creatures::builder::CritBuilder;
use super::{DoubleBuffer, TelemetrySnapshot};
#[cfg(feature = "dev-tools")]
use super::PerceptionDebugBuffer;
use crate::ipc::SimCommand;
use bevy_ecs::prelude::Entity;
use crossbeam_channel::Receiver;
use parking_lot::Mutex;
use std::sync::Arc;

/// NAPI-specific Bevy app wrapper
pub struct NapiApp {
    simulation: Simulation,
    command_rx: Receiver<SimCommand>,
}

impl NapiApp {
    /// Create new NapiApp with command receiver
    ///
    /// If `save_state_path` is provided and the file exists, loads simulation from save state.
    /// Otherwise, creates a new simulation and spawns `initial_count` creatures.
    pub fn new(
        command_rx: Receiver<SimCommand>,
        initial_count: u32,
        assets_path: String,
        save_state_path: Option<String>,
    ) -> Self {
        use crate::persistence::WorldSaveState;
        use std::path::Path;

        let mut simulation = if let Some(ref path_str) = save_state_path {
            let path = Path::new(path_str);

            if path.exists() {
                eprintln!("[NAPI] Loading save state from: {}", path_str);
                match WorldSaveState::load_from_file(path) {
                    Ok(save_state) => {
                        eprintln!("[NAPI] ✅ Loaded save state: {} creatures at tick {}",
                            save_state.metadata.creature_count,
                            save_state.metadata.tick_number);
                        match Simulation::from_save_state(save_state) {
                            Ok(sim) => sim,
                            Err(e) => {
                                eprintln!("[NAPI] ⚠️  Failed to restore simulation from save state: {}. Starting fresh.", e);
                                SimulationBuilder::new()
                                    .set_boundaries(MAX_WORLD_SIZE, MAX_WORLD_SIZE)
                                    .build()
                            }
                        }
                    }
                    Err(e) => {
                        eprintln!("[NAPI] ⚠️  Failed to load save state: {}. Starting fresh.", e);
                        SimulationBuilder::new()
                            .set_boundaries(MAX_WORLD_SIZE, MAX_WORLD_SIZE)
                            .build()
                    }
                }
            } else {
                eprintln!("[NAPI] Save state not found at: {}. Starting fresh.", path_str);
                SimulationBuilder::new()
                    .set_boundaries(MAX_WORLD_SIZE, MAX_WORLD_SIZE)
                    .build()
            }
        } else {
            eprintln!("[NAPI] No save state provided. Starting fresh.");
            SimulationBuilder::new()
                .set_boundaries(MAX_WORLD_SIZE, MAX_WORLD_SIZE)
                .build()
        };

        // Set assets path for trial loading
        simulation.set_assets_path(&assets_path);

        // Spawn initial creatures ONLY if save state was not loaded
        if save_state_path.is_none() || !save_state_path.as_ref().map(|p| Path::new(p).exists()).unwrap_or(false) {
            for _i in 0..initial_count {
                let x = (rand::random::<f32>() - 0.5) * 1000.0;  // ±500 units
                let y = (rand::random::<f32>() - 0.5) * 1000.0;  // ±500 units

                let builder = CritBuilder::new()
                    .at(x, y)
                    .with_all_capabilities()
                    .in_behavior(BehaviorMode::Wandering);

                simulation.spawn_crit(builder);
            }
        }

        Self {
            simulation,
            command_rx,
        }
    }

    /// Process commands from JavaScript (non-blocking)
    pub fn process_commands(&mut self) {
        while let Ok(cmd) = self.command_rx.try_recv() {
            match cmd {
                SimCommand::Spawn(count) => {
                    for _ in 0..count {
                        let x = (rand::random::<f32>() - 0.5) * 1000.0;  // ±500 units
                        let y = (rand::random::<f32>() - 0.5) * 1000.0;  // ±500 units

                        let builder = CritBuilder::new()
                            .at(x, y)
                            .with_all_capabilities()
                            .in_behavior(BehaviorMode::Wandering);

                        self.simulation.spawn_crit(builder);
                    }
                    // Flush world to ensure entities are added to archetypes immediately
                    // Without this, queries in export_positions() won't see newly spawned entities
                    self.simulation.world.flush();
                }
                SimCommand::SpawnAt { x, y } => {
                    self.simulation.spawn_crit_at(x, y);
                    self.simulation.world.flush();
                }
                SimCommand::KillAll => {
                    self.simulation.despawn_all();
                    // Flush to ensure entities are removed from archetypes immediately
                    self.simulation.world.flush();
                }
                SimCommand::LoadTrial { trial_name } => {
                    self.simulation.load_trial(&trial_name, |result| {
                        if result.success {
                            eprintln!("[NAPI] ✅ {}", result.message);
                        } else {
                            eprintln!("[NAPI] ❌ {}", result.message);
                        }
                    });
                    self.simulation.world.flush();
                }
                SimCommand::SelectCreatureDebug(creature_id) => {
                    #[cfg(feature = "dev-tools")]
                    {
                        use crate::simulation::perception::PerceptionDebugTarget;

                        let entity = creature_id.and_then(|id| {
                            self.lookup_entity_by_crit_id(id)
                        });

                        if let Some(mut target) = self.simulation.world.get_resource_mut::<PerceptionDebugTarget>() {
                            target.0 = entity;
                        }
                    }
                    #[cfg(not(feature = "dev-tools"))]
                    {
                        let _ = creature_id;
                    }
                }
            }
        }
    }

    /// Lookup entity by CritId (reserved for future use)
    #[allow(dead_code)]
    fn lookup_entity_by_crit_id(&mut self, crit_id: u32) -> Option<Entity> {
        self.simulation.world
            .query::<(Entity, &CritId)>()
            .iter(&self.simulation.world)
            .find(|(_, id)| id.0 == crit_id)
            .map(|(entity, _)| entity)
    }

    /// Update simulation one tick
    pub fn update(&mut self, delta_time: f32) {
        self.simulation.update(delta_time);
    }

    /// Export positions to DoubleBuffer (SoA layout)
    ///
    /// Layout: [ID₁, ID₂..., X₁, X₂..., Y₁, Y₂..., Rot₁, Rot₂...]
    ///
    /// Returns: The number of creatures actually exported to the buffer
    pub fn export_positions(&mut self, buffer: &Arc<Mutex<DoubleBuffer>>) -> usize {
        let world = &mut self.simulation.world;

        // Query all living creatures
        let mut query = world.query::<(&CritId, &Position, &Rotation)>();

        let creatures: Vec<_> = query.iter(world).collect();
        let creature_count = creatures.len();

        if creature_count == 0 {
            return 0;
        }

        // Lock buffer and get write slice
        let mut buffer_guard = buffer.lock();
        let buffer_size = buffer_guard.size();

        // Check if buffer capacity (in creatures, not f32s)
        let buffer_capacity = buffer_size / 4;

        // Only export what fits in the buffer
        let export_count = creature_count.min(buffer_capacity);

        let write_slice = buffer_guard.get_write_slice();

        // SoA offsets
        let id_offset = 0;
        let x_offset = export_count;
        let y_offset = export_count * 2;
        let rot_offset = export_count * 3;

        // Write data in SoA layout (only up to export_count)
        for (i, (id, pos, rot)) in creatures.iter().take(export_count).enumerate() {
            write_slice[id_offset + i] = id.0 as f32;
            write_slice[x_offset + i] = pos.x;
            write_slice[y_offset + i] = pos.y;
            write_slice[rot_offset + i] = rot.radians;
        }

        export_count
    }

    /// Export perception debug data to buffer (dev-tools only)
    ///
    /// Called every tick to provide smooth visualization updates.
    /// Buffer is swapped after this call.
    #[cfg(feature = "dev-tools")]
    pub fn export_perception_debug(&mut self, buffer: &Arc<Mutex<PerceptionDebugBuffer>>) {
        use crate::simulation::perception::{PerceptionDebugSnapshot, PerceptionDebugTarget};

        let world = &self.simulation.world;

        // Check if there's an active debug target
        let has_target = world
            .get_resource::<PerceptionDebugTarget>()
            .map(|t| t.get().is_some())
            .unwrap_or(false);

        let mut buffer_guard = buffer.lock();

        if !has_target {
            buffer_guard.clear_write();
            return;
        }

        // Get the snapshot data
        if let Some(snapshot) = world.get_resource::<PerceptionDebugSnapshot>() {
            if snapshot.entity_id > 0 {
                // Convert neighbors to tuple format
                let neighbors: Vec<(u32, f32, f32)> = snapshot.neighbors
                    .iter()
                    .map(|n| (n.id, n.x, n.y))
                    .collect();

                buffer_guard.write_debug_data(
                    snapshot.entity_id,
                    snapshot.x,
                    snapshot.y,
                    snapshot.perception_range,
                    snapshot.fov_angle,
                    snapshot.rotation,
                    &neighbors,
                );

                // Write cell data for grid visualization
                let grid = world.get_resource::<crate::simulation::spatial::SpatialGrid>();
                let cell_size = grid.map(|g| g.cell_size()).unwrap_or(10.0);
                let queried_cells: Vec<(i32, i32)> = snapshot.queried_cells
                    .iter()
                    .map(|c| (c.x, c.y))
                    .collect();
                let checked_cells: Vec<(i32, i32)> = snapshot.checked_cells
                    .iter()
                    .map(|c| (c.x, c.y))
                    .collect();
                buffer_guard.write_cell_data(
                    cell_size,
                    (snapshot.creature_cell.x, snapshot.creature_cell.y),
                    &queried_cells,
                    &checked_cells,
                );
            } else {
                buffer_guard.clear_write();
            }
        } else {
            buffer_guard.clear_write();
        }
    }

    /// Get telemetry snapshot
    pub fn get_telemetry(&mut self, tick: u64, tick_rate_hz: f32) -> TelemetrySnapshot {
        use crate::simulation::spatial::DoubleBufferedSpatialGrid;

        // Query creature count directly (no EntityIdMap dependency)
        let count = self.simulation.world
            .query::<&CritId>()
            .iter(&self.simulation.world)
            .count();
        let system_timings = self.simulation.get_system_timings();

        // Get actual spatial grid bounds
        let grid = self.simulation.world.resource::<DoubleBufferedSpatialGrid>();
        let read_grid = grid.read_grid();
        let cell_size = read_grid.cell_size();
        let (min_cell_x, min_cell_y) = read_grid.bounds();
        let (width, height) = read_grid.dimensions();

        // Convert cell bounds to world coordinates
        let grid_min_x = min_cell_x as f32 * cell_size;
        let grid_min_y = min_cell_y as f32 * cell_size;
        let grid_max_x = grid_min_x + (width as f32 * cell_size);
        let grid_max_y = grid_min_y + (height as f32 * cell_size);
        let grid_bounds = (grid_min_x, grid_max_x, grid_min_y, grid_max_y);

        #[cfg(feature = "dev-tools")]
        {
            let hardware_metrics = self.simulation.get_hardware_metrics();
            let parallelization_metrics = self.simulation.get_parallelization_metrics();

            TelemetrySnapshot::new(
                tick,
                count,
                tick_rate_hz,
                cell_size,
                grid_bounds,
                system_timings,
                hardware_metrics,
                parallelization_metrics,
            )
        }

        #[cfg(not(feature = "dev-tools"))]
        {
            TelemetrySnapshot::new(
                tick,
                count,
                tick_rate_hz,
                cell_size,
                grid_bounds,
                system_timings,
            )
        }
    }

    /// Record total tick timing (for NAPI run loop)
    #[cfg(feature = "dev-tools")]
    pub fn record_total_tick_timing(&mut self, elapsed_us: u64) {
        self.simulation.world()
            .resource::<crate::instrumentation::SystemTimings>()
            .total_tick_us
            .store(elapsed_us, std::sync::atomic::Ordering::Relaxed);
    }

    /// Record total tick timing (for NAPI run loop) - no-op without dev-tools
    #[cfg(not(feature = "dev-tools"))]
    pub fn record_total_tick_timing(&mut self, _elapsed_us: u64) {
        // No-op: SystemTimings resource doesn't exist without dev-tools feature
    }

    /// Read hardware counters and store snapshot (for NAPI run loop, dev-tools only)
    #[cfg(feature = "dev-tools")]
    pub fn read_hardware_counters(&mut self) {
        // Read hardware counters (they stay enabled continuously from initialization)
        let hw_snapshot = self.simulation.world_mut()
            .resource_mut::<crate::instrumentation::HardwareMetrics>()
            .read();

        self.simulation.world_mut()
            .resource_mut::<crate::instrumentation::HardwareSnapshotResource>()
            .0 = hw_snapshot;
    }

    /// Create save state from current simulation (for periodic/shutdown saves)
    pub fn to_save_state(&mut self) -> Result<crate::persistence::WorldSaveState, crate::persistence::SaveStateError> {
        self.simulation.to_save_state()
    }
}
