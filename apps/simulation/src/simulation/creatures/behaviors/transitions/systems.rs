use crate::simulation::creatures::constants::{
    AGE_INCREMENT_PER_TICK, ENERGY_COST_WANDERING, TICK_INTERVAL_SECONDS,
};
use crate::simulation::core::components::PhysicsTick;
use crate::simulation::creatures::components::{BehaviorMode, Brain, BrainMode, CreatureState};
use bevy_ecs::prelude::*;
use rayon::prelude::*;

pub fn behavior_transition_system(
    physics_tick: Res<PhysicsTick>,
    mut query: Query<(&mut CreatureState, &mut Brain)>,
    #[cfg(feature = "dev-tools")] timings: bevy_ecs::system::Res<
        crate::instrumentation::SystemTimings,
    >,
) {
    #[cfg(feature = "dev-tools")]
    crate::time_system!(timings, "behavior_transition");

    let current_time = physics_tick.get() as f64 * TICK_INTERVAL_SECONDS;

    // Collect into Vec for Rayon parallelization (same pattern as steering system)
    let mut entities: Vec<_> = query.iter_mut().collect();

    // Parallel processing - each creature's updates are independent
    entities
        .par_iter_mut()
        .for_each(|(creature_state, brain)| {
            creature_state.age += AGE_INCREMENT_PER_TICK;

            if creature_state.behavior == BehaviorMode::Wandering {
                creature_state.consume_energy(ENERGY_COST_WANDERING);
            }

            if brain.mode == BrainMode::Normal {
                let age = creature_state.age;
                let energy = creature_state.energy;
                if brain.can_decide(current_time, age, energy) {
                    brain.record_decision(current_time);
                }
            }
        });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_creature_aging() {
        let mut world = World::new();

        let entity = world.spawn((CreatureState::new(), Brain::default())).id();

        let initial_age = world.get::<CreatureState>(entity).unwrap().age;
        for _ in 0..10 {
            let mut query = world.query::<&mut CreatureState>();
            for mut state in query.iter_mut(&mut world) {
                state.age += AGE_INCREMENT_PER_TICK;
            }
        }

        let final_age = world.get::<CreatureState>(entity).unwrap().age;
        assert!(final_age > initial_age);
        assert_eq!(final_age, initial_age + (AGE_INCREMENT_PER_TICK * 10.0));
    }

    #[test]
    fn test_dormant_brain_stays_in_behavior() {
        let mut world = World::new();

        let entity = world
            .spawn((CreatureState::default(), Brain::dormant()))
            .id();

        let state = world.get::<CreatureState>(entity).unwrap();
        assert_eq!(state.behavior, BehaviorMode::Catatonic);
    }

}
