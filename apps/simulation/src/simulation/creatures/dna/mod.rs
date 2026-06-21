use bevy_ecs::prelude::*;
use bevy_reflect::Reflect;
use rand_distr::{Distribution as _, LogNormal};
use serde::{Deserialize, Serialize};

use super::constants::{MAX_FOV_DEGREES, MIN_FOV_DEGREES};

pub const SIZE_MIN: f32 = 0.1;
pub const SIZE_MAX: f32 = 10.0;

pub const DEFAULT_SIZE_GENE: f32 = 0.09;
pub const DEFAULT_FOV_GENE: f32 = 0.46;

#[derive(Clone, Copy, Debug)]
pub struct SizeDistributionParams {
    log_normal: LogNormal<f32>,
}

impl SizeDistributionParams {
    pub fn new(median_meters: f32, sigma_log10: f32) -> Self {
        let mu = median_meters.ln();
        let sigma = sigma_log10 * std::f32::consts::LN_10; // log10→natural-log: σ_ln = σ_log10 × ln(10)
        Self { log_normal: LogNormal::new(mu, sigma).expect("sigma_log10 must be > 0") }
    }

    pub fn realistic() -> Self {
        Self::new(0.5, 0.45) // sigma_log10=0.45 → ~1-2% giants (>5m), p99≈5.4m; 0.40 falls short of the p99>5m target
    }
}

#[derive(Component, Clone, Copy, Debug, PartialEq, Serialize, Deserialize, Reflect)]
#[reflect(Component)]
pub struct Dna {
    pub size_gene: f32,
    pub fov_gene: f32,
}

impl Dna {
    pub fn new(size_gene: f32, fov_gene: f32) -> Self {
        Self {
            size_gene: size_gene.clamp(0.0, 1.0),
            fov_gene: fov_gene.clamp(0.0, 1.0),
        }
    }

    pub fn random() -> Self {
        let mut rng = rand::thread_rng();
        Self::random_seeded(&mut rng)
    }

    pub fn random_seeded(rng: &mut impl rand::Rng) -> Self {
        Self::new(rng.gen(), rng.gen())
    }

    pub fn random_realistic_seeded(rng: &mut impl rand::Rng, params: &SizeDistributionParams) -> Self {
        // rand_distr PINNED at 0.4 for seed-reproducibility (a major bump may change sampling)
        let size_m = params.log_normal.sample(rng).clamp(SIZE_MIN, SIZE_MAX);
        let size_gene = (size_m - SIZE_MIN) / (SIZE_MAX - SIZE_MIN);
        Self::new(size_gene, rng.gen())
    }

    pub fn expressed_size(&self) -> f32 {
        express_gene(self.size_gene, SIZE_MIN, SIZE_MAX)
    }

    pub fn expressed_fov(&self) -> f32 {
        express_gene(self.fov_gene, MIN_FOV_DEGREES, MAX_FOV_DEGREES)
    }
}

impl Default for Dna {
    fn default() -> Self {
        Self::new(DEFAULT_SIZE_GENE, DEFAULT_FOV_GENE)
    }
}

pub fn express_gene(gene: f32, min: f32, max: f32) -> f32 {
    min + gene.clamp(0.0, 1.0) * (max - min)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn realistic_gene_always_in_unit_range() {
        use rand::{rngs::StdRng, SeedableRng};
        let p = SizeDistributionParams::realistic();
        let mut rng = StdRng::seed_from_u64(1);
        for _ in 0..10_000 {
            let d = Dna::random_realistic_seeded(&mut rng, &p);
            assert!((0.0..=1.0).contains(&d.size_gene));
            assert!((0.0..=1.0).contains(&d.fov_gene));
        }
    }

    #[test]
    fn realistic_is_deterministic_for_same_seed() {
        use rand::{rngs::StdRng, SeedableRng};
        let p = SizeDistributionParams::realistic();
        let mut a = StdRng::seed_from_u64(42);
        let mut b = StdRng::seed_from_u64(42);
        assert_eq!(Dna::random_realistic_seeded(&mut a, &p), Dna::random_realistic_seeded(&mut b, &p));
    }

    #[test]
    fn realistic_median_near_half_metre() {
        use rand::{rngs::StdRng, SeedableRng};
        let p = SizeDistributionParams::realistic();
        let mut rng = StdRng::seed_from_u64(99);
        let mut sizes: Vec<f32> = (0..10_000).map(|_| Dna::random_realistic_seeded(&mut rng, &p).expressed_size()).collect();
        sizes.sort_by(|a, b| a.partial_cmp(b).unwrap());
        let median = sizes[5000];
        assert!((median - 0.5).abs() < 0.15, "median {median} not ~0.5 m");
    }

    #[test]
    fn realistic_is_right_skewed_with_rare_giants() {
        use rand::{rngs::StdRng, SeedableRng};
        let p = SizeDistributionParams::realistic();
        let mut rng = StdRng::seed_from_u64(7);
        let sizes: Vec<f32> = (0..10_000).map(|_| Dna::random_realistic_seeded(&mut rng, &p).expressed_size()).collect();
        let mut s = sizes.clone();
        s.sort_by(|a, b| a.partial_cmp(b).unwrap());
        let p50 = s[5000];
        let p90 = s[9000];
        let p99 = s[9900];
        assert!(p50 < 2.0, "median should be small: {p50}");
        assert!(p90 > p50 * 3.0, "right skew: p90 {p90} vs p50 {p50}");
        assert!(p99 > 5.0, "long tail should reach giants: p99 {p99}");
        let giants = sizes.iter().filter(|&&x| x > 5.0).count();
        assert!((giants as f32 / sizes.len() as f32) < 0.05, "giants (>5m) must be <5%, got {giants}/10000");
    }

    #[test]
    fn realistic_fov_gene_stays_uniform() {
        use rand::{rngs::StdRng, SeedableRng};
        let p = SizeDistributionParams::realistic();
        let mut rng = StdRng::seed_from_u64(3);
        let mean = (0..2000).map(|_| Dna::random_realistic_seeded(&mut rng, &p).fov_gene).sum::<f32>() / 2000.0;
        assert!((mean - 0.5).abs() < 0.05, "fov_gene should be uniform, mean {mean}");
    }

    #[test]
    fn test_dna_new_stores_values() {
        let dna = Dna::new(0.5, 0.7);
        assert_eq!(dna.size_gene, 0.5);
        assert_eq!(dna.fov_gene, 0.7);
    }

    #[test]
    fn test_dna_new_clamps_below_zero() {
        let dna = Dna::new(-0.5, -1.0);
        assert_eq!(dna.size_gene, 0.0);
        assert_eq!(dna.fov_gene, 0.0);
    }

    #[test]
    fn test_dna_new_clamps_above_one() {
        let dna = Dna::new(1.5, 2.0);
        assert_eq!(dna.size_gene, 1.0);
        assert_eq!(dna.fov_gene, 1.0);
    }

    #[test]
    fn test_dna_default_produces_backward_compat_genes() {
        let dna = Dna::default();
        assert_eq!(dna.size_gene, DEFAULT_SIZE_GENE);
        assert_eq!(dna.fov_gene, DEFAULT_FOV_GENE);
    }

    #[test]
    fn test_dna_default_expresses_to_1m() {
        let dna = Dna::default();
        let size = dna.expressed_size();
        assert!(
            (size - 1.0).abs() < 0.05,
            "Default size gene should express to ~1.0m, got {}",
            size
        );
    }

    #[test]
    fn test_dna_default_expresses_to_180_degrees() {
        let dna = Dna::default();
        let fov = dna.expressed_fov();
        assert!(
            (fov - 180.0).abs() < 1.0,
            "Default FOV gene should express to ~180 degrees, got {}",
            fov
        );
    }

    #[test]
    fn test_express_gene_min() {
        assert_eq!(express_gene(0.0, 0.5, 5.0), 0.5);
        assert_eq!(express_gene(0.0, 45.0, 340.0), 45.0);
    }

    #[test]
    fn test_express_gene_max() {
        assert_eq!(express_gene(1.0, 0.5, 5.0), 5.0);
        assert_eq!(express_gene(1.0, 45.0, 340.0), 340.0);
    }

    #[test]
    fn test_express_gene_middle() {
        let mid_size = express_gene(0.5, 0.5, 5.0);
        assert!((mid_size - 2.75).abs() < 0.001);

        let mid_fov = express_gene(0.5, 45.0, 340.0);
        assert!((mid_fov - 192.5).abs() < 0.001);
    }

    #[test]
    fn test_express_gene_clamps_input() {
        assert_eq!(express_gene(-0.5, 0.0, 100.0), 0.0);
        assert_eq!(express_gene(1.5, 0.0, 100.0), 100.0);
    }

    #[test]
    fn test_expressed_size_min_gene() {
        let dna = Dna::new(0.0, 0.5);
        assert_eq!(dna.expressed_size(), SIZE_MIN);
    }

    #[test]
    fn test_expressed_size_max_gene() {
        let dna = Dna::new(1.0, 0.5);
        assert_eq!(dna.expressed_size(), SIZE_MAX);
    }

    #[test]
    fn test_expressed_fov_min_gene() {
        let dna = Dna::new(0.5, 0.0);
        assert_eq!(dna.expressed_fov(), MIN_FOV_DEGREES);
    }

    #[test]
    fn test_expressed_fov_max_gene() {
        let dna = Dna::new(0.5, 1.0);
        assert_eq!(dna.expressed_fov(), MAX_FOV_DEGREES);
    }

    #[test]
    fn test_dna_serde_roundtrip() {
        let original = Dna::new(0.3, 0.7);
        let json = serde_json::to_string(&original).unwrap();
        let deserialized: Dna = serde_json::from_str(&json).unwrap();
        assert_eq!(original, deserialized);
    }

    #[test]
    fn test_dna_random_produces_valid_genes() {
        for _ in 0..10 {
            let dna = Dna::random();
            assert!(dna.size_gene >= 0.0 && dna.size_gene <= 1.0);
            assert!(dna.fov_gene >= 0.0 && dna.fov_gene <= 1.0);
        }
    }

    #[test]
    fn random_seeded_is_deterministic_for_same_seed() {
        use rand::rngs::StdRng;
        use rand::SeedableRng;
        let mut rng_a = StdRng::seed_from_u64(42);
        let mut rng_b = StdRng::seed_from_u64(42);
        let a = Dna::random_seeded(&mut rng_a);
        let b = Dna::random_seeded(&mut rng_b);
        assert_eq!(a, b, "same seed must produce identical DNA");
    }

    #[test]
    fn random_seeded_varies_across_draws() {
        use rand::rngs::StdRng;
        use rand::SeedableRng;
        let mut rng = StdRng::seed_from_u64(7);
        let first = Dna::random_seeded(&mut rng);
        let second = Dna::random_seeded(&mut rng);
        assert_ne!(first, second, "successive draws from one rng must differ");
    }

    #[test]
    fn random_seeded_stays_in_gene_range() {
        use rand::rngs::StdRng;
        use rand::SeedableRng;
        let mut rng = StdRng::seed_from_u64(1);
        for _ in 0..1000 {
            let dna = Dna::random_seeded(&mut rng);
            assert!((0.0..=1.0).contains(&dna.size_gene));
            assert!((0.0..=1.0).contains(&dna.fov_gene));
        }
    }
}
