use crate::simulation::creatures::constants::DEFAULT_MASS;

#[derive(Clone, Copy, Default, Debug)]
pub struct BioSignature {
    pub total_mass: f32,
    pub max_size: f32,
    pub creature_count: u16,
}

impl BioSignature {
    pub fn is_empty(&self) -> bool {
        self.creature_count == 0
    }

    pub fn clear(&mut self) {
        *self = Self::default();
    }

    pub fn add(&mut self, mass: f32, size: f32) {
        self.total_mass += mass;
        self.max_size = self.max_size.max(size);
        self.creature_count += 1;
    }

    /// Merge another BioSignature into this one (used for L1→L2 aggregation).
    pub fn merge(&mut self, other: &BioSignature) {
        self.total_mass += other.total_mass;
        self.max_size = self.max_size.max(other.max_size);
        self.creature_count += other.creature_count;
    }

    /// Derive mass from radius using the same formula as BodySize::mass()
    /// mass = DEFAULT_MASS * length^3 where length = radius * 2
    pub fn mass_from_radius(radius: f32) -> f32 {
        let length = radius * 2.0;
        DEFAULT_MASS * length.powi(3)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_by_default() {
        let sig = BioSignature::default();
        assert!(sig.is_empty());
        assert_eq!(sig.total_mass, 0.0);
        assert_eq!(sig.max_size, 0.0);
        assert_eq!(sig.creature_count, 0);
    }

    #[test]
    fn add_accumulates_correctly() {
        let mut sig = BioSignature::default();

        sig.add(10.0, 1.0);
        assert!(!sig.is_empty());
        assert_eq!(sig.total_mass, 10.0);
        assert_eq!(sig.max_size, 1.0);
        assert_eq!(sig.creature_count, 1);

        sig.add(20.0, 0.5);
        assert_eq!(sig.total_mass, 30.0);
        assert_eq!(sig.max_size, 1.0); // max unchanged
        assert_eq!(sig.creature_count, 2);

        sig.add(5.0, 2.0);
        assert_eq!(sig.total_mass, 35.0);
        assert_eq!(sig.max_size, 2.0); // new max
        assert_eq!(sig.creature_count, 3);
    }

    #[test]
    fn clear_resets_to_default() {
        let mut sig = BioSignature::default();
        sig.add(100.0, 5.0);
        sig.add(50.0, 3.0);

        sig.clear();

        assert!(sig.is_empty());
        assert_eq!(sig.total_mass, 0.0);
        assert_eq!(sig.max_size, 0.0);
        assert_eq!(sig.creature_count, 0);
    }

    #[test]
    fn mass_from_radius_matches_body_size_formula() {
        // radius = 0.5 -> length = 1.0 -> mass = DEFAULT_MASS * 1.0^3
        let mass = BioSignature::mass_from_radius(0.5);
        assert_eq!(mass, DEFAULT_MASS);

        // radius = 1.0 -> length = 2.0 -> mass = DEFAULT_MASS * 8.0
        let mass = BioSignature::mass_from_radius(1.0);
        assert_eq!(mass, DEFAULT_MASS * 8.0);
    }

    #[test]
    fn merge_combines_biosignatures() {
        let mut l2_sig = BioSignature::default();

        // First L1 cell: 3 creatures, total 30.0 mass, max size 2.0
        let l1_a = BioSignature {
            total_mass: 30.0,
            max_size: 2.0,
            creature_count: 3,
        };

        // Second L1 cell: 5 creatures, total 50.0 mass, max size 1.5
        let l1_b = BioSignature {
            total_mass: 50.0,
            max_size: 1.5,
            creature_count: 5,
        };

        l2_sig.merge(&l1_a);
        assert_eq!(l2_sig.total_mass, 30.0);
        assert_eq!(l2_sig.max_size, 2.0);
        assert_eq!(l2_sig.creature_count, 3);

        l2_sig.merge(&l1_b);
        assert_eq!(l2_sig.total_mass, 80.0); // 30 + 50
        assert_eq!(l2_sig.max_size, 2.0); // max(2.0, 1.5)
        assert_eq!(l2_sig.creature_count, 8); // 3 + 5
    }
}
