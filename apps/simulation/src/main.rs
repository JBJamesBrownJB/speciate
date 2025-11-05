//! Speciate - Console Simulation Server
//!
//! Headless simulation engine running at 20 Hz with console output

mod config;
mod nats;
mod simulation;
mod snapshot;
mod snapshot_worker;
mod spawner;
mod state;
mod state_loader;

use clap::Parser;
use config::{SnapshotConfig, SpawningConfig, TimingConfig, WorldConfig};
use log::{info, warn};
use simulation::timing::TickTimer;
use simulation::{Simulation, SimulationBuilder};
use snapshot_worker::{SnapshotType, SnapshotWorker};
use spawner::spawn_initial_creatures;
use state_loader::SimStateFile;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};

#[derive(Parser, Debug)]
#[command(name = "speciate")]
#[command(about = "Speciate simulation server", long_about = None)]
struct Args {
    #[arg(long, value_name = "PATH", help = "Path to simulation state file (TOML)")]
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
    env_logger::Builder::from_default_env()
        .filter_level(log::LevelFilter::Info)
        .init();

    info!("=== Speciate Simulation Server ===");
    info!("Console-only mode: No network, no serialization\n");

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
        info!("Loading simulation from snapshot: {}", snapshot_path.display());
        let snapshot = snapshot::WorldSnapshot::load_from_file(&snapshot_path)?;

        info!(
            "Snapshot loaded: v{} - {} creatures (created {})",
            snapshot.metadata.version,
            snapshot.metadata.creature_count,
            snapshot.metadata.created_at
        );

        let simulation = Simulation::from_snapshot(snapshot);
        let (width, height) = simulation.get_boundaries();
        info!("Restored {} creatures", simulation.creature_count());
        info!("World boundaries: {}x{}\n", width, height);

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

        let spawning = SpawningConfig {
            initial_population: state_file.spawn.random_creatures,
            min_size: state_file.spawn.size_min.unwrap_or(0.5),
            max_size: state_file.spawn.size_max.unwrap_or(2.0),
            spawn_x_min: state_file.spawn.area_x_min.unwrap_or(0.0),
            spawn_x_max: state_file.spawn.area_x_max.unwrap_or(world_width),
            spawn_y_min: state_file.spawn.area_y_min.unwrap_or(0.0),
            spawn_y_max: state_file.spawn.area_y_max.unwrap_or(world_height),
        };

        // Build simulation with all systems registered
        let mut simulation = SimulationBuilder::new()
            .set_boundaries(world_width, world_height)
            .build();

        spawn_initial_creatures(&mut simulation, &spawning);
        let initial_count = simulation.creature_count();
        info!("Spawned {} initial creatures", initial_count);
        info!("World boundaries: {}x{}\n", world_width, world_height);

        simulation
    } else {
        // Use default configuration
        info!("Using default configuration");
        let config = WorldConfig::new();

        // Build simulation with all systems registered
        let mut simulation = SimulationBuilder::new()
            .set_boundaries(config.world.width, config.world.height)
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

    // Start snapshot worker thread
    let snapshot_config = SnapshotConfig::default();
    info!(
        "Periodic snapshots enabled: every {} seconds, keeping last {}",
        snapshot_config.interval_secs, snapshot_config.keep_last_n
    );
    let snapshot_worker = SnapshotWorker::start(snapshot_config.clone());

    // Run simulation loop
    let result = run_simulation_loop(
        simulation,
        TimingConfig::default(),
        snapshot_config,
        snapshot_worker,
        running,
    );

    result
}

fn run_simulation_loop(
    mut simulation: Simulation,
    timing_config: config::TimingConfig,
    snapshot_config: SnapshotConfig,
    snapshot_worker: SnapshotWorker,
    running: Arc<AtomicBool>,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut tick: u64 = 0;
    let tick_duration = Duration::from_secs_f64(1.0 / timing_config.target_tick_rate as f64);
    let mut last_time = Instant::now();

    // Tick timing instrumentation
    let mut tick_timer = TickTimer::new(
        timing_config.timing_window_size,
        timing_config.timing_report_interval,
    );
    let mut last_creature_log = Instant::now();

    // Periodic snapshot timing
    let snapshot_interval = Duration::from_secs(snapshot_config.interval_secs);
    let mut last_snapshot = Instant::now();

    info!(
        "Starting simulation loop at {} Hz (delta time based)...\n",
        timing_config.target_tick_rate
    );

    // Main simulation loop - runs until shutdown signal received
    while running.load(Ordering::Relaxed) {
        let tick_start = Instant::now();

        let now = Instant::now();
        let delta = now.duration_since(last_time);
        let delta_time = delta.as_secs_f32();
        last_time = now;

        simulation.update(delta_time);

        tick += 1;

        let tick_elapsed = tick_start.elapsed();
        tick_timer.record_tick(tick_elapsed);

        log_sim_stats(
            &simulation,
            &timing_config,
            tick,
            tick_duration,
            &tick_timer,
            &mut last_creature_log,
        );

        // Periodic snapshot save
        if snapshot_config.enabled && last_snapshot.elapsed() >= snapshot_interval {
            let snapshot = simulation.to_snapshot();
            snapshot_worker.save_snapshot(snapshot, SnapshotType::Periodic);
            last_snapshot = Instant::now();
        }

        if tick_elapsed < tick_duration {
            thread::sleep(tick_duration - tick_elapsed);
        }
    }

    // Graceful shutdown: save final snapshot
    info!("Simulation stopped at tick {}", tick);
    info!("Creating final shutdown snapshot...");

    let final_snapshot = simulation.to_snapshot();
    snapshot_worker.save_snapshot(final_snapshot, SnapshotType::Shutdown);

    // Wait for all snapshots to finish saving
    info!("Waiting for snapshot worker to finish...");
    snapshot_worker.shutdown();
    info!("Snapshot worker finished. Shutdown complete.");

    Ok(())
}

fn log_sim_stats(
    simulation: &Simulation,
    timing_config: &config::TimingConfig,
    tick: u64,
    tick_duration: Duration,
    tick_timer: &TickTimer,
    last_creature_log: &mut Instant,
) {
    if last_creature_log.elapsed()
        >= Duration::from_secs(timing_config.creature_count_log_interval_secs)
    {
        let count = simulation.creature_count();
        let avg = tick_timer
            .average_duration()
            .map(|d| d.as_secs_f64() * 1000.0)
            .unwrap_or(0.0);
        let current = tick_timer
            .current_duration()
            .map(|d| d.as_secs_f64() * 1000.0)
            .unwrap_or(0.0);

        info!(
            "Tick {}: {} creatures | Avg: {:.2}ms, Current: {:.2}ms",
            tick, count, avg, current
        );

        // Warn if exceeding budget
        if let Some(avg_tick) = tick_timer.average_duration() {
            if avg_tick > tick_duration {
                warn!(
                    "⚠️  Performance degraded: {:.2}ms avg exceeds {:.2}ms budget",
                    avg_tick.as_secs_f64() * 1000.0,
                    tick_duration.as_secs_f64() * 1000.0
                );
            }
        }

        *last_creature_log = Instant::now();
    }
}
