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

/// Cores reserved for the renderer/main processes when the `SPECIATE_RESERVED_CORES`
/// override is unset. See memory `1m-pan-stutter-root-cause`.
pub const DEFAULT_RESERVED_CORES: usize = 2;

/// Threads to give the simulation's parallel pools (Rayon + Bevy `ComputeTaskPool`),
/// reserving `reserved` cores so the Electron renderer/main processes still get
/// scheduled and their frame loop isn't starved by the per-tick compute burst.
/// Always leaves at least 1 sim thread, even when `reserved >= total_cores`.
pub fn sim_thread_count_with_reserved(total_cores: usize, reserved: usize) -> usize {
    total_cores.saturating_sub(reserved).max(1)
}

/// Convenience: reserve the default core count.
pub fn sim_thread_count(total_cores: usize) -> usize {
    sim_thread_count_with_reserved(total_cores, DEFAULT_RESERVED_CORES)
}

/// Parse the `SPECIATE_RESERVED_CORES` override. Falls back to the default on
/// absent/blank/invalid input so a typo can never wedge startup.
pub fn parse_reserved_cores(raw: Option<&str>) -> usize {
    raw.and_then(|s| s.trim().parse::<usize>().ok())
        .unwrap_or(DEFAULT_RESERVED_CORES)
}

#[cfg(test)]
mod thread_count_tests {
    use super::{
        parse_reserved_cores, sim_thread_count, sim_thread_count_with_reserved,
        DEFAULT_RESERVED_CORES,
    };

    #[test]
    fn reserves_two_cores_on_typical_machines() {
        assert_eq!(sim_thread_count(16), 14);
        assert_eq!(sim_thread_count(8), 6);
        assert_eq!(sim_thread_count(4), 2);
    }

    #[test]
    fn never_returns_below_one() {
        assert_eq!(sim_thread_count(3), 1);
        assert_eq!(sim_thread_count(2), 1);
        assert_eq!(sim_thread_count(1), 1);
        assert_eq!(sim_thread_count(0), 1);
    }

    #[test]
    fn reserved_override_sizes_the_pool() {
        // 16-core experiment matrix (A/B/C/D).
        assert_eq!(sim_thread_count_with_reserved(16, 12), 4); // heavy reserve
        assert_eq!(sim_thread_count_with_reserved(16, 2), 14); // current default
        assert_eq!(sim_thread_count_with_reserved(16, 1), 15); // light
        assert_eq!(sim_thread_count_with_reserved(16, 0), 16); // control, all cores
    }

    #[test]
    fn override_never_starves_below_one() {
        assert_eq!(sim_thread_count_with_reserved(16, 16), 1);
        assert_eq!(sim_thread_count_with_reserved(16, 99), 1);
    }

    #[test]
    fn parse_falls_back_to_default_on_bad_input() {
        assert_eq!(parse_reserved_cores(None), DEFAULT_RESERVED_CORES);
        assert_eq!(parse_reserved_cores(Some("")), DEFAULT_RESERVED_CORES);
        assert_eq!(parse_reserved_cores(Some("abc")), DEFAULT_RESERVED_CORES);
        assert_eq!(parse_reserved_cores(Some("-1")), DEFAULT_RESERVED_CORES);
    }

    #[test]
    fn parse_reads_valid_overrides() {
        assert_eq!(parse_reserved_cores(Some("0")), 0);
        assert_eq!(parse_reserved_cores(Some("12")), 12);
        assert_eq!(parse_reserved_cores(Some("  3  ")), 3);
    }
}
