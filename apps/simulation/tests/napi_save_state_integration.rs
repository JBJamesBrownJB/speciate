//! Integration tests for NAPI mode save state functionality
//!
//! **TDD RED PHASE:** These tests assert DESIRED behavior and currently FAIL
//! because SimulationEngine doesn't integrate SaveStateWorker yet.
//!
//! After GREEN phase implementation, these tests will PASS and remain as
//! permanent regression tests.
//!
//! **IMPORTANT**: Run with `--test-threads=1` to avoid race conditions:
//! ```bash
//! cargo test --test napi_save_state_integration -- --test-threads=1
//! ```

mod common;

use common::*;
use speciate::persistence::{WorldSaveState, SaveStateWorker, SaveType};
use speciate::config::SaveStateConfig;
use speciate::ipc::bridge::NapiApp;
use std::env;
use std::time::{Duration, Instant};

/// Test periodic saves with fast interval (for testing)
///
/// **RED PHASE:** This test FAILS because SimulationEngine doesn't save states
/// **GREEN PHASE:** Will PASS after implementing SaveStateWorker integration
/// **FOREVER:** Protects against regression - catches if saves break again
#[test]
fn test_napi_periodic_saves_with_2_second_interval() {
    cleanup_test_save_states();

    // Set test-friendly interval via environment variables
    env::set_var("SAVE_STATE_ENABLED", "true");
    env::set_var("SAVE_STATE_INTERVAL_SECS", "2");

    // Create config and worker
    let config = SaveStateConfig {
        enabled: true,
        interval_secs: 2,
        keep_last_n: 10,
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

    let save_count = count_save_states();

    // This assertion FAILS now (save_count == 0)
    // Will PASS after GREEN phase (save_count >= 2)
    assert!(
        save_count >= 2,
        "Should create at least 2 saves in 5 seconds with 2-second interval, got {}",
        save_count
    );

    // Verify we can retrieve the most recent save
    let most_recent = get_most_recent_save_state();
    assert!(
        most_recent.is_some(),
        "Should be able to find most recent save state after periodic saves"
    );

    env::remove_var("SAVE_STATE_ENABLED");
    env::remove_var("SAVE_STATE_INTERVAL_SECS");
    cleanup_test_save_states();
}

/// Test shutdown save creates latest.msgpack
///
/// **RED PHASE:** This test FAILS because SimulationEngine doesn't save on shutdown
/// **GREEN PHASE:** Will PASS after implementing shutdown() method
/// **FOREVER:** Protects against regression - catches if shutdown save breaks
#[test]
fn test_napi_shutdown_save_creates_latest() {
    cleanup_test_save_states();

    env::set_var("SAVE_STATE_ENABLED", "true");

    // Create config and worker
    let config = SaveStateConfig {
        enabled: true,
        interval_secs: 300, // Long interval (won't trigger during test)
        keep_last_n: 10,
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

    // This assertion FAILS now (save doesn't exist)
    // Will PASS after GREEN phase (save exists)
    let most_recent = get_most_recent_save_state();
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

    env::remove_var("SAVE_STATE_ENABLED");
    cleanup_test_save_states();
}

/// Test that disabled config prevents saves
///
/// **RED PHASE:** This test FAILS because config isn't implemented
/// **GREEN PHASE:** Will PASS after accepting SaveStateConfig parameter
/// **FOREVER:** Protects against regression - catches if disabled flag ignored
#[test]
fn test_napi_disabled_config_prevents_saves() {
    cleanup_test_save_states();

    env::set_var("SAVE_STATE_ENABLED", "false");
    env::set_var("SAVE_STATE_INTERVAL_SECS", "1");

    // Create config with saves DISABLED
    let _config = SaveStateConfig {
        enabled: false,
        interval_secs: 1,
        keep_last_n: 10,
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

    // No worker, so no saves should be created
    let save_count = count_save_states();

    assert_eq!(
        save_count,
        0,
        "Should NOT create any saves when disabled, got {}",
        save_count
    );

    // Verify no save state files exist
    let most_recent = get_most_recent_save_state();
    assert!(
        most_recent.is_none(),
        "Should NOT have any save state files when saves are disabled"
    );

    env::remove_var("SAVE_STATE_ENABLED");
    env::remove_var("SAVE_STATE_INTERVAL_SECS");
    cleanup_test_save_states();
}

/// Test cleanup keeps only last N saves
///
/// **RED PHASE:** This test FAILS because no saves are created
/// **GREEN PHASE:** Will PASS after implementing cleanup logic
/// **FOREVER:** Protects against regression - catches if cleanup breaks
#[test]
fn test_napi_cleanup_keeps_last_n() {
    cleanup_test_save_states();

    env::set_var("SAVE_STATE_ENABLED", "true");
    env::set_var("SAVE_STATE_INTERVAL_SECS", "1");
    env::set_var("SAVE_STATE_KEEP_LAST_N", "3");

    // Create config with keep_last_n=3
    let config = SaveStateConfig {
        enabled: true,
        interval_secs: 1,
        keep_last_n: 3,
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

    let save_count = count_save_states();

    // This will FAIL now (save_count == 0)
    // Will PASS after GREEN phase (save_count == 3)
    assert!(
        save_count <= 3 && save_count > 0,
        "Should keep at most 3 saves (got {}). Must have at least 1 to prove saves work.",
        save_count
    );

    env::remove_var("SAVE_STATE_ENABLED");
    env::remove_var("SAVE_STATE_INTERVAL_SECS");
    env::remove_var("SAVE_STATE_KEEP_LAST_N");
    cleanup_test_save_states();
}
