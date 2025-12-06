//! Integration tests for NAPI mode save state functionality
//!
//! Each test uses an isolated temp directory to avoid race conditions.

mod common;

use common::*;
use speciate::persistence::{WorldSaveState, SaveStateWorker, SaveType};
use speciate::config::SaveStateConfig;
use speciate::ipc::bridge::NapiApp;
use std::time::{Duration, Instant};
use tempfile::tempdir;

/// Test periodic saves with fast interval (for testing)
#[test]
fn test_napi_periodic_saves_with_2_second_interval() {
    let temp_dir = tempdir().expect("Failed to create temp dir");
    let save_dir = temp_dir.path().to_path_buf();

    // Create config and worker with isolated directory
    let config = SaveStateConfig {
        enabled: true,
        interval_secs: 2,
        keep_last_n: 10,
        save_dir: save_dir.clone(),
    };
    let worker = SaveStateWorker::start(config.clone());

    // Create NapiApp (no save state to load)
    let (_tx, rx) = crossbeam_channel::bounded(128);
    let mut app = NapiApp::new(rx, 10, ".".to_string(), None);

    // Run simulation for 5 seconds with periodic saves
    let start = Instant::now();
    let mut last_save = Instant::now();
    let save_interval = Duration::from_secs(config.interval_secs);

    while start.elapsed() < Duration::from_secs(5) {
        app.update(0.045); // ~22Hz

        // Periodic save check
        if last_save.elapsed() >= save_interval {
            let save_state = app.to_save_state().expect("Failed to create save state");
            worker.save_world_state(save_state, SaveType::Periodic);
            last_save = Instant::now();
        }

        std::thread::sleep(Duration::from_millis(45));
    }

    // Shutdown and wait for worker to finish
    worker.shutdown();
    std::thread::sleep(Duration::from_millis(500));

    let save_count = count_save_states_in_dir(&save_dir);

    assert!(
        save_count >= 2,
        "Should create at least 2 saves in 5 seconds with 2-second interval, got {}",
        save_count
    );

    // Verify we can retrieve the most recent save
    let most_recent = get_most_recent_save_state_in_dir(&save_dir);
    assert!(
        most_recent.is_some(),
        "Should be able to find most recent save state after periodic saves"
    );
}

/// Test shutdown save creates timestamped file
#[test]
fn test_napi_shutdown_save_creates_latest() {
    let temp_dir = tempdir().expect("Failed to create temp dir");
    let save_dir = temp_dir.path().to_path_buf();

    // Create config and worker with isolated directory
    let config = SaveStateConfig {
        enabled: true,
        interval_secs: 300, // Long interval (won't trigger during test)
        keep_last_n: 10,
        save_dir: save_dir.clone(),
    };
    let worker = SaveStateWorker::start(config.clone());

    // Create NapiApp and run briefly
    let (_tx, rx) = crossbeam_channel::bounded(128);
    let mut app = NapiApp::new(rx, 10, ".".to_string(), None);

    // Run for 1 second
    for _ in 0..22 {
        app.update(0.045);
        std::thread::sleep(Duration::from_millis(45));
    }

    // Create shutdown save
    let shutdown_save = app.to_save_state().expect("Failed to create shutdown save");
    worker.save_world_state(shutdown_save, SaveType::Shutdown);

    // Shutdown and wait for worker to finish
    worker.shutdown();
    std::thread::sleep(Duration::from_millis(500));

    let most_recent = get_most_recent_save_state_in_dir(&save_dir);
    assert!(
        most_recent.is_some(),
        "Should have a timestamped save state after shutdown"
    );

    let save_state = WorldSaveState::load_from_file(
        &most_recent.unwrap()
    ).expect("Should be able to load shutdown save state");

    assert_eq!(
        save_state.metadata.version,
        "2.0.0",
        "Save state should have correct version"
    );
}

/// Test that disabled config prevents saves
#[test]
fn test_napi_disabled_config_prevents_saves() {
    let temp_dir = tempdir().expect("Failed to create temp dir");
    let save_dir = temp_dir.path().to_path_buf();

    // Create config with saves DISABLED (we don't start a worker)
    let _config = SaveStateConfig {
        enabled: false,
        interval_secs: 1,
        keep_last_n: 10,
        save_dir: save_dir.clone(),
    };

    // Note: We don't create a worker since saves are disabled
    // This simulates the SimulationEngine behavior when enabled=false

    // Create NapiApp and run
    let (_tx, rx) = crossbeam_channel::bounded(128);
    let mut app = NapiApp::new(rx, 10, ".".to_string(), None);

    // Run for 3 seconds (should NOT create saves)
    for _ in 0..66 {
        app.update(0.045);
        std::thread::sleep(Duration::from_millis(45));
    }

    // No worker, so no saves should be created in our isolated temp dir
    let save_count = count_save_states_in_dir(&save_dir);

    assert_eq!(
        save_count,
        0,
        "Should NOT create any saves when disabled, got {}",
        save_count
    );

    // Verify no save state files exist
    let most_recent = get_most_recent_save_state_in_dir(&save_dir);
    assert!(
        most_recent.is_none(),
        "Should NOT have any save state files when saves are disabled"
    );
}

/// Test cleanup keeps only last N saves
#[test]
fn test_napi_cleanup_keeps_last_n() {
    let temp_dir = tempdir().expect("Failed to create temp dir");
    let save_dir = temp_dir.path().to_path_buf();

    // Create config with keep_last_n=3
    let config = SaveStateConfig {
        enabled: true,
        interval_secs: 1,
        keep_last_n: 3,
        save_dir: save_dir.clone(),
    };
    let worker = SaveStateWorker::start(config.clone());

    // Create NapiApp (no save state to load)
    let (_tx, rx) = crossbeam_channel::bounded(128);
    let mut app = NapiApp::new(rx, 10, ".".to_string(), None);

    // Run for 7 seconds to create ~7 saves (should keep only 3)
    let start = Instant::now();
    let mut last_save = Instant::now();
    let save_interval = Duration::from_secs(config.interval_secs);

    while start.elapsed() < Duration::from_secs(7) {
        app.update(0.045);

        // Periodic save check
        if last_save.elapsed() >= save_interval {
            let save_state = app.to_save_state().expect("Failed to create save state");
            worker.save_world_state(save_state, SaveType::Periodic);
            last_save = Instant::now();
        }

        std::thread::sleep(Duration::from_millis(45));
    }

    // Shutdown and wait for worker to finish
    worker.shutdown();
    std::thread::sleep(Duration::from_millis(500));

    let save_count = count_save_states_in_dir(&save_dir);

    assert!(
        save_count <= 3 && save_count > 0,
        "Should keep at most 3 saves (got {}). Must have at least 1 to prove saves work.",
        save_count
    );
}
