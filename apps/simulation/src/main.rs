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
        help = "Load simulation from binary snapshot (MessagePack)",
        conflicts_with = "state"
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

    let simulation = if let Some(snapshot_path) = args.load_snapshot {
        // Load from binary snapshot (takes precedence over --state)
        info!(
            "Loading simulation from snapshot: {}",
            snapshot_path.display()
        );
        let snapshot = WorldSnapshot::load_from_file(&snapshot_path)?;

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
