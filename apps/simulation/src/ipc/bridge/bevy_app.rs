//! Bevy App wrapper for NAPI integration
//!
//! This module bridges the existing Bevy simulation with the NAPI custom run loop.
//! It handles:
//! - Command processing from JavaScript (spawn, kill_all, load_trial)
//! - Position export to DoubleBuffer (SoA layout)
//! - Telemetry collection and reporting

use crate::simulation::core::{Simulation, SimulationBuilder, MAX_WORLD_SIZE};
use crate::simulation::core::components::{BodySize, Position, Rotation};
use crate::simulation::creatures::components::{BehaviorMode, CritId};
use crate::simulation::creatures::builder::CritBuilder;
use super::{DoubleBuffer, TelemetrySnapshot};
#[cfg(feature = "dev-tools")]
use super::PerceptionDebugBuffer;
use crate::ipc::SimCommand;
use bevy_ecs::prelude::Resource;
use crossbeam_channel::Receiver;
use parking_lot::Mutex;
use rayon::prelude::*;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU32};

#[derive(Resource, Debug, Clone)]
pub struct ViewportBounds {
    pub min_x: f32,
    pub min_y: f32,
    pub max_x: f32,
    pub max_y: f32,
    pub margin: f32,
    pub enabled: bool,
}

impl Default for ViewportBounds {
    fn default() -> Self {
        Self {
            min_x: -10000.0,
            min_y: -10000.0,
            max_x: 10000.0,
            max_y: 10000.0,
            margin: 50.0,
            enabled: false,
        }
    }
}

impl ViewportBounds {
    pub fn contains(&self, x: f32, y: f32) -> bool {
        if !self.enabled {
            return true;
        }
        x >= self.min_x - self.margin
            && x <= self.max_x + self.margin
            && y >= self.min_y - self.margin
            && y <= self.max_y + self.margin
    }
}

/// NAPI-specific Bevy app wrapper
pub struct NapiApp {
    simulation: Simulation,
    command_rx: Receiver<SimCommand>,
    paused: Option<Arc<AtomicBool>>,
    time_scale: Option<Arc<AtomicU32>>,
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

        simulation.world.insert_resource(ViewportBounds::default());

        Self {
            simulation,
            command_rx,
            paused: None,
            time_scale: None,
        }
    }

    pub fn set_paused_flag(&mut self, paused: Arc<AtomicBool>) {
        self.paused = Some(paused);
    }

    pub fn set_time_scale_flag(&mut self, time_scale: Arc<AtomicU32>) {
        self.time_scale = Some(time_scale);
    }

    #[cfg(feature = "test-helpers")]
    pub fn simulation(&self) -> &Simulation {
        &self.simulation
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
                SimCommand::SpawnAt { x, y, dna } => {
                    self.simulation.spawn_crit_at_with_dna(x, y, dna);
                    self.simulation.world.flush();
                }
                SimCommand::KillAll => {
                    self.simulation.despawn_all();
                    // Flush to ensure entities are removed from archetypes immediately
                    self.simulation.world.flush();
                }
                SimCommand::LoadTrial { trial_name, randomize_dna, dna } => {
                    self.simulation.load_trial(&trial_name, randomize_dna, dna, |result| {
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
                        use bevy_ecs::prelude::Entity;

                        let entity: Option<Entity> = creature_id.and_then(|id| {
                            self.simulation.world
                                .query::<(Entity, &CritId)>()
                                .iter(&self.simulation.world)
                                .find(|(_, crit_id)| crit_id.0 == id)
                                .map(|(entity, _)| entity)
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
                SimCommand::SetPaused(is_paused) => {
                    if let Some(paused_ref) = &self.paused {
                        paused_ref.store(is_paused, std::sync::atomic::Ordering::SeqCst);
                        eprintln!("[NAPI] Simulation {}", if is_paused { "PAUSED" } else { "RESUMED" });
                    }
                }
                SimCommand::SetTimeScale(scale) => {
                    if let Some(time_scale_ref) = &self.time_scale {
                        time_scale_ref.store(scale.to_bits(), std::sync::atomic::Ordering::SeqCst);
                        eprintln!("[NAPI] Time scale set to {}x", scale);
                    }
                }
                SimCommand::SetViewportBounds { min_x, min_y, max_x, max_y, margin } => {
                    if let Some(mut bounds) = self.simulation.world.get_resource_mut::<ViewportBounds>() {
                        bounds.min_x = min_x;
                        bounds.min_y = min_y;
                        bounds.max_x = max_x;
                        bounds.max_y = max_y;
                        bounds.margin = margin;
                        bounds.enabled = true;
                    }
                }
            }
        }
    }

    /// Update simulation one tick
    pub fn update(&mut self, delta_time: f32) {
        self.simulation.update(delta_time);
    }

    /// Export positions to DoubleBuffer (SoA layout)
    ///
    /// Layout: [ID₁, ID₂..., X₁, X₂..., Y₁, Y₂..., Rot₁, Rot₂...]
    ///
    /// Creatures are sorted by CritId to ensure stable ordering across ticks.
    /// This prevents ghost-crits caused by ECS query order instability during spawn/despawn.
    /// See: docs/testing/bugs/ghost-crits.md
    ///
    /// If viewport culling is enabled, only exports creatures within the viewport bounds + margin.
    /// Frontend handles ID-based interpolation, so creatures entering/leaving viewport work correctly.
    ///
    /// Returns: The number of creatures actually exported to the buffer
    pub fn export_positions(&mut self, buffer: &Arc<Mutex<DoubleBuffer>>) -> usize {
        let world = &mut self.simulation.world;

        // Get viewport bounds for culling (clone to avoid borrow issues)
        let viewport = world
            .get_resource::<ViewportBounds>()
            .cloned()
            .unwrap_or_default();

        // Lock buffer first to get capacity
        let mut buffer_guard = buffer.lock();
        let buffer_size = buffer_guard.size();
        let buffer_capacity = buffer_size / 5; // 5 f32s per creature: ID, X, Y, Rot, Size

        if buffer_capacity == 0 {
            return 0;
        }

        let write_slice = buffer_guard.get_write_slice();

        // Collect entities into Vec for sorting (required for stable index ordering)
        // ECS query order is unstable across spawn/despawn - sorting by CritId fixes ghost-crits
        let mut query = world.query::<(&CritId, &Position, &Rotation, &BodySize)>();
        let mut entities: Vec<_> = query.iter(world).collect();

        // Parallel sort by CritId for stable ordering
        // Benchmarked: 1.35ms at 400K creatures (3% of 45ms tick budget)
        entities.par_sort_unstable_by_key(|(id, _, _, _)| id.0);

        // Filter by viewport bounds (after sorting to maintain stable ordering)
        // Frontend uses ID-based interpolation, so creatures can enter/leave buffer freely
        let visible_entities: Vec<_> = entities
            .into_iter()
            .filter(|(_, pos, _, _)| viewport.contains(pos.x, pos.y))
            .collect();

        let export_count = visible_entities.len().min(buffer_capacity);

        if export_count == 0 {
            return 0;
        }

        // Write to SoA layout
        // Layout: [ID₁...IDₙ, X₁...Xₙ, Y₁...Yₙ, Rot₁...Rotₙ, Size₁...Sizeₙ]
        let x_offset = export_count;
        let y_offset = export_count * 2;
        let rot_offset = export_count * 3;
        let size_offset = export_count * 4;

        for (i, (id, pos, rot, size)) in visible_entities.iter().take(export_count).enumerate() {
            write_slice[i] = id.0 as f32;
            write_slice[x_offset + i] = pos.x;
            write_slice[y_offset + i] = pos.y;
            write_slice[rot_offset + i] = rot.radians;
            write_slice[size_offset + i] = size.length;
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
        let target_entity = world
            .get_resource::<PerceptionDebugTarget>()
            .and_then(|t| t.get());
        let has_target = target_entity.is_some();

        let mut buffer_guard = buffer.lock();

        if !has_target {
            buffer_guard.clear_write();
            return;
        }

        // Get the snapshot data
        if let Some(snapshot) = world.get_resource::<PerceptionDebugSnapshot>() {
            if snapshot.entity_id > 0 {
                // Pass iterators directly - no intermediate Vec allocations
                buffer_guard.write_debug_data(
                    snapshot.entity_id,
                    snapshot.x,
                    snapshot.y,
                    snapshot.perception_range,
                    snapshot.query_radius,
                    snapshot.fov_angle,
                    snapshot.rotation,
                    snapshot.ax,
                    snapshot.ay,
                    snapshot.neighbors.iter(),
                );

                // Write cell data for grid visualization
                let grid = world.get_resource::<crate::simulation::spatial::SpatialGrid>();
                let cell_size = grid.map(|g| g.cell_size()).unwrap_or(10.0);
                buffer_guard.write_cell_data(
                    cell_size,
                    (snapshot.creature_cell.x, snapshot.creature_cell.y),
                    snapshot.queried_cells.iter(),
                    snapshot.checked_cells.iter(),
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
        use crate::simulation::spatial::HierarchicalGrid;

        // Query creature count directly (no EntityIdMap dependency)
        let count = self.simulation.world
            .query::<&CritId>()
            .iter(&self.simulation.world)
            .count();
        let system_timings = self.simulation.get_system_timings();

        // Get actual spatial grid bounds (from L0 grid)
        let grid = self.simulation.world.resource::<HierarchicalGrid>();
        let read_grid = grid.l0.read_grid();
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

    /// Record export_positions timing (for NAPI run loop)
    #[cfg(feature = "dev-tools")]
    pub fn record_export_positions_timing(&mut self, elapsed_us: u64) {
        self.simulation.world()
            .resource::<crate::instrumentation::SystemTimings>()
            .export_positions_us
            .store(elapsed_us, std::sync::atomic::Ordering::Relaxed);
    }

    /// Record export_positions timing (for NAPI run loop) - no-op without dev-tools
    #[cfg(not(feature = "dev-tools"))]
    pub fn record_export_positions_timing(&mut self, _elapsed_us: u64) {
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
