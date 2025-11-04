//! Centralized configuration for the simulation world
//!
//! This module contains all configurable parameters for the simulation,
//! making it easy to tweak world settings without modifying code.

/// World configuration parameters
#[derive(Debug, Clone)]
pub struct WorldConfig {
    /// World boundaries
    pub world: WorldBoundaries,

    /// Initial creature spawning parameters
    pub spawning: SpawningConfig,

    /// Simulation timing parameters
    pub timing: TimingConfig,
}

#[derive(Debug, Clone)]
pub struct WorldBoundaries {
    /// World width (in world coordinates)
    pub width: f32,
    /// World height (in world coordinates)
    pub height: f32,
}

#[derive(Debug, Clone)]
pub struct SpawningConfig {
    /// Number of creatures to spawn at startup
    pub initial_population: usize,

    /// Minimum creature size
    pub min_size: f32,

    /// Maximum creature size
    pub max_size: f32,

    /// Spawn area - minimum X coordinate (screen space)
    pub spawn_x_min: f32,

    /// Spawn area - maximum X coordinate (screen space)
    pub spawn_x_max: f32,

    /// Spawn area - minimum Y coordinate (screen space)
    pub spawn_y_min: f32,

    /// Spawn area - maximum Y coordinate (screen space)
    pub spawn_y_max: f32,
}

#[derive(Debug, Clone)]
pub struct TimingConfig {
    /// Target simulation tick rate in Hz (e.g., 60 for 60 ticks per second)
    pub target_tick_rate: u32,

    /// Tick timing window size (number of samples for rolling average)
    pub timing_window_size: usize,

    /// Report timing stats every N ticks
    pub timing_report_interval: u64,

    /// Log creature count every N seconds (real time)
    pub creature_count_log_interval_secs: u64,
}

/// Automatic snapshot configuration
#[derive(Debug, Clone)]
pub struct SnapshotConfig {
    /// Enable periodic automatic snapshots
    pub enabled: bool,

    /// Interval between periodic snapshots in seconds (default: 300 = 5 minutes)
    pub interval_secs: u64,

    /// Maximum number of periodic snapshots to keep (older ones are auto-deleted)
    pub keep_last_n: usize,
}

impl Default for SnapshotConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            interval_secs: 300, // 5 minutes
            keep_last_n: 10,
        }
    }
}

impl Default for WorldConfig {
    fn default() -> Self {
        Self {
            world: WorldBoundaries {
                width: 180.0,
                height: 130.0,
            },
            spawning: SpawningConfig {
                initial_population: 1,
                min_size: 0.5,
                max_size: 2.0,
                spawn_x_min: 40.0,
                spawn_x_max: 140.0,
                spawn_y_min: 30.0,
                spawn_y_max: 100.0,
            },
            timing: TimingConfig {
                target_tick_rate: 20,
                timing_window_size: 100,
                timing_report_interval: 200,
                creature_count_log_interval_secs: 5,
            },
        }
    }
}

impl Default for TimingConfig {
    fn default() -> Self {
        Self {
            target_tick_rate: 20,
            timing_window_size: 100,
            timing_report_interval: 200,
            creature_count_log_interval_secs: 5,
        }
    }
}

impl WorldConfig {
    /// Create a new WorldConfig with default values
    pub fn new() -> Self {
        Self::default()
    }
}
