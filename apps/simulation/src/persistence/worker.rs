use super::snapshot::WorldSaveState;
use crate::config::SaveStateConfig;
use chrono::Local;
use log::{error, info, warn};
use std::fs;
use std::path::PathBuf;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::thread::{self, JoinHandle};

enum WorkerMessage {
    SaveWorldState(WorldSaveState, SaveType),
    Shutdown,
}

#[derive(Debug, Clone, Copy)]
pub enum SaveType {
    Periodic,
    Shutdown,
}

pub struct SaveStateWorker {
    sender: Sender<WorkerMessage>,
    thread_handle: Option<JoinHandle<()>>,
}

impl SaveStateWorker {
    pub fn start(config: SaveStateConfig) -> Self {
        let (sender, receiver) = channel();

        let thread_handle = thread::spawn(move || {
            worker_thread_loop(receiver, config);
        });

        Self {
            sender,
            thread_handle: Some(thread_handle),
        }
    }

    pub fn save_world_state(&self, save_state: WorldSaveState, save_type: SaveType) {
        if let Err(e) = self
            .sender
            .send(WorkerMessage::SaveWorldState(save_state, save_type))
        {
            error!("Failed to send save state to worker thread: {}", e);
        }
    }

    pub fn shutdown(mut self) {
        if let Err(e) = self.sender.send(WorkerMessage::Shutdown) {
            warn!("Failed to send shutdown message to worker: {}", e);
        }

        if let Some(handle) = self.thread_handle.take() {
            if let Err(e) = handle.join() {
                error!("Save state worker thread panicked: {:?}", e);
            }
        }
    }
}

fn worker_thread_loop(receiver: Receiver<WorkerMessage>, config: SaveStateConfig) {
    if let Err(e) = fs::create_dir_all(&config.save_dir) {
        error!("Failed to create save-states directory: {}", e);
        return;
    }

    loop {
        match receiver.recv() {
            Ok(WorkerMessage::SaveWorldState(save_state, save_type)) => {
                handle_save_world_state(save_state, save_type, &config);
            }
            Ok(WorkerMessage::Shutdown) => {
                info!("Save state worker shutting down gracefully");
                break;
            }
            Err(_) => {
                break;
            }
        }
    }
}

fn handle_save_world_state(
    save_state: WorldSaveState,
    save_type: SaveType,
    config: &SaveStateConfig,
) {
    let timestamp = Local::now().format("%Y-%m-%d_%H-%M-%S");
    let save_path = config.save_dir.join(format!("{}.msgpack", timestamp));

    let type_label = match save_type {
        SaveType::Periodic => "periodic",
        SaveType::Shutdown => "shutdown",
    };

    match save_state.save_to_file(&save_path) {
        Ok(_) => {
            info!(
                "Saved {} save state: {} ({} creatures)",
                type_label,
                save_path.display(),
                save_state.metadata.creature_count
            );
        }
        Err(e) => {
            error!(
                "Failed to save {} save state {}: {}",
                type_label,
                save_path.display(),
                e
            );
            return;
        }
    }

    // Run cleanup after every save (periodic or shutdown)
    cleanup_old_save_states(&config.save_dir, config.keep_last_n);
}

fn cleanup_old_save_states(save_dir: &PathBuf, keep_last_n: usize) {
    let snapshots_dir = save_dir;

    let entries = match fs::read_dir(&snapshots_dir) {
        Ok(entries) => entries,
        Err(e) => {
            warn!("Failed to read save-states directory: {}", e);
            return;
        }
    };

    // Collect all .msgpack files (no prefix filtering)
    let mut save_states: Vec<PathBuf> = entries
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

    // Sort by filename (which is timestamp-based: YYYY-MM-DD_HH-MM-SS.msgpack)
    save_states.sort();

    if save_states.len() > keep_last_n {
        let to_delete = save_states.len() - keep_last_n;

        for path in save_states.iter().take(to_delete) {
            match fs::remove_file(path) {
                Ok(_) => {
                    info!("Deleted old save state: {}", path.display());
                }
                Err(e) => {
                    warn!("Failed to delete old save state {}: {}", path.display(), e);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_save_state_config_default() {
        let config = SaveStateConfig::default();
        assert!(config.enabled);
        assert_eq!(config.interval_secs, 300);
        assert_eq!(config.keep_last_n, 20);
    }

    #[test]
    fn test_cleanup_with_no_files() {
        let temp_dir = tempdir().expect("Failed to create temp dir");
        let save_dir = temp_dir.path().to_path_buf();
        fs::create_dir_all(&save_dir).ok();
        cleanup_old_save_states(&save_dir, 10);
    }

    #[test]
    fn test_cleanup_with_fewer_than_keep() {
        let temp_dir = tempdir().expect("Failed to create temp dir");
        let test_dir = temp_dir.path().to_path_buf();
        fs::create_dir_all(&test_dir).ok();

        for i in 1..=3 {
            let path = test_dir.join(format!("2025-01-0{}_12-00-00.msgpack", i));
            fs::write(&path, b"test").ok();
        }

        cleanup_old_save_states(&test_dir, 10);

        let count = fs::read_dir(&test_dir)
            .unwrap()
            .filter(|e| {
                e.as_ref()
                    .unwrap()
                    .path()
                    .extension()
                    .map(|ext| ext == "msgpack")
                    .unwrap_or(false)
            })
            .count();

        assert_eq!(count, 3);
    }
}
