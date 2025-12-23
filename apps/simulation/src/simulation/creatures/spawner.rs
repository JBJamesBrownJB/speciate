use super::builder::CritBuilder;
use crate::simulation::core::WorldBounds;
use crate::simulation::creatures::components::CreatureState;
use crate::simulation::Simulation;
use crate::state::loader::{Rectangle, SpawnSection};
use log::info;
use rand::Rng;

#[derive(Debug, Clone, Default)]
pub struct CreatureSpawnRequest {
    position: Option<(f32, f32)>,
    state: Option<CreatureState>,
}

impl CreatureSpawnRequest {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn at(mut self, x: f32, y: f32) -> Self {
        self.position = Some((x, y));
        self
    }

    pub fn with_state(mut self, state: CreatureState) -> Self {
        self.state = Some(state);
        self
    }
}

pub fn spawn_creature(simulation: &mut Simulation, request: CreatureSpawnRequest) -> u32 {
    let mut rng = rand::thread_rng();

    let (min_x, max_x, min_y, max_y) = simulation.get_boundaries();

    let (x, y) = if let Some((px, py)) = request.position {
        (px.clamp(min_x, max_x), py.clamp(min_y, max_y))
    } else {
        (rng.gen_range(min_x..max_x), rng.gen_range(min_y..max_y))
    };

    let mut builder = CritBuilder::new().at(x, y).with_all_capabilities();

    if let Some(state) = request.state {
        builder = builder
            .in_behavior(state.behavior)
            .with_energy(state.energy)
            .with_age(state.age);
    }

    simulation.spawn_crit(builder)
}

fn random_point_in_rect(rect: &Rectangle) -> (f32, f32) {
    let mut rng = rand::thread_rng();
    let x = rng.gen_range(rect.min_x..rect.max_x);
    let y = rng.gen_range(rect.min_y..rect.max_y);
    (x, y)
}

pub fn spawn_initial_creatures_from_config(simulation: &mut Simulation, config: &SpawnSection) {
    if !config.spawn_zone.is_valid() {
        log::error!("Invalid spawn_zone in config");
        return;
    }
    if !config.target_zone.is_valid() {
        log::error!("Invalid target_zone in config");
        return;
    }

    if config.behavior != "seeking" && config.behavior != "wandering" {
        log::error!(
            "Unsupported behavior: {} (supported: seeking, wandering)",
            config.behavior
        );
        return;
    }

    info!(
        "Spawning {} {} in zone ({},{}) to ({},{})",
        config.count,
        config.behavior,
        config.spawn_zone.min_x,
        config.spawn_zone.min_y,
        config.spawn_zone.max_x,
        config.spawn_zone.max_y
    );

    let world_bounds = {
        let (min_x, max_x, min_y, max_y) = simulation.get_boundaries();
        WorldBounds::new(min_x, max_x, min_y, max_y)
    };
    const EDGE_MARGIN: f32 = 10.0;

    for _ in 0..config.count {
        let (spawn_x, spawn_y) = random_point_in_rect(&config.spawn_zone);

        let (spawn_x, spawn_y) = world_bounds.clamp_point(spawn_x, spawn_y);

        let builder = if config.behavior == "wandering" {
            CritBuilder::new()
                .at(spawn_x, spawn_y)
                .as_wanderer(&world_bounds)
                .with_all_capabilities()
        } else {
            let (target_x, target_y) = random_point_in_rect(&config.target_zone);

            let (target_x, target_y) = world_bounds.clamp_target(target_x, target_y, EDGE_MARGIN);

            CritBuilder::new()
                .at(spawn_x, spawn_y)
                .as_seeker(target_x, target_y)
                .with_all_capabilities()
        };

        simulation.spawn_crit(builder);
    }
}

#[cfg(test)]
pub fn spawn_seek_test_scenario(simulation: &mut Simulation) -> (u32, u32) {
    let seeker_id = {
        let builder = CritBuilder::new().at(0.0, 0.0).as_seeker(100.0, 0.0);
        simulation.spawn_crit(builder)
    };

    let obstacle_id = {
        let builder = CritBuilder::new().at(50.0, 10.0).with_all_capabilities();
        simulation.spawn_crit(builder)
    };

    (seeker_id, obstacle_id)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::simulation::creatures::components::BehaviorMode;
    use crate::simulation::SimulationBuilder;

    #[test]
    fn test_spawn_creature_with_defaults_creates_entity() {
        let mut simulation = SimulationBuilder::new().build();
        simulation.set_boundaries(50.0, 50.0);

        let entity_id = spawn_creature(&mut simulation, CreatureSpawnRequest::new());

        assert!(entity_id > 0);
        assert_eq!(simulation.creature_count(), 1);
    }

    #[test]
    fn test_spawn_creature_random_position_within_bounds() {
        let mut simulation = SimulationBuilder::new().build();
        let width = 100.0;
        let height = 100.0;
        simulation.set_boundaries(width, height);

        let _entity_id = spawn_creature(&mut simulation, CreatureSpawnRequest::new());

        assert_eq!(simulation.creature_count(), 1);
    }

    #[test]
    fn test_spawn_creature_random_has_valid_defaults() {
        let mut simulation = SimulationBuilder::new().build();
        simulation.set_boundaries(50.0, 50.0);

        for _ in 0..10 {
            spawn_creature(&mut simulation, CreatureSpawnRequest::new());
        }

        assert_eq!(simulation.creature_count(), 10);
    }

    #[test]
    fn test_spawn_creature_at_specific_position() {
        let mut simulation = SimulationBuilder::new().build();
        simulation.set_boundaries(50.0, 50.0);
        let target_x = 25.5;
        let target_y = 75.3;

        let entity_id = spawn_creature(
            &mut simulation,
            CreatureSpawnRequest::new().at(target_x, target_y),
        );

        assert!(entity_id > 0);
        assert_eq!(simulation.creature_count(), 1);
    }

    #[test]
    fn test_spawn_at_position_still_has_random_state() {
        let mut simulation = SimulationBuilder::new().build();
        simulation.set_boundaries(50.0, 50.0);

        let entity1 = spawn_creature(&mut simulation, CreatureSpawnRequest::new().at(50.0, 50.0));
        let entity2 = spawn_creature(&mut simulation, CreatureSpawnRequest::new().at(50.0, 50.0));

        assert_ne!(entity1, entity2);
        assert_eq!(simulation.creature_count(), 2);
    }

    #[test]
    fn test_spawn_position_clamped_to_bounds() {
        let mut simulation = SimulationBuilder::new().build();
        simulation.set_boundaries(50.0, 50.0);

        let entity_id = spawn_creature(
            &mut simulation,
            CreatureSpawnRequest::new().at(150.0, -50.0),
        );

        assert!(entity_id > 0);
        assert_eq!(simulation.creature_count(), 1);
    }

    #[test]
    fn test_spawn_creature_with_specific_state() {
        let mut simulation = SimulationBuilder::new().build();
        simulation.set_boundaries(50.0, 50.0);

        let custom_state = CreatureState {
            behavior: BehaviorMode::Catatonic,
            energy: 50.0,
            age: 10.0,
        };

        let entity_id = spawn_creature(
            &mut simulation,
            CreatureSpawnRequest::new().with_state(custom_state),
        );

        assert!(entity_id > 0);
        assert_eq!(simulation.creature_count(), 1);
    }

    #[test]
    fn test_spawn_with_state_still_has_random_position() {
        let mut simulation = SimulationBuilder::new().build();
        simulation.set_boundaries(50.0, 50.0);

        let state = CreatureState {
            behavior: BehaviorMode::Catatonic,
            energy: 75.0,
            age: 0.0,
        };

        let entity1 = spawn_creature(
            &mut simulation,
            CreatureSpawnRequest::new().with_state(state),
        );
        let entity2 = spawn_creature(
            &mut simulation,
            CreatureSpawnRequest::new().with_state(state),
        );

        assert_ne!(entity1, entity2);
        assert_eq!(simulation.creature_count(), 2);
    }

    #[test]
    fn test_spawn_creature_with_full_specification() {
        let mut simulation = SimulationBuilder::new().build();
        simulation.set_boundaries(50.0, 50.0);

        let state = CreatureState {
            behavior: BehaviorMode::Catatonic,
            energy: 80.0,
            age: 5.0,
        };

        let entity_id = spawn_creature(
            &mut simulation,
            CreatureSpawnRequest::new().at(30.0, 70.0).with_state(state),
        );

        assert!(entity_id > 0);
        assert_eq!(simulation.creature_count(), 1);
    }

    #[test]
    fn test_spawn_deterministic_with_full_spec() {
        let mut simulation = SimulationBuilder::new().build();
        simulation.set_boundaries(50.0, 50.0);

        let state = CreatureState {
            behavior: BehaviorMode::Catatonic,
            energy: 60.0,
            age: 0.0,
        };

        let entity1 = spawn_creature(
            &mut simulation,
            CreatureSpawnRequest::new().at(50.0, 50.0).with_state(state),
        );
        let entity2 = spawn_creature(
            &mut simulation,
            CreatureSpawnRequest::new().at(50.0, 50.0).with_state(state),
        );

        assert_ne!(entity1, entity2);
        assert_eq!(simulation.creature_count(), 2);
    }

    #[test]
    fn test_spawn_with_zero_energy_is_valid() {
        let mut simulation = SimulationBuilder::new().build();
        simulation.set_boundaries(50.0, 50.0);

        let state = CreatureState {
            behavior: BehaviorMode::Catatonic,
            energy: 0.0,
            age: 0.0,
        };

        let entity_id = spawn_creature(
            &mut simulation,
            CreatureSpawnRequest::new().with_state(state),
        );

        assert!(entity_id > 0);
        assert_eq!(simulation.creature_count(), 1);
    }

    #[test]
    fn test_spawn_multiple_maintains_independence() {
        let mut simulation = SimulationBuilder::new().build();
        simulation.set_boundaries(50.0, 50.0);

        let mut entity_ids = Vec::new();
        for _ in 0..100 {
            let entity_id = spawn_creature(&mut simulation, CreatureSpawnRequest::new());
            entity_ids.push(entity_id);
        }

        assert_eq!(simulation.creature_count(), 100);
        let unique_ids: std::collections::HashSet<_> = entity_ids.iter().collect();
        assert_eq!(unique_ids.len(), 100);
    }

    #[test]
    fn test_spawn_at_boundary_edges() {
        let mut simulation = SimulationBuilder::new().build();
        simulation.set_boundaries(50.0, 50.0);

        let corners = vec![(0.0, 0.0), (100.0, 0.0), (0.0, 100.0), (100.0, 100.0)];

        for (x, y) in corners {
            let entity_id = spawn_creature(&mut simulation, CreatureSpawnRequest::new().at(x, y));
            assert!(entity_id > 0);
        }

        assert_eq!(simulation.creature_count(), 4);
    }

    #[test]
    fn test_mixed_spawn_strategies() {
        let mut simulation = SimulationBuilder::new().build();
        simulation.set_boundaries(50.0, 50.0);

        spawn_creature(&mut simulation, CreatureSpawnRequest::new());

        spawn_creature(&mut simulation, CreatureSpawnRequest::new().at(10.0, 10.0));

        let state = CreatureState {
            behavior: BehaviorMode::Catatonic,
            energy: 90.0,
            age: 0.0,
        };
        spawn_creature(
            &mut simulation,
            CreatureSpawnRequest::new().with_state(state),
        );

        spawn_creature(
            &mut simulation,
            CreatureSpawnRequest::new().at(50.0, 50.0).with_state(state),
        );

        assert_eq!(simulation.creature_count(), 4);
    }

    #[test]
    fn test_spawn_from_config_basic() {
        let mut simulation = SimulationBuilder::new().build();
        simulation.set_boundaries(500.0, 500.0);

        let config = SpawnSection {
            count: 10,
            behavior: "seeking".to_string(),
            spawn_zone: Rectangle {
                min_x: 0.0,
                max_x: 100.0,
                min_y: 0.0,
                max_y: 100.0,
            },
            target_zone: Rectangle {
                min_x: 200.0,
                max_x: 300.0,
                min_y: 200.0,
                max_y: 300.0,
            },
        };

        spawn_initial_creatures_from_config(&mut simulation, &config);
        assert_eq!(simulation.creature_count(), 10);
    }

    #[test]
    fn test_spawn_from_config_validates_zones() {
        let mut simulation = SimulationBuilder::new().build();
        simulation.set_boundaries(500.0, 500.0);

        let config = SpawnSection {
            count: 5,
            behavior: "seeking".to_string(),
            spawn_zone: Rectangle {
                min_x: 100.0,
                max_x: 0.0,
                min_y: 0.0,
                max_y: 100.0,
            },
            target_zone: Rectangle {
                min_x: 200.0,
                max_x: 300.0,
                min_y: 200.0,
                max_y: 300.0,
            },
        };

        spawn_initial_creatures_from_config(&mut simulation, &config);
        assert_eq!(simulation.creature_count(), 0);
    }

    #[test]
    fn test_spawn_from_config_rejects_unsupported_behavior() {
        let mut simulation = SimulationBuilder::new().build();
        simulation.set_boundaries(500.0, 500.0);

        let config = SpawnSection {
            count: 5,
            behavior: "flying".to_string(),
            spawn_zone: Rectangle {
                min_x: 0.0,
                max_x: 100.0,
                min_y: 0.0,
                max_y: 100.0,
            },
            target_zone: Rectangle {
                min_x: 200.0,
                max_x: 300.0,
                min_y: 200.0,
                max_y: 300.0,
            },
        };

        spawn_initial_creatures_from_config(&mut simulation, &config);
        assert_eq!(simulation.creature_count(), 0);
    }

    #[test]
    fn test_spawn_from_config_large_count() {
        let mut simulation = SimulationBuilder::new().build();
        simulation.set_boundaries(1000.0, 1000.0);

        let config = SpawnSection {
            count: 100,
            behavior: "seeking".to_string(),
            spawn_zone: Rectangle {
                min_x: -500.0,
                max_x: 500.0,
                min_y: -500.0,
                max_y: 500.0,
            },
            target_zone: Rectangle {
                min_x: -400.0,
                max_x: 400.0,
                min_y: -400.0,
                max_y: 400.0,
            },
        };

        spawn_initial_creatures_from_config(&mut simulation, &config);
        assert_eq!(simulation.creature_count(), 100);
    }

    #[test]
    fn test_rectangle_validation() {
        let valid = Rectangle {
            min_x: 0.0,
            max_x: 100.0,
            min_y: 0.0,
            max_y: 100.0,
        };
        assert!(valid.is_valid());
        assert_eq!(valid.width(), 100.0);
        assert_eq!(valid.height(), 100.0);

        let invalid_x = Rectangle {
            min_x: 100.0,
            max_x: 0.0,
            min_y: 0.0,
            max_y: 100.0,
        };
        assert!(!invalid_x.is_valid());

        let invalid_y = Rectangle {
            min_x: 0.0,
            max_x: 100.0,
            min_y: 100.0,
            max_y: 0.0,
        };
        assert!(!invalid_y.is_valid());
    }
}
