pub const MAX_PERCEIVED_NEIGHBORS: usize = 8; // Maximum neighbors tracked per creature
pub const PERCEPTION_MULTIPLIER: f32 = 10.0; // Base perception range as multiple of body length
pub const PERSONAL_SPACE: f32 = 1.5; // Spacing buffer distance in meters
pub const PANIC_THRESHOLD_RATIO: f32 = 0.5; // Panic threshold as fraction of personal_space

// Field of View constants
// Range scales with FOV: narrow FOV = longer range (more photoreceptors per degree)
// Formula: range = base_range × (180° / fov_angle)^FOV_RANGE_EXPONENT
pub const FOV_RANGE_EXPONENT: f32 = 0.4; // Biological tradeoff exponent
pub const MIN_FOV_DEGREES: f32 = 45.0; // Extreme specialist (mantis shrimp, owl focus)
pub const MAX_FOV_DEGREES: f32 = 340.0; // Near-panoramic (true 360° unrealistic)
pub const DEFAULT_FOV_DEGREES: f32 = 180.0; // Neutral baseline

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

    #[test]
    fn test_fov_constants_valid_range() {
        assert!(MIN_FOV_DEGREES > 0.0);
        assert!(MIN_FOV_DEGREES < MAX_FOV_DEGREES);
        assert!(MAX_FOV_DEGREES < 360.0);
        assert!(DEFAULT_FOV_DEGREES >= MIN_FOV_DEGREES);
        assert!(DEFAULT_FOV_DEGREES <= MAX_FOV_DEGREES);
    }

    #[test]
    fn test_fov_range_exponent_reasonable() {
        assert!(FOV_RANGE_EXPONENT > 0.0);
        assert!(FOV_RANGE_EXPONENT < 1.0);
    }
}
