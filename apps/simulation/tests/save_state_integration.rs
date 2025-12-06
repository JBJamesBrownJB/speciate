//! Integration tests for automatic save state functionality
//!
//! **IMPORTANT**: Run with `--test-threads=1` to avoid race conditions:
//! ```bash
//! cargo test --test save_state_integration -- --test-threads=1
//! ```

mod common;

use common::*;
use speciate::config::SaveStateConfig;
use speciate::simulation::Simulation;
use speciate::persistence::WorldSaveState;
use speciate::persistence::{SaveType, SaveStateWorker};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

#[test]
fn test_snapshot_worker_creation_and_shutdown() {
    cleanup_test_save_states();

    let config = test_save_state_config(60, 5);
    let worker = SaveStateWorker::start(config);

    // Worker should start successfully
    worker.shutdown();

    cleanup_test_save_states();
}

#[test]
fn test_snapshot_cleanup_keeps_last_n() {
    cleanup_test_save_states();

    // Config: 1 second intervals, keep last 5
    let config = test_save_state_config(1, 5);
    let worker = SaveStateWorker::start(config.clone());

    let mut simulation = setup_test_simulation(10);

    // Run for 7 seconds → should create 7 snapshots, but only keep last 5
    let start = std::time::Instant::now();
    let interval = Duration::from_secs(config.interval_secs);
    let mut last_snapshot = std::time::Instant::now();
    let mut snapshot_count = 0;

    while start.elapsed() < Duration::from_secs(7) {
        simulation.update(0.016);

        if last_snapshot.elapsed() >= interval {
            let snapshot = simulation.to_save_state().expect("Failed to create save state");
            worker.save_world_state(snapshot, SaveType::Periodic);
            last_snapshot = std::time::Instant::now();
            snapshot_count += 1;
        }

        thread::sleep(Duration::from_millis(100));
    }

    // Shutdown worker to ensure all pending work completes
    worker.shutdown();

    // Give a moment for final cleanup
    thread::sleep(Duration::from_millis(500));

    let save_count = count_save_states();

    // Check that we created snapshots and cleanup is working
    if snapshot_count > 0 {
        assert!(
            save_count <= 5,
            "Should keep at most 5 save states, got {} (created {})",
            save_count,
            snapshot_count
        );

        // Verify we can find the most recent save state
        let most_recent = get_most_recent_save_state();
        assert!(
            most_recent.is_some(),
            "Should have at least one save state file"
        );
    }

    cleanup_test_save_states();
}

#[test]
fn test_periodic_and_shutdown_both_create_timestamped_files() {
    let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
    let save_dir = temp_dir.path().to_path_buf();

    let config = test_save_state_config_with_dir(60, 5, save_dir.clone());
    let worker = SaveStateWorker::start(config);

    let mut simulation = setup_test_simulation(25);

    // Create one periodic save
    let snapshot = simulation.to_save_state().expect("Failed to create save state");
    worker.save_world_state(snapshot, SaveType::Periodic);

    // Sleep to ensure different timestamp (1+ second granularity)
    thread::sleep(Duration::from_millis(1100));

    // Create one shutdown save
    let shutdown_snapshot = simulation.to_save_state().expect("Failed to create save state");
    worker.save_world_state(shutdown_snapshot, SaveType::Shutdown);

    // Give worker time to finish
    thread::sleep(Duration::from_millis(500));

    // Both periodic and shutdown saves create timestamped files
    let save_count = count_save_states_in_dir(&save_dir);
    assert_eq!(
        save_count, 2,
        "Should have 2 timestamped save files (1 periodic + 1 shutdown)"
    );

    // Verify we can get the most recent one
    let most_recent = get_most_recent_save_state_in_dir(&save_dir);
    assert!(
        most_recent.is_some(),
        "Should be able to find most recent save state"
    );

    worker.shutdown();
}

#[test]
fn test_snapshot_preserves_creature_count() {
    cleanup_test_save_states();

    let config = test_save_state_config(60, 5);
    let worker = SaveStateWorker::start(config);

    let simulation = setup_test_simulation(75);
    let original_count = simulation.creature_count();

    // Take snapshot
    let mut sim_clone = simulation;
    let snapshot = sim_clone.to_save_state().expect("Failed to create save state");

    // Save and reload
    worker.save_world_state(snapshot.clone(), SaveType::Periodic);
    thread::sleep(Duration::from_millis(500));

    // Verify snapshot metadata
    assert_eq!(
        snapshot.metadata.creature_count, original_count,
        "Snapshot metadata should match original count"
    );

    // Restore and verify
    let restored = Simulation::from_save_state(snapshot).expect("Failed to restore from save state");
    assert_eq!(
        restored.creature_count(),
        original_count,
        "Restored simulation should have same creature count"
    );

    worker.shutdown();
    cleanup_test_save_states();
}

#[test]
fn test_graceful_shutdown_flag() {
    cleanup_test_save_states();

    let running = Arc::new(AtomicBool::new(true));
    let running_clone = running.clone();

    let config = test_save_state_config(60, 5);
    let worker = SaveStateWorker::start(config);

    let mut simulation = setup_test_simulation(10);

    // Simulate running for a short time then shutting down
    let thread_handle = thread::spawn(move || {
        thread::sleep(Duration::from_millis(500));
        running_clone.store(false, Ordering::Relaxed);
    });

    // Simulate main loop checking the flag
    let start = std::time::Instant::now();
    while running.load(Ordering::Relaxed) && start.elapsed() < Duration::from_secs(2) {
        simulation.update(0.016);
        thread::sleep(Duration::from_millis(50));
    }

    // Should have exited loop due to flag
    assert!(
        !running.load(Ordering::Relaxed),
        "Shutdown flag should be set"
    );

    thread_handle.join().unwrap();

    // Create final snapshot (simulating shutdown behavior)
    let final_snapshot = simulation.to_save_state().expect("Failed to create save state");
    worker.save_world_state(final_snapshot, SaveType::Shutdown);

    // Wait for snapshot to be saved with retry loop (more robust than single sleep)
    let start = std::time::Instant::now();
    let max_wait = Duration::from_secs(2);
    let mut save_exists = false;

    while start.elapsed() < max_wait {
        save_exists = get_most_recent_save_state().is_some();
        if save_exists {
            break;
        }
        thread::sleep(Duration::from_millis(100));
    }

    // Verify shutdown snapshot was saved (as timestamped file)
    assert!(
        save_exists,
        "Timestamped save file should exist after shutdown"
    );

    worker.shutdown();
    cleanup_test_save_states();
}

#[test]
fn test_disabled_snapshots_creates_no_files() {
    let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
    let save_dir = temp_dir.path().to_path_buf();

    let config = SaveStateConfig {
        enabled: false,
        interval_secs: 1,
        keep_last_n: 10,
        save_dir: save_dir.clone(),
    };

    let worker = SaveStateWorker::start(config.clone());
    let mut simulation = setup_test_simulation(10);

    // Simulate periodic check (but snapshots are disabled)
    let start = std::time::Instant::now();
    let interval = Duration::from_secs(config.interval_secs);
    let mut last_snapshot = std::time::Instant::now();

    while start.elapsed() < Duration::from_secs(3) {
        simulation.update(0.016);

        // This check would normally trigger a save, but config.enabled is false
        if config.enabled && last_snapshot.elapsed() >= interval {
            let snapshot = simulation.to_save_state().expect("Failed to create save state");
            worker.save_world_state(snapshot, SaveType::Periodic);
            last_snapshot = std::time::Instant::now();
        }

        thread::sleep(Duration::from_millis(100));
    }

    thread::sleep(Duration::from_millis(500));

    // No save states should have been created in our isolated temp dir
    let save_count = count_save_states_in_dir(&save_dir);
    assert_eq!(
        save_count, 0,
        "Should have 0 save states when disabled"
    );

    worker.shutdown();
}

#[test]
fn test_most_recent_save_state_can_be_retrieved() {
    cleanup_test_save_states();

    let config = test_save_state_config(1, 5);
    let worker = SaveStateWorker::start(config.clone());

    let mut simulation = setup_test_simulation(20);

    // Create 3 saves over 3 seconds (ensuring different timestamps)
    for _ in 0..3 {
        thread::sleep(Duration::from_secs(1));
        simulation.update(0.016);
        let snapshot = simulation.to_save_state().expect("Failed to create save state");
        worker.save_world_state(snapshot, SaveType::Periodic);
    }

    thread::sleep(Duration::from_millis(500));

    // Should have 3 timestamped save files
    let save_count = count_save_states();
    assert_eq!(save_count, 3, "Should have 3 timestamped save files");

    // Should be able to get most recent
    let most_recent = get_most_recent_save_state();
    assert!(
        most_recent.is_some(),
        "Should be able to find most recent save state"
    );

    // Load most recent and verify it's valid
    let loaded = WorldSaveState::load_from_file(&most_recent.unwrap());
    assert!(loaded.is_ok(), "Should be able to load most recent save state");

    worker.shutdown();
    cleanup_test_save_states();
}
