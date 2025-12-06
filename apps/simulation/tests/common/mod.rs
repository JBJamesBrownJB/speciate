//! Common test utilities for integration tests

#![allow(dead_code)]

use speciate::config::SaveStateConfig;
use speciate::simulation::{Simulation, SimulationBuilder};
use speciate::{spawn_creature, CreatureSpawnRequest};
use std::fs;
use std::path::PathBuf;

/// Create a test simulation with specified number of creatures
pub fn setup_test_simulation(creature_count: usize) -> Simulation {
    let mut simulation = SimulationBuilder::new()
        .set_boundaries(180.0, 130.0)
        .build();

    for _ in 0..creature_count {
        spawn_creature(&mut simulation, CreatureSpawnRequest::new());
    }

    simulation
}

/// Create a save state config with fast intervals for testing
pub fn test_save_state_config(interval_secs: u64, keep_last_n: usize) -> SaveStateConfig {
    SaveStateConfig {
        enabled: true,
        interval_secs,
        keep_last_n,
        save_dir: PathBuf::from("save-states"),
    }
}

/// Create a save state config with custom save directory (for test isolation)
pub fn test_save_state_config_with_dir(interval_secs: u64, keep_last_n: usize, save_dir: PathBuf) -> SaveStateConfig {
    SaveStateConfig {
        enabled: true,
        interval_secs,
        keep_last_n,
        save_dir,
    }
}

/// Count total number of save states (all .msgpack files) in default directory
pub fn count_save_states() -> usize {
    count_save_states_in_dir(&PathBuf::from("save-states"))
}

/// Count total number of save states (all .msgpack files) in specified directory
pub fn count_save_states_in_dir(snapshots_dir: &PathBuf) -> usize {
    if !snapshots_dir.exists() {
        return 0;
    }

    fs::read_dir(snapshots_dir)
        .unwrap()
        .filter_map(|entry| entry.ok())
        .filter(|entry| {
            let path = entry.path();
            path.is_file()
                && path
                    .file_name()
                    .and_then(|name| name.to_str())
                    .map(|name| name.ends_with(".msgpack"))
                    .unwrap_or(false)
        })
        .count()
}

/// Count number of periodic save states (DEPRECATED - returns same as count_save_states())
///
/// In the new simplified system, there's no distinction between periodic and shutdown saves.
/// This function is kept for backwards compatibility with existing tests.
#[deprecated(note = "Use count_save_states() instead - no distinction between periodic/shutdown")]
pub fn count_periodic_save_states() -> usize {
    count_save_states()
}

/// Count number of shutdown save states (DEPRECATED - always returns 0)
///
/// In the new simplified system, shutdown saves create the same timestamped files as periodic saves.
/// This function is kept for backwards compatibility with existing tests.
#[deprecated(note = "No longer distinguishes shutdown saves - use count_save_states()")]
pub fn count_shutdown_save_states() -> usize {
    0  // No separate shutdown saves in new system
}

/// Check if latest.msgpack exists (DEPRECATED - always returns false)
///
/// In the new simplified system, we no longer maintain a latest.msgpack file.
/// All saves are timestamped files. Use get_most_recent_save_state() instead.
#[deprecated(note = "latest.msgpack no longer exists - use get_most_recent_save_state()")]
pub fn latest_save_state_exists() -> bool {
    false  // latest.msgpack no longer exists in new system
}

/// Get the most recent save state file path (sorted by timestamp in filename) from default directory
pub fn get_most_recent_save_state() -> Option<PathBuf> {
    get_most_recent_save_state_in_dir(&PathBuf::from("save-states"))
}

/// Get the most recent save state file path from specified directory
pub fn get_most_recent_save_state_in_dir(snapshots_dir: &PathBuf) -> Option<PathBuf> {
    if !snapshots_dir.exists() {
        return None;
    }

    let mut save_files: Vec<PathBuf> = fs::read_dir(snapshots_dir)
        .ok()?
        .filter_map(|entry| entry.ok())
        .filter_map(|entry| {
            let path = entry.path();
            if path.is_file()
                && path
                    .file_name()
                    .and_then(|name| name.to_str())
                    .map(|name| name.ends_with(".msgpack"))
                    .unwrap_or(false)
            {
                Some(path)
            } else {
                None
            }
        })
        .collect();

    // Sort by filename (which is timestamp: YYYY-MM-DD_HH-MM-SS.msgpack)
    save_files.sort();

    // Return the last one (most recent)
    save_files.last().cloned()
}

/// Clean up all test save states
pub fn cleanup_test_save_states() {
    let snapshots_dir = PathBuf::from("save-states");

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
