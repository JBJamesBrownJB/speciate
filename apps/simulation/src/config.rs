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

    /// Movement behavior parameters
    pub movement: MovementConfig,
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

/// Movement behavior configuration
#[derive(Debug, Clone, bevy_ecs::system::Resource)]
pub struct MovementConfig {
    /// Base magnitude for locomotion noise (Newtons)
    /// Controls how much random wobble creatures have when moving
    /// Scales with speed² and inversely with body size (smaller creatures wobble more)
    pub locomotion_noise_base: f32,

    /// Time scale for Perlin noise (controls wobble frequency)
    /// Lower values = smoother, slower wobbles (e.g., 0.01 = very smooth)
    /// Higher values = jittery, rapid wobbles (e.g., 0.2 = very jittery)
    pub noise_time_scale: f32,
}

impl Default for MovementConfig {
    fn default() -> Self {
        Self {
            locomotion_noise_base: 99.5,
            noise_time_scale: 0.01,
        }
    }
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
                // 2,000,000m × 2,000,000m (2,000 km × 2,000 km) as per World_Scale.md
                width: 2_000_000.0,
                height: 2_000_000.0,
            },
            spawning: SpawningConfig {
                initial_population: 1,
                min_size: 0.5,
                max_size: 5.0,
            },
            timing: TimingConfig::default(), // Use default() to avoid duplication
            movement: MovementConfig::default(),
        }
    }
}

impl Default for TimingConfig {
    fn default() -> Self {
        Self {
            target_tick_rate: 60,
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
