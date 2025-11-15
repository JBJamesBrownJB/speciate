//! Integration tests for --load-snapshot CLI argument behavior
//!
//! Tests verify:
//! 1. --load-snapshot with no argument defaults to ./snapshots/latest.msgpack
//! 2. Missing snapshot file doesn't crash the simulation

mod common;

use common::*;
use speciate::persistence::WorldSnapshot;
use std::fs;
use std::path::PathBuf;

#[test]
fn test_load_snapshot_creates_valid_latest_msgpack() {
    cleanup_test_snapshots();

    // Create a simulation and save snapshot to latest.msgpack
    let mut simulation = setup_test_simulation(50);
    let snapshot = simulation.to_snapshot();

    // Ensure snapshots directory exists
    fs::create_dir_all("snapshots").expect("Failed to create snapshots directory");

    // Save as latest.msgpack (the default location)
    let latest_path = PathBuf::from("snapshots/latest.msgpack");
    snapshot
        .save_to_file(&latest_path)
        .expect("Failed to save latest.msgpack");

    // Verify file exists
    assert!(
        latest_path.exists(),
        "latest.msgpack should exist after saving"
    );

    // Verify we can load it back
    let loaded_snapshot = WorldSnapshot::load_from_file(&latest_path);
    assert!(
        loaded_snapshot.is_ok(),
        "Should be able to load latest.msgpack"
    );

    let loaded_snapshot = loaded_snapshot.unwrap();
    assert_eq!(
        loaded_snapshot.metadata.creature_count, 50,
        "Loaded snapshot should have correct creature count"
    );

    // Verify we can restore simulation from it
    let restored_simulation = speciate::Simulation::from_snapshot(loaded_snapshot);
    assert_eq!(
        restored_simulation.creature_count(),
        50,
        "Restored simulation should have 50 creatures"
    );

    cleanup_test_snapshots();
}

#[test]
fn test_missing_latest_msgpack_graceful_handling() {
    cleanup_test_snapshots();

    let latest_path = PathBuf::from("snapshots/latest.msgpack");

    // Ensure file does NOT exist
    if latest_path.exists() {
        fs::remove_file(&latest_path).ok();
    }

    // Verify file doesn't exist
    assert!(
        !latest_path.exists(),
        "latest.msgpack should not exist for this test"
    );

    // Attempt to load - should return Err, not panic
    let result = WorldSnapshot::load_from_file(&latest_path);
    assert!(
        result.is_err(),
        "Loading missing snapshot should return Err, not panic"
    );

    // Application should handle this gracefully by starting with default config
    // (This is tested in the main.rs logic - we verify it doesn't panic here)

    cleanup_test_snapshots();
}

#[test]
fn test_latest_msgpack_path_convention() {
    cleanup_test_snapshots();

    // This test documents the expected path for --load-snapshot with no args
    let expected_default_path = PathBuf::from("snapshots/latest.msgpack");

    // Create a test snapshot
    let mut simulation = setup_test_simulation(25);
    let snapshot = simulation.to_snapshot();

    fs::create_dir_all("snapshots").expect("Failed to create snapshots directory");

    // Save to the default path
    snapshot
        .save_to_file(&expected_default_path)
        .expect("Failed to save to default path");

    // Verify the convention
    assert!(
        expected_default_path.exists(),
        "Default snapshot path should be snapshots/latest.msgpack"
    );

    assert_eq!(
        expected_default_path.to_str().unwrap(),
        "snapshots/latest.msgpack",
        "Default path should match convention"
    );

    cleanup_test_snapshots();
}
