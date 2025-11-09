//! Integration tests for dev commands
//!
//! These tests verify that dev commands (spawn, clear, speed) work correctly
//! and integrate properly with the simulation state.

use crossbeam_channel::Sender;
use speciate::dev_commands::{DevCommand, DevCommandListener};
use speciate::simulation::{Simulation, SimulationBuilder};

/// Helper function to create a simulation with a test dev command listener
///
/// Returns a tuple of (Simulation, Sender<DevCommand>) for injecting test commands
fn setup_test_simulation_with_dev_commands() -> (Simulation, Sender<DevCommand>) {
    let mut simulation = SimulationBuilder::new()
        .set_boundaries(100.0, 100.0)
        .build();

    // Create test listener and inject it into the simulation's world
    let (listener, sender) = DevCommandListener::new_for_test(16);
    simulation.world_mut().insert_resource(listener);

    (simulation, sender)
}

/// Test that dev spawn command updates creature count
///
/// This test verifies the BUG: Dev-spawned creatures should be counted
/// in simulation.creature_count(), but currently they bypass entity_id_map
/// registration.
#[test]
fn dev_spawn_updates_creature_count() {
    // Given: Empty simulation with test dev command listener
    let (mut simulation, sender) = setup_test_simulation_with_dev_commands();

    let initial_count = simulation.creature_count();
    assert_eq!(initial_count, 0, "Should start with no creatures");

    // When: Send dev spawn command
    sender.send(DevCommand::Spawn {
        x: 50.0,
        y: 50.0,
        behavior: "wandering".to_string(),
        target_x: None,
        target_y: None,
        energy: Some(100.0),
        max_speed: Some(20.0),
    }).expect("Failed to send command");

    // Process one frame to execute the command
    simulation.update(0.05);

    // Then: Creature count should reflect the new spawn
    let final_count = simulation.creature_count();
    assert_eq!(
        final_count, 1,
        "Creature count should be 1 after dev spawn (was {}, BUG!)",
        final_count
    );
}

/// Test that multiple dev spawns increment creature count correctly
#[test]
fn multiple_dev_spawns_increment_count() {
    // Given: Simulation with 2 initial creatures
    let (mut simulation, sender) = setup_test_simulation_with_dev_commands();

    // Spawn 2 creatures normally (via spawn_crit)
    simulation.spawn_crit(
        speciate::CritBuilder::new()
            .at(0.0, 0.0)
            .with_all_capabilities()
    );
    simulation.spawn_crit(
        speciate::CritBuilder::new()
            .at(10.0, 0.0)
            .with_all_capabilities()
    );

    assert_eq!(simulation.creature_count(), 2, "Should have 2 initial creatures");

    // When: Spawn 3 more via dev commands
    for i in 0..3 {
        sender.send(DevCommand::Spawn {
            x: (i * 10) as f32,
            y: 0.0,
            behavior: "catatonic".to_string(),
            target_x: None,
            target_y: None,
            energy: None,
            max_speed: None,
        }).expect("Failed to send command");
    }

    simulation.update(0.05);

    // Then: Count should be 5 (2 initial + 3 dev spawns)
    assert_eq!(
        simulation.creature_count(),
        5,
        "Should have 5 total creatures (2 initial + 3 dev spawned)"
    );
}

/// Test that dev spawned creatures appear in ECS queries
/// (Snapshots use ECS queries, so if query finds them, snapshot will too)
#[test]
fn dev_spawned_creatures_queryable() {
    use speciate::simulation::components::CritId;
    use bevy_ecs::prelude::*;

    // Given: Simulation with 1 normal creature
    let (mut simulation, sender) = setup_test_simulation_with_dev_commands();

    simulation.spawn_crit(
        speciate::CritBuilder::new()
            .at(0.0, 0.0)
            .with_all_capabilities()
    );

    // When: Spawn 2 via dev command
    for _ in 0..2 {
        sender.send(DevCommand::Spawn {
            x: 50.0,
            y: 50.0,
            behavior: "wandering".to_string(),
            target_x: None,
            target_y: None,
            energy: None,
            max_speed: None,
        }).expect("Failed to send command");
    }

    simulation.update(0.05);

    // Then: ECS query should find all 3 creatures
    let mut query_state: QueryState<&CritId> = simulation.world_mut().query();
    let creature_count = query_state.iter(simulation.world()).count();

    assert_eq!(
        creature_count,
        3,
        "ECS query should find all creatures (1 normal + 2 dev spawned)"
    );
}

/// Test that dev clear command works
#[test]
fn dev_clear_removes_all_creatures() {
    // Given: Simulation with creatures
    let (mut simulation, sender) = setup_test_simulation_with_dev_commands();

    simulation.spawn_crit(
        speciate::CritBuilder::new()
            .at(0.0, 0.0)
            .with_all_capabilities()
    );
    simulation.spawn_crit(
        speciate::CritBuilder::new()
            .at(10.0, 10.0)
            .with_all_capabilities()
    );

    assert_eq!(simulation.creature_count(), 2);

    // When: Send clear command
    sender.send(DevCommand::Clear).expect("Failed to send command");

    simulation.update(0.05);

    // Then: All creatures removed
    assert_eq!(simulation.creature_count(), 0, "All creatures should be cleared");
}

/// Test that dev speed command changes delta time
#[test]
fn dev_speed_command_changes_delta_time() {
    use speciate::simulation::core::components::DeltaTime;

    // Given: Simulation with default speed
    let (mut simulation, sender) = setup_test_simulation_with_dev_commands();

    let initial_dt = simulation.world().resource::<DeltaTime>().0;
    assert_eq!(initial_dt, 0.05, "Default delta time should be 0.05 (20 Hz)");

    // When: Send speed command for 2x speed
    sender.send(DevCommand::Speed { multiplier: 2.0 }).expect("Failed to send command");

    simulation.update(0.05);

    // Then: Delta time doubled
    let final_dt = simulation.world().resource::<DeltaTime>().0;
    assert_eq!(final_dt, 0.1, "Delta time should be 0.1 (2x speed)");
}
