// Wander behavior constants

pub const WANDER_FORCE: f32 = 5.0; // Random exploration force (Newtons)
pub const WANDER_RADIUS: f32 = 5.0; // Radius of wander circle (meters)
pub const WANDER_DISTANCE: f32 = 3.0; // Distance ahead to project wander circle (meters)
pub const ANGLE_CHANGE: f32 = 20.5; // Maximum angle change per tick (degrees)

// Territory constants (moved from movement::TERRITORY)
pub const COMFORT_RADIUS: f32 = 100.0; // Territory core radius where creature explores freely (meters)
pub const BLEND_CENTER: f32 = 20.0; // Distance where wander/homeward forces are 50/50 (meters)
pub const MAX_WANDER_DISTANCE: f32 = 200.0; // Hard limit for excursions from home (meters)
pub const HOMEWARD_FORCE: f32 = 50.0; // Force magnitude pulling creature toward home (Newtons)
pub const SIGMOID_STEEPNESS: f32 = 1.5; // Steepness of elastic tether transition curve

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wander_constants_positive() {
        assert!(WANDER_RADIUS > 0.0);
        assert!(WANDER_DISTANCE > 0.0);
        assert!(ANGLE_CHANGE > 0.0);
        assert!(WANDER_FORCE > 0.0);
    }

    #[test]
    fn test_territory_constants_ordered() {
        assert!(COMFORT_RADIUS < BLEND_CENTER);
        assert!(BLEND_CENTER < MAX_WANDER_DISTANCE);
    }

    #[test]
    fn test_territory_forces_positive() {
        assert!(HOMEWARD_FORCE > 0.0);
        assert!(SIGMOID_STEEPNESS > 0.0);
    }
}
