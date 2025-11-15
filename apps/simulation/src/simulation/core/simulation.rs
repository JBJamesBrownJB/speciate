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

pub struct SimulationBuilder {
    world: World,
    schedule: Schedule,
}

impl SimulationBuilder {
    pub fn new() -> Self {
        let mut world = World::new();
        let mut schedule = Schedule::default();

        use bevy_ecs::prelude::AppTypeRegistry;
        use crate::simulation::components::*;
        use crate::simulation::core::components::*;
        use crate::simulation::perception::{AvoidanceBehavior, Perception};

        world.init_resource::<AppTypeRegistry>();
        {
            let registry = world.resource::<AppTypeRegistry>();
            let mut type_registry = registry.write();

            type_registry.register::<Position>();
            type_registry.register::<Velocity>();
            type_registry.register::<Acceleration>();
            type_registry.register::<BodySize>();
            type_registry.register::<Rotation>();
            type_registry.register::<Catatonic>();

            type_registry.register::<CritId>();
            type_registry.register::<CreatureState>();
            type_registry.register::<BehaviorMode>();
            type_registry.register::<HomePosition>();

            type_registry.register::<CanSeek>();
            type_registry.register::<CanFlee>();
            type_registry.register::<CanWander>();
            type_registry.register::<CanAvoidObstacles>();

            type_registry.register::<Perception>();
            type_registry.register::<AvoidanceBehavior>();
            type_registry.register::<Target>();

            type_registry.register::<WanderState>();
            type_registry.register::<FleeState>();
        }

        schedule.add_systems(process_spawn_events);

        #[cfg(feature = "dev-tools")]
        schedule.add_systems(crate::ipc::command_executor_system);

        schedule.add_systems((
            perception::update_perception_system,
            behavior_transition_system,
            territory_wandering_system,
            flee_system,
            seek_system,
            behaviors::avoidance_system,
            integrate_motion_system,
            rotation_system,
        ));


        world.insert_resource(DeltaTime::default());
        world.insert_resource(BoundaryConfig::default());
        world.insert_resource(WorldBounds::default());
        world.insert_resource(PhysicsTick::default());
        world.insert_resource(ActualTickRate::default());
        world.insert_resource(MovementConfig::default());

        world.init_resource::<Events<SpawnCreatureEvent>>();
        world.insert_resource(NextCreatureId::default());
        world.insert_resource(EntityIdMap::default());

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
        self.world.insert_resource(WorldBounds::new(-extent_x, extent_x, -extent_y, extent_y));
        self
    }

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

pub struct Simulation {
    pub(crate) world: World,
    schedule: Schedule,
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
        self.world.insert_resource(WorldBounds::new(-extent_x, extent_x, -extent_y, extent_y));
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

        let entity = self.world.spawn(builder.build(id)).id();

        self.world.resource_mut::<EntityIdMap>().insert(entity, id);

        id
    }

    #[cfg(test)]
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

        self.world.resource_mut::<Events<SpawnCreatureEvent>>().update();

        self.world.resource_mut::<PhysicsTick>().increment();
    }

    pub fn creature_count(&self) -> usize {
        self.world.resource::<EntityIdMap>().len()
    }

    #[cfg(any(test, feature = "test-helpers", feature = "dev-tools"))]
    pub fn world(&self) -> &World {
        &self.world
    }

    #[cfg(any(test, feature = "test-helpers", feature = "dev-tools"))]
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
        let simulation = SimulationBuilder::new().build();
        let (min_x, max_x, min_y, max_y) = simulation.get_boundaries();
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
}
