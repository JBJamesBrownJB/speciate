//! Simulation orchestration and builder

use super::agent_systems::*;
use super::components::*;
use crate::nats::{self, NatsPublisher, SimulationTick};
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
        schedule.add_systems((
            // Behavior systems - Bevy will parallelize these automatically
            behavior_transition_system,
            wander_system,
            flee_system,
            boundary_seek_system,
            // Physics integration
            update_physics_system,
            // CRITICAL: Boundary enforcement must run after physics to prevent escape
            boundary_enforcement_system.after(update_physics_system),
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

        // Initialize NATS publisher
        let nats_url = std::env::var("NATS_URL").unwrap_or_else(|_| "nats://nats:4222".to_string());
        let (publisher, _nats_handle) = NatsPublisher::new(nats_url, 4);
        world.insert_resource(publisher);
        world.insert_resource(SimulationTick::default());

        Self { world, schedule }
    }

    /// Sets the boundary configuration for the simulation world
    pub fn set_boundaries(mut self, width: f32, height: f32) -> Self {
        self.world.insert_resource(BoundaryConfig {
            width,
            height,
            margin: 20.0,
            max_force: 1.0,
        });
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
            next_id: 1,
            entity_id_map: std::collections::HashMap::new(),
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
pub struct Simulation {
    pub(crate) world: World,
    schedule: Schedule,
    pub(crate) next_id: u32,
    pub(crate) entity_id_map: std::collections::HashMap<bevy_ecs::entity::Entity, u32>,
}

impl Simulation {
    /// Sets the boundary configuration
    pub fn set_boundaries(&mut self, width: f32, height: f32) {
        self.world.insert_resource(BoundaryConfig {
            width,
            height,
            margin: 20.0,
            max_force: 1.0,
        });
    }

    /// Gets the current boundary configuration
    pub fn get_boundaries(&self) -> (f32, f32) {
        let config = self.world.resource::<BoundaryConfig>();
        (config.width, config.height)
    }

    /// Spawns a new creature entity
    pub fn spawn_creature(&mut self, x: f32, y: f32, _width: f32, _height: f32) -> u32 {
        use rand::Rng;
        let mut rng = rand::thread_rng();

        let angle = rng.gen_range(0.0..std::f32::consts::TAU);
        let speed = rng.gen_range(30.0..60.0);

        let id = self.next_id;
        self.next_id += 1;

        let entity = self
            .world
            .spawn((
                AgentId(id), // Attach stable ID as component
                Position { x, y },
                Velocity {
                    vx: angle.cos() * speed,
                    vy: angle.sin() * speed,
                },
                Acceleration { ax: 0.0, ay: 0.0 },
                Rotation { radians: angle },
                CreatureState::new(),
                WanderState {
                    wander_angle: rng.gen_range(0.0..std::f32::consts::TAU),
                    wander_radius: 25.0,
                    wander_distance: 50.0,
                    angle_change: 0.15,
                },
            ))
            .id();

        self.entity_id_map.insert(entity, id);

        id
    }

    /// Updates the simulation by one step
    pub fn update(&mut self, delta_time: f32) {
        self.world.insert_resource(DeltaTime(delta_time));
        self.schedule.run(&mut self.world);
    }

    /// Returns the number of active creatures
    pub fn creature_count(&self) -> usize {
        self.entity_id_map.len()
    }

    /// Get mutable access to the ECS world (internal use only)
    pub(crate) fn world_mut(&mut self) -> &mut World {
        &mut self.world
    }

    // NOTE: schedule_mut() has been intentionally removed to prevent
    // dynamically adding systems after initialization, which causes
    // Bevy ECS archetype caching issues. All systems must be added
    // during SimulationBuilder::new().
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

        let (width, height) = simulation.get_boundaries();
        assert_eq!(width, 200.0);
        assert_eq!(height, 150.0);
    }

    #[test]
    fn test_builder_default_boundaries() {
        let simulation = SimulationBuilder::new().build();
        let (width, height) = simulation.get_boundaries();
        assert_eq!(width, 180.0); // Default from BoundaryConfig
        assert_eq!(height, 130.0);
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
