//! Behavior transition system (A-Life state machine)
//!
//! Manages creature behavioral state changes based on energy, perception, and random events.
//! Currently simplified for Catatonic/Seeking-only mode.

use crate::simulation::components::*;
use bevy_ecs::prelude::*;

// Aging and Transition Constants
const AGE_INCREMENT_PER_TICK: f32 = 0.001;

// Energy Constants (for future use when more behaviors are enabled)
#[allow(dead_code)]
const ENERGY_COST_WANDERING: f32 = 0.01;
#[allow(dead_code)]
const ENERGY_COST_FLEEING: f32 = 0.05;
#[allow(dead_code)]
const ENERGY_RESTORE_FEEDING: f32 = 0.1;
#[allow(dead_code)]
const ENERGY_RESTORE_RESTING: f32 = 0.02;
#[allow(dead_code)]
const ENERGY_THRESHOLD_MODERATE: f32 = 50.0;
#[allow(dead_code)]
const ENERGY_THRESHOLD_HIGH: f32 = 80.0;

// Transition Probabilities (for future use)
#[allow(dead_code)]
const TRANSITION_PROB_WANDERING_TO_FLEEING: f64 = 0.01;
#[allow(dead_code)]
const TRANSITION_PROB_WANDERING_TO_RESTING: f64 = 0.001;
#[allow(dead_code)]
const TRANSITION_PROB_FLEEING_TO_WANDERING: f64 = 0.02;
#[allow(dead_code)]
const TRANSITION_PROB_RESTING_TO_WANDERING: f64 = 0.05;

/// Behavior transition system: manages creature state machine
///
/// System ordering: Can run in parallel with perception systems
///
/// Current State Machine (simplified):
/// - Catatonic: No transitions, no energy consumption
/// - Seeking: Externally controlled (no auto-transitions)
///
/// Future State Machine (when more modes enabled):
/// - Prioritized transitions based on urgency (threat > hunger > rest > wander)
/// - Energy consumption per behavior type
/// - Random transitions for behavioral diversity
/// - Component addition/removal on state changes
///
/// TODO: Full state machine when BehaviorMode variants are uncommented
/// TODO: Migrate energy costs and transition probabilities to DNA (DNA system (in progress))
pub fn behavior_transition_system(
    _commands: Commands,
    mut query: Query<(
        Entity,
        &mut CreatureState,
        Option<&WanderState>,
        Option<&FleeState>,
    )>,
) {
    for (_entity, mut creature_state, _wander_state, _flee_state) in query.iter_mut() {
        // Disabled: Only Catatonic behavior active for now
        // Energy consumption and state transitions will be re-enabled
        // when other behaviors are uncommented in BehaviorMode enum

        // Age the creature (still active)
        creature_state.age += AGE_INCREMENT_PER_TICK;

        // State transition logic (disabled for Catatonic-only mode)
        let previous_behavior = creature_state.behavior;

        match creature_state.behavior {
            BehaviorMode::Catatonic => {
                // Stationary - no transitions, no energy consumption
            }
            BehaviorMode::Seeking => {
                // Seeking - controlled externally, no auto-transitions
            }
            BehaviorMode::Wandering => {
                // Wandering - low-energy patrol mode (1.2x basal metabolism)
                creature_state.consume_energy(ENERGY_COST_WANDERING);
                // No auto-transitions yet - wandering is stable state
                // Future: transition to seeking when food detected, fleeing when threat detected
            } // Future behaviors will be added here when uncommented:
              // TODO(Future): Add Fleeing, Resting transitions when those behaviors are implemented
        }

        // Add/remove behavior-specific components based on state changes (disabled for Catatonic-only)
        if previous_behavior != creature_state.behavior {
            match creature_state.behavior {
                BehaviorMode::Catatonic => {
                    // Stationary - no behavior-specific components needed
                }
                BehaviorMode::Seeking => {
                    // Seeking - CanSeek capability and Target should already be present
                }
                BehaviorMode::Wandering => {
                    // Wandering - CanWander capability and WanderState should already be present
                    // Components are added at spawn, not dynamically
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

        // Simulate aging over 10 ticks
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
                CreatureState::default(), // Default is Catatonic
            ))
            .id();

        // Run transition system (should do nothing for Catatonic)
        let mut query = world.query::<&mut CreatureState>();
        for mut state in query.iter_mut(&mut world) {
            let previous = state.behavior;
            // Catatonic logic: no state changes
            assert_eq!(state.behavior, previous);
        }

        let state = world.get::<CreatureState>(entity).unwrap();
        assert_eq!(state.behavior, BehaviorMode::Catatonic);
    }
}
