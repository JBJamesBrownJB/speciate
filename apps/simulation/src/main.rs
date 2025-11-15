//! Speciate - Console Simulation Server
//!
//! Headless simulation engine with console output (tick rate configured via TimingConfig)

use clap::Parser;
use speciate::config::{TimingConfig, WorldConfig};
use speciate::runner::{RunnerConfig, SimulationRunner};
use speciate::StdioHooks;
use log::info;
use speciate::simulation::creatures::spawner::{spawn_initial_creatures, spawn_initial_creatures_from_config};
use speciate::{Simulation, SimulationBuilder};
use speciate::persistence::WorldSnapshot;
use speciate::state::SimStateFile;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::fs;

// Dev tools command system (enabled with dev-tools feature)
#[cfg(feature = "dev-tools")]
use std::sync::Mutex;
#[cfg(feature = "dev-tools")]
use speciate::ipc::{spawn_stdin_reader_thread, CommandReceiver};

#[derive(Parser, Debug)]
#[command(name = "speciate")]
#[command(about = "Speciate simulation server", long_about = None)]
struct Args {
    #[arg(
        long,
        value_name = "PATH",
        help = "Path to simulation state file (TOML)"
    )]
    state: Option<PathBuf>,

    #[arg(
        long,
        value_name = "PATH",
        help = "Load simulation from binary snapshot (MessagePack). Defaults to most recent snapshot in ./snapshots/ if no path provided",
        conflicts_with = "state",
        num_args = 0..=1,
        default_missing_value = "auto"
    )]
    load_snapshot: Option<PathBuf>,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Configure logging to stderr ONLY (stdout is for MessagePack frames)
    env_logger::Builder::from_default_env()
        .filter_level(log::LevelFilter::Info)
        .target(env_logger::Target::Stderr)
        .init();

    info!("=== Speciate Simulation Server ===");
    info!("Stdio mode: MessagePack frames on stdout\n");

    let args = Args::parse();

    // Setup signal handler for graceful shutdown
    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();

    ctrlc::set_handler(move || {
        info!("\nReceived shutdown signal (Ctrl+C), saving snapshot and stopping...");
        r.store(false, Ordering::Relaxed);
    })?;

    let mut simulation = if let Some(snapshot_path) = args.load_snapshot {
        // Load from binary snapshot (takes precedence over --state)

        // Auto-discover most recent snapshot if no path provided
        let actual_path_opt = if snapshot_path.as_os_str() == "auto" {
            match find_most_recent_snapshot() {
                Some(path) => {
                    info!("Auto-discovered most recent snapshot: {}", path.display());
                    Some(path)
                }
                None => {
                    info!("No snapshots found in ./snapshots/ - starting with default configuration");
                    None
                }
            }
        } else {
            Some(snapshot_path)
        };

        // If we have a path, try to load it
        if let Some(actual_path) = actual_path_opt {

        info!("Loading simulation from snapshot: {}", actual_path.display());

        // Check if snapshot file exists
        if !actual_path.exists() {
            info!(
                "Snapshot file not found: {} - starting with default configuration",
                actual_path.display()
            );

            // Fall back to default config
            let config = WorldConfig::new();
            let mut simulation = SimulationBuilder::new()
                .set_boundaries(config.world.width / 2.0, config.world.height / 2.0)
                .build();

            spawn_initial_creatures(&mut simulation, &config.spawning);
            let initial_count = simulation.creature_count();
            info!("Spawned {} initial creatures", initial_count);
            info!(
                "World boundaries: {}x{}\n",
                config.world.width, config.world.height
            );

            simulation
        } else {
            // Load snapshot from file (with error handling for corrupted files)
            match WorldSnapshot::load_from_file(&actual_path) {
                Ok(snapshot) => {
                    info!(
                        "Snapshot loaded: v{} - {} creatures (created {})",
                        snapshot.metadata.version,
                        snapshot.metadata.creature_count,
                        snapshot.metadata.created_at
                    );

                    let simulation = Simulation::from_snapshot(snapshot);
                    let (min_x, max_x, min_y, max_y) = simulation.get_boundaries();
                    let world_width = max_x - min_x;
                    let world_height = max_y - min_y;
                    info!("Restored {} creatures", simulation.creature_count());
                    info!(
                        "World boundaries: {}x{} (centered: {} to {}, {} to {})\n",
                        world_width, world_height, min_x, max_x, min_y, max_y
                    );

                    simulation
                }
                Err(e) => {
                    // Corrupted snapshot → gracefully fall back to default config
                    info!(
                        "Failed to load snapshot ({}): {} - starting with default configuration",
                        actual_path.display(),
                        e
                    );

                    let config = WorldConfig::new();
                    let mut simulation = SimulationBuilder::new()
                        .set_boundaries(config.world.width / 2.0, config.world.height / 2.0)
                        .build();

                    spawn_initial_creatures(&mut simulation, &config.spawning);
                    let initial_count = simulation.creature_count();
                    info!("Spawned {} initial creatures", initial_count);
                    info!(
                        "World boundaries: {}x{}\n",
                        config.world.width, config.world.height
                    );

                    simulation
                }
            }
        }
        } else {
            // No snapshot found (auto-discovery returned None)
            // Fall back to default config
            let config = WorldConfig::new();
            let mut simulation = SimulationBuilder::new()
                .set_boundaries(config.world.width / 2.0, config.world.height / 2.0)
                .build();

            spawn_initial_creatures(&mut simulation, &config.spawning);
            let initial_count = simulation.creature_count();
            info!("Spawned {} initial creatures", initial_count);
            info!(
                "World boundaries: {}x{}\n",
                config.world.width, config.world.height
            );

            simulation
        }
    } else if let Some(state_path) = args.state {
        // Load configuration from TOML
        info!("Loading state from: {}", state_path.display());
        let state_file = SimStateFile::load_from_file(&state_path)?;

        info!(
            "State file loaded: v{} - {}",
            state_file.metadata.version, state_file.metadata.description
        );

        let world_width = state_file.world.width;
        let world_height = state_file.world.height;

        // Build simulation with all systems registered
        // Note: set_boundaries now takes extents (half-widths), not full dimensions
        let mut simulation = SimulationBuilder::new()
            .set_boundaries(world_width / 2.0, world_height / 2.0)
            .build();

        spawn_initial_creatures_from_config(&mut simulation, &state_file.spawn);
        let initial_count = simulation.creature_count();
        info!("Spawned {} initial creatures", initial_count);
        info!("World boundaries: {}x{}\n", world_width, world_height);

        simulation
    } else {
        // Use default configuration
        info!("Using default configuration");
        let config = WorldConfig::new();

        // Build simulation with all systems registered
        // Note: set_boundaries now takes extents (half-widths), not full dimensions
        let mut simulation = SimulationBuilder::new()
            .set_boundaries(config.world.width / 2.0, config.world.height / 2.0)
            .build();

        spawn_initial_creatures(&mut simulation, &config.spawning);
        let initial_count = simulation.creature_count();
        info!("Spawned {} initial creatures", initial_count);
        info!(
            "World boundaries: {}x{}\n",
            config.world.width, config.world.height
        );

        simulation
    };

    // Start stdin command reader (dev tools only)
    #[cfg(feature = "dev-tools")]
    {
        let (tx, rx) = std::sync::mpsc::channel();
        let _stdin_thread = spawn_stdin_reader_thread(tx);
        simulation.world_mut().insert_resource(CommandReceiver(Arc::new(Mutex::new(rx))));
        info!("Dev tools: stdin command reader started");
    }

    // Create runner with stdio hooks (outputs MessagePack to stdout)
    let timing_config = TimingConfig::default();
    info!(
        "Starting simulation loop at {} Hz (stdio IPC mode)...\n",
        timing_config.target_tick_rate
    );

    let hooks = StdioHooks::new();
    let runner_config = RunnerConfig {
        timing: timing_config,
        shutdown_signal: Some(running), // Graceful shutdown on Ctrl+C
    };

    let mut runner = SimulationRunner::new(runner_config, hooks);
    runner.run(simulation)
}

/// Find the most recent snapshot file in the snapshots directory
///
/// Scans for `simulation_*.msgpack` files and returns the one with the
/// most recent modification time. Returns None if no snapshots exist.
fn find_most_recent_snapshot() -> Option<PathBuf> {
    use std::path::Path;

    let snapshots_dir = Path::new("snapshots");
    if !snapshots_dir.exists() {
        return None;
    }

    let entries = fs::read_dir(snapshots_dir).ok()?;

    let mut snapshot_files: Vec<_> = entries
        .filter_map(|entry| entry.ok())
        .filter(|entry| {
            entry.path()
                .file_name()
                .and_then(|name| name.to_str())
                .map(|name| name.starts_with("simulation_") && name.ends_with(".msgpack"))
                .unwrap_or(false)
        })
        .collect();

    // Sort by modification time (most recent first)
    snapshot_files.sort_by_key(|entry| {
        entry.metadata()
            .and_then(|m| m.modified())
            .unwrap_or(std::time::SystemTime::UNIX_EPOCH)
    });
    snapshot_files.reverse(); // Most recent first

    snapshot_files.first().map(|entry| entry.path())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::Path;

    /// Helper to create test snapshots directory
    fn create_test_snapshots_dir() {
        let snapshots_dir = Path::new("snapshots");
        fs::create_dir_all(snapshots_dir).unwrap();
    }

    /// Helper to cleanup test snapshots
    fn cleanup_test_snapshots() {
        let snapshots_dir = Path::new("snapshots");
        if snapshots_dir.exists() {
            if let Ok(entries) = fs::read_dir(snapshots_dir) {
                for entry in entries.filter_map(|e| e.ok()) {
                    let path = entry.path();
                    if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("msgpack") {
                        fs::remove_file(path).ok();
                    }
                }
            }
        }
    }

    #[test]
    fn test_find_most_recent_snapshot_empty_directory() {
        cleanup_test_snapshots();
        create_test_snapshots_dir();

        let result = find_most_recent_snapshot();
        assert!(result.is_none(), "Should return None when no snapshots exist");

        cleanup_test_snapshots();
    }

    #[test]
    fn test_find_most_recent_snapshot_no_directory() {
        cleanup_test_snapshots();

        // Remove the directory entirely
        let snapshots_dir = Path::new("snapshots");
        if snapshots_dir.exists() {
            fs::remove_dir_all(snapshots_dir).ok();
        }

        let result = find_most_recent_snapshot();
        assert!(result.is_none(), "Should return None when snapshots directory doesn't exist");
    }

    #[test]
    fn test_find_most_recent_snapshot_single_file() {
        cleanup_test_snapshots();
        create_test_snapshots_dir();

        // Create a single snapshot file
        let snapshot_path = Path::new("snapshots/simulation_2025-11-15_10-00-00.msgpack");
        fs::write(snapshot_path, b"test data").unwrap();

        let result = find_most_recent_snapshot();
        assert!(result.is_some(), "Should find the snapshot file");
        assert_eq!(
            result.unwrap().file_name().unwrap().to_str().unwrap(),
            "simulation_2025-11-15_10-00-00.msgpack"
        );

        cleanup_test_snapshots();
    }

    #[test]
    fn test_find_most_recent_snapshot_multiple_files() {
        cleanup_test_snapshots();
        create_test_snapshots_dir();

        // Create multiple snapshot files with delays to ensure different modification times
        let old_path = Path::new("snapshots/simulation_2025-11-15_10-00-00.msgpack");
        fs::write(old_path, b"old data").unwrap();

        std::thread::sleep(std::time::Duration::from_millis(10));

        let mid_path = Path::new("snapshots/simulation_2025-11-15_11-00-00.msgpack");
        fs::write(mid_path, b"mid data").unwrap();

        std::thread::sleep(std::time::Duration::from_millis(10));

        let recent_path = Path::new("snapshots/simulation_2025-11-15_12-00-00.msgpack");
        fs::write(recent_path, b"recent data").unwrap();

        let result = find_most_recent_snapshot();
        assert!(result.is_some(), "Should find the most recent snapshot");

        // Should return the most recently modified file
        let path = result.unwrap();
        let found_name = path.file_name().unwrap().to_str().unwrap();
        assert_eq!(
            found_name,
            "simulation_2025-11-15_12-00-00.msgpack",
            "Should return most recently modified file"
        );

        cleanup_test_snapshots();
    }

    #[test]
    fn test_find_most_recent_snapshot_ignores_non_simulation_files() {
        cleanup_test_snapshots();
        create_test_snapshots_dir();

        // Create various files
        fs::write("snapshots/readme.txt", b"readme").unwrap();
        fs::write("snapshots/other.msgpack", b"other").unwrap();
        fs::write("snapshots/simulation_backup.msgpack", b"backup").unwrap(); // Matches pattern!

        std::thread::sleep(std::time::Duration::from_millis(10));

        fs::write("snapshots/simulation_2025-11-15_10-00-00.msgpack", b"valid").unwrap();

        let result = find_most_recent_snapshot();
        assert!(result.is_some(), "Should find valid snapshot file");

        let path = result.unwrap();
        let found_name = path.file_name().unwrap().to_str().unwrap();
        assert!(
            found_name.starts_with("simulation_") && found_name.ends_with(".msgpack"),
            "Should only find files matching simulation_*.msgpack pattern"
        );

        cleanup_test_snapshots();
    }
}
