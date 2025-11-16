use crate::simulation::components::*;
use crate::simulation::core::components::*;
use bevy_ecs::prelude::*;

pub fn flee_system(
    mut query: Query<(&mut Acceleration, &mut Velocity, &FleeState, &CreatureState)>,
    #[cfg(feature = "dev-tools")] timings: bevy_ecs::system::Res<
        crate::instrumentation::SystemTimings,
    >,
) {
    #[cfg(feature = "dev-tools")]
    crate::time_system!(timings, "flee");

    for (_acceleration, _velocity, _flee_state, creature_state) in query.iter_mut() {
        if creature_state.behavior == BehaviorMode::Catatonic {
            continue;
        }
    }
}
