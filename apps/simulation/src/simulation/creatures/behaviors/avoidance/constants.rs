// Avoidance behavior constants

pub const AVOIDANCE_FORCE: f32 = 35.0; // Force for obstacle avoidance (Newtons)
pub const PANIC_FORCE: f32 = 90.0; // Emergency evasion force (Newtons)
pub const SEEKING_PERSONAL_SPACE_BUFFER: f32 = 0.1; // Seekers tolerate very close proximity (body + 10cm buffer)

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_seeking_buffer_positive() {
        assert!(SEEKING_PERSONAL_SPACE_BUFFER > 0.0);
        assert!(SEEKING_PERSONAL_SPACE_BUFFER < 1.0);
    }

    #[test]
    fn test_forces_positive() {
        assert!(AVOIDANCE_FORCE > 0.0);
        assert!(PANIC_FORCE > 0.0);
    }
}
