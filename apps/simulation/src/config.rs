#[derive(Debug, Clone, bevy_ecs::system::Resource)]
pub struct MovementConfig {
    pub locomotion_noise_base: f32,
    pub noise_time_scale: f32,
}

impl Default for MovementConfig {
    fn default() -> Self {
        Self {
            // Reduced from 99.5 to account for lower drag (0.5 vs 2.0).
            // With old drag 2.0, noise quickly decayed. With drag 0.5,
            // noise accumulates and causes wild veering.
            // Target: ~5% of max_speed perpendicular drift per second.
            locomotion_noise_base: 3.0,
            noise_time_scale: 0.01,
        }
    }
}

use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct SaveStateConfig {
    pub enabled: bool,
    pub interval_secs: u64,
    pub keep_last_n: usize,
    pub save_dir: PathBuf,
}

impl Default for SaveStateConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            interval_secs: 300,
            keep_last_n: 20,
            save_dir: PathBuf::from("save-states"),
        }
    }
}
