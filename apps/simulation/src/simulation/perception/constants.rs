pub const MAX_PERCEIVED_NEIGHBORS: usize = 21; // Maximum neighbors tracked per creature
pub const DEFAULT_PERCEPTION_RANGE: f32 = 100.0; // Default perception range in meters

// Perception constants (moved from movement::PERCEPTION)
pub const PERCEPTION_MULTIPLIER: f32 = 10.0; // Perception range as multiple of body length
pub const PERSONAL_SPACE: f32 = 1.5; // Spacing buffer distance in meters
pub const PANIC_THRESHOLD_RATIO: f32 = 0.5; // Panic threshold as fraction of personal_space

#[derive(Debug, Clone, Copy)]
pub struct EnergyModifierConstants {
    pub min_modifier: f32, // Minimum personal space multiplier at zero energy (hungry creatures tolerate closer proximity)
    pub max_modifier: f32, // Maximum personal space multiplier at full energy
}

impl Default for EnergyModifierConstants {
    fn default() -> Self {
        Self {
            min_modifier: 0.4,
            max_modifier: 1.0,
        }
    }
}

pub static ENERGY_MODIFIER: EnergyModifierConstants = EnergyModifierConstants {
    min_modifier: 0.4,
    max_modifier: 1.0,
};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_max_perceived_neighbors_positive() {
        assert!(MAX_PERCEIVED_NEIGHBORS > 0);
        assert!(MAX_PERCEIVED_NEIGHBORS <= 255);
    }

    #[test]
    fn test_default_perception_range_positive() {
        assert!(DEFAULT_PERCEPTION_RANGE > 0.0);
    }

    #[test]
    fn test_energy_modifier_valid_range() {
        let modifier = EnergyModifierConstants::default();
        assert!(modifier.min_modifier > 0.0);
        assert!(modifier.min_modifier < modifier.max_modifier);
        assert_eq!(modifier.max_modifier, 1.0);
    }

    #[test]
    fn test_energy_modifier_global_instance() {
        assert_eq!(ENERGY_MODIFIER.min_modifier, 0.4);
        assert_eq!(ENERGY_MODIFIER.max_modifier, 1.0);
    }

    #[test]
    fn test_perception_constants_positive() {
        assert!(PERCEPTION_MULTIPLIER > 0.0);
        assert!(PERSONAL_SPACE > 0.0);
        assert!(PANIC_THRESHOLD_RATIO > 0.0);
        assert!(PANIC_THRESHOLD_RATIO < 1.0);
    }
}
