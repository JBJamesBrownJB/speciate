//! Simulation loop orchestration with pluggable hooks
//!
//! This module provides a unified simulation runner that handles:
//! - Delta time calculation and tick timing
//! - FPS measurement and instrumentation (TickTimer)
//! - Sleep management to maintain target tick rate
//! - Entry-point-specific behavior via RunnerHooks trait

use crate::config::TimingConfig;
use crate::simulation::core::timing::TickTimer;
use crate::Simulation;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

/// Trait for entry-point-specific hooks during simulation loop
pub trait RunnerHooks {
    /// Called once per tick after simulation.update()
    ///
    /// # Arguments
    /// * `tick` - Current tick number
    /// * `tick_elapsed` - Time taken for this tick
    /// * `simulation` - Mutable reference to simulation (for snapshots, state mutations)
    fn on_tick(&mut self, tick: u64, tick_elapsed: Duration, simulation: &mut Simulation);

    /// Called periodically for stats/logging (interval configured via TimingConfig)
    ///
    /// # Arguments
    /// * `tick` - Current tick number
    /// * `simulation` - Reference to simulation
    /// * `tick_timer` - Timing instrumentation (for avg/current duration)
    /// * `tick_duration` - Target tick budget (e.g., 16.67ms for 60 Hz)
    fn on_stats_interval(
        &mut self,
        tick: u64,
        simulation: &Simulation,
        tick_timer: &TickTimer,
        tick_duration: Duration,
    );

    /// Called when loop is about to exit (cleanup, final snapshots)
    ///
    /// # Arguments
    /// * `tick` - Final tick number
    /// * `simulation` - Mutable reference for final operations
    fn on_shutdown(&mut self, tick: u64, simulation: &mut Simulation);
}

/// Configuration for the simulation runner
pub struct RunnerConfig {
    /// Timing configuration (tick rate, logging intervals)
    pub timing: TimingConfig,

    /// Shutdown signal (Arc<AtomicBool>, sets to false to stop loop)
    /// If None, runs forever (infinite loop, Electron subprocess mode)
    pub shutdown_signal: Option<Arc<AtomicBool>>,
}

/// Manages the simulation loop with pluggable hooks
pub struct SimulationRunner<H: RunnerHooks> {
    config: RunnerConfig,
    hooks: H,
}

impl<H: RunnerHooks> SimulationRunner<H> {
    /// Create a new simulation runner
    pub fn new(config: RunnerConfig, hooks: H) -> Self {
        Self { config, hooks }
    }

    /// Run the simulation loop until shutdown signal or forever
    pub fn run(
        &mut self,
        mut simulation: Simulation,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mut tick: u64 = 0;
        let tick_duration = Duration::from_secs_f64(
            1.0 / self.config.timing.target_tick_rate as f64
        );
        let mut last_time = Instant::now();

        // Tick timing instrumentation
        let mut tick_timer = TickTimer::new(
            self.config.timing.timing_window_size,
            self.config.timing.timing_report_interval,
        );

        let mut last_stats_log = Instant::now();
        let stats_interval = Duration::from_secs(
            self.config.timing.creature_count_log_interval_secs
        );

        // Main simulation loop
        loop {
            // Check shutdown signal (if provided)
            if let Some(ref signal) = self.config.shutdown_signal {
                if !signal.load(Ordering::Relaxed) {
                    break; // Exit loop
                }
            }

            let tick_start = Instant::now();

            // Calculate delta time
            let now = Instant::now();
            let delta = now.duration_since(last_time);
            let delta_time = delta.as_secs_f32();
            last_time = now;

            // Update simulation
            simulation.update(delta_time);
            tick += 1;

            // Measure tick duration (execution time only, for performance monitoring)
            let tick_elapsed = tick_start.elapsed();
            tick_timer.record_tick(tick_elapsed);

            // Update measured tick rate from wall-clock interval (includes sleep)
            // This gives the actual tick rate (e.g., 20 Hz), not execution speed
            let measured_tick_rate = 1.0 / delta_time;
            simulation.set_tick_rate(measured_tick_rate);

            // Call hook: per-tick callback
            self.hooks.on_tick(tick, tick_elapsed, &mut simulation);

            // Call hook: periodic stats logging
            if last_stats_log.elapsed() >= stats_interval {
                self.hooks.on_stats_interval(
                    tick,
                    &simulation,
                    &tick_timer,
                    tick_duration,
                );
                last_stats_log = Instant::now();
            }

            // Sleep to maintain target tick rate
            if tick_elapsed < tick_duration {
                std::thread::sleep(tick_duration - tick_elapsed);
            }
        }

        // Graceful shutdown
        self.hooks.on_shutdown(tick, &mut simulation);

        Ok(())
    }
}

/// No-op hooks for minimal overhead (useful for testing or embedded use cases)
pub struct NoOpHooks;

impl RunnerHooks for NoOpHooks {
    fn on_tick(&mut self, _tick: u64, _tick_elapsed: Duration, _simulation: &mut Simulation) {}

    fn on_stats_interval(
        &mut self,
        _tick: u64,
        _simulation: &Simulation,
        _tick_timer: &TickTimer,
        _tick_duration: Duration,
    ) {
    }

    fn on_shutdown(&mut self, _tick: u64, _simulation: &mut Simulation) {}
}

// Console server-specific hooks (snapshot worker + detailed logging)
use crate::config::SnapshotConfig;
use crate::persistence::{SnapshotType, SnapshotWorker};
use log::{info, warn};

/// Console server implementation of runner hooks
///
/// Provides:
/// - Periodic snapshot saving via SnapshotWorker
/// - Detailed console logging (tick stats, creature count)
/// - Performance warnings (when tick duration exceeds budget)
pub struct ConsoleHooks {
    snapshot_worker: Option<SnapshotWorker>,
    snapshot_config: SnapshotConfig,
    last_snapshot: Instant,
}

impl ConsoleHooks {
    /// Create new console hooks with snapshot worker integration
    pub fn new(
        snapshot_worker: SnapshotWorker,
        snapshot_config: SnapshotConfig,
    ) -> Self {
        Self {
            snapshot_worker: Some(snapshot_worker),
            snapshot_config,
            last_snapshot: Instant::now(),
        }
    }
}

impl RunnerHooks for ConsoleHooks {
    fn on_tick(&mut self, _tick: u64, _tick_elapsed: Duration, simulation: &mut Simulation) {
        // Check if it's time for a periodic snapshot
        let snapshot_interval = Duration::from_secs(self.snapshot_config.interval_secs);
        if self.snapshot_config.enabled && self.last_snapshot.elapsed() >= snapshot_interval {
            let snapshot = simulation.to_snapshot();
            if let Some(ref worker) = self.snapshot_worker {
                worker.save_snapshot(snapshot, SnapshotType::Periodic);
            }
            self.last_snapshot = Instant::now();
        }
    }

    fn on_stats_interval(
        &mut self,
        tick: u64,
        simulation: &Simulation,
        tick_timer: &TickTimer,
        tick_duration: Duration,
    ) {
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
    }

    fn on_shutdown(&mut self, tick: u64, simulation: &mut Simulation) {
        info!("Simulation stopped at tick {}", tick);
        info!("Creating final shutdown snapshot...");

        let final_snapshot = simulation.to_snapshot();

        // Take ownership of snapshot worker to call shutdown
        if let Some(worker) = self.snapshot_worker.take() {
            worker.save_snapshot(final_snapshot, SnapshotType::Shutdown);
            info!("Waiting for snapshot worker to finish...");
            worker.shutdown();
            info!("Snapshot worker finished. Shutdown complete.");
        }
    }
}
