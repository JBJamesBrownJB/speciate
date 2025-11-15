
use noise::{NoiseFn, Perlin};

pub fn perlin_locomotion_noise(entity_id: u32, tick: u64, axis: u8, time_scale: f32) -> f32 {
    let perlin = Perlin::new(entity_id.wrapping_mul(37) + axis as u32 * 1009);



    let spatial_offset = (entity_id as f64) * 100.0;
    let t = (tick as f64) * (time_scale as f64) + spatial_offset;


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
