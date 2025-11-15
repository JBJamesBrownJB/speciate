use super::snapshot::WorldSnapshot;
use crate::config::SnapshotConfig;
use chrono::Local;
use log::{error, info, warn};
use std::fs;
use std::path::PathBuf;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::thread::{self, JoinHandle};

enum WorkerMessage {
    SaveSnapshot(WorldSnapshot, SnapshotType),
    Shutdown,
}

#[derive(Debug, Clone, Copy)]
pub enum SnapshotType {
    Periodic,
    Shutdown,
}

pub struct SnapshotWorker {
    sender: Sender<WorkerMessage>,
    thread_handle: Option<JoinHandle<()>>,
}

impl SnapshotWorker {
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

    pub fn save_snapshot(&self, snapshot: WorldSnapshot, snapshot_type: SnapshotType) {
        if let Err(e) = self
            .sender
            .send(WorkerMessage::SaveSnapshot(snapshot, snapshot_type))
        {
            error!("Failed to send snapshot to worker thread: {}", e);
        }
    }

    pub fn shutdown(mut self) {
        if let Err(e) = self.sender.send(WorkerMessage::Shutdown) {
            warn!("Failed to send shutdown message to worker: {}", e);
        }

        if let Some(handle) = self.thread_handle.take() {
            if let Err(e) = handle.join() {
                error!("Snapshot worker thread panicked: {:?}", e);
            }
        }
    }
}

fn worker_thread_loop(receiver: Receiver<WorkerMessage>, config: SnapshotConfig) {
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
                break;
            }
        }
    }
}

fn handle_save_snapshot(
    snapshot: WorldSnapshot,
    snapshot_type: SnapshotType,
    config: &SnapshotConfig,
) {
    match snapshot_type {
        SnapshotType::Periodic => {
            let timestamp = Local::now().format("%Y-%m-%d_%H-%M-%S");
            let timestamped_path =
                PathBuf::from(format!("snapshots/simulation_{}.msgpack", timestamp));

            match snapshot.save_to_file(&timestamped_path) {
                Ok(_) => {
                    info!(
                        "Saved periodic snapshot: {} ({} creatures)",
                        timestamped_path.display(),
                        snapshot.metadata.creature_count
                    );
                }
                Err(e) => {
                    error!(
                        "Failed to save snapshot {}: {}",
                        timestamped_path.display(),
                        e
                    );
                    return;
                }
            }

            let latest_path = PathBuf::from("snapshots/latest.msgpack");
            if let Err(e) = snapshot.save_to_file(&latest_path) {
                error!("Failed to update latest snapshot: {}", e);
            }

            cleanup_old_snapshots(config.keep_last_n);
        }
        SnapshotType::Shutdown => {
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

fn cleanup_old_snapshots(keep_last_n: usize) {
    let snapshots_dir = PathBuf::from("snapshots");

    let entries = match fs::read_dir(&snapshots_dir) {
        Ok(entries) => entries,
        Err(e) => {
            warn!("Failed to read snapshots directory: {}", e);
            return;
        }
    };

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

    periodic_snapshots.sort();

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
        cleanup_old_snapshots(10);
    }

    #[test]
    fn test_cleanup_with_fewer_than_keep() {
        let test_dir = PathBuf::from("snapshots");
        fs::create_dir_all(&test_dir).ok();

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

        for i in 1..=3 {
            let path = test_dir.join(format!("simulation_2025-01-0{}_12-00-00.msgpack", i));
            fs::write(&path, b"test").ok();
        }

        cleanup_old_snapshots(10);

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

        for i in 1..=3 {
            let path = test_dir.join(format!("simulation_2025-01-0{}_12-00-00.msgpack", i));
            fs::remove_file(path).ok();
        }
    }
}
