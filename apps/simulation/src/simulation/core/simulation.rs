//! Simulation orchestration and builder

use super::components::*;
use super::world_bounds::WorldBounds;
use crate::nats::{self, NatsPublisher, SimulationTick};
use crate::simulation::creatures::behaviors::{
    self, behavior_transition_system, flee_system, seek_system,
    territory_wandering_system,
};
use crate::simulation::creatures::builder::CritBuilder;
use crate::simulation::creatures::events::SpawnCreatureEvent;
use crate::simulation::creatures::systems::{process_spawn_events, EntityIdMap, NextCreatureId};
use crate::simulation::movement::{
    integrate_motion_system, rotation_system,
};
use crate::simulation::perception;
use bevy_ecs::prelude::*;

#[cfg(feature = "dev-commands")]
use crate::dev_commands::{DevCommandListener, DevSpawnIdCounter, process_dev_commands_system};

/// Builder for creating a Simulation with proper system initialization
///
/// This builder ensures all systems are registered BEFORE any entities are spawned,
/// preventing Bevy ECS archetype caching issues. Once built, the Simulation cannot
/// have new systems dynamically added, enforcing correct initialization order.
pub struct SimulationBuilder {
    world: World,
    schedule: Schedule,
}

impl SimulationBuilder {
    /// Create a new simulation builder with all systems registered
    pub fn new() -> Self {
        let mut world = World::new();
        let mut schedule = Schedule::default();

        // Register ALL systems here - this is the only place systems should be added
        // Minimal constraints: Let Bevy parallelize automatically based on data access patterns

        // Spawn event processing (MUST run FIRST to process spawn requests)
        schedule.add_systems(process_spawn_events);

        // Dev commands system (feature-gated, runs after spawn events)
        #[cfg(feature = "dev-commands")]
        schedule.add_systems(process_dev_commands_system.after(process_spawn_events));

        schedule.add_systems((
            // Perception systems - MUST run before behaviors
            perception::update_perception_system,
            // Behavior systems - Bevy will parallelize these automatically
            // Note: These all run AFTER perception and BEFORE physics integration
            behavior_transition_system,
            territory_wandering_system, // NEW Sprint 6: Territory-based wandering (elastic tether)
            flee_system,
            seek_system, // NEW Sprint 6: Seeking behavior
            behaviors::avoidance_system, // NEW Sprint 6: Obstacle avoidance
            // Movement integration (Euler integration: accel → vel → pos)
            integrate_motion_system,
            // Rotation can run whenever (one-frame delay is acceptable)
            rotation_system,
            // NATS publishing systems
            nats::systems::publish_frame_system,
            // Tick must increment after frame is published
            nats::systems::increment_tick_system.after(nats::systems::publish_frame_system),
        ));

        // Initialize default resources
        world.insert_resource(DeltaTime::default());
        world.insert_resource(BoundaryConfig::default());
        world.insert_resource(WorldBounds::default());
        world.insert_resource(PhysicsTick::default());

        // Initialize spawn event system resources
        world.init_resource::<Events<SpawnCreatureEvent>>();
        world.insert_resource(NextCreatureId::default());
        world.insert_resource(EntityIdMap::default());

        // Initialize NATS publisher
        let nats_url = std::env::var("NATS_URL").unwrap_or_else(|_| "nats://nats:4222".to_string());
        let (publisher, _nats_handle) = NatsPublisher::new(nats_url.clone(), 4);
        world.insert_resource(publisher);
        world.insert_resource(SimulationTick::default());

        // Initialize dev command listener (feature-gated)
        #[cfg(feature = "dev-commands")]
        {
            let (listener, _listener_handle) = DevCommandListener::new(nats_url, 16);
            world.insert_resource(listener);
            world.insert_resource(DevSpawnIdCounter::default());
            log::info!("[DEV] Dev commands enabled - listening on dev.sim.*");
        }

        Self { world, schedule }
    }

    /// Sets the boundary configuration for the simulation world (centered coordinate system)
    ///
    /// # Arguments
    /// * `extent_x` - Half-width of world (world spans from -extent_x to +extent_x)
    /// * `extent_y` - Half-height of world (world spans from -extent_y to +extent_y)
    pub fn set_boundaries(mut self, extent_x: f32, extent_y: f32) -> Self {
        self.world.insert_resource(BoundaryConfig {
            min_x: -extent_x,
            max_x: extent_x,
            min_y: -extent_y,
            max_y: extent_y,
            margin: (extent_x / 100.0).max(1000.0), // 1% of world size, min 1km
            max_force: 1.0,
        });
        self.world.insert_resource(WorldBounds::new(-extent_x, extent_x, -extent_y, extent_y));
        self
    }

    /// Builds the simulation, consuming the builder
    ///
    /// After this point, no new systems can be added. This prevents the archetype
    /// caching issue where late-added systems can't see entities spawned before
    /// the systems were registered.
    pub fn build(self) -> Simulation {
        Simulation {
            world: self.world,
            schedule: self.schedule,
        }
    }
}

impl Default for SimulationBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Running simulation that cannot have systems dynamically added
///
/// This struct is returned by SimulationBuilder::build() and represents
/// a locked-down simulation that can only update and spawn entities.
///
/// Creature spawning state (next_id, entity_id_map) is now managed as ECS resources
/// within the World, allowing systems to access them directly.
pub struct Simulation {
    pub(crate) world: World,
    schedule: Schedule,
}

impl Simulation {
    /// Sets the boundary configuration (centered coordinate system)
    ///
    /// # Arguments
    /// * `extent_x` - Half-width of world (world spans from -extent_x to +extent_x)
    /// * `extent_y` - Half-height of world (world spans from -extent_y to +extent_y)
    pub fn set_boundaries(&mut self, extent_x: f32, extent_y: f32) {
        self.world.insert_resource(BoundaryConfig {
            min_x: -extent_x,
            max_x: extent_x,
            min_y: -extent_y,
            max_y: extent_y,
            margin: (extent_x / 100.0).max(1000.0), // 1% of world size, min 1km
            max_force: 1.0,
        });
        self.world.insert_resource(WorldBounds::new(-extent_x, extent_x, -extent_y, extent_y));
    }

    /// Gets the current boundary configuration (centered coordinate system)
    /// Returns (min_x, max_x, min_y, max_y)
    pub fn get_boundaries(&self) -> (f32, f32, f32, f32) {
        let config = self.world.resource::<BoundaryConfig>();
        (config.min_x, config.max_x, config.min_y, config.max_y)
    }

    /// Spawns a new crit using the builder pattern
    ///
    /// This is the preferred way to spawn crits. Use `CritBuilder` to configure
    /// the crit's capabilities, behavior, and initial state.
    ///
    /// # Example
    /// ```no_run
    /// # use speciate::{CritBuilder, SimulationBuilder};
    /// # let mut sim = SimulationBuilder::new().build();
    /// let builder = CritBuilder::new()
    ///     .at(0.0, 0.0)
    ///     .as_seeker(100.0, 0.0);
    /// let id = sim.spawn_crit(builder);
    /// ```
    pub fn spawn_crit(&mut self, builder: CritBuilder) -> u32 {
        // Get next ID from resource
        let id = self.world.resource_mut::<NextCreatureId>().next();

        // Spawn entity
        let entity = self.world.spawn(builder.build(id)).id();

        // Register in entity map
        self.world.resource_mut::<EntityIdMap>().insert(entity, id);

        id
    }

    /// Spawns a new creature entity (deprecated)
    ///
    /// # Deprecated
    /// Use `spawn_crit` with `CritBuilder` instead. This method is kept for
    /// backward compatibility during the transition but will be removed in future versions.
    ///
    /// # Example Migration
    /// ```no_run
    /// # use speciate::{CritBuilder, SimulationBuilder};
    /// # let mut sim = SimulationBuilder::new().build();
    /// // OLD: sim.spawn_creature(10.0, 20.0, 0.0, 0.0);
    /// // NEW:
    /// let builder = CritBuilder::new().at(10.0, 20.0).with_all_capabilities();
    /// sim.spawn_crit(builder);
    /// ```
    #[deprecated(
        since = "0.1.0",
        note = "Use spawn_crit with CritBuilder instead for better configurability"
    )]
    pub fn spawn_creature(&mut self, x: f32, y: f32, _width: f32, _height: f32) -> u32 {
        let builder = CritBuilder::new().at(x, y).with_all_capabilities();
        self.spawn_crit(builder)
    }

    /// Quick spawn for testing - crit with all capabilities, catatonic at position
    ///
    /// This is a convenience method for tests and quick prototyping.
    /// For production code, use `spawn_crit` with a fully configured builder.
    #[cfg(test)]
    pub fn spawn_test_crit(&mut self, x: f32, y: f32) -> u32 {
        let builder = CritBuilder::new().at(x, y).with_all_capabilities();
        self.spawn_crit(builder)
    }

    /// Spawn a seeker aimed at a target
    ///
    /// Convenience method that spawns a crit with seeking capability enabled
    /// and behavior set to Seeking, with a target position.
    ///
    /// # Example
    /// ```no_run
    /// # use speciate::SimulationBuilder;
    /// # let mut sim = SimulationBuilder::new().build();
    /// // Spawn a seeker at (0, 0) targeting (100, 0)
    /// let id = sim.spawn_seeker(0.0, 0.0, 100.0, 0.0);
    /// ```
    pub fn spawn_seeker(&mut self, x: f32, y: f32, target_x: f32, target_y: f32) -> u32 {
        let builder = CritBuilder::new().at(x, y).as_seeker(target_x, target_y);
        self.spawn_crit(builder)
    }

    /// Updates the simulation by one step
    pub fn update(&mut self, delta_time: f32) {
        self.world.insert_resource(DeltaTime(delta_time));

        // Increment physics tick for temporal variation (Perlin noise)
        self.world.resource_mut::<PhysicsTick>().increment();

        // Run all systems (spawn events are emitted and processed in this frame)
        self.schedule.run(&mut self.world);

        // Update events AFTER systems run (prepare for next frame)
        self.world.resource_mut::<Events<SpawnCreatureEvent>>().update();
    }

    /// Returns the number of active creatures
    pub fn creature_count(&self) -> usize {
        self.world.resource::<EntityIdMap>().len()
    }

    /// Get immutable access to the ECS world (test helper)
    #[cfg(any(test, feature = "test-helpers"))]
    pub fn world(&self) -> &World {
        &self.world
    }

    /// Get mutable access to the ECS world (test helper)
    #[cfg(any(test, feature = "test-helpers"))]
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

    /// Builder Pattern Tests
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
        let simulation = SimulationBuilder::new().build();
        let (min_x, max_x, min_y, max_y) = simulation.get_boundaries();
        // Default from BoundaryConfig: ±1,000,000m world
        assert_eq!(min_x, -1_000_000.0);
        assert_eq!(max_x, 1_000_000.0);
        assert_eq!(min_y, -1_000_000.0);
        assert_eq!(max_y, 1_000_000.0);
    }

    #[test]
    fn test_builder_systems_functional() {
        let mut simulation = SimulationBuilder::new()
            .set_boundaries(100.0, 100.0)
            .build();

        // Spawn a creature
        let entity_id = simulation.spawn_creature(50.0, 50.0, 0.0, 0.0);
        assert!(entity_id > 0);
        assert_eq!(simulation.creature_count(), 1);

        // Update should not crash
        simulation.update(0.016);
        assert_eq!(simulation.creature_count(), 1);
    }

    #[test]
    fn test_builder_multiple_spawns_unique_ids() {
        let mut simulation = SimulationBuilder::new()
            .set_boundaries(100.0, 100.0)
            .build();

        let id1 = simulation.spawn_creature(10.0, 10.0, 0.0, 0.0);
        let id2 = simulation.spawn_creature(20.0, 20.0, 0.0, 0.0);
        let id3 = simulation.spawn_creature(30.0, 30.0, 0.0, 0.0);

        assert_ne!(id1, id2);
        assert_ne!(id2, id3);
        assert_ne!(id1, id3);
        assert_eq!(simulation.creature_count(), 3);
    }
}
