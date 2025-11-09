//! Fleeing behavior system (currently disabled)
//!
//! Will be enabled when Fleeing behavior mode is uncommented.
//! Follows the force accumulation pattern: ADDs to Acceleration.

use crate::simulation::components::*;
use crate::simulation::core::components::*;
use bevy_ecs::prelude::*;

/// Fleeing behavior: steer away from threats
///
/// Currently disabled - only Catatonic and Seeking modes are active.
/// Will be implemented when BehaviorMode::Fleeing is uncommented.
pub fn flee_system(
    mut query: Query<(&mut Acceleration, &mut Velocity, &FleeState, &CreatureState)>,
) {
    // Disabled: Fleeing behavior not implemented yet (only Catatonic/Seeking active)
    // Will be enabled when other behaviors are uncommented in BehaviorMode enum
    for (_acceleration, _velocity, _flee_state, creature_state) in query.iter_mut() {
        // Early return since Fleeing is not active
        if creature_state.behavior == BehaviorMode::Catatonic {
            continue;
        }
    }
}
