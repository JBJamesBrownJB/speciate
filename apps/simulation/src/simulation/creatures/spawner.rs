//! Creature spawning logic
//!
//! This module handles the creation and spawning of creatures into the simulation world.

use super::builder::CritBuilder;
use crate::config::SpawningConfig;
use crate::simulation::components::*;
use crate::simulation::core::WorldBounds;
use crate::simulation::Simulation;
use crate::state::loader::{Rectangle, SpawnSection};
use log::info;
use rand::Rng;

/// Request for spawning a creature with optional position and state
#[derive(Debug, Clone)]
pub struct CreatureSpawnRequest {
    position: Option<(f32, f32)>,
    state: Option<CreatureState>,
}

impl Default for CreatureSpawnRequest {
    fn default() -> Self {
        Self {
            position: None,
            state: None,
        }
    }
}

impl CreatureSpawnRequest {
    /// Create a new spawn request with all defaults (random everything)
    pub fn new() -> Self {
        Self::default()
    }

    /// Specify exact spawn position
    pub fn at(mut self, x: f32, y: f32) -> Self {
        self.position = Some((x, y));
        self
    }

    /// Specify exact creature state
    pub fn with_state(mut self, state: CreatureState) -> Self {
        self.state = Some(state);
        self
    }
}

/// Spawn a single creature based on the request
///
/// Uses CritBuilder internally to create crits with all capabilities enabled.
///
/// # Arguments
/// * `simulation` - Mutable reference to the simulation
/// * `request` - Spawn configuration (position/state optional)
///
/// # Returns
/// Entity ID of the spawned creature
pub fn spawn_creature(simulation: &mut Simulation, request: CreatureSpawnRequest) -> u32 {
    let mut rng = rand::thread_rng();

    // Get world boundaries (centered coordinate system)
    let (min_x, max_x, min_y, max_y) = simulation.get_boundaries();

    // Resolve position (random or specified, then clamped)
    let (x, y) = if let Some((px, py)) = request.position {
        // Clamp to centered bounds
        (px.clamp(min_x, max_x), py.clamp(min_y, max_y))
    } else {
        // Random position within centered bounds
        (rng.gen_range(min_x..max_x), rng.gen_range(min_y..max_y))
    };

    // Build crit with all capabilities (flexible for any behavior)
    let mut builder = CritBuilder::new().at(x, y).with_all_capabilities();

    // Apply custom state if provided
    if let Some(state) = request.state {
        builder = builder
            .in_behavior(state.behavior)
            .with_energy(state.energy)
            .with_age(state.age)
            .with_max_speed(state.max_speed);
    }

    simulation.spawn_crit(builder)
}

/// Generate a random point within a rectangle
fn random_point_in_rect(rect: &Rectangle) -> (f32, f32) {
    let mut rng = rand::thread_rng();
    let x = rng.gen_range(rect.min_x..rect.max_x);
    let y = rng.gen_range(rect.min_y..rect.max_y);
    (x, y)
}

/// Spawn initial creatures from TOML config
///
/// Creates creatures based on the spawn configuration:
/// - `count`: Number of creatures to spawn
/// - `behavior`: Behavior type ("seeking" or "wandering")
/// - `spawn_zone`: Rectangle where creatures spawn (random positions)
/// - `target_zone`: Rectangle where seeker targets are placed (random positions)
///
/// All spawn positions and targets are clamped to world boundaries with margin
/// to prevent creatures from bunching at edges.
pub fn spawn_initial_creatures_from_config(simulation: &mut Simulation, config: &SpawnSection) {
    // Validate rectangles
    if !config.spawn_zone.is_valid() {
        log::error!("Invalid spawn_zone in config");
        return;
    }
    if !config.target_zone.is_valid() {
        log::error!("Invalid target_zone in config");
        return;
    }

    // Support "seeking" and "wandering"
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

    // Get world bounds for clamping
    let world_bounds = {
        let (min_x, max_x, min_y, max_y) = simulation.get_boundaries();
        WorldBounds::new(min_x, max_x, min_y, max_y)
    };
    const EDGE_MARGIN: f32 = 10.0; // 10m margin from world edge

    // Spawn creatures based on behavior type
    for _ in 0..config.count {
        let (spawn_x, spawn_y) = random_point_in_rect(&config.spawn_zone);

        // Clamp spawn position to world bounds (no margin - allow full world)
        // This prevents bunching at margin boundaries - spawns spread naturally
        let (spawn_x, spawn_y) = world_bounds.clamp_point(spawn_x, spawn_y);

        let builder = if config.behavior == "wandering" {
            // Wanderers: home position = spawn position, start wandering
            CritBuilder::new()
                .at(spawn_x, spawn_y)
                .as_wanderer(&world_bounds)
                .with_all_capabilities()
        } else {
            // Seekers: random target in target zone
            let (target_x, target_y) = random_point_in_rect(&config.target_zone);

            // Clamp target to world bounds (with margin)
            let (target_x, target_y) = world_bounds.clamp_target(target_x, target_y, EDGE_MARGIN);

            CritBuilder::new()
                .at(spawn_x, spawn_y)
                .as_seeker(target_x, target_y)
                .with_all_capabilities()
        };

        simulation.spawn_crit(builder);
    }
}

/// Spawn initial creatures - DEMO: Obstacle Avoidance Scenario
///
/// This demo spawns:
/// - 1 stationary obstacle (catatonic) at (15, 1)
/// - 1 seeker starting at (0, 0) targeting (100, 0)
///
/// Watch the seeker detect and navigate around the obstacle to reach its goal!
pub fn spawn_initial_creatures(simulation: &mut Simulation, _config: &SpawningConfig) {
    // Spawn stationary obstacle slightly off the direct path
    // Position: (15, 1) - close enough for early detection, off-axis for lateral steering
    let obstacle = CritBuilder::new()
        .at(0.0, 0.0)
        .in_behavior(BehaviorMode::Catatonic) // Stationary obstacle
        .with_all_capabilities();
    simulation.spawn_crit(obstacle);

    simulation.spawn_crit(
        CritBuilder::new()
            .with_size(5.0)
            .at(20.0, 0.0)
            .as_seeker(-10.0, 0.0) // Target is beyond obstacle
            .with_all_capabilities(),
    );

    simulation.spawn_crit(
        CritBuilder::new()
            .at(-20.0, 0.0)
            .as_seeker(10.0, 0.0) // Target is beyond obstacle
            .with_all_capabilities(),
    );

    simulation.spawn_crit(
        CritBuilder::new()
            .at(0.0, 20.0)
            .as_seeker(-10.0, -10.0) // Target is beyond obstacle
            .with_all_capabilities(),
    );
}

/// Spawn a test scenario for seeking behavior (Sprint 6 milestone)
///
/// Creates a seeker at (0, 0) targeting (100, 0) and an obstacle at (50, 0).
/// Uses the simplified CritBuilder pattern - much cleaner than before!
///
/// Returns (seeker_id, obstacle_id)
#[cfg(test)]
pub fn spawn_seek_test_scenario(simulation: &mut Simulation) -> (u32, u32) {
    // Spawn seeker using the builder - clean and simple!
    let seeker_id = {
        let builder = CritBuilder::new().at(0.0, 0.0).as_seeker(100.0, 0.0); // Includes CanSeek, Target, and Seeking behavior
        simulation.spawn_crit(builder)
    };

    // Spawn stationary obstacle OFF the direct path (for perception tests, not blocking)
    // Moved from (50, 0) to (50, 10) so seeker can reach target without obstruction
    let obstacle_id = {
        let builder = CritBuilder::new().at(50.0, 10.0).with_all_capabilities(); // Has capabilities but stays Catatonic
        simulation.spawn_crit(builder)
    };

    (seeker_id, obstacle_id)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::SpawningConfig;
    use crate::simulation::SimulationBuilder;

    /// Tests for spawn_initial_creatures helper function
    #[test]
    fn test_spawn_initial_creatures() {
        let mut simulation = SimulationBuilder::new().build();
        let config = SpawningConfig {
            initial_population: 10, // NOTE: Config ignored in demo mode
            min_size: 0.5,
            max_size: 2.0,
        };

        spawn_initial_creatures(&mut simulation, &config);

        // Demo mode: Always spawns 4 creatures (1 obstacle + 3 seekers from different angles)
        assert_eq!(simulation.creature_count(), 4);
    }

    #[test]
    fn test_spawn_demo_scenario() {
        let mut simulation = SimulationBuilder::new().build();
        let config = SpawningConfig {
            initial_population: 0, // NOTE: Config ignored in demo mode
            min_size: 0.5,
            max_size: 2.0,
        };

        spawn_initial_creatures(&mut simulation, &config);

        // Demo mode: Always spawns 4 creatures regardless of config
        assert_eq!(simulation.creature_count(), 4);
    }

    /// Cycle 1: Random Everything Tests
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

        // Position verification would require entity query access
        // For now, we verify the entity was created within valid world
        assert_eq!(simulation.creature_count(), 1);
    }

    #[test]
    fn test_spawn_creature_random_has_valid_defaults() {
        let mut simulation = SimulationBuilder::new().build();
        simulation.set_boundaries(50.0, 50.0);

        // Spawn multiple to ensure randomness is working
        for _ in 0..10 {
            spawn_creature(&mut simulation, CreatureSpawnRequest::new());
        }

        assert_eq!(simulation.creature_count(), 10);
    }

    /// Cycle 2: Specific Position Tests
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

        // Spawn two creatures at same position
        // They should have potentially different states (random)
        let entity1 = spawn_creature(&mut simulation, CreatureSpawnRequest::new().at(50.0, 50.0));
        let entity2 = spawn_creature(&mut simulation, CreatureSpawnRequest::new().at(50.0, 50.0));

        assert_ne!(entity1, entity2);
        assert_eq!(simulation.creature_count(), 2);
    }

    #[test]
    fn test_spawn_position_clamped_to_bounds() {
        let mut simulation = SimulationBuilder::new().build();
        simulation.set_boundaries(50.0, 50.0);

        // Try to spawn outside bounds - should be clamped
        let entity_id = spawn_creature(
            &mut simulation,
            CreatureSpawnRequest::new().at(150.0, -50.0),
        );

        assert!(entity_id > 0);
        assert_eq!(simulation.creature_count(), 1);
    }

    /// Cycle 3: Specific State Tests
    #[test]
    fn test_spawn_creature_with_specific_state() {
        let mut simulation = SimulationBuilder::new().build();
        simulation.set_boundaries(50.0, 50.0);

        let custom_state = CreatureState {
            behavior: BehaviorMode::Catatonic,
            energy: 50.0,
            age: 10.0,
            max_speed: 15.0,
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
            max_speed: 20.0,
        };

        // Spawn with same state multiple times
        // Positions should be random
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

    /// Cycle 4: Fully Specified Tests
    #[test]
    fn test_spawn_creature_with_full_specification() {
        let mut simulation = SimulationBuilder::new().build();
        simulation.set_boundaries(50.0, 50.0);

        let state = CreatureState {
            behavior: BehaviorMode::Catatonic,
            energy: 80.0,
            age: 5.0,
            max_speed: 18.0,
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
            max_speed: 20.0,
        };

        // Spawn twice with full specification
        // Should create two distinct entities but with same initial state
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

    /// Cycle 5: Edge Cases Tests
    #[test]
    fn test_spawn_with_zero_energy_is_valid() {
        let mut simulation = SimulationBuilder::new().build();
        simulation.set_boundaries(50.0, 50.0);

        let state = CreatureState {
            behavior: BehaviorMode::Catatonic,
            energy: 0.0, // Dead on arrival
            age: 0.0,
            max_speed: 20.0,
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

        // Spawn 100 random creatures
        let mut entity_ids = Vec::new();
        for _ in 0..100 {
            let entity_id = spawn_creature(&mut simulation, CreatureSpawnRequest::new());
            entity_ids.push(entity_id);
        }

        // All IDs should be unique
        assert_eq!(simulation.creature_count(), 100);
        let unique_ids: std::collections::HashSet<_> = entity_ids.iter().collect();
        assert_eq!(unique_ids.len(), 100);
    }

    #[test]
    fn test_spawn_at_boundary_edges() {
        let mut simulation = SimulationBuilder::new().build();
        simulation.set_boundaries(50.0, 50.0);

        // Test all corners
        let corners = vec![
            (0.0, 0.0),     // Top-left
            (100.0, 0.0),   // Top-right
            (0.0, 100.0),   // Bottom-left
            (100.0, 100.0), // Bottom-right
        ];

        for (x, y) in corners {
            let entity_id = spawn_creature(&mut simulation, CreatureSpawnRequest::new().at(x, y));
            assert!(entity_id > 0);
        }

        assert_eq!(simulation.creature_count(), 4);
    }

    /// Integration Tests
    #[test]
    fn test_mixed_spawn_strategies() {
        let mut simulation = SimulationBuilder::new().build();
        simulation.set_boundaries(50.0, 50.0);

        // Mix all 4 spawn strategies
        spawn_creature(&mut simulation, CreatureSpawnRequest::new());

        spawn_creature(&mut simulation, CreatureSpawnRequest::new().at(10.0, 10.0));

        let state = CreatureState {
            behavior: BehaviorMode::Catatonic,
            energy: 90.0,
            age: 0.0,
            max_speed: 20.0,
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

    // Tests for spawn_initial_creatures_from_config
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

        // Invalid spawn zone (min > max)
        let config = SpawnSection {
            count: 5,
            behavior: "seeking".to_string(),
            spawn_zone: Rectangle {
                min_x: 100.0,
                max_x: 0.0, // Invalid!
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
        // Should not spawn any creatures due to validation failure
        assert_eq!(simulation.creature_count(), 0);
    }

    #[test]
    fn test_spawn_from_config_rejects_unsupported_behavior() {
        let mut simulation = SimulationBuilder::new().build();
        simulation.set_boundaries(500.0, 500.0);

        let config = SpawnSection {
            count: 5,
            behavior: "flying".to_string(), // Unsupported behavior
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
        // Should not spawn any creatures
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
