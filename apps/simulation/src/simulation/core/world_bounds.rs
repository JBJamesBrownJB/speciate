//! World boundary utilities
//!
//! Provides clamping and validation for positions and targets to ensure they stay
//! within the valid world region. This prevents creatures from chasing targets
//! outside the world, which eliminates boundary bunching.
//!
//! Philosophy: Clamp targets, not creatures. Let physics handle momentum overshoot.

use bevy_ecs::prelude::*;

/// World boundary configuration and utility functions
///
/// Represents the valid rectangular region of the simulation world.
/// All target selection should go through these clamping functions to ensure
/// creatures never chase targets outside the world.
#[derive(Resource, Clone, Copy, Debug)]
pub struct WorldBounds {
    pub min_x: f32,
    pub max_x: f32,
    pub min_y: f32,
    pub max_y: f32,
}

impl WorldBounds {
    /// Create world bounds from min/max coordinates
    pub fn new(min_x: f32, max_x: f32, min_y: f32, max_y: f32) -> Self {
        Self {
            min_x,
            max_x,
            min_y,
            max_y,
        }
    }

    /// Create centered world bounds from width/height
    ///
    /// Creates a world centered at (0, 0) with the given dimensions.
    ///
    /// # Arguments
    /// * `width` - Total world width (e.g., 2000 km)
    /// * `height` - Total world height (e.g., 2000 km)
    ///
    /// # Example
    /// ```
    /// use speciate::simulation::core::WorldBounds;
    /// let bounds = WorldBounds::from_dimensions(2000.0, 2000.0);
    /// assert_eq!(bounds.min_x, -1000.0);
    /// assert_eq!(bounds.max_x, 1000.0);
    /// ```
    pub fn from_dimensions(width: f32, height: f32) -> Self {
        let half_width = width / 2.0;
        let half_height = height / 2.0;
        Self {
            min_x: -half_width,
            max_x: half_width,
            min_y: -half_height,
            max_y: half_height,
        }
    }

    /// Clamp a point to world boundaries
    ///
    /// Ensures the point is within valid world coordinates.
    /// Use for spawn positions to ensure they're always valid.
    ///
    /// # Example
    /// ```
    /// use speciate::simulation::core::WorldBounds;
    /// let bounds = WorldBounds::new(-100.0, 100.0, -100.0, 100.0);
    /// let (x, y) = bounds.clamp_point(150.0, -150.0);
    /// assert_eq!(x, 100.0);  // Clamped to max_x
    /// assert_eq!(y, -100.0); // Clamped to min_y
    /// ```
    pub fn clamp_point(&self, x: f32, y: f32) -> (f32, f32) {
        (
            x.clamp(self.min_x, self.max_x),
            y.clamp(self.min_y, self.max_y),
        )
    }

    /// Clamp a target position with margin from edges
    ///
    /// Keeps targets away from world boundaries by the specified margin.
    /// This prevents creatures from seeking targets right at the edge,
    /// which would cause bunching behavior.
    ///
    /// # Arguments
    /// * `x`, `y` - Target coordinates
    /// * `margin` - Minimum distance from boundary (e.g., 10m)
    ///
    /// # Example
    /// ```
    /// use speciate::simulation::core::WorldBounds;
    /// let bounds = WorldBounds::new(-100.0, 100.0, -100.0, 100.0);
    /// let (x, y) = bounds.clamp_target(95.0, 0.0, 10.0);
    /// assert_eq!(x, 90.0);  // 10m margin from max_x (100)
    /// assert_eq!(y, 0.0);   // No clamping needed
    /// ```
    pub fn clamp_target(&self, x: f32, y: f32, margin: f32) -> (f32, f32) {
        let x = x.clamp(self.min_x + margin, self.max_x - margin);
        let y = y.clamp(self.min_y + margin, self.max_y - margin);
        (x, y)
    }

    /// Check if a point is inside the world boundaries
    ///
    /// # Example
    /// ```
    /// use speciate::simulation::core::WorldBounds;
    /// let bounds = WorldBounds::new(-100.0, 100.0, -100.0, 100.0);
    /// assert!(bounds.contains(50.0, 50.0));
    /// assert!(!bounds.contains(150.0, 0.0));
    /// ```
    pub fn contains(&self, x: f32, y: f32) -> bool {
        x >= self.min_x && x <= self.max_x && y >= self.min_y && y <= self.max_y
    }

    /// Get world width
    pub fn width(&self) -> f32 {
        self.max_x - self.min_x
    }

    /// Get world height
    pub fn height(&self) -> f32 {
        self.max_y - self.min_y
    }
}

impl Default for WorldBounds {
    /// Default world: 2000 km × 2000 km centered at origin
    fn default() -> Self {
        Self::from_dimensions(2_000_000.0, 2_000_000.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_dimensions() {
        let bounds = WorldBounds::from_dimensions(200.0, 100.0);
        assert_eq!(bounds.min_x, -100.0);
        assert_eq!(bounds.max_x, 100.0);
        assert_eq!(bounds.min_y, -50.0);
        assert_eq!(bounds.max_y, 50.0);
    }

    #[test]
    fn test_clamp_point_inside() {
        let bounds = WorldBounds::new(-100.0, 100.0, -50.0, 50.0);
        let (x, y) = bounds.clamp_point(25.0, -25.0);
        assert_eq!(x, 25.0); // No change
        assert_eq!(y, -25.0); // No change
    }

    #[test]
    fn test_clamp_point_outside() {
        let bounds = WorldBounds::new(-100.0, 100.0, -50.0, 50.0);

        // Test all edges
        let (x, y) = bounds.clamp_point(150.0, 0.0);
        assert_eq!(x, 100.0); // Clamped to max_x

        let (x, y) = bounds.clamp_point(-150.0, 0.0);
        assert_eq!(x, -100.0); // Clamped to min_x

        let (x, y) = bounds.clamp_point(0.0, 75.0);
        assert_eq!(y, 50.0); // Clamped to max_y

        let (x, y) = bounds.clamp_point(0.0, -75.0);
        assert_eq!(y, -50.0); // Clamped to min_y
    }

    #[test]
    fn test_clamp_target_with_margin() {
        let bounds = WorldBounds::new(-100.0, 100.0, -50.0, 50.0);
        let margin = 10.0;

        // Point inside margin zone gets clamped
        let (x, y) = bounds.clamp_target(95.0, 45.0, margin);
        assert_eq!(x, 90.0); // max_x - margin = 100 - 10
        assert_eq!(y, 40.0); // max_y - margin = 50 - 10

        // Point outside gets clamped
        let (x, y) = bounds.clamp_target(200.0, -200.0, margin);
        assert_eq!(x, 90.0); // max_x - margin
        assert_eq!(y, -40.0); // min_y + margin = -50 + 10
    }

    #[test]
    fn test_contains() {
        let bounds = WorldBounds::new(-100.0, 100.0, -50.0, 50.0);

        assert!(bounds.contains(0.0, 0.0)); // Center
        assert!(bounds.contains(100.0, 50.0)); // Corner (inclusive)
        assert!(bounds.contains(-100.0, -50.0)); // Corner (inclusive)

        assert!(!bounds.contains(101.0, 0.0)); // Outside
        assert!(!bounds.contains(0.0, -51.0)); // Outside
    }

    #[test]
    fn test_dimensions() {
        let bounds = WorldBounds::new(-100.0, 100.0, -50.0, 50.0);
        assert_eq!(bounds.width(), 200.0);
        assert_eq!(bounds.height(), 100.0);
    }

    #[test]
    fn test_default_bounds() {
        let bounds = WorldBounds::default();
        assert_eq!(bounds.width(), 2_000_000.0);
        assert_eq!(bounds.height(), 2_000_000.0);
        assert!(bounds.contains(0.0, 0.0));
    }
}
