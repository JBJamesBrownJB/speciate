use bevy_ecs::prelude::*;

#[derive(Resource, Clone, Copy, Debug)]
pub struct WorldBounds {
    pub min_x: f32,
    pub max_x: f32,
    pub min_y: f32,
    pub max_y: f32,
}

impl WorldBounds {
    pub fn new(min_x: f32, max_x: f32, min_y: f32, max_y: f32) -> Self {
        Self {
            min_x,
            max_x,
            min_y,
            max_y,
        }
    }

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

    pub fn clamp_point(&self, x: f32, y: f32) -> (f32, f32) {
        (
            x.clamp(self.min_x, self.max_x),
            y.clamp(self.min_y, self.max_y),
        )
    }

    pub fn clamp_target(&self, x: f32, y: f32, margin: f32) -> (f32, f32) {
        let x = x.clamp(self.min_x + margin, self.max_x - margin);
        let y = y.clamp(self.min_y + margin, self.max_y - margin);
        (x, y)
    }

    pub fn contains(&self, x: f32, y: f32) -> bool {
        x >= self.min_x && x <= self.max_x && y >= self.min_y && y <= self.max_y
    }

    pub fn width(&self) -> f32 {
        self.max_x - self.min_x
    }

    pub fn height(&self) -> f32 {
        self.max_y - self.min_y
    }
}

impl Default for WorldBounds {
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
        assert_eq!(x, 25.0);
        assert_eq!(y, -25.0);
    }

    #[test]
    fn test_clamp_point_outside() {
        let bounds = WorldBounds::new(-100.0, 100.0, -50.0, 50.0);

        let (x, _) = bounds.clamp_point(150.0, 0.0);
        assert_eq!(x, 100.0);

        let (x, _) = bounds.clamp_point(-150.0, 0.0);
        assert_eq!(x, -100.0);

        let (_, y) = bounds.clamp_point(0.0, 75.0);
        assert_eq!(y, 50.0);

        let (_, y) = bounds.clamp_point(0.0, -75.0);
        assert_eq!(y, -50.0);
    }

    #[test]
    fn test_clamp_target_with_margin() {
        let bounds = WorldBounds::new(-100.0, 100.0, -50.0, 50.0);
        let margin = 10.0;

        let (x, y) = bounds.clamp_target(95.0, 45.0, margin);
        assert_eq!(x, 90.0);
        assert_eq!(y, 40.0);

        let (x, y) = bounds.clamp_target(200.0, -200.0, margin);
        assert_eq!(x, 90.0);
        assert_eq!(y, -40.0);
    }

    #[test]
    fn test_contains() {
        let bounds = WorldBounds::new(-100.0, 100.0, -50.0, 50.0);

        assert!(bounds.contains(0.0, 0.0));
        assert!(bounds.contains(100.0, 50.0));
        assert!(bounds.contains(-100.0, -50.0));

        assert!(!bounds.contains(101.0, 0.0));
        assert!(!bounds.contains(0.0, -51.0));
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
