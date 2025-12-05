use bevy_ecs::prelude::*;
use rand::Rng;

/// Pre-computed noise table for locomotion variability.
/// Populated once at simulation startup, accessed as a ring buffer.
/// At 100k creatures, this is just 200k array lookups per tick (no computation).
#[derive(Resource)]
pub struct NoiseTable {
    values: Vec<f32>,
    mask: usize, // For fast modulo (table_size - 1)
}

impl NoiseTable {
    /// Create a new noise table with 2^power entries
    pub fn new(power: u32) -> Self {
        let size = 1usize << power; // 2^power
        let mut rng = rand::thread_rng();

        let values: Vec<f32> = (0..size)
            .map(|_| rng.gen_range(-1.0..1.0))
            .collect();

        Self {
            values,
            mask: size - 1,
        }
    }

    /// Look up noise value for given entity/tick/axis combination
    #[inline(always)]
    pub fn get(&self, entity_id: u32, tick: u64, axis: u8, time_scale: f32) -> f32 {
        // Scale tick by time_scale
        let scaled_tick = ((tick as f32) * time_scale) as u64;

        // Combine into index (fast mixing)
        let index = (entity_id as usize)
            .wrapping_add(scaled_tick as usize)
            .wrapping_add((axis as usize) * 65537);

        // Fast modulo using bitmask (table size is power of 2)
        self.values[index & self.mask]
    }
}

impl Default for NoiseTable {
    fn default() -> Self {
        // 2^16 = 65536 entries, ~256KB, plenty of variation
        Self::new(16)
    }
}

/// Legacy function signature for compatibility (uses hash fallback if no table provided)
#[inline(always)]
pub fn perlin_locomotion_noise(entity_id: u32, tick: u64, axis: u8, time_scale: f32) -> f32 {
    // Fast hash fallback (used when NoiseTable not available)
    let scaled_tick = ((tick as f32) * time_scale) as u64;
    let seed = entity_id as u64 ^ (scaled_tick.wrapping_mul(2654435761)) ^ ((axis as u64) << 32);

    let mut h = seed;
    h ^= h >> 33;
    h = h.wrapping_mul(0xff51afd7ed558ccd);
    h ^= h >> 33;
    h = h.wrapping_mul(0xc4ceb9fe1a85ec53);
    h ^= h >> 33;

    ((h as i64) as f32) / (i64::MAX as f32)
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

        let noise1 = perlin_locomotion_noise(42, 100, 0, 0.05);
        let noise2 = perlin_locomotion_noise(42, 100, 0, 0.05);
        assert_eq!(noise1, noise2);
    }

    #[test]
    fn test_noise_varies_with_tick() {

        let noises: Vec<f32> = (0..10).map(|t| perlin_locomotion_noise(1, t * 10, 0, 0.05)).collect();


        let all_same = noises.windows(2).all(|w| (w[0] - w[1]).abs() < 0.001);
        assert!(!all_same, "Noise should vary across ticks");
    }

    #[test]
    fn test_noise_varies_with_entity() {

        let noises: Vec<f32> = (1..=10).map(|e| perlin_locomotion_noise(e, 50, 0, 0.05)).collect();


        let all_same = noises.windows(2).all(|w| (w[0] - w[1]).abs() < 0.001);
        assert!(!all_same, "Noise should vary across entities");
    }

    #[test]
    fn test_noise_independent_axes() {

        let noises_x: Vec<f32> = (0..10).map(|t| perlin_locomotion_noise(1, t, 0, 0.05)).collect();
        let noises_y: Vec<f32> = (0..10).map(|t| perlin_locomotion_noise(1, t, 1, 0.05)).collect();


        let identical = noises_x.iter().zip(noises_y.iter())
            .all(|(x, y)| (x - y).abs() < 0.001);
        assert!(!identical, "X and Y axes should have independent noise");
    }
}
