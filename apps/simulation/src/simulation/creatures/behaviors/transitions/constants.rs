// Behavior transition constants

pub const AGE_INCREMENT_PER_TICK: f32 = 0.001; // Age increase per simulation tick
pub const ENERGY_COST_WANDERING: f32 = 0.01; // Energy consumed per tick while wandering
pub const TICK_INTERVAL_SECONDS: f64 = 0.05; // Time per simulation tick (20 Hz)

pub const RANDOM_TARGET_MIN_DISTANCE: f32 = 50.0; // Minimum distance for random seek targets (meters)
pub const RANDOM_TARGET_MAX_DISTANCE: f32 = 200.0; // Maximum distance for random seek targets (meters)

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_constants_positive() {
        assert!(AGE_INCREMENT_PER_TICK > 0.0);
        assert!(ENERGY_COST_WANDERING > 0.0);
        assert!(TICK_INTERVAL_SECONDS > 0.0);
    }

    #[test]
    fn test_random_target_range_valid() {
        assert!(RANDOM_TARGET_MIN_DISTANCE > 0.0);
        assert!(RANDOM_TARGET_MAX_DISTANCE > RANDOM_TARGET_MIN_DISTANCE);
    }

    #[test]
    fn test_tick_interval_matches_expected_rate() {
        let expected_hz = 20.0;
        let calculated_hz = 1.0 / TICK_INTERVAL_SECONDS;
        assert!((calculated_hz - expected_hz).abs() < 0.1);
    }
}
