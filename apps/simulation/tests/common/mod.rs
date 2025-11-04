//! Common test utilities for integration tests

use speciate::config::SnapshotConfig;
use speciate::simulation::Simulation;
use speciate::spawner::{spawn_creature, CreatureSpawnRequest};
use std::fs;
use std::path::PathBuf;

/// Create a test simulation with specified number of creatures
pub fn setup_test_simulation(creature_count: usize) -> Simulation {
    let mut simulation = Simulation::new();
    simulation.set_boundaries(180.0, 130.0);

    for _ in 0..creature_count {
        spawn_creature(&mut simulation, CreatureSpawnRequest::new());
    }

    simulation
}

/// Create a snapshot config with fast intervals for testing
pub fn test_snapshot_config(interval_secs: u64, keep_last_n: usize) -> SnapshotConfig {
    SnapshotConfig {
        enabled: true,
        interval_secs,
        keep_last_n,
    }
}

/// Count number of periodic snapshots in the snapshots directory
pub fn count_periodic_snapshots() -> usize {
    let snapshots_dir = PathBuf::from("snapshots");

    if !snapshots_dir.exists() {
        return 0;
    }

    fs::read_dir(&snapshots_dir)
        .unwrap()
        .filter_map(|entry| entry.ok())
        .filter(|entry| {
            let path = entry.path();
            path.is_file()
                && path
                    .file_name()
                    .and_then(|name| name.to_str())
                    .map(|name| name.starts_with("simulation_") && name.ends_with(".msgpack"))
                    .unwrap_or(false)
        })
        .count()
}

/// Count number of shutdown snapshots in the snapshots directory
pub fn count_shutdown_snapshots() -> usize {
    let snapshots_dir = PathBuf::from("snapshots");

    if !snapshots_dir.exists() {
        return 0;
    }

    fs::read_dir(&snapshots_dir)
        .unwrap()
        .filter_map(|entry| entry.ok())
        .filter(|entry| {
            let path = entry.path();
            path.is_file()
                && path
                    .file_name()
                    .and_then(|name| name.to_str())
                    .map(|name| name.starts_with("shutdown_") && name.ends_with(".msgpack"))
                    .unwrap_or(false)
        })
        .count()
}

/// Check if latest.msgpack exists
pub fn latest_snapshot_exists() -> bool {
    PathBuf::from("snapshots/latest.msgpack").exists()
}

/// Clean up all test snapshots
pub fn cleanup_test_snapshots() {
    let snapshots_dir = PathBuf::from("snapshots");

    if !snapshots_dir.exists() {
        return;
    }

    if let Ok(entries) = fs::read_dir(&snapshots_dir) {
        for entry in entries.filter_map(|e| e.ok()) {
            let path = entry.path();
            if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("msgpack") {
                fs::remove_file(path).ok();
            }
        }
    }
}

/// Get list of all snapshot files sorted by name (which sorts by timestamp)
pub fn list_snapshots() -> Vec<PathBuf> {
    let snapshots_dir = PathBuf::from("snapshots");

    if !snapshots_dir.exists() {
        return Vec::new();
    }

    let mut snapshots: Vec<PathBuf> = fs::read_dir(&snapshots_dir)
        .unwrap()
        .filter_map(|entry| entry.ok())
        .map(|entry| entry.path())
        .filter(|path| {
            path.is_file()
                && path
                    .file_name()
                    .and_then(|name| name.to_str())
                    .map(|name| name.ends_with(".msgpack"))
                    .unwrap_or(false)
        })
        .collect();

    snapshots.sort();
    snapshots
}
