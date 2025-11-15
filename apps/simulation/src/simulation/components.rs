//! ECS Components for the Speciate simulation
//!
//! NOTE: This file is being gradually split across domain modules.
//! Components have been moved to core/ and creatures/ and are re-exported here for backward compatibility.

use bevy_ecs::prelude::*;
use serde::{Deserialize, Serialize};

// Re-export core components for backward compatibility during refactor
pub use super::core::components::{
    Acceleration, BoundaryConfig, Catatonic, DeltaTime, Position, Velocity,
};

// Re-export creature components for backward compatibility during refactor
pub use super::creatures::components::{
    // State
    BehaviorMode,
    CanAvoidObstacles,
    CanFlee,
    // Capabilities
    CanSeek,
    CanWander,
    CreatureState,
    HomePosition,
    // Identity
    CritId,
    FleeState,
    // Perception
    Target,
    WanderState,
};

/// Rotation component for creature orientation
/// TODO: Move to rendering module in Phase 5
#[derive(Component, Clone, Copy, Debug, Default, Serialize, Deserialize)]
pub struct Rotation {
    pub radians: f32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_creature_state_energy_management() {
        let mut state = CreatureState::new();
        let initial_energy = state.energy;

        state.consume_energy(10.0);
        assert_eq!(state.energy, initial_energy - 10.0);

        state.restore_energy(5.0);
        assert_eq!(state.energy, initial_energy - 5.0);
    }

    #[test]
    fn test_creature_state_exhaustion() {
        let mut state = CreatureState::new();

        // Drain to low energy (< 30)
        state.consume_energy(75.0); // 100 - 75 = 25
        assert!(state.is_low_energy());
        assert!(!state.is_exhausted());

        // Drain further to exhausted (< 10)
        state.consume_energy(20.0); // 25 - 20 = 5
        assert!(state.is_exhausted());
    }
}
