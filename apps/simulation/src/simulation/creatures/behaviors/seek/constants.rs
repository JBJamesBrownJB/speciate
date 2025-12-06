// Seek behavior constants

pub const SEEK_FORCE: f32 = 10.0; // Force for goal-directed movement (Newtons)
pub const ARRIVAL_THRESHOLD: f32 = 0.1; // Edge-to-edge distance when target is reached (meters)
pub const POUNCE_THRESHOLD: f32 = 0.1; // Distance for pounce snap (meters)

// Seeking behavior constants (moved from movement::SEEKING)
pub const MAX_FORCE: f32 = 50.0; // Maximum seeking force (Newtons)
pub const BRAKE_FORCE: f32 = 170.0; // Emergency brake force when overshooting (Newtons)
pub const POUNCE_DISTANCE: f32 = 0.5; // Snap-to-target distance threshold (meters)
pub const POUNCE_SPEED: f32 = 5.5; // Maximum speed for pounce snap (m/s)
pub const ARRIVAL_TOLERANCE: f32 = 0.5; // Stop when this close to target (meters)
pub const SLOW_ZONE_DECAY: f32 = 1.5; // Exponential decay factor for deceleration curve
pub const SLOW_ZONE_DECAY_EXP: f32 = 4.4816890703; // Precomputed: 1.5_f32.exp()

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_arrival_threshold_positive() {
        assert!(ARRIVAL_THRESHOLD > 0.0);
        assert!(POUNCE_THRESHOLD > 0.0);
    }

    #[test]
    fn test_forces_positive() {
        assert!(SEEK_FORCE > 0.0);
        assert!(MAX_FORCE > 0.0);
        assert!(BRAKE_FORCE > 0.0);
        assert!(POUNCE_DISTANCE > 0.0);
        assert!(POUNCE_SPEED > 0.0);
        assert!(ARRIVAL_TOLERANCE > 0.0);
        assert!(SLOW_ZONE_DECAY > 0.0);
    }
}
