/// Determine if a specific entity should be perceived.
///
/// Pure function - no ECS, fully unit testable.
///
/// An entity is perceived if ALL conditions are true:
/// - Target mass >= my_threshold (size domination filter)
/// - Distance squared <= perception_range squared (range check)
/// - Target is within FOV (already computed by caller)
///
/// The `target_mass` parameter should be computed from the entity's body size
/// using the standard mass formula.
pub fn should_perceive_entity(
    my_threshold: f32,
    target_mass: f32,
    distance_sq: f32,
    perception_range_sq: f32,
    in_fov: bool,
) -> bool {
    target_mass >= my_threshold && distance_sq <= perception_range_sq && in_fov
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_target_below_threshold_not_perceived() {
        let result = should_perceive_entity(5.0, 4.0, 100.0, 10000.0, true);
        assert!(
            !result,
            "Target below mass threshold should not be perceived"
        );
    }

    #[test]
    fn test_target_above_threshold_in_range_in_fov_perceived() {
        let result = should_perceive_entity(5.0, 10.0, 100.0, 10000.0, true);
        assert!(result, "Target meeting all conditions should be perceived");
    }

    #[test]
    fn test_target_out_of_range_not_perceived() {
        let result = should_perceive_entity(5.0, 10.0, 20000.0, 10000.0, true);
        assert!(!result, "Target out of range should not be perceived");
    }

    #[test]
    fn test_target_out_of_fov_not_perceived() {
        let result = should_perceive_entity(5.0, 10.0, 100.0, 10000.0, false);
        assert!(!result, "Target outside FOV should not be perceived");
    }

    #[test]
    fn test_target_at_exact_threshold_perceived() {
        let result = should_perceive_entity(5.0, 5.0, 100.0, 10000.0, true);
        assert!(result, "Target at exact threshold should be perceived");
    }

    #[test]
    fn test_target_at_exact_range_perceived() {
        let result = should_perceive_entity(5.0, 10.0, 10000.0, 10000.0, true);
        assert!(result, "Target at exact range should be perceived");
    }

    #[test]
    fn test_zero_threshold_perceives_everything() {
        let result = should_perceive_entity(0.0, 0.001, 100.0, 10000.0, true);
        assert!(result, "Zero threshold should perceive any non-zero mass");
    }

    #[test]
    fn test_large_threshold_filters_most() {
        let result = should_perceive_entity(1000.0, 100.0, 100.0, 10000.0, true);
        assert!(!result, "Large threshold should filter small targets");
    }
}
