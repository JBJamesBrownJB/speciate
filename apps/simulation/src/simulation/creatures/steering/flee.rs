//! Flee steering behavior pure function (stub).
//!
//! This module provides a placeholder for flee behavior calculation.
//! Implementation pending - currently returns zero acceleration.

use crate::simulation::math::SteeringContext;

/// Calculate flee acceleration (stub - not yet implemented).
///
/// Future implementation will:
/// 1. Identify threats from perception data
/// 2. Calculate weighted flee direction (away from nearest/most dangerous threats)
/// 3. Apply proper F=ma physics
///
/// Currently returns zero acceleration.
#[allow(unused_variables)]
pub fn calculate_flee_force(
    ctx: &SteeringContext,
    threats: &[(f32, f32)], // Placeholder: positions of threats
) -> (f32, f32) {
    // Stub - awaiting implementation
    (0.0, 0.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn default_context() -> SteeringContext {
        SteeringContext {
            velocity: (10.0, 0.0),
            max_speed: 15.0,
            max_force: 390.0,
            mass: 65.0,
        }
    }

    #[test]
    fn flee_stub_returns_zero() {
        let ctx = default_context();
        let threats = vec![(10.0, 0.0)];

        let (ax, ay) = calculate_flee_force(&ctx, &threats);

        assert_eq!(ax, 0.0);
        assert_eq!(ay, 0.0);
    }
}
