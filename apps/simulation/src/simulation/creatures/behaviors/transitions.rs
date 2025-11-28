
use crate::simulation::components::*;
use bevy_ecs::prelude::*;


const AGE_INCREMENT_PER_TICK: f32 = 0.001;
const ENERGY_COST_WANDERING: f32 = 0.01;

pub fn behavior_transition_system(
    _commands: Commands,
    mut query: Query<(
        Entity,
        &mut CreatureState,
        Option<&WanderState>,
        Option<&FleeState>,
    )>,
    #[cfg(feature = "dev-tools")] timings: bevy_ecs::system::Res<
        crate::instrumentation::SystemTimings,
    >,
) {
    #[cfg(feature = "dev-tools")]
    crate::time_system!(timings, "behavior_transition");

    for (_entity, mut creature_state, _wander_state, _flee_state) in query.iter_mut() {





        creature_state.age += AGE_INCREMENT_PER_TICK;


        let previous_behavior = creature_state.behavior;

        match creature_state.behavior {
            BehaviorMode::Catatonic => {

            }
            BehaviorMode::Seeking => {

            }
            BehaviorMode::Wandering => {

                creature_state.consume_energy(ENERGY_COST_WANDERING);


            }
              // TODO(Future): Add Fleeing, Resting transitions when those behaviors are implemented
        }


        if previous_behavior != creature_state.behavior {
            match creature_state.behavior {
                BehaviorMode::Catatonic => {

                }
                BehaviorMode::Seeking => {

                }
                BehaviorMode::Wandering => {


                }
                // TODO(Future): Add Fleeing component management when implemented
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_creature_aging() {
        let mut world = World::new();

        let entity = world.spawn((CreatureState::new(),)).id();

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
    fn test_catatonic_stays_catatonic() {
        let mut world = World::new();

        let entity = world
            .spawn((
                CreatureState::default(),
            ))
            .id();


        let mut query = world.query::<&mut CreatureState>();
        for mut state in query.iter_mut(&mut world) {
            let previous = state.behavior;

            assert_eq!(state.behavior, previous);
        }

        let state = world.get::<CreatureState>(entity).unwrap();
        assert_eq!(state.behavior, BehaviorMode::Catatonic);
    }
}
