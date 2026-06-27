use crate::simulation::core::components::{FreqConfig, PhysicsTick};
use crate::simulation::core::FrequencyThrottle;
use crate::simulation::creatures::components::{BehaviorMode, Brain, BrainMode, CreatureState};
use crate::simulation::creatures::constants::{
    AGE_INCREMENT_PER_TICK, ENERGY_COST_WANDERING, TICK_INTERVAL_SECONDS,
};
use bevy_ecs::prelude::*;
use rayon::prelude::*;

pub fn behavior_transition_system(
    physics_tick: Res<PhysicsTick>,
    freq: Res<FreqConfig>,
    mut query: Query<(Entity, &mut CreatureState, &mut Brain)>,
    #[cfg(feature = "dev-tools")] timings: bevy_ecs::system::Res<
        crate::instrumentation::SystemTimings,
    >,
) {
    #[cfg(feature = "dev-tools")]
    crate::time_system!(timings, "behavior_transition");

    let current_time = physics_tick.get() as f64 * TICK_INTERVAL_SECONDS;

    // Frequency throttling: entity-ID bucketing with power-of-2 optimization.
    // Hoisted above the collect so the throttle gate runs once, serially, during the
    // (already-serial) ECS scan rather than once per dispatched closure in parallel.
    let throttle = FrequencyThrottle::new(freq.behavior_divisor, physics_tick.get());

    // Compact the active set BEFORE the parallel dispatch: keep only the entities in
    // this tick's throttle bucket. This shrinks the dispatched Vec ~`behavior_divisor`×
    // and removes the no-op Rayon work units (and their scheduler overhead) that
    // throttled entities incur when collected unconditionally.
    let mut entities: Vec<_> = query
        .iter_mut()
        .filter(|(entity, _, _)| throttle.should_process(entity.index()))
        .collect();

    // Transitions: Light workload - moderate chunks.
    // Throttle check is omitted here — every entity in `entities` already passed the filter.
    entities.par_iter_mut().with_min_len(256).for_each(|(_, creature_state, brain)| {
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
    fn test_compact_active_set_cadence_preserved() {
        // Verifies that the compact-active-set filter preserves the throttle cadence.
        // With behavior_divisor=4, only 1-in-4 entities are processed per tick.
        // Entity index 0 falls in bucket 0, so it is processed at ticks 0, 4, 8, ...
        // Over 100 ticks that is 25 processing events → 25 × AGE_INCREMENT_PER_TICK.
        let divisor = 4u8;
        let mut age = 0.0f32;

        for tick in 0u64..100 {
            let throttle = FrequencyThrottle::new(divisor, tick);
            // Simulate what the filter does: only proceed when should_process is true.
            if throttle.should_process(0) {
                age += AGE_INCREMENT_PER_TICK;
            }
        }

        // 100 ticks / divisor-4 = 25 processing events.
        // Use approximate equality: 25 float additions accumulate rounding error (~1e-7).
        let expected = AGE_INCREMENT_PER_TICK * 25.0;
        assert!(
            (age - expected).abs() < 1e-6,
            "age {age} differed from expected {expected} by more than 1e-6"
        );
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
