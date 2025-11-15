//! Locomotion noise for natural movement variability
//!
//! Provides Perlin noise generation for realistic animal movement patterns.
//! Animals don't move in perfect straight lines - they have motor control variability,
//! terrain micro-irregularities, and decision-making fluctuations.

use noise::{NoiseFn, Perlin};

/// Generate Perlin noise for locomotion variability
///
/// Creates smooth, organic noise that varies over time for each entity.
/// Used to add lateral (side-to-side) wobble to creature movement.
///
/// # Arguments
/// * `entity_id` - Unique creature identifier (spatial seed)
/// * `tick` - Simulation tick counter (temporal variation)
/// * `axis` - 0 for X-axis, 1 for Y-axis (different noise per dimension)
/// * `time_scale` - Controls wobble frequency (lower = smoother, higher = jittery)
///
/// # Returns
/// Noise value in range [-1.0, 1.0]
///
/// # Example
/// ```ignore
/// let noise_x = perlin_locomotion_noise(entity_id, tick, 0, 0.05);
/// let noise_y = perlin_locomotion_noise(entity_id, tick, 1, 0.05);
/// ```
pub fn perlin_locomotion_noise(entity_id: u32, tick: u64, axis: u8, time_scale: f32) -> f32 {
    let perlin = Perlin::new(entity_id.wrapping_mul(37) + axis as u32 * 1009);

    // Sample Perlin noise using tick as time parameter
    // Use entity_id as spatial offset to ensure different creatures have different patterns
    let spatial_offset = (entity_id as f64) * 100.0; // Spatial separation between creatures
    let t = (tick as f64) * (time_scale as f64) + spatial_offset;

    // Get 2D Perlin noise (use axis as second dimension)
    perlin.get([t, axis as f64 * 50.0]) as f32
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_noise_in_valid_range() {
        for tick in 0..100 {
            let noise = perlin_locomotion_noise(1, tick, 0, 0.05);
            assert!(noise >= -1.0 && noise <= 1.0, "Noise out of range: {}", noise);
        }
    }

    #[test]
    fn test_noise_deterministic() {
        // Same inputs should give same output
        let noise1 = perlin_locomotion_noise(42, 100, 0, 0.05);
        let noise2 = perlin_locomotion_noise(42, 100, 0, 0.05);
        assert_eq!(noise1, noise2);
    }

    #[test]
    fn test_noise_varies_with_tick() {
        // Different ticks should give different noise (check over range)
        let noises: Vec<f32> = (0..10).map(|t| perlin_locomotion_noise(1, t * 10, 0, 0.05)).collect();

        // At least some values should differ
        let all_same = noises.windows(2).all(|w| (w[0] - w[1]).abs() < 0.001);
        assert!(!all_same, "Noise should vary across ticks");
    }

    #[test]
    fn test_noise_varies_with_entity() {
        // Different entities should have different noise patterns
        let noises: Vec<f32> = (1..=10).map(|e| perlin_locomotion_noise(e, 50, 0, 0.05)).collect();

        // At least some values should differ
        let all_same = noises.windows(2).all(|w| (w[0] - w[1]).abs() < 0.001);
        assert!(!all_same, "Noise should vary across entities");
    }

    #[test]
    fn test_noise_independent_axes() {
        // X and Y axes should generate different patterns
        let noises_x: Vec<f32> = (0..10).map(|t| perlin_locomotion_noise(1, t, 0, 0.05)).collect();
        let noises_y: Vec<f32> = (0..10).map(|t| perlin_locomotion_noise(1, t, 1, 0.05)).collect();

        // Check that patterns are different (not identical)
        let identical = noises_x.iter().zip(noises_y.iter())
            .all(|(x, y)| (x - y).abs() < 0.001);
        assert!(!identical, "X and Y axes should have independent noise");
    }
}
