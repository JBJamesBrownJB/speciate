//! Simulation orchestration and builder

use super::components::*;
use super::world_bounds::WorldBounds;
use crate::config::MovementConfig;
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
        ));


        // Initialize default resources
        world.insert_resource(DeltaTime::default());
        world.insert_resource(BoundaryConfig::default());
        world.insert_resource(WorldBounds::default());
        world.insert_resource(PhysicsTick::default());
        world.insert_resource(ActualTickRate::default());
        world.insert_resource(MovementConfig::default());

        // Initialize spawn event system resources
        world.init_resource::<Events<SpawnCreatureEvent>>();
        world.insert_resource(NextCreatureId::default());
        world.insert_resource(EntityIdMap::default());

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

    /// Update the measured tick rate resource
    ///
    /// This should be called by the main loop after measuring actual tick duration
    /// to provide accurate tick rate display in the frontend.
    pub fn set_tick_rate(&mut self, tick_rate: f32) {
        self.world.resource_mut::<ActualTickRate>().0 = tick_rate;
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
        let id = self.world.resource_mut::<NextCreatureId>().generate();

        // Spawn entity
        let entity = self.world.spawn(builder.build(id)).id();

        // Register in entity map
        self.world.resource_mut::<EntityIdMap>().insert(entity, id);

        id
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

        // Run all systems (spawn events are emitted and processed in this frame)
        // Systems run at the CURRENT tick (0-indexed)
        self.schedule.run(&mut self.world);

        // Update events AFTER systems run (prepare for next frame)
        self.world.resource_mut::<Events<SpawnCreatureEvent>>().update();

        // Increment physics tick for next frame (Perlin noise temporal variation)
        // This ensures first update runs at tick 0, second at tick 1, etc.
        self.world.resource_mut::<PhysicsTick>().increment();
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

        // Spawn a creature using CritBuilder
        let builder = CritBuilder::new().at(50.0, 50.0).with_all_capabilities();
        let entity_id = simulation.spawn_crit(builder);
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

        let id1 = simulation.spawn_crit(CritBuilder::new().at(10.0, 10.0).with_all_capabilities());
        let id2 = simulation.spawn_crit(CritBuilder::new().at(20.0, 20.0).with_all_capabilities());
        let id3 = simulation.spawn_crit(CritBuilder::new().at(30.0, 30.0).with_all_capabilities());

        assert_ne!(id1, id2);
        assert_ne!(id2, id3);
        assert_ne!(id1, id3);
        assert_eq!(simulation.creature_count(), 3);
    }
}
