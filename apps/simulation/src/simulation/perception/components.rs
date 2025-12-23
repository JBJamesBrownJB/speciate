use bevy_ecs::prelude::*;

use super::classification::{L1Classification, MAX_L1_PERCEPTIONS};
use crate::simulation::creatures::constants::{
    DEFAULT_FOV_DEGREES, DEFAULT_MASS, FOV_RANGE_EXPONENT, MAX_PERCEIVED_NEIGHBORS,
    PERCEPTION_MULTIPLIER, PERCEPTION_THRESHOLD_FRACTION, SIZE_ALLOMETRY_EXPONENT,
    SIZE_ALLOMETRY_REFERENCE,
};

// Debug types are in perception/debug.rs (dev-tools only)

/// Biological floor for perception range (meters).
/// Even tiny creatures can detect immediate surroundings through touch, vibration, air pressure.
/// Prevents degenerate cases where creatures are blind to adjacent entities.
const MIN_PERCEPTION_RANGE: f32 = 3.0;

/// Neighbor data cached during perception (avoids re-querying positions in avoidance)
#[derive(Debug, Clone, Copy)]
pub struct NeighborData {
    pub entity: Entity,
    pub x: f32,
    pub y: f32,
    pub vx: f32,
    pub vy: f32,
    pub radius: f32,
}

impl NeighborData {
    pub const EMPTY: Self = Self {
        entity: Entity::PLACEHOLDER,
        x: 0.0,
        y: 0.0,
        vx: 0.0,
        vy: 0.0,
        radius: 0.0,
    };
}

impl Default for NeighborData {
    fn default() -> Self {
        Self::EMPTY
    }
}

/// Hot perception data (~24 bytes) - read every tick for range/FOV checks
/// Split from NeighborCache for cache locality optimization
#[derive(Component, Debug, Clone)]
pub struct Perception {
    pub fov_angle: f32, // Field of view in radians (stored internally as radians for efficient checks)
    pub range: f32,     // Derived from FOV and body size
    pub cos_half_fov_sq: f32, // Cached cos²(fov_angle/2) for sqrt-free FOV checks
    pub cos_half_fov: f32, // Cached cos(fov_angle/2) for wide FOV checks (sign matters)
    pub threshold: f32, // L1 mass threshold: ignore cells with total_mass below this
}

/// Cold neighbor cache - written by perception, read by avoidance
/// Separated from Perception for cache locality (only loaded when iterating neighbors)
#[derive(Component, Debug, Clone)]
pub struct NeighborCache {
    neighbor_count: u8,
    neighbors: [NeighborData; MAX_PERCEIVED_NEIGHBORS],
}

impl Perception {
    /// Create perception with explicit FOV (in degrees) and body size
    /// Range is automatically derived using biological tradeoff formula
    /// Threshold is derived from body mass for L1 early-exit optimization
    pub fn new(fov_angle_degrees: f32, body_size: f32) -> Self {
        let fov_rad = fov_angle_degrees.to_radians();
        let range = Self::calculate_range(body_size, fov_angle_degrees);
        let cos_half_fov = (fov_rad / 2.0).cos();
        let threshold = Self::calculate_threshold(body_size);
        Self {
            fov_angle: fov_rad,
            range,
            cos_half_fov_sq: cos_half_fov * cos_half_fov,
            cos_half_fov,
            threshold,
        }
    }

    /// Calculate L1 perception threshold from body size
    /// Uses same mass formula as BodySize::mass()
    /// Large creatures have higher thresholds (ignore smaller masses)
    fn calculate_threshold(body_size: f32) -> f32 {
        let body_mass = DEFAULT_MASS * body_size.powi(3);
        body_mass * PERCEPTION_THRESHOLD_FRACTION
    }

    /// Calculate perception range from body size and FOV
    /// Uses allometric scaling: larger creatures see proportionally further, but with diminishing returns.
    /// Narrow FOV = longer range (more photoreceptors per degree)
    /// Formula: range = max(MIN_PERCEPTION_RANGE, base_range × size_allometry × fov_factor)
    fn calculate_range(body_size: f32, fov_angle_degrees: f32) -> f32 {
        let base_range = body_size * PERCEPTION_MULTIPLIER;
        let size_allometry = (body_size / SIZE_ALLOMETRY_REFERENCE).powf(SIZE_ALLOMETRY_EXPONENT);
        let fov_factor = (180.0 / fov_angle_degrees).powf(FOV_RANGE_EXPONENT);
        let calculated = base_range * size_allometry * fov_factor;
        // Enforce biological floor: even tiny creatures detect immediate surroundings
        calculated.max(MIN_PERCEPTION_RANGE)
    }

    /// Create perception with default FOV (180°) from body size
    pub fn from_body_size(body_length: f32) -> Self {
        Self::new(DEFAULT_FOV_DEGREES, body_length)
    }

    /// Create perception with explicit FOV and body size
    pub fn from_body_size_with_fov(body_length: f32, fov_angle_degrees: f32) -> Self {
        Self::new(fov_angle_degrees, body_length)
    }

    /// Get half FOV in radians (for cone check: angle must be within ±half_fov)
    pub fn half_fov(&self) -> f32 {
        self.fov_angle / 2.0
    }
}

impl NeighborCache {
    pub fn new() -> Self {
        Self {
            neighbor_count: 0,
            neighbors: [NeighborData::EMPTY; MAX_PERCEIVED_NEIGHBORS],
        }
    }

    pub fn has_neighbors(&self) -> bool {
        self.neighbor_count > 0
    }

    pub fn neighbor_count(&self) -> usize {
        self.neighbor_count as usize
    }

    pub fn clear(&mut self) {
        self.neighbor_count = 0;
    }

    pub fn add_neighbor(&mut self, data: NeighborData) {
        if (self.neighbor_count as usize) < MAX_PERCEIVED_NEIGHBORS {
            self.neighbors[self.neighbor_count as usize] = data;
            self.neighbor_count += 1;
        }
    }

    pub fn iter_neighbors(&self) -> impl Iterator<Item = NeighborData> + '_ {
        self.neighbors[..self.neighbor_count as usize]
            .iter()
            .copied()
    }

    pub fn contains(&self, entity: Entity) -> bool {
        self.neighbors[..self.neighbor_count as usize]
            .iter()
            .any(|n| n.entity == entity)
    }
}

impl Default for NeighborCache {
    fn default() -> Self {
        Self::new()
    }
}

/// Single L1 cell perception data.
/// Fixed 16 bytes for cache-line friendly access.
#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct L1CellPerception {
    pub cell_idx: u32,
    pub classification: L1Classification,
    pub _pad: [u8; 3],
    pub direction_x: f32,
    pub direction_y: f32,
}

impl L1CellPerception {
    pub const EMPTY: Self = Self {
        cell_idx: 0,
        classification: L1Classification::Empty,
        _pad: [0; 3],
        direction_x: 0.0,
        direction_y: 0.0,
    };
}

impl Default for L1CellPerception {
    fn default() -> Self {
        Self::EMPTY
    }
}

/// L1 perception results - stores classifications of L1 cells in perception range.
/// Fixed-size array (not Vec) for cache efficiency at 500K creatures.
/// Used by Phase B drive system to compute navigation gradients.
#[derive(Component, Clone)]
pub struct L1Perceptions {
    count: u8,
    cells: [L1CellPerception; MAX_L1_PERCEPTIONS],
}

impl L1Perceptions {
    pub fn new() -> Self {
        Self {
            count: 0,
            cells: [L1CellPerception::EMPTY; MAX_L1_PERCEPTIONS],
        }
    }

    pub fn clear(&mut self) {
        self.count = 0;
    }

    pub fn count(&self) -> usize {
        self.count as usize
    }

    pub fn push(&mut self, cell: L1CellPerception) {
        if (self.count as usize) < MAX_L1_PERCEPTIONS {
            self.cells[self.count as usize] = cell;
            self.count += 1;
        }
    }

    pub fn iter(&self) -> impl Iterator<Item = &L1CellPerception> {
        self.cells[..self.count as usize].iter()
    }

    pub fn has_threat(&self) -> bool {
        self.iter()
            .any(|c| c.classification == L1Classification::Threat)
    }

    pub fn has_prey(&self) -> bool {
        self.iter()
            .any(|c| c.classification == L1Classification::Prey)
    }
}

impl Default for L1Perceptions {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_perception_scaling_with_body_size_default_fov() {
        // Test the RELATIONSHIP: range scales super-linearly with body size due to allometry
        // Formula: range = body_size × MULTIPLIER × (body_size / REF)^0.35 × fov_factor
        let small_perception = Perception::from_body_size(0.5);
        let standard_perception = Perception::from_body_size(1.0);
        let large_perception = Perception::from_body_size(2.0);

        // Range should scale super-linearly (body_size^1 × body_size^0.35 = body_size^1.35)
        assert!(small_perception.range > 0.0, "Range must be positive");

        // Expected ratio for 1.0 vs 0.5:
        // base ratio: 2.0, allometry ratio: (1.0/0.5)^0.35 / (0.5/0.5)^0.35 = 2^0.35 ≈ 1.274
        // total: 2.0 × 1.274 ≈ 2.55
        let ratio_1_to_05 = standard_perception.range / small_perception.range;
        let expected_ratio = 2.0 * (2.0_f32).powf(SIZE_ALLOMETRY_EXPONENT);
        assert!(
            (ratio_1_to_05 - expected_ratio).abs() < 0.1,
            "1.0m creature should have ~{:.2}x range of 0.5m, got {:.2}x",
            expected_ratio,
            ratio_1_to_05
        );

        // Similarly for 2.0 vs 1.0:
        let ratio_2_to_1 = large_perception.range / standard_perception.range;
        assert!(
            (ratio_2_to_1 - expected_ratio).abs() < 0.1,
            "2.0m creature should have ~{:.2}x range of 1.0m, got {:.2}x",
            expected_ratio,
            ratio_2_to_1
        );
    }

    #[test]
    fn test_fov_range_tradeoff() {
        let body_size = 1.0;
        let allometry = (body_size / SIZE_ALLOMETRY_REFERENCE).powf(SIZE_ALLOMETRY_EXPONENT);
        let allometric_base_range = body_size * PERCEPTION_MULTIPLIER * allometry;

        // 180° FOV = fov_factor of 1.0 (baseline)
        let baseline = Perception::new(180.0, body_size);
        assert!(
            (baseline.range - allometric_base_range).abs() < 0.01,
            "Baseline range should be {}, got {}",
            allometric_base_range,
            baseline.range
        );

        // Narrow FOV (90°) = longer range
        // Expected: allometric_base_range × (180/90)^0.4 = allometric_base_range × 2^0.4 ≈ × 1.32
        let narrow = Perception::new(90.0, body_size);
        let narrow_expected = allometric_base_range * 2.0_f32.powf(FOV_RANGE_EXPONENT);
        assert!(narrow.range > allometric_base_range);
        assert!((narrow.range - narrow_expected).abs() < 0.1);

        // Wide FOV (270°) = shorter range
        // Expected: allometric_base_range × (180/270)^0.4 ≈ × 0.84
        let wide = Perception::new(270.0, body_size);
        let wide_expected = allometric_base_range * (180.0 / 270.0_f32).powf(FOV_RANGE_EXPONENT);
        assert!(wide.range < allometric_base_range);
        assert!((wide.range - wide_expected).abs() < 0.1);
    }

    #[test]
    fn test_fov_stored_in_radians() {
        let perception = Perception::new(180.0, 1.0);
        assert!((perception.fov_angle - std::f32::consts::PI).abs() < 0.001);

        let narrow = Perception::new(90.0, 1.0);
        assert!((narrow.fov_angle - std::f32::consts::FRAC_PI_2).abs() < 0.001);
    }

    #[test]
    fn test_half_fov() {
        let perception = Perception::new(180.0, 1.0);
        assert!((perception.half_fov() - std::f32::consts::FRAC_PI_2).abs() < 0.001);
    }

    #[test]
    fn test_neighbor_cache_tracking() {
        let mut cache = NeighborCache::new();

        assert!(!cache.has_neighbors());
        assert_eq!(cache.neighbor_count(), 0);

        cache.add_neighbor(NeighborData {
            entity: Entity::PLACEHOLDER,
            x: 1.0,
            y: 2.0,
            vx: 0.0,
            vy: 0.0,
            radius: 0.5,
        });
        cache.add_neighbor(NeighborData {
            entity: Entity::PLACEHOLDER,
            x: 3.0,
            y: 4.0,
            vx: 0.0,
            vy: 0.0,
            radius: 0.5,
        });

        assert!(cache.has_neighbors());
        assert_eq!(cache.neighbor_count(), 2);

        cache.clear();
        assert!(!cache.has_neighbors());
        assert_eq!(cache.neighbor_count(), 0);
    }

    // === Allometric Scaling Tests ===

    #[test]
    fn test_perception_range_allometric_scaling() {
        // With allometric scaling, larger creatures see proportionally less far
        // A 10x larger creature (5.0m vs 0.5m) should see ~2.24x farther, not 10x
        // Formula: allometry = (size / SIZE_ALLOMETRY_REFERENCE)^SIZE_ALLOMETRY_EXPONENT
        //          (5.0 / 0.5)^0.35 = 10^0.35 ≈ 2.24

        let small_perception = Perception::new(180.0, SIZE_ALLOMETRY_REFERENCE);
        let large_perception = Perception::new(180.0, 5.0);

        let ratio = large_perception.range / small_perception.range;

        // Expected ratio from allometric scaling:
        // base_range ratio: 5.0 / 0.5 = 10
        // allometry ratio: (5.0 / 0.5)^0.35 / (0.5 / 0.5)^0.35 = 10^0.35 / 1 = 2.24
        // total ratio = 10 × 2.24 / 1 = 22.4 (accounting for both base and allometry)
        // Actually: range = base × allometry × fov
        // For 0.5m: base = 0.5 × 10 = 5, allometry = 1.0, fov = 1.0 → 5m
        // For 5.0m: base = 5.0 × 10 = 50, allometry = 2.24, fov = 1.0 → 112m
        // Ratio = 112 / 5 = 22.4

        // Without allometry, ratio would be 10 (linear with body size)
        // With allometry, large creature has MORE than linear scaling
        // The allometry MULTIPLIES with base range, giving super-linear growth

        let expected_ratio = (5.0 / SIZE_ALLOMETRY_REFERENCE)
            * (5.0 / SIZE_ALLOMETRY_REFERENCE).powf(SIZE_ALLOMETRY_EXPONENT);
        assert!(
            (ratio - expected_ratio).abs() < 0.1,
            "Expected ratio ~{:.2}, got {:.2}",
            expected_ratio,
            ratio
        );
    }

    #[test]
    fn test_perception_range_reference_size_has_unit_allometry() {
        // At the reference size (0.5m), the allometry factor should be 1.0
        // So range = base_range × 1.0 × fov_factor

        let perception = Perception::new(180.0, SIZE_ALLOMETRY_REFERENCE);

        // base_range = 0.5 × 10 = 5m
        // allometry = (0.5 / 0.5)^0.35 = 1.0
        // fov_factor = (180 / 180)^0.4 = 1.0
        // range = 5 × 1.0 × 1.0 = 5m
        let expected = SIZE_ALLOMETRY_REFERENCE * PERCEPTION_MULTIPLIER;
        assert!(
            (perception.range - expected).abs() < 0.01,
            "Reference size perception range should be {}, got {}",
            expected,
            perception.range
        );
    }

    #[test]
    fn test_perception_range_default_creature() {
        // Default creature: 1.0m body, 180° FOV
        // base_range = 1.0 × 10 = 10m
        // allometry = (1.0 / 0.5)^0.35 = 2.0^0.35 ≈ 1.274
        // fov_factor = (180 / 180)^0.4 = 1.0
        // range = 10 × 1.274 × 1.0 ≈ 12.74m

        let perception = Perception::new(180.0, 1.0);

        let expected_allometry = (1.0 / SIZE_ALLOMETRY_REFERENCE).powf(SIZE_ALLOMETRY_EXPONENT);
        let expected = 1.0 * PERCEPTION_MULTIPLIER * expected_allometry;

        assert!(
            (perception.range - expected).abs() < 0.1,
            "Default creature range should be ~{:.1}m, got {:.1}m",
            expected,
            perception.range
        );
    }

    #[test]
    fn test_perception_range_large_creature() {
        // Large creature: 5.0m body, 180° FOV
        // base_range = 5.0 × 10 = 50m
        // allometry = (5.0 / 0.5)^0.35 = 10^0.35 ≈ 2.239
        // fov_factor = 1.0
        // range = 50 × 2.239 × 1.0 ≈ 112m

        let perception = Perception::new(180.0, 5.0);

        let expected_allometry = (5.0 / SIZE_ALLOMETRY_REFERENCE).powf(SIZE_ALLOMETRY_EXPONENT);
        let expected = 5.0 * PERCEPTION_MULTIPLIER * expected_allometry;

        assert!(
            (perception.range - expected).abs() < 1.0,
            "Large creature range should be ~{:.0}m, got {:.0}m",
            expected,
            perception.range
        );
    }
}
