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
use speciate::simulation::core::components::{Catatonic, Position};
use speciate::simulation::creatures::components::Target;

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
        trial_file_exists("default-spawn-baseline"),
        "default-spawn-baseline.toml not found in trials/"
    );
    assert!(
        trial_file_exists("crowd-navigation"),
        "crowd-navigation.toml not found in trials/"
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
    assert_eq!(config.name, "Crowd Navigation Test");
    assert!(config.description.contains("200 static obstacles"));

    // Verify total creature count (200 catatonic + 50 seekers = 250)
    let mut pos_query = world.query::<&Position>();
    let total_count = pos_query.iter(&world).count();
    assert_eq!(
        total_count, 250,
        "Should spawn 250 creatures (200 obstacles + 50 seekers)"
    );

    // Verify catatonic count (20×10 grid = 200)
    let mut catatonic_query = world.query::<&Catatonic>();
    let catatonic_count = catatonic_query.iter(&world).count();
    assert_eq!(catatonic_count, 200, "Should have 200 static obstacles");

    // Verify seeker count (circle with 50 creatures)
    let mut seeker_query = world.query::<&Target>();
    let seeker_count = seeker_query.iter(&world).count();
    assert_eq!(seeker_count, 50, "Should have 50 mobile seekers");
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

    // Load crowd-navigation trial (has custom boundary config)
    load_trial(&mut world, "crowd-navigation").expect("Should load trial");

    // Verify boundary override was applied
    let boundary = world.get_resource::<BoundaryConfig>().unwrap();
    assert_eq!(boundary.min_x, -500.0);
    assert_eq!(boundary.max_x, 500.0);
    assert_eq!(boundary.min_y, -500.0);
    assert_eq!(boundary.max_y, 500.0);
}

#[test]
#[cfg(feature = "dev-tools")]
fn test_trial_spawns_valid_components() {
    use speciate::simulation::core::components::{Acceleration, BodySize, Velocity};

    let mut world = World::new();
    world.insert_resource(speciate::simulation::creatures::systems::NextCreatureId::default());

    load_trial(&mut world, "default-spawn-baseline").expect("Should load trial");

    // All creatures should have core physics components
    let pos_count = world.query::<&Position>().iter(&world).count();
    let vel_count = world.query::<&Velocity>().iter(&world).count();
    let accel_count = world.query::<&Acceleration>().iter(&world).count();
    let body_count = world.query::<&BodySize>().iter(&world).count();

    assert_eq!(pos_count, 100);
    assert_eq!(vel_count, 100);
    assert_eq!(accel_count, 100);
    assert_eq!(body_count, 100);
}

#[test]
#[cfg(feature = "dev-tools")]
fn test_trial_circle_pattern_radius() {
    let mut world = World::new();
    world.insert_resource(speciate::simulation::creatures::systems::NextCreatureId::default());

    load_trial(&mut world, "crowd-navigation").expect("Should load trial");

    // Get seeker positions (circle pattern with radius 100)
    let mut query = world.query::<(&Position, &Target)>();
    let seekers: Vec<_> = query.iter(&world).collect();

    // Verify all seekers are ~100m from center (0, 0)
    for (pos, _) in seekers {
        let dist = (pos.x * pos.x + pos.y * pos.y).sqrt();
        assert!(
            (dist - 100.0).abs() < 0.1,
            "Seeker at ({}, {}) should be ~100m from origin, got {}m",
            pos.x,
            pos.y,
            dist
        );
    }
}

#[test]
#[cfg(feature = "dev-tools")]
fn test_trial_grid_spacing() {
    let mut world = World::new();
    world.insert_resource(speciate::simulation::creatures::systems::NextCreatureId::default());

    load_trial(&mut world, "default-spawn-baseline").expect("Should load trial");

    let mut query = world.query::<&Position>();
    let mut positions: Vec<_> = query.iter(&world).collect();

    // Sort by Y, then X for predictable ordering
    positions.sort_by(|a, b| {
        a.y.partial_cmp(&b.y)
            .unwrap()
            .then(a.x.partial_cmp(&b.x).unwrap())
    });

    // Verify first row has 5m spacing
    for i in 0..9 {
        let spacing = positions[i + 1].x - positions[i].x;
        assert!(
            (spacing - 5.0).abs() < 0.01,
            "Grid spacing should be 5m, got {}",
            spacing
        );
    }
}
