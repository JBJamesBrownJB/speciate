/// Integration test for trial loading system
///
/// Tests the full pipeline:
/// 1. Read TOML trial template from file
/// 2. Parse into TrialConfig
/// 3. Spawn creatures into ECS World
/// 4. Verify creature positions and component composition
///
/// NOTE: These tests verify trial files exist at the expected locations
/// relative to the binary (../../trials/ from target/debug)

use bevy_ecs::prelude::*;
use speciate::simulation::core::components::Position;
use speciate::simulation::creatures::components::{BehaviorMode, CreatureState};

#[cfg(feature = "dev-tools")]
use speciate::trials::loader::load_trial;

/// Helper to verify trial files exist
fn trial_file_exists(name: &str) -> bool {
    let exe_dir = std::env::current_exe()
        .expect("Failed to get exe path")
        .parent()
        .expect("Failed to get exe dir")
        .to_path_buf();

    let trials_relative_path = if exe_dir.ends_with("deps") {
        "../../../trials"
    } else {
        "../../trials"
    };

    let trial_path = exe_dir.join(trials_relative_path).join(format!("{}.toml", name));
    trial_path.exists()
}

#[test]
#[cfg(feature = "dev-tools")]
fn test_trial_files_exist() {
    // This test verifies the trial files are in the expected location
    assert!(
        trial_file_exists("crowd-navigation"),
        "crowd-navigation.toml not found in trials/"
    );
    assert!(
        trial_file_exists("many-wanderers-dense"),
        "many-wanderers-dense.toml not found in trials/"
    );
}

#[test]
#[cfg(feature = "dev-tools")]
fn test_load_default_spawn_baseline_trial() {
    if !trial_file_exists("default-spawn-baseline") {
        eprintln!("⚠️  Skipping test: default-spawn-baseline.toml not found");
        return;
    }

    let mut world = World::new();
    world.insert_resource(speciate::simulation::creatures::systems::NextCreatureId::default());

    // Load trial template
    let config = load_trial(&mut world, "default-spawn-baseline")
        .expect("Should load default-spawn-baseline.toml");

    // Verify metadata
    assert_eq!(config.name, "Default Spawn Baseline");
    assert!(config.description.contains("10×10 grid"));

    // Verify creatures spawned (10×10 = 100 creatures)
    let mut query = world.query::<&Position>();
    let positions: Vec<_> = query.iter(&world).collect();
    assert_eq!(
        positions.len(),
        100,
        "Should spawn 100 creatures from 10×10 grid"
    );

    // Verify grid layout (should span -25 to +25 with 5m spacing)
    let min_x = positions.iter().map(|p| p.x).fold(f32::INFINITY, f32::min);
    let max_x = positions
        .iter()
        .map(|p| p.x)
        .fold(f32::NEG_INFINITY, f32::max);
    let min_y = positions.iter().map(|p| p.y).fold(f32::INFINITY, f32::min);
    let max_y = positions
        .iter()
        .map(|p| p.y)
        .fold(f32::NEG_INFINITY, f32::max);

    assert_eq!(min_x, -25.0, "Grid should start at -25m X");
    assert_eq!(max_x, 20.0, "Grid should end at 20m X (9 * 5 spacing)");
    assert_eq!(min_y, -25.0, "Grid should start at -25m Y");
    assert_eq!(max_y, 20.0, "Grid should end at 20m Y");
}

#[test]
#[cfg(feature = "dev-tools")]
fn test_load_crowd_navigation_trial() {
    if !trial_file_exists("crowd-navigation") {
        eprintln!("⚠️  Skipping test: crowd-navigation.toml not found");
        return;
    }

    let mut world = World::new();
    world.insert_resource(speciate::simulation::creatures::systems::NextCreatureId::default());

    // Load trial template
    let config = load_trial(&mut world, "crowd-navigation")
        .expect("Should load crowd-navigation.toml");

    // Verify metadata
    assert_eq!(config.name, "Crowd Navigation");
    assert!(config.description.contains("10x10 grid"));

    // Verify total creature count (100 catatonic + 1 seeker = 101)
    let mut pos_query = world.query::<&Position>();
    let total_count = pos_query.iter(&world).count();
    assert_eq!(
        total_count, 101,
        "Should spawn 101 creatures (100 obstacles + 1 seeker)"
    );

    // Verify catatonic count (10×10 grid = 100)
    let mut catatonic_query = world.query::<&CreatureState>();
    let catatonic_count = catatonic_query.iter(&world)
        .filter(|state| state.behavior == BehaviorMode::Catatonic)
        .count();
    assert_eq!(catatonic_count, 100, "Should have 100 static obstacles");

    // Verify seeker count (1 seeker with Seeking behavior)
    let mut seeker_query = world.query::<&CreatureState>();
    let seeker_count = seeker_query.iter(&world)
        .filter(|state| state.behavior == BehaviorMode::Seeking)
        .count();
    assert_eq!(seeker_count, 1, "Should have 1 mobile seeker");
}

#[test]
#[cfg(feature = "dev-tools")]
fn test_trial_nonexistent_file() {
    let mut world = World::new();

    // Try to load nonexistent trial
    let result = load_trial(&mut world, "nonexistent-trial");

    assert!(result.is_err(), "Should fail for nonexistent file");
    let err_msg = result.unwrap_err();
    assert!(
        err_msg.contains("Trial file not found") || err_msg.contains("No such file"),
        "Error should mention file not found, got: {}",
        err_msg
    );
}

#[test]
#[cfg(feature = "dev-tools")]
fn test_trial_world_config_override() {
    use speciate::simulation::core::components::BoundaryConfig;

    let mut world = World::new();
    world.insert_resource(speciate::simulation::creatures::systems::NextCreatureId::default());

    // Load crowd-navigation trial (this trial doesn't define boundaries)
    load_trial(&mut world, "crowd-navigation").expect("Should load trial");

    // Trial doesn't define boundaries, so BoundaryConfig should NOT be present
    // (only trials with explicit boundary config will have this resource)
    let boundary = world.get_resource::<BoundaryConfig>();
    assert!(boundary.is_none(), "BoundaryConfig should NOT be present for trials without boundary config");
}

#[test]
#[cfg(feature = "dev-tools")]
fn test_trial_spawns_valid_components() {
    use speciate::simulation::core::components::{Acceleration, BodySize, Velocity};

    let mut world = World::new();
    world.insert_resource(speciate::simulation::creatures::systems::NextCreatureId::default());

    load_trial(&mut world, "crowd-navigation").expect("Should load trial");

    // All creatures should have core physics components (101 total: 100 grid + 1 seeker)
    let pos_count = world.query::<&Position>().iter(&world).count();
    let vel_count = world.query::<&Velocity>().iter(&world).count();
    let accel_count = world.query::<&Acceleration>().iter(&world).count();
    let body_count = world.query::<&BodySize>().iter(&world).count();

    assert_eq!(pos_count, 101);
    assert_eq!(vel_count, 101);
    assert_eq!(accel_count, 101);
    assert_eq!(body_count, 101);
}

#[test]
#[cfg(feature = "dev-tools")]
fn test_trial_seeker_starting_position() {
    let mut world = World::new();
    world.insert_resource(speciate::simulation::creatures::systems::NextCreatureId::default());

    load_trial(&mut world, "crowd-navigation").expect("Should load trial");

    // Get seeker positions (single seeker with Seeking behavior)
    let mut query = world.query::<(&Position, &CreatureState)>();
    let seekers: Vec<_> = query.iter(&world)
        .filter(|(_, state)| state.behavior == BehaviorMode::Seeking)
        .collect();

    assert_eq!(seekers.len(), 1, "Should have exactly 1 seeker");

    // Verify seeker starts at (-30, 0)
    let (pos, _) = seekers[0];
    assert!(
        (pos.x - (-30.0)).abs() < 0.1 && pos.y.abs() < 0.1,
        "Seeker should start at (-30, 0), got ({}, {})",
        pos.x,
        pos.y
    );
}

#[test]
#[cfg(feature = "dev-tools")]
fn test_trial_grid_staggered_pattern() {
    let mut world = World::new();
    world.insert_resource(speciate::simulation::creatures::systems::NextCreatureId::default());

    load_trial(&mut world, "crowd-navigation").expect("Should load trial");

    // Get only the catatonic grid positions (not the seeker)
    let mut query = world.query::<(&Position, &CreatureState)>();
    let positions: Vec<_> = query.iter(&world)
        .filter(|(_, state)| state.behavior == BehaviorMode::Catatonic)
        .map(|(pos, _)| (pos.x, pos.y))
        .collect();

    // Verify correct creature count (10x10 grid = 100 catatonic creatures)
    assert_eq!(positions.len(), 100, "Should have 100 catatonic creatures in grid");

    // Verify staggered pattern creates multiple distinct Y levels
    // With grid_offset_y = 1.0 and spacing = 2.0, we expect 20 distinct Y values
    // (10 base rows + 10 offset rows)
    let mut unique_y_values: Vec<f32> = positions.iter().map(|(_, y)| *y).collect();
    unique_y_values.sort_by(|a, b| a.partial_cmp(b).unwrap());
    unique_y_values.dedup_by(|a, b| (*a - *b).abs() < 0.01);

    assert!(
        unique_y_values.len() > 10,
        "Staggered grid should create more than 10 Y levels, got {}",
        unique_y_values.len()
    );

    // Verify grid spans reasonable bounds (creatures spread across area)
    let min_x = positions.iter().map(|(x, _)| *x).fold(f32::INFINITY, f32::min);
    let max_x = positions.iter().map(|(x, _)| *x).fold(f32::NEG_INFINITY, f32::max);
    let min_y = positions.iter().map(|(_, y)| *y).fold(f32::INFINITY, f32::min);
    let max_y = positions.iter().map(|(_, y)| *y).fold(f32::NEG_INFINITY, f32::max);

    // Grid should span a reasonable area (not all creatures at same point)
    assert!(max_x - min_x > 10.0, "Grid should span significant X range");
    assert!(max_y - min_y > 10.0, "Grid should span significant Y range");
}
