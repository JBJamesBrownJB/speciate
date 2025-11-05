//! Background worker thread for non-blocking snapshot saves
//!
//! This module handles automatic snapshot saving in a separate thread to prevent
//! blocking the main simulation loop during file I/O operations.

use crate::config::SnapshotConfig;
use crate::snapshot::WorldSnapshot;
use chrono::Local;
use log::{error, info, warn};
use std::fs;
use std::path::PathBuf;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::thread::{self, JoinHandle};

/// Message types sent to the snapshot worker thread
enum WorkerMessage {
    /// Save a snapshot with the given type
    SaveSnapshot(WorldSnapshot, SnapshotType),
    /// Shutdown the worker thread
    Shutdown,
}

/// Type of snapshot being saved
#[derive(Debug, Clone, Copy)]
pub enum SnapshotType {
    /// Periodic automatic snapshot (subject to cleanup)
    Periodic,
    /// Shutdown snapshot (never auto-deleted)
    Shutdown,
}

/// Handle to the snapshot worker thread
pub struct SnapshotWorker {
    sender: Sender<WorkerMessage>,
    thread_handle: Option<JoinHandle<()>>,
}

impl SnapshotWorker {
    /// Start a new snapshot worker thread
    pub fn start(config: SnapshotConfig) -> Self {
        let (sender, receiver) = channel();

        let thread_handle = thread::spawn(move || {
            worker_thread_loop(receiver, config);
        });

        Self {
            sender,
            thread_handle: Some(thread_handle),
        }
    }

    /// Send a snapshot to be saved (non-blocking)
    pub fn save_snapshot(&self, snapshot: WorldSnapshot, snapshot_type: SnapshotType) {
        if let Err(e) = self.sender.send(WorkerMessage::SaveSnapshot(snapshot, snapshot_type)) {
            error!("Failed to send snapshot to worker thread: {}", e);
        }
    }

    /// Shutdown the worker thread gracefully (blocks until all pending saves complete)
    pub fn shutdown(mut self) {
        // Send shutdown message
        if let Err(e) = self.sender.send(WorkerMessage::Shutdown) {
            warn!("Failed to send shutdown message to worker: {}", e);
        }

        // Wait for thread to finish
        if let Some(handle) = self.thread_handle.take() {
            if let Err(e) = handle.join() {
                error!("Snapshot worker thread panicked: {:?}", e);
            }
        }
    }
}

/// Main loop for the snapshot worker thread
fn worker_thread_loop(receiver: Receiver<WorkerMessage>, config: SnapshotConfig) {
    // Create snapshots directory if it doesn't exist
    if let Err(e) = fs::create_dir_all("snapshots") {
        error!("Failed to create snapshots directory: {}", e);
        return;
    }

    loop {
        match receiver.recv() {
            Ok(WorkerMessage::SaveSnapshot(snapshot, snapshot_type)) => {
                handle_save_snapshot(snapshot, snapshot_type, &config);
            }
            Ok(WorkerMessage::Shutdown) => {
                info!("Snapshot worker shutting down gracefully");
                break;
            }
            Err(_) => {
                // Channel closed, shutdown
                break;
            }
        }
    }
}

/// Handle saving a single snapshot
fn handle_save_snapshot(snapshot: WorldSnapshot, snapshot_type: SnapshotType, config: &SnapshotConfig) {
    match snapshot_type {
        SnapshotType::Periodic => {
            // Generate timestamp for filename
            let timestamp = Local::now().format("%Y-%m-%d_%H-%M-%S");
            let timestamped_path = PathBuf::from(format!("snapshots/simulation_{}.msgpack", timestamp));

            // Save timestamped snapshot
            match snapshot.save_to_file(&timestamped_path) {
                Ok(_) => {
                    info!(
                        "Saved periodic snapshot: {} ({} creatures)",
                        timestamped_path.display(),
                        snapshot.metadata.creature_count
                    );
                }
                Err(e) => {
                    error!("Failed to save snapshot {}: {}", timestamped_path.display(), e);
                    return; // Don't update latest if save failed
                }
            }

            // Update latest.msgpack
            let latest_path = PathBuf::from("snapshots/latest.msgpack");
            if let Err(e) = snapshot.save_to_file(&latest_path) {
                error!("Failed to update latest snapshot: {}", e);
            }

            // Cleanup old snapshots
            cleanup_old_snapshots(config.keep_last_n);
        }
        SnapshotType::Shutdown => {
            // On shutdown, only update latest.msgpack (no timestamped file)
            let latest_path = PathBuf::from("snapshots/latest.msgpack");
            match snapshot.save_to_file(&latest_path) {
                Ok(_) => {
                    info!(
                        "Saved shutdown snapshot to latest.msgpack ({} creatures)",
                        snapshot.metadata.creature_count
                    );
                }
                Err(e) => {
                    error!("Failed to save shutdown snapshot: {}", e);
                }
            }
        }
    }
}

/// Remove old periodic snapshots, keeping only the last N
fn cleanup_old_snapshots(keep_last_n: usize) {
    let snapshots_dir = PathBuf::from("snapshots");

    // Read all files in snapshots directory
    let entries = match fs::read_dir(&snapshots_dir) {
        Ok(entries) => entries,
        Err(e) => {
            warn!("Failed to read snapshots directory: {}", e);
            return;
        }
    };

    // Collect all periodic snapshot files (simulation_*.msgpack)
    let mut periodic_snapshots: Vec<PathBuf> = entries
        .filter_map(|entry| entry.ok())
        .map(|entry| entry.path())
        .filter(|path| {
            path.is_file()
                && path
                    .file_name()
                    .and_then(|name| name.to_str())
                    .map(|name| name.starts_with("simulation_") && name.ends_with(".msgpack"))
                    .unwrap_or(false)
        })
        .collect();

    // Sort by filename (timestamp is in filename, so lexical sort works)
    periodic_snapshots.sort();

    // Keep only the last N snapshots
    if periodic_snapshots.len() > keep_last_n {
        let to_delete = periodic_snapshots.len() - keep_last_n;

        for path in periodic_snapshots.iter().take(to_delete) {
            match fs::remove_file(path) {
                Ok(_) => {
                    info!("Deleted old snapshot: {}", path.display());
                }
                Err(e) => {
                    warn!("Failed to delete old snapshot {}: {}", path.display(), e);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_snapshot_config_default() {
        let config = SnapshotConfig::default();
        assert!(config.enabled);
        assert_eq!(config.interval_secs, 300);
        assert_eq!(config.keep_last_n, 10);
    }

    #[test]
    fn test_cleanup_with_no_files() {
        // Should not panic when directory is empty
        cleanup_old_snapshots(10);
    }

    #[test]
    fn test_cleanup_with_fewer_than_keep() {
        // Create test directory
        let test_dir = PathBuf::from("snapshots");
        fs::create_dir_all(&test_dir).ok();

        // Clean up any existing simulation snapshot files first (test isolation)
        if let Ok(entries) = fs::read_dir(&test_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                    if name.starts_with("simulation_") && name.ends_with(".msgpack") {
                        fs::remove_file(path).ok();
                    }
                }
            }
        }

        // Create 3 test snapshots
        for i in 1..=3 {
            let path = test_dir.join(format!("simulation_2025-01-0{}_12-00-00.msgpack", i));
            fs::write(&path, b"test").ok();
        }

        // Cleanup with keep_last_n = 10 (should delete nothing)
        cleanup_old_snapshots(10);

        // Verify all 3 still exist
        let count = fs::read_dir(&test_dir)
            .unwrap()
            .filter(|e| {
                e.as_ref()
                    .unwrap()
                    .path()
                    .file_name()
                    .unwrap()
                    .to_str()
                    .unwrap()
                    .starts_with("simulation_")
            })
            .count();

        assert_eq!(count, 3);

        // Cleanup test files
        for i in 1..=3 {
            let path = test_dir.join(format!("simulation_2025-01-0{}_12-00-00.msgpack", i));
            fs::remove_file(path).ok();
        }
    }
}
