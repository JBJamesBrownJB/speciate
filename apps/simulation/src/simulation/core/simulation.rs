use super::components::{ActualTickRate, BoundaryConfig, DeltaTime, PhysicsTick};
use super::world_bounds::WorldBounds;
use crate::config::MovementConfig;
use crate::simulation::creatures::behaviors::behavior_transition_system;
use crate::simulation::creatures::steering::update_steering_system;
use crate::simulation::creatures::builder::CritBuilder;
use crate::simulation::creatures::dna::Dna;
use crate::simulation::creatures::events::SpawnCreatureEvent;
use crate::simulation::creatures::systems::{process_spawn_events, NextCreatureId};
use crate::simulation::movement::{integrate_motion_system, update_body_size_cache};
use crate::simulation::perception;
use crate::simulation::spatial::{
    aggregate_l1_system, rebuild_spatial_grid_system, swap_spatial_grid_buffers_system, HierarchicalGrid,
};
use bevy_ecs::prelude::*;

/// Result of loading a trial
pub struct LoadTrialResult {
    pub success: bool,
    pub message: String,
    pub command_type: String,
}

pub struct SimulationBuilder {
    world: World,
    schedule: Schedule,
}

impl SimulationBuilder {
    pub fn new() -> Self {
        let mut world = World::new();
        let mut schedule = Schedule::default();

        use crate::simulation::core::components::{Acceleration, BodySize, Position, Rotation, Velocity};
        use crate::simulation::creatures::components::{
            BehaviorMode, Brain, BrainMode, CanAvoidObstacles, CanFlee, CanSeek, CanWander,
            CreatureState, CritId, FleeState, HomePosition, Target, WanderState,
        };
        use crate::simulation::perception::AvoidanceBehavior;
        use bevy_ecs::prelude::AppTypeRegistry;

        world.init_resource::<AppTypeRegistry>();
        {
            let registry = world.resource::<AppTypeRegistry>();
            let mut type_registry = registry.write();

            type_registry.register::<Position>();
            type_registry.register::<Velocity>();
            type_registry.register::<Acceleration>();
            type_registry.register::<BodySize>();
            type_registry.register::<Rotation>();

            type_registry.register::<CritId>();
            type_registry.register::<CreatureState>();
            type_registry.register::<BehaviorMode>();
            type_registry.register::<Brain>();
            type_registry.register::<BrainMode>();
            type_registry.register::<HomePosition>();

            type_registry.register::<CanSeek>();
            type_registry.register::<CanFlee>();
            type_registry.register::<CanWander>();
            type_registry.register::<CanAvoidObstacles>();

            type_registry.register::<AvoidanceBehavior>();
            type_registry.register::<Target>();

            type_registry.register::<WanderState>();
            type_registry.register::<FleeState>();
        }

        schedule.add_systems(process_spawn_events);

        schedule.add_systems((
            rebuild_spatial_grid_system,
            // L1 aggregation runs after L0 rebuild
            aggregate_l1_system.after(rebuild_spatial_grid_system),
            // Perception MUST run after L1 aggregation for early-exit optimization
            perception::update_perception_system.after(aggregate_l1_system),
            // Behavior transition runs after perception (may use perception data in future)
            behavior_transition_system.after(perception::update_perception_system),
            // FUSED STEERING: Single system replaces 4 separate systems (wander, seek, avoidance, flee)
            // Sprint 20 optimization: 1 query + 1 iteration instead of 4
            // Also includes steering cap (was separate system, now fused for performance)
            update_steering_system.after(behavior_transition_system),
            update_body_size_cache,
            // integrate_motion MUST run AFTER steering (which now includes capping)
            // Rotation is now fused into integrate_motion_system for parallelization
            integrate_motion_system.after(update_steering_system),
            // Swap grid buffers at END of tick - next tick sees newly rebuilt grid
            swap_spatial_grid_buffers_system.after(integrate_motion_system),
        ));

        // Debug acceleration capture runs AFTER steering (which includes capping) but BEFORE movement integration
        // This ensures force visualization shows CAPPED acceleration values
        #[cfg(feature = "dev-tools")]
        schedule.add_systems(
            perception::capture_debug_acceleration_system
                .after(update_steering_system)
                .before(integrate_motion_system),
        );

        world.insert_resource(DeltaTime::default());
        world.insert_resource(BoundaryConfig::default());
        world.insert_resource(WorldBounds::default());
        world.insert_resource(PhysicsTick::default());
        world.insert_resource(ActualTickRate::default());
        world.insert_resource(MovementConfig::default());
        world.insert_resource(crate::simulation::movement::noise::NoiseTable::default());

        #[cfg(feature = "dev-tools")]
        super::dev_tools::register_dev_resources(&mut world);

        world.init_resource::<Events<SpawnCreatureEvent>>();
        world.insert_resource(NextCreatureId::default());
        world.insert_resource(HierarchicalGrid::new());

        Self { world, schedule }
    }

    pub fn set_boundaries(mut self, extent_x: f32, extent_y: f32) -> Self {
        self.world.insert_resource(BoundaryConfig {
            min_x: -extent_x,
            max_x: extent_x,
            min_y: -extent_y,
            max_y: extent_y,
            margin: (extent_x / 100.0).max(1000.0),
            max_force: 1.0,
        });
        self.world
            .insert_resource(WorldBounds::new(-extent_x, extent_x, -extent_y, extent_y));

        // Set spatial grid bounds for fixed-bounds optimization
        if let Some(mut grid) = self.world.get_resource_mut::<HierarchicalGrid>() {
            grid.set_world_bounds(-extent_x, extent_x, -extent_y, extent_y);
        }

        self
    }

    pub fn build(self) -> Simulation {
        Simulation {
            world: self.world,
            schedule: self.schedule,
            assets_path: None,
        }
    }
}

impl Default for SimulationBuilder {
    fn default() -> Self {
        Self::new()
    }
}

pub struct Simulation {
    pub(crate) world: World,
    schedule: Schedule,
    assets_path: Option<std::path::PathBuf>,
}

impl Simulation {
    pub fn set_boundaries(&mut self, extent_x: f32, extent_y: f32) {
        self.world.insert_resource(BoundaryConfig {
            min_x: -extent_x,
            max_x: extent_x,
            min_y: -extent_y,
            max_y: extent_y,
            margin: (extent_x / 100.0).max(1000.0),
            max_force: 1.0,
        });
        self.world
            .insert_resource(WorldBounds::new(-extent_x, extent_x, -extent_y, extent_y));

        // Set spatial grid bounds for fixed-bounds optimization
        if let Some(mut grid) = self.world.get_resource_mut::<HierarchicalGrid>() {
            grid.set_world_bounds(-extent_x, extent_x, -extent_y, extent_y);
        }
    }

    pub fn get_boundaries(&self) -> (f32, f32, f32, f32) {
        let config = self.world.resource::<BoundaryConfig>();
        (config.min_x, config.max_x, config.min_y, config.max_y)
    }

    pub fn set_tick_rate(&mut self, tick_rate: f32) {
        self.world.resource_mut::<ActualTickRate>().0 = tick_rate;
    }

    pub fn spawn_crit(&mut self, builder: CritBuilder) -> u32 {
        let id = self.world.resource_mut::<NextCreatureId>().generate();

        let _entity = self.world.spawn(builder.build(id)).id();

        id
    }

    #[cfg(any(test, feature = "test-helpers"))]
    pub fn spawn_test_crit(&mut self, x: f32, y: f32) -> u32 {
        let builder = CritBuilder::new().at(x, y).with_all_capabilities();
        self.spawn_crit(builder)
    }

    pub fn spawn_seeker(&mut self, x: f32, y: f32, target_x: f32, target_y: f32) -> u32 {
        let builder = CritBuilder::new().at(x, y).as_seeker(target_x, target_y);
        self.spawn_crit(builder)
    }

    pub fn update(&mut self, delta_time: f32) {
        self.world.insert_resource(DeltaTime(delta_time));

        self.schedule.run(&mut self.world);

        self.world
            .resource_mut::<Events<SpawnCreatureEvent>>()
            .update();

        self.world.resource_mut::<PhysicsTick>().increment();
    }

    /// Count creatures in the simulation.
    ///
    /// # Safety Note
    /// Uses `UnsafeWorldCell` for read-only query access from `&self`.
    /// This is safe because:
    /// - We only read data (no mutations)
    /// - We don't store the query or any references beyond this call
    /// - `as_unsafe_world_cell_readonly()` is Bevy's API for exactly this pattern
    pub fn creature_count(&self) -> usize {
        use crate::simulation::creatures::components::CritId;

        // SAFETY: Read-only query on immutable reference. No aliasing occurs because
        // we consume the QueryState immediately and don't hold any references to world data.
        unsafe {
            let world_cell = self.world.as_unsafe_world_cell_readonly();
            let mut query = world_cell.world_mut().query::<&CritId>();
            query.iter(world_cell.world()).count()
        }
    }

    pub fn despawn_all(&mut self) {
        use crate::simulation::creatures::components::CritId;

        // Clear debug target before despawning to prevent use-after-free
        #[cfg(feature = "dev-tools")]
        {
            use crate::simulation::perception::PerceptionDebugTarget;
            if let Some(mut debug_target) = self.world.get_resource_mut::<PerceptionDebugTarget>() {
                debug_target.clear();
            }
        }

        let entities: Vec<Entity> = self.world
            .query::<(Entity, &CritId)>()
            .iter(&self.world)
            .map(|(entity, _)| entity)
            .collect();

        for entity in entities {
            self.world.despawn(entity);
        }
    }

    pub fn set_assets_path(&mut self, path: &str) {
        self.assets_path = Some(std::path::PathBuf::from(path));
    }

    pub fn spawn_crit_at(&mut self, x: f32, y: f32) -> u32 {
        self.spawn_crit_at_with_dna(x, y, None)
    }

    pub fn spawn_crit_at_with_dna(&mut self, x: f32, y: f32, dna: Option<crate::simulation::creatures::dna::Dna>) -> u32 {
        use crate::simulation::creatures::builder::CritBuilder;
        use crate::simulation::creatures::components::state::BehaviorMode;

        let mut builder = CritBuilder::new()
            .at(x, y)
            .with_all_capabilities()
            .in_behavior(BehaviorMode::Wandering);

        if let Some(dna) = dna {
            builder = builder.with_dna(dna);
        }

        self.spawn_crit(builder)
    }

    #[cfg_attr(not(feature = "dev-tools"), allow(unused_variables))]
    pub fn load_trial<F>(&mut self, trial_name: &str, randomize_dna: bool, dna: Option<Dna>, callback: F)
    where
        F: FnOnce(LoadTrialResult) + 'static,
    {
        let Some(ref _assets_path) = self.assets_path else {
            callback(LoadTrialResult {
                success: false,
                message: "Assets path not set. Call set_assets_path() first.".to_string(),
                command_type: "LoadTrial".to_string(),
            });
            return;
        };

        #[cfg(feature = "dev-tools")]
        {
            use crate::trials;
            match trials::loader::load_trial(&mut self.world, trial_name, randomize_dna, dna) {
                Ok(config) => {
                    let dna_msg = if randomize_dna { " with randomized DNA" } else { "" };
                    callback(LoadTrialResult {
                        success: true,
                        message: format!("Loaded trial '{}' ({} spawn patterns){}", config.name, config.spawns.len(), dna_msg),
                        command_type: "LoadTrial".to_string(),
                    });
                }
                Err(e) => {
                    callback(LoadTrialResult {
                        success: false,
                        message: format!("Failed to load trial '{}': {}", trial_name, e),
                        command_type: "LoadTrial".to_string(),
                    });
                }
            }
        }

        #[cfg(not(feature = "dev-tools"))]
        {
            let _ = randomize_dna;
            let _ = dna;
            callback(LoadTrialResult {
                success: false,
                message: "Trial loading requires dev-tools feature".to_string(),
                command_type: "LoadTrial".to_string(),
            });
        }
    }

    pub fn get_system_timings(&self) -> crate::instrumentation::SystemTimingsSnapshot {
        #[cfg(feature = "dev-tools")]
        {
            self.world.resource::<crate::instrumentation::SystemTimings>().snapshot()
        }

        #[cfg(not(feature = "dev-tools"))]
        {
            crate::instrumentation::SystemTimingsSnapshot::default()
        }
    }

    #[cfg(feature = "dev-tools")]
    pub fn get_hardware_metrics(&self) -> crate::instrumentation::HardwareSnapshot {
        self.world
            .resource::<crate::instrumentation::HardwareSnapshotResource>()
            .0
            .clone()
            .unwrap_or_default()
    }

    #[cfg(feature = "dev-tools")]
    pub fn get_parallelization_metrics(&mut self) -> crate::instrumentation::ParallelizationSnapshot {
        self.world.resource_mut::<crate::instrumentation::ParallelizationMetrics>().read()
    }

    pub fn world(&self) -> &World {
        &self.world
    }

    pub fn world_mut(&mut self) -> &mut World {
        &mut self.world
    }
}

impl Default for Simulation {
    fn default() -> Self {
        SimulationBuilder::new().build()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_builder_creates_empty_simulation() {
        let simulation = SimulationBuilder::new().build();
        assert_eq!(simulation.creature_count(), 0);
    }

    #[test]
    fn test_builder_with_boundaries() {
        let simulation = SimulationBuilder::new()
            .set_boundaries(200.0, 150.0)
            .build();

        let (min_x, max_x, min_y, max_y) = simulation.get_boundaries();
        assert_eq!(min_x, -200.0);
        assert_eq!(max_x, 200.0);
        assert_eq!(min_y, -150.0);
        assert_eq!(max_y, 150.0);
    }

    #[test]
    fn test_builder_default_boundaries() {
        use crate::simulation::core::MAX_WORLD_SIZE;
        let simulation = SimulationBuilder::new().build();
        let (min_x, max_x, min_y, max_y) = simulation.get_boundaries();
        assert_eq!(min_x, -MAX_WORLD_SIZE);
        assert_eq!(max_x, MAX_WORLD_SIZE);
        assert_eq!(min_y, -MAX_WORLD_SIZE);
        assert_eq!(max_y, MAX_WORLD_SIZE);
    }

    #[test]
    fn test_builder_systems_functional() {
        let mut simulation = SimulationBuilder::new()
            .set_boundaries(100.0, 100.0)
            .build();

        let builder = CritBuilder::new().at(50.0, 50.0).with_all_capabilities();
        let entity_id = simulation.spawn_crit(builder);
        assert!(entity_id > 0);
        assert_eq!(simulation.creature_count(), 1);

        simulation.update(0.016);
        assert_eq!(simulation.creature_count(), 1);
    }

    #[test]
    fn test_builder_multiple_spawns_unique_ids() {
        let mut simulation = SimulationBuilder::new()
            .set_boundaries(100.0, 100.0)
            .build();

        let id1 = simulation.spawn_crit(CritBuilder::new().at(10.0, 10.0).with_all_capabilities());
        let id2 = simulation.spawn_crit(CritBuilder::new().at(20.0, 20.0).with_all_capabilities());
        let id3 = simulation.spawn_crit(CritBuilder::new().at(30.0, 30.0).with_all_capabilities());

        assert_ne!(id1, id2);
        assert_ne!(id2, id3);
        assert_ne!(id1, id3);
        assert_eq!(simulation.creature_count(), 3);
    }

    // Trial loading tests
    #[test]
    #[cfg(feature = "dev-tools")]
    fn test_trial_opposing_seekers_loads() {
        use crate::simulation::creatures::components::CritId;
        use std::time::Duration;

        let mut sim = SimulationBuilder::new().build();
        sim.set_assets_path(".");

        let (tx, rx) = std::sync::mpsc::channel();
        sim.load_trial("opposing-seekers-1", false, None, move |result| {
            tx.send(result).unwrap();
        });

        let result = rx.recv_timeout(Duration::from_secs(1))
            .expect("Trial load should complete within 1 second");

        assert!(result.success, "Trial load should succeed: {}", result.message);

        let count = sim.world_mut().query::<&CritId>()
            .iter(sim.world_mut())
            .count();
        assert_eq!(count, 3, "Should spawn 3 creatures from trial");
    }

    #[test]
    #[cfg(feature = "dev-tools")]
    fn test_trial_additive_spawning() {
        use crate::simulation::creatures::components::CritId;
        use std::time::Duration;

        let mut sim = SimulationBuilder::new().build();
        sim.set_assets_path(".");

        for _ in 0..10 {
            sim.spawn_test_crit(0.0, 0.0);
        }

        let count_before = sim.world_mut().query::<&CritId>()
            .iter(sim.world_mut())
            .count();
        assert_eq!(count_before, 10, "Should have 10 runtime-spawned creatures");

        let (tx, rx) = std::sync::mpsc::channel();
        sim.load_trial("opposing-seekers-1", false, None, move |result| {
            tx.send(result).unwrap();
        });

        let result = rx.recv_timeout(Duration::from_secs(1))
            .expect("Trial load should complete within 1 second");
        assert!(result.success, "Trial load should succeed: {}", result.message);

        let count_after = sim.world_mut().query::<&CritId>()
            .iter(sim.world_mut())
            .count();
        assert_eq!(count_after, 13, "Should have 13 total creatures (10 + 3)");
    }

    // Despawn all tests
    #[test]
    fn test_despawn_all_removes_runtime_spawned() {
        use crate::simulation::creatures::components::CritId;

        let mut sim = SimulationBuilder::new().build();

        for i in 0..100 {
            let x = (i as f32 % 10.0) * 10.0;
            let y = (i as f32 / 10.0) * 10.0;
            sim.spawn_test_crit(x, y);
        }

        let count_before = sim.world_mut().query::<&CritId>()
            .iter(sim.world_mut())
            .count();
        assert_eq!(count_before, 100, "Should have 100 spawned creatures");

        sim.despawn_all();

        let count_after = sim.world_mut().query::<&CritId>()
            .iter(sim.world_mut())
            .count();
        assert_eq!(count_after, 0, "Should have 0 creatures after clear all");
    }

    #[test]
    fn test_despawn_all_removes_all_entities() {
        use crate::simulation::creatures::components::CritId;

        let mut sim = SimulationBuilder::new().build();

        for _ in 0..20 {
            sim.spawn_test_crit(0.0, 0.0);
        }

        let count_before = sim.world_mut().query::<&CritId>()
            .iter(sim.world_mut())
            .count();
        assert_eq!(count_before, 20, "Should have 20 creatures");

        sim.despawn_all();

        let count_after = sim.world_mut().query::<&CritId>()
            .iter(sim.world_mut())
            .count();
        assert_eq!(count_after, 0, "Should have 0 creatures after despawn_all");
    }

    #[test]
    fn test_multiple_despawns_idempotent() {
        use crate::simulation::creatures::components::CritId;

        let mut sim = SimulationBuilder::new().build();

        for _ in 0..20 {
            sim.spawn_test_crit(0.0, 0.0);
        }

        sim.despawn_all();
        sim.despawn_all();
        sim.despawn_all();

        let count = sim.world_mut().query::<&CritId>()
            .iter(sim.world_mut())
            .count();
        assert_eq!(count, 0, "Should still be 0 after multiple clears");
    }
}
