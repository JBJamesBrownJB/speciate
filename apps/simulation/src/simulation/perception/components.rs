use bevy_ecs::prelude::*;
use bevy_reflect::Reflect;
use serde::{Deserialize, Serialize};

use crate::simulation::creatures::constants::{
    DEFAULT_FOV_DEGREES, DEFAULT_MASS, ENERGY_MODIFIER, FOV_RANGE_EXPONENT, MAX_PERCEIVED_NEIGHBORS,
    PERCEPTION_MULTIPLIER, PERCEPTION_THRESHOLD_FRACTION, PERSONAL_SPACE_MULTIPLIER,
    SIZE_ALLOMETRY_EXPONENT, SIZE_ALLOMETRY_REFERENCE,
};

// Debug types are in perception/debug.rs (dev-tools only)

/// Neighbor data cached during perception (avoids re-querying positions in avoidance)
#[derive(Debug, Clone, Copy)]
pub struct NeighborData {
    pub entity: Entity,
    pub x: f32,
    pub y: f32,
    pub radius: f32,
}

impl NeighborData {
    pub const EMPTY: Self = Self {
        entity: Entity::PLACEHOLDER,
        x: 0.0,
        y: 0.0,
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
    pub fov_angle: f32,        // Field of view in radians (stored internally as radians for efficient checks)
    pub range: f32,            // Derived from FOV and body size
    pub cos_half_fov_sq: f32,  // Cached cos²(fov_angle/2) for sqrt-free FOV checks
    pub cos_half_fov: f32,     // Cached cos(fov_angle/2) for wide FOV checks (sign matters)
    pub threshold: f32,        // L1 mass threshold: ignore cells with total_mass below this
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
    /// Formula: range = base_range × size_allometry × fov_factor
    fn calculate_range(body_size: f32, fov_angle_degrees: f32) -> f32 {
        let base_range = body_size * PERCEPTION_MULTIPLIER;
        let size_allometry =
            (body_size / SIZE_ALLOMETRY_REFERENCE).powf(SIZE_ALLOMETRY_EXPONENT);
        let fov_factor = (180.0 / fov_angle_degrees).powf(FOV_RANGE_EXPONENT);
        base_range * size_allometry * fov_factor
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
        self.neighbors[..self.neighbor_count as usize].iter().copied()
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

#[derive(Component, Debug, Clone, Copy, Reflect, Serialize, Deserialize)]
#[reflect(Component)]
pub struct AvoidanceBehavior {
    body_radius: f32,
}

// Energy-driven personal space reduction (biological realism)
// Hungry creatures tolerate closer proximity to reach resources
// 100% energy = max_modifier space, 0% energy = min_modifier space
// TODO(DNA): Replace hardcoded modifier range with energy_sensitivity gene
fn calculate_energy_modifier(energy_fraction: f32) -> f32 {
    let clamped = energy_fraction.clamp(0.0, 1.0);
    let range = ENERGY_MODIFIER.max_modifier - ENERGY_MODIFIER.min_modifier;
    ENERGY_MODIFIER.min_modifier + (range * clamped)
}

impl AvoidanceBehavior {
    pub fn new(body_radius: f32) -> Self {
        Self { body_radius }
    }

    pub fn from_body_size(body_length: f32) -> Self {
        Self::new(body_length / 2.0)
    }

    // Personal space: comfort zone for wandering = body_radius × multiplier
    pub fn personal_space(&self) -> f32 {
        self.body_radius * PERSONAL_SPACE_MULTIPLIER
    }

    // Energy-modified personal space (hungry creatures tolerate closer proximity)
    pub fn effective_personal_space(&self, energy_fraction: f32) -> f32 {
        self.personal_space() * calculate_energy_modifier(energy_fraction)
    }

    pub fn body_radius(&self) -> f32 {
        self.body_radius
    }
}

impl Default for AvoidanceBehavior {
    fn default() -> Self {
        Self::new(0.5) // Default 1m creature = 0.5m radius
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
    fn test_avoidance_scaling_with_body_size() {
        // New multiplicative model: personal_space = body_radius × PERSONAL_SPACE_MULTIPLIER
        // body_radius = body_length / 2

        let small_avoid = AvoidanceBehavior::from_body_size(0.5);
        // body_radius = 0.25, personal_space = 0.25 × 2.0 = 0.5
        assert_eq!(small_avoid.personal_space(), 0.5);

        let standard_avoid = AvoidanceBehavior::from_body_size(1.0);
        // body_radius = 0.5, personal_space = 0.5 × 2.0 = 1.0
        assert_eq!(standard_avoid.personal_space(), 1.0);

        let large_avoid = AvoidanceBehavior::from_body_size(2.0);
        // body_radius = 1.0, personal_space = 1.0 × 2.0 = 2.0
        assert_eq!(large_avoid.personal_space(), 2.0);
    }

    #[test]
    fn test_neighbor_cache_tracking() {
        let mut cache = NeighborCache::new();

        assert!(!cache.has_neighbors());
        assert_eq!(cache.neighbor_count(), 0);

        cache.add_neighbor(NeighborData { entity: Entity::PLACEHOLDER, x: 1.0, y: 2.0, radius: 0.5 });
        cache.add_neighbor(NeighborData { entity: Entity::PLACEHOLDER, x: 3.0, y: 4.0, radius: 0.5 });

        assert!(cache.has_neighbors());
        assert_eq!(cache.neighbor_count(), 2);

        cache.clear();
        assert!(!cache.has_neighbors());
        assert_eq!(cache.neighbor_count(), 0);
    }

    #[test]
    fn test_effective_personal_space_at_full_energy() {
        // body_radius = 5.0 → personal_space = 5.0 × 2.0 = 10.0
        let body_radius = 5.0;
        let avoidance = AvoidanceBehavior::new(body_radius);
        let personal_space = avoidance.personal_space(); // 10.0
        // At full energy (1.0), effective space = personal_space × max_modifier (1.0)
        assert_eq!(avoidance.effective_personal_space(1.0), personal_space);
    }

    #[test]
    fn test_effective_personal_space_at_zero_energy() {
        // body_radius = 5.0 → personal_space = 5.0 × 2.0 = 10.0
        let body_radius = 5.0;
        let avoidance = AvoidanceBehavior::new(body_radius);
        let personal_space = avoidance.personal_space();
        // At zero energy, effective space = personal_space × min_modifier (0.1)
        let expected = personal_space * ENERGY_MODIFIER.min_modifier;
        assert_eq!(avoidance.effective_personal_space(0.0), expected);
    }

    #[test]
    fn test_effective_personal_space_at_half_energy() {
        // body_radius = 5.0 → personal_space = 5.0 × 2.0 = 10.0
        let body_radius = 5.0;
        let avoidance = AvoidanceBehavior::new(body_radius);
        let personal_space = avoidance.personal_space();
        let range = ENERGY_MODIFIER.max_modifier - ENERGY_MODIFIER.min_modifier;
        let expected = personal_space * (ENERGY_MODIFIER.min_modifier + range * 0.5);
        let result = avoidance.effective_personal_space(0.5);
        assert!((result - expected).abs() < 0.001, "Expected ~{}, got {}", expected, result);
    }

    #[test]
    fn test_energy_fraction_clamped() {
        // body_radius = 5.0 → personal_space = 5.0 × 2.0 = 10.0
        let body_radius = 5.0;
        let avoidance = AvoidanceBehavior::new(body_radius);
        let personal_space = avoidance.personal_space();
        let min_space = personal_space * ENERGY_MODIFIER.min_modifier;
        let max_space = personal_space * ENERGY_MODIFIER.max_modifier;
        // Negative energy clamps to min_modifier
        assert_eq!(avoidance.effective_personal_space(-1.0), min_space);
        // Energy > 1.0 clamps to max_modifier
        assert_eq!(avoidance.effective_personal_space(2.0), max_space);
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
