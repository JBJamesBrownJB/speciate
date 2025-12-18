use bevy_ecs::prelude::*;
use bevy_reflect::Reflect;
use serde::{Deserialize, Serialize};

use super::constants::{MAX_FOV_DEGREES, MIN_FOV_DEGREES};

pub const SIZE_MIN: f32 = 0.5;
pub const SIZE_MAX: f32 = 5.0;

pub const DEFAULT_SIZE_GENE: f32 = 0.11;
pub const DEFAULT_FOV_GENE: f32 = 0.46;

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
        use rand::Rng;
        let mut rng = rand::thread_rng();
        Self::new(rng.gen(), rng.gen())
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
}
