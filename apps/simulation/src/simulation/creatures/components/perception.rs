use bevy_ecs::prelude::*;
use bevy_reflect::Reflect;
use serde::{Deserialize, Serialize};

/// A positive, non-zero radius value
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, Reflect)]
pub struct PositiveRadius(f32);

impl PositiveRadius {
    /// Create a PositiveRadius, panicking if value <= 0.0
    /// Use this for constants and known-good values
    pub const fn new(value: f32) -> Self {
        assert!(value > 0.0, "Radius must be positive");
        Self(value)
    }

    /// Try to create a PositiveRadius, returning None if value <= 0.0
    /// Use this when radius comes from user input or calculations
    pub fn try_new(value: f32) -> Option<Self> {
        if value > 0.0 {
            Some(Self(value))
        } else {
            None
        }
    }

    pub fn get(self) -> f32 {
        self.0
    }
}

impl Default for PositiveRadius {
    fn default() -> Self {
        Self(1.0)
    }
}

#[derive(Component, Clone, Copy, Debug, Serialize, Deserialize, Reflect)]
#[reflect(Component)]
pub struct Target {
    pub x: f32,
    pub y: f32,
    pub radius: PositiveRadius,
}

impl Target {
    /// Create target at position with explicit radius
    pub fn new(x: f32, y: f32, radius: PositiveRadius) -> Self {
        Self { x, y, radius }
    }

    /// Create target at position with default radius (1.0)
    pub fn at_point(x: f32, y: f32) -> Self {
        Self {
            x,
            y,
            radius: PositiveRadius::default(),
        }
    }
}

#[derive(Component, Clone, Copy, Debug, Default, Serialize, Deserialize, Reflect)]
#[reflect(Component)]
pub struct WanderState {
    pub wander_angle: f32,
    pub wander_radius: f32,   // TODO: from DNA
    pub wander_distance: f32, // TODO: from DNA
    pub angle_change: f32,    // TODO: from DNA
}

#[derive(Component, Clone, Copy, Debug, Serialize, Deserialize, Reflect)]
#[reflect(Component)]
pub struct FleeState {
    pub flee_speed_multiplier: f32, // TODO: from DNA
}

impl FleeState {
    pub fn new(_threat: Option<Entity>) -> Self {
        Self {
            flee_speed_multiplier: 1.5,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_positive_radius_valid() {
        let r = PositiveRadius::new(1.0);
        assert_eq!(r.get(), 1.0);

        let r = PositiveRadius::new(5.5);
        assert_eq!(r.get(), 5.5);
    }

    #[test]
    #[should_panic(expected = "Radius must be positive")]
    fn test_positive_radius_zero_panics() {
        let _r = PositiveRadius::new(0.0);
    }

    #[test]
    #[should_panic(expected = "Radius must be positive")]
    fn test_positive_radius_negative_panics() {
        let _r = PositiveRadius::new(-1.0);
    }

    #[test]
    fn test_positive_radius_try_new() {
        assert!(PositiveRadius::try_new(1.0).is_some());
        assert!(PositiveRadius::try_new(0.5).is_some());
        assert!(PositiveRadius::try_new(0.0).is_none());
        assert!(PositiveRadius::try_new(-1.0).is_none());
    }

    #[test]
    fn test_positive_radius_default() {
        let r = PositiveRadius::default();
        assert_eq!(r.get(), 1.0);
    }

    #[test]
    fn test_target_at_point() {
        let target = Target::at_point(10.0, 20.0);
        assert_eq!(target.x, 10.0);
        assert_eq!(target.y, 20.0);
        assert_eq!(target.radius.get(), 1.0);
    }

    #[test]
    fn test_target_new_with_explicit_radius() {
        let target = Target::new(5.0, 15.0, PositiveRadius::new(2.5));
        assert_eq!(target.x, 5.0);
        assert_eq!(target.y, 15.0);
        assert_eq!(target.radius.get(), 2.5);
    }
}
