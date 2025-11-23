#[derive(Debug, Clone)]
pub struct WorldConfig {
    pub world: WorldBoundaries,
    pub spawning: SpawningConfig,
    pub timing: TimingConfig,
    pub movement: MovementConfig,
}

#[derive(Debug, Clone)]
pub struct WorldBoundaries {
    pub width: f32,
    pub height: f32,
}

#[derive(Debug, Clone)]
pub struct SpawningConfig {
    pub initial_population: usize,
    pub min_size: f32,
    pub max_size: f32,
}

#[derive(Debug, Clone)]
pub struct TimingConfig {
    pub target_tick_rate: u32,
    pub timing_window_size: usize,
    pub timing_report_interval: u64,
    pub creature_count_log_interval_secs: u64,
}

#[derive(Debug, Clone, bevy_ecs::system::Resource)]
pub struct MovementConfig {
    pub locomotion_noise_base: f32,
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

#[derive(Debug, Clone)]
pub struct SaveStateConfig {
    pub enabled: bool,
    pub interval_secs: u64,
    pub keep_last_n: usize,
}

impl Default for SaveStateConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            interval_secs: 300,
            keep_last_n: 20,
        }
    }
}

impl Default for WorldConfig {
    fn default() -> Self {
        Self {
            world: WorldBoundaries {
                width: 2_000_000.0,
                height: 2_000_000.0,
            },
            spawning: SpawningConfig {
                initial_population: 1,
                min_size: 0.5,
                max_size: 5.0,
            },
            timing: TimingConfig::default(),
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
    pub fn new() -> Self {
        Self::default()
    }
}
