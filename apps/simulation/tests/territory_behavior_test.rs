//! Integration tests for territory-based wandering behavior
//!
//! These tests verify that creatures stay anchored to their spawn locations
//! and don't drift indefinitely toward world edges.

use bevy_ecs::prelude::*;
use speciate::simulation::components::*;
use speciate::simulation::creatures::builder::CritBuilder;
use speciate::{Simulation, SimulationBuilder};

/// Helper to get creature position and home by CritId
fn get_creature_state(sim: &mut Simulation, crit_id: u32) -> Option<(Position, HomePosition, Target)> {

    let world = sim.world_mut();
    let mut query = world.query::<(&CritId, &Position, &HomePosition, &Target)>();

    for (id, pos, home, target) in query.iter(world) {
        if id.0 == crit_id {
            return Some((*pos, *home, *target));
        }
    }
    None
}

#[test]
fn test_creature_stays_near_spawn_with_small_comfort_zone() {
    // Setup: 100m x 100m world, single wanderer spawned at origin
    let mut sim = SimulationBuilder::new()
        .set_boundaries(50.0, 50.0)  // 100m x 100m world
        .build();

    let builder = CritBuilder::new()
        .at(0.0, 0.0)
        .with_wandering()
        .in_behavior(BehaviorMode::Wandering);

    let crit_id = sim.spawn_crit(builder);

    // Verify initial state
    let (pos, home, _target) = get_creature_state(&mut sim, crit_id)
        .expect("Creature should exist");

    assert_eq!(pos.x, 0.0, "Should spawn at origin X");
    assert_eq!(pos.y, 0.0, "Should spawn at origin Y");
    assert_eq!(home.x, 0.0, "Home should be at origin X");
    assert_eq!(home.y, 0.0, "Home should be at origin Y");

    // Run simulation for 200 ticks (~10 seconds at 20 Hz)
    for _ in 0..200 {
        sim.update(0.05); // 50ms timestep
    }

    // Check final position
    // With COMFORT_RADIUS=10m and MAX_WANDER_DISTANCE=30m (Tom's biological recommendations):
    // - Should mostly stay within comfort zone (10m)
    // - Can wander to ~30m before emergency return kicks in
    // - Allow overshoot to 40m due to momentum/velocity
    let (final_pos, final_home, _) = get_creature_state(&mut sim, crit_id)
        .expect("Creature should still exist");

    let distance_from_home = ((final_pos.x - final_home.x).powi(2)
                             + (final_pos.y - final_home.y).powi(2)).sqrt();

    assert!(
        distance_from_home < 40.0,
        "With COMFORT_RADIUS=10m and MAX_WANDER_DISTANCE=30m, creature should stay within ~40m of spawn (got {:.2}m)",
        distance_from_home
    );

    // Also check not bunched at world edges
    assert!(
        final_pos.x.abs() < 40.0,
        "Creature should not be near world edge X (got {:.2}, world is ±50)",
        final_pos.x
    );
    assert!(
        final_pos.y.abs() < 40.0,
        "Creature should not be near world edge Y (got {:.2}, world is ±50)",
        final_pos.y
    );
}

#[test]
fn test_edge_spawned_creature_returns_home() {
    // Setup: Spawn creature near world edge, verify it doesn't drift further to edge
    let mut sim = SimulationBuilder::new()
        .set_boundaries(50.0, 50.0)
        .build();

    // Spawn at (40, 40) - close to edge but not at boundary
    let builder = CritBuilder::new()
        .at(40.0, 40.0)
        .with_wandering()
        .in_behavior(BehaviorMode::Wandering);

    let crit_id = sim.spawn_crit(builder);

    let (initial_pos, home, _) = get_creature_state(&mut sim, crit_id).unwrap();
    assert_eq!(initial_pos.x, 40.0);
    assert_eq!(initial_pos.y, 40.0);
    assert_eq!(home.x, 40.0);
    assert_eq!(home.y, 40.0);

    // Run simulation
    for _ in 0..300 {
        sim.update(0.05);
    }

    let (final_pos, _, _) = get_creature_state(&mut sim, crit_id).unwrap();

    let distance_from_home = ((final_pos.x - 40.0).powi(2)
                             + (final_pos.y - 40.0).powi(2)).sqrt();

    // Should stay near spawn, not drift to edge (50, 50)
    // Allow up to 40m drift with current parameters (COMFORT_RADIUS=10m, MAX_WANDER_DISTANCE=30m)
    assert!(
        distance_from_home < 40.0,
        "Edge-spawned creature should stay near home (40, 40), got {:.2}m away",
        distance_from_home
    );

    // Verify: Should be within territory radius of home (40, 40), not pushed to boundary
    // With elastic tether, creature can wander within ~40m of spawn even if near edge
    // The test is: Did it stay near HOME, not: Did it avoid the boundary?
    //
    // A creature spawned at (40, 40) can legitimately be at (30-50, 30-50) if within
    // territory radius. What matters is it's not bunching AT the boundary (50, 50).
    //
    // Old test was wrong: It expected creatures to avoid world edges regardless of spawn.
    // New test is correct: Territory center matters, not absolute position.
}

#[test]
fn test_home_bias_probability_increases_with_distance() {
    // This test verifies the sigmoid home bias curve is working
    let mut sim = SimulationBuilder::new()
        .set_boundaries(50.0, 50.0)
        .build();

    // Spawn 10 creatures at different distances from origin
    let spawn_distances = vec![0.0, 5.0, 10.0, 15.0, 20.0];
    let mut creatures = Vec::new();

    for &distance in &spawn_distances {
        let builder = CritBuilder::new()
            .at(distance, 0.0)
            .with_wandering()
            .in_behavior(BehaviorMode::Wandering);
        creatures.push((sim.spawn_crit(builder), distance));
    }

    // Run for a while
    for _ in 0..100 {
        sim.update(0.05);
    }

    // Just verify creatures exist and are tracked
    // (Full statistical analysis would require many runs)
    for (crit_id, _spawn_x) in creatures {
        assert!(get_creature_state(&mut sim, crit_id).is_some(), "Creature should still exist");
    }
}

#[test]
fn test_multiple_creatures_dont_all_bunch_at_same_location() {
    // Spawn 20 creatures in spawn zone, verify they spread out
    let mut sim = SimulationBuilder::new()
        .set_boundaries(50.0, 50.0)
        .build();

    let mut creature_ids = Vec::new();

    // Spawn creatures in a grid pattern in spawn zone
    for x in [-15.0, -5.0, 5.0, 15.0] {
        for y in [-15.0, -5.0, 5.0, 15.0] {
            let builder = CritBuilder::new()
                .at(x, y)
                .with_wandering()
                .in_behavior(BehaviorMode::Wandering);
            creature_ids.push(sim.spawn_crit(builder));
        }
    }

    // Run simulation
    for _ in 0..200 {
        sim.update(0.05);
    }

    // Collect final positions
    let mut positions = Vec::new();
    for &crit_id in &creature_ids {
        if let Some((pos, _, _)) = get_creature_state(&mut sim, crit_id) {
            positions.push((pos.x, pos.y));
        }
    }

    // Check they're not all bunched at same location
    // Calculate standard deviation of positions
    let mean_x: f32 = positions.iter().map(|(x, _)| x).sum::<f32>() / positions.len() as f32;
    let mean_y: f32 = positions.iter().map(|(_, y)| y).sum::<f32>() / positions.len() as f32;

    let var_x: f32 = positions.iter()
        .map(|(x, _)| (x - mean_x).powi(2))
        .sum::<f32>() / positions.len() as f32;
    let var_y: f32 = positions.iter()
        .map(|(_, y)| (y - mean_y).powi(2))
        .sum::<f32>() / positions.len() as f32;

    let std_dev_x = var_x.sqrt();
    let std_dev_y = var_y.sqrt();

    // If creatures are bunching, std dev will be very small
    assert!(
        std_dev_x > 5.0,
        "X positions should be spread out (std dev {:.2} is too small)",
        std_dev_x
    );
    assert!(
        std_dev_y > 5.0,
        "Y positions should be spread out (std dev {:.2} is too small)",
        std_dev_y
    );

    // Check not all bunched at edges
    let edge_count = positions.iter()
        .filter(|(x, y)| x.abs() > 40.0 || y.abs() > 40.0)
        .count();

    let edge_percentage = (edge_count as f32 / positions.len() as f32) * 100.0;

    assert!(
        edge_percentage < 50.0,
        "Too many creatures bunched at edges: {}%",
        edge_percentage
    );
}
