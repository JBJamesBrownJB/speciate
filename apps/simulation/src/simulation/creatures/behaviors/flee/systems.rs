use crate::simulation::components::*;
use crate::simulation::core::components::*;
use bevy_ecs::prelude::*;

// TODO: Flee behavior not yet implemented
// This system is registered in the simulation schedule but currently does nothing.
// FleeState component exists and is spawned on creatures for future implementation.
// When implemented, this should apply flee forces away from threats detected in perception.
pub fn flee_system(
    mut query: Query<(&mut Acceleration, &mut Velocity, &FleeState, &CreatureState)>,
    #[cfg(feature = "dev-tools")] timings: bevy_ecs::system::Res<
        crate::instrumentation::SystemTimings,
    >,
) {
    #[cfg(feature = "dev-tools")]
    crate::time_system!(timings, "flee");

    // Stub - awaiting implementation
    for (_acceleration, _velocity, _flee_state, creature_state) in query.iter_mut() {
        if creature_state.behavior == BehaviorMode::Catatonic {
            continue;
        }
    }
}
