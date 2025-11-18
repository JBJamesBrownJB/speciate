use crate::config::TimingConfig;
use crate::simulation::core::timing::TickTimer;
use crate::Simulation;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

pub trait RunnerHooks {
    fn on_tick(&mut self, tick: u64, tick_elapsed: Duration, simulation: &mut Simulation);

    fn on_stats_interval(
        &mut self,
        tick: u64,
        simulation: &Simulation,
        tick_timer: &TickTimer,
        tick_duration: Duration,
    );

    fn on_shutdown(&mut self, tick: u64, simulation: &mut Simulation);
}

pub struct RunnerConfig {
    pub timing: TimingConfig,
    pub shutdown_signal: Option<Arc<AtomicBool>>,
}

pub struct SimulationRunner<H: RunnerHooks> {
    config: RunnerConfig,
    hooks: H,
}

impl<H: RunnerHooks> SimulationRunner<H> {
    pub fn new(config: RunnerConfig, hooks: H) -> Self {
        Self { config, hooks }
    }

    pub fn run(
        &mut self,
        mut simulation: Simulation,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mut tick: u64 = 0;
        let tick_duration = Duration::from_secs_f64(
            1.0 / self.config.timing.target_tick_rate as f64
        );
        let mut last_time = Instant::now();

        let mut tick_timer = TickTimer::new(
            self.config.timing.timing_window_size,
            self.config.timing.timing_report_interval,
        );

        let mut last_stats_log = Instant::now();
        let stats_interval = Duration::from_secs(
            self.config.timing.creature_count_log_interval_secs
        );

        loop {
            if let Some(ref signal) = self.config.shutdown_signal {
                if !signal.load(Ordering::Relaxed) {
                    break;
                }
            }

            let tick_start = Instant::now();

            let now = Instant::now();
            let delta = now.duration_since(last_time);
            let delta_time = delta.as_secs_f32();
            last_time = now;

            simulation.update(delta_time);

            let measured_tick_rate = 1.0 / delta_time;
            simulation.set_tick_rate(measured_tick_rate);

            self.hooks.on_tick(tick, tick_start.elapsed(), &mut simulation);

            let total_tick_elapsed = tick_start.elapsed();

            #[cfg(feature = "dev-tools")]
            {
                let elapsed_us = total_tick_elapsed.as_micros() as u64;
                simulation.world()
                    .resource::<crate::instrumentation::SystemTimings>()
                    .total_tick_us
                    .store(elapsed_us, std::sync::atomic::Ordering::Relaxed);
            }

            tick += 1;
            tick_timer.record_tick(total_tick_elapsed);

            if last_stats_log.elapsed() >= stats_interval {
                self.hooks.on_stats_interval(
                    tick,
                    &simulation,
                    &tick_timer,
                    tick_duration,
                );
                last_stats_log = Instant::now();
            }

            if total_tick_elapsed < tick_duration {
                std::thread::sleep(tick_duration - total_tick_elapsed);
            }
        }

        self.hooks.on_shutdown(tick, &mut simulation);

        Ok(())
    }
}

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

use crate::config::SnapshotConfig;
use crate::persistence::{SnapshotType, SnapshotWorker};
use log::{info, warn};

pub struct ConsoleHooks {
    snapshot_worker: Option<SnapshotWorker>,
    snapshot_config: SnapshotConfig,
    last_snapshot: Instant,
}

impl ConsoleHooks {
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

        if let Some(avg_tick) = tick_timer.average_duration() {
            if avg_tick > tick_duration {
                warn!(
                    "Performance degraded: {:.2}ms avg exceeds {:.2}ms budget",
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

        if let Some(worker) = self.snapshot_worker.take() {
            worker.save_snapshot(final_snapshot, SnapshotType::Shutdown);
            info!("Waiting for snapshot worker to finish...");
            worker.shutdown();
            info!("Snapshot worker finished. Shutdown complete.");
        }
    }
}
