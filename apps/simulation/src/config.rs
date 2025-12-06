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
