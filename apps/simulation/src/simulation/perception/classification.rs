use crate::simulation::creatures::constants::PERCEPTION_THRESHOLD_FRACTION;
use crate::simulation::spatial::biosignature::BioSignature;

pub const MAX_L1_VISION: usize = 48;
pub const PREY_SIZE_RATIO: f32 = 0.3;

#[repr(u8)]
#[derive(Clone, Copy, Default, PartialEq, Eq, Debug)]
pub enum L1Classification {
    #[default]
    Empty = 0,
    Threat = 1,
    Prey = 2,
    Crowded = 3,
}

/// Classify an L1 cell for this creature.
///
/// Pure function - no ECS, fully unit testable.
///
/// Classification logic:
/// - EMPTY: Cell has no creatures OR total_mass < my_mass * threshold_fraction
/// - THREAT: Cell contains creature larger than me (max_size > my_size)
/// - PREY: Cell contains creatures smaller than me × PREY_SIZE_RATIO
/// - CROWDED: Has mass, but no threat/prey (medium-sized creatures)
///
/// When `is_my_cell` is true, subtracts own mass and decrements count
/// before classification (self-pollution handling).
pub fn classify_l1_cell(
    biosig: &BioSignature,
    my_mass: f32,
    my_size: f32,
    is_my_cell: bool,
) -> L1Classification {
    let (effective_mass, effective_count) = if is_my_cell {
        (
            biosig.total_mass - my_mass,
            biosig.creature_count.saturating_sub(1),
        )
    } else {
        (biosig.total_mass, biosig.creature_count)
    };

    let threshold = my_mass * PERCEPTION_THRESHOLD_FRACTION;

    if effective_count == 0 || effective_mass < threshold {
        return L1Classification::Empty;
    }

    if biosig.max_size > my_size {
        return L1Classification::Threat;
    }

    if biosig.max_size < my_size * PREY_SIZE_RATIO {
        return L1Classification::Prey;
    }

    L1Classification::Crowded
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_classify_empty_cell_returns_empty() {
        let biosig = BioSignature::default();
        let result = classify_l1_cell(&biosig, 100.0, 1.0, false);
        assert_eq!(result, L1Classification::Empty);
    }

    #[test]
    fn test_classify_cell_below_threshold_returns_empty() {
        let mut biosig = BioSignature::default();
        biosig.add(4.0, 0.5);
        let result = classify_l1_cell(&biosig, 100.0, 1.0, false);
        assert_eq!(result, L1Classification::Empty);
    }

    #[test]
    fn test_classify_cell_with_threat_returns_threat() {
        let mut biosig = BioSignature::default();
        biosig.add(50.0, 2.0);
        let result = classify_l1_cell(&biosig, 100.0, 1.0, false);
        assert_eq!(result, L1Classification::Threat);
    }

    #[test]
    fn test_classify_cell_with_prey_returns_prey() {
        let mut biosig = BioSignature::default();
        biosig.add(10.0, 0.2);
        let result = classify_l1_cell(&biosig, 100.0, 1.0, false);
        assert_eq!(result, L1Classification::Prey);
    }

    #[test]
    fn test_classify_cell_with_medium_creatures_returns_crowded() {
        let mut biosig = BioSignature::default();
        biosig.add(50.0, 0.5);
        let result = classify_l1_cell(&biosig, 100.0, 1.0, false);
        assert_eq!(result, L1Classification::Crowded);
    }

    #[test]
    fn test_classify_own_cell_subtracts_self() {
        let mut biosig = BioSignature::default();
        biosig.add(100.0, 1.0);
        let result = classify_l1_cell(&biosig, 100.0, 1.0, true);
        assert_eq!(result, L1Classification::Empty);
    }

    #[test]
    fn test_classify_own_cell_with_others() {
        let mut biosig = BioSignature::default();
        biosig.add(100.0, 1.0);
        biosig.add(200.0, 2.0);
        let result = classify_l1_cell(&biosig, 100.0, 1.0, true);
        assert_eq!(result, L1Classification::Threat);
    }

    #[test]
    fn test_threat_takes_priority_over_prey() {
        let mut biosig = BioSignature::default();
        biosig.add(10.0, 0.1);
        biosig.add(200.0, 2.0);
        let result = classify_l1_cell(&biosig, 100.0, 1.0, false);
        assert_eq!(result, L1Classification::Threat);
    }

    #[test]
    fn test_prey_at_exact_threshold() {
        let mut biosig = BioSignature::default();
        biosig.add(10.0, 0.3);
        let result = classify_l1_cell(&biosig, 100.0, 1.0, false);
        assert_eq!(result, L1Classification::Crowded);
    }

    #[test]
    fn test_prey_just_below_threshold() {
        let mut biosig = BioSignature::default();
        biosig.add(10.0, 0.29);
        let result = classify_l1_cell(&biosig, 100.0, 1.0, false);
        assert_eq!(result, L1Classification::Prey);
    }
}
