use crate::simulation::components::*;
use crate::simulation::core::components::PhysicsTick;
use bevy_ecs::prelude::*;

const AGE_INCREMENT_PER_TICK: f32 = 0.001;
const ENERGY_COST_WANDERING: f32 = 0.01;
const TICK_INTERVAL_SECONDS: f64 = 0.05;

pub fn behavior_transition_system(
    physics_tick: Res<PhysicsTick>,
    mut query: Query<(
        &mut CreatureState,
        &mut Brain,
        &Position,
        &mut Target,
    )>,
    #[cfg(feature = "dev-tools")] timings: bevy_ecs::system::Res<
        crate::instrumentation::SystemTimings,
    >,
) {
    #[cfg(feature = "dev-tools")]
    crate::time_system!(timings, "behavior_transition");

    let current_time = physics_tick.get() as f64 * TICK_INTERVAL_SECONDS;

    for (mut creature_state, mut brain, position, mut target) in query.iter_mut() {
        creature_state.age += AGE_INCREMENT_PER_TICK;

        match creature_state.behavior {
            BehaviorMode::Catatonic => {}
            BehaviorMode::Seeking => {}
            BehaviorMode::Wandering => {
                creature_state.consume_energy(ENERGY_COST_WANDERING);
            }
        }

        let age = creature_state.age;
        let energy = creature_state.energy;

        match brain.mode {
            BrainMode::Normal => {
                if brain.can_decide(current_time, age, energy) {
                    brain.record_decision(current_time);
                    // Future: perception-based decision logic
                    // For now, no automatic transitions
                }
            }
            BrainMode::Cycling => {
                if brain.can_decide(current_time, age, energy) {
                    creature_state.behavior = cycle_behavior_with_target(
                        creature_state.behavior,
                        position,
                        &mut target
                    );
                    brain.record_decision(current_time);
                }
            }
            BrainMode::Dormant => {}
        }
    }
}

fn generate_random_target(position: &Position) -> Target {
    use rand::Rng;
    let mut rng = rand::thread_rng();

    let distance = rng.gen_range(50.0..200.0);
    let angle = rng.gen_range(0.0..std::f32::consts::TAU);

    Target::new(
        position.x + distance * angle.cos(),
        position.y + distance * angle.sin(),
    )
}

fn cycle_behavior_with_target(current: BehaviorMode, position: &Position, target: &mut Target) -> BehaviorMode {
    match current {
        BehaviorMode::Catatonic => BehaviorMode::Wandering,
        BehaviorMode::Wandering => {
            *target = generate_random_target(position);
            BehaviorMode::Seeking
        },
        BehaviorMode::Seeking => BehaviorMode::Catatonic,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_creature_aging() {
        let mut world = World::new();

        let entity = world.spawn((CreatureState::new(), Brain::new())).id();

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

    #[test]
    fn test_cycling_brain_changes_behavior() {
        let mut brain = Brain::cycling();

        let mut state = CreatureState::new();
        state.behavior = BehaviorMode::Catatonic;
        let position = Position { x: 100.0, y: 100.0 };
        let mut target = Target::new(0.0, 0.0);

        // Young creature with full energy - base cooldown ~150ms
        assert!(brain.can_decide(0.15, 0.0, 100.0));
        state.behavior = cycle_behavior_with_target(state.behavior, &position, &mut target);
        brain.record_decision(0.15);

        assert_eq!(state.behavior, BehaviorMode::Wandering);

        // After 150ms more, can decide again
        assert!(brain.can_decide(0.30, 0.0, 100.0));
        let old_target = target;
        state.behavior = cycle_behavior_with_target(state.behavior, &position, &mut target);

        assert_eq!(state.behavior, BehaviorMode::Seeking);
        assert_ne!(target.x, old_target.x, "Target should be updated when transitioning to Seeking");
        assert_ne!(target.y, old_target.y, "Target should be updated when transitioning to Seeking");
    }

    #[test]
    fn test_cycle_behavior_sequence() {
        let mut target = Target::new(0.0, 0.0);
        let position = Position { x: 100.0, y: 100.0 };

        assert_eq!(
            cycle_behavior_with_target(BehaviorMode::Catatonic, &position, &mut target),
            BehaviorMode::Wandering
        );

        let old_target = target;
        assert_eq!(
            cycle_behavior_with_target(BehaviorMode::Wandering, &position, &mut target),
            BehaviorMode::Seeking
        );
        assert_ne!(target.x, old_target.x, "Target should be updated");
        assert_ne!(target.y, old_target.y);

        assert_eq!(
            cycle_behavior_with_target(BehaviorMode::Seeking, &position, &mut target),
            BehaviorMode::Catatonic
        );
    }
}
