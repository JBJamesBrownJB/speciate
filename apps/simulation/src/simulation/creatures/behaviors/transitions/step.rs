use crate::simulation::creatures::components::{BehaviorMode, Brain, BrainMode, CreatureState};
use crate::simulation::creatures::constants::{AGE_INCREMENT_PER_TICK, ENERGY_COST_WANDERING};

pub struct BehaviorStepCtx {
    pub current_time: f64,
}

pub fn step(state: &mut CreatureState, brain: &mut Brain, ctx: &BehaviorStepCtx) {
    state.age += AGE_INCREMENT_PER_TICK;

    if state.behavior == BehaviorMode::Wandering {
        state.consume_energy(ENERGY_COST_WANDERING);
    }

    if brain.mode == BrainMode::Normal {
        let age = state.age;
        let energy = state.energy;
        if brain.can_decide(ctx.current_time, age, energy) {
            brain.record_decision(ctx.current_time);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::simulation::creatures::components::CreatureState;
    use crate::simulation::creatures::constants::{AGE_INCREMENT_PER_TICK, ENERGY_COST_WANDERING};

    fn default_ctx(current_time: f64) -> BehaviorStepCtx {
        BehaviorStepCtx { current_time }
    }

    #[test]
    fn step_increments_age_by_exact_constant() {
        let mut state = CreatureState::new();
        let mut brain = Brain::default();
        let ctx = default_ctx(0.0);

        step(&mut state, &mut brain, &ctx);

        assert!(
            (state.age - AGE_INCREMENT_PER_TICK).abs() < 1e-7,
            "age {} did not match expected {}",
            state.age,
            AGE_INCREMENT_PER_TICK
        );
    }

    #[test]
    fn step_multiple_calls_accumulate_age_exactly() {
        let mut state = CreatureState::new();
        let mut brain = Brain::default();

        for tick in 0u64..50 {
            let ctx = default_ctx(tick as f64 * 0.05);
            step(&mut state, &mut brain, &ctx);
        }

        let expected = AGE_INCREMENT_PER_TICK * 50.0;
        assert!(
            (state.age - expected).abs() < 1e-5,
            "age {} differed from expected {} by more than 1e-5",
            state.age,
            expected
        );
    }

    #[test]
    fn step_wandering_drains_energy_exactly() {
        let mut state = CreatureState::new();
        state.behavior = BehaviorMode::Wandering;
        let energy_before = state.energy;
        let mut brain = Brain::default();
        let ctx = default_ctx(0.0);

        step(&mut state, &mut brain, &ctx);

        let expected_energy = energy_before - ENERGY_COST_WANDERING;
        assert!(
            (state.energy - expected_energy).abs() < 1e-7,
            "energy {} did not match expected {}",
            state.energy,
            expected_energy
        );
    }

    #[test]
    fn step_seeking_does_not_drain_energy() {
        let mut state = CreatureState::new();
        state.behavior = BehaviorMode::Seeking;
        let energy_before = state.energy;
        let mut brain = Brain::default();
        let ctx = default_ctx(0.0);

        step(&mut state, &mut brain, &ctx);

        assert!(
            (state.energy - energy_before).abs() < 1e-7,
            "seeking should not drain energy; got {} expected {}",
            state.energy,
            energy_before
        );
    }

    #[test]
    fn step_catatonic_does_not_drain_energy() {
        let mut state = CreatureState::new();
        let energy_before = state.energy;
        let mut brain = Brain::default();
        let ctx = default_ctx(0.0);

        step(&mut state, &mut brain, &ctx);

        assert!(
            (state.energy - energy_before).abs() < 1e-7,
            "catatonic should not drain energy; got {} expected {}",
            state.energy,
            energy_before
        );
    }

    #[test]
    fn step_wandering_energy_clamps_at_zero() {
        let mut state = CreatureState::new();
        state.behavior = BehaviorMode::Wandering;
        state.energy = 0.0;
        let mut brain = Brain::default();
        let ctx = default_ctx(0.0);

        step(&mut state, &mut brain, &ctx);

        assert_eq!(state.energy, 0.0, "energy must not go below 0.0");
    }

    #[test]
    fn step_normal_brain_records_decision_after_cooldown_elapsed() {
        let mut state = CreatureState::new();
        let mut brain = Brain::default();
        brain.last_decision_time = 0.0;

        let ctx = default_ctx(0.15);

        step(&mut state, &mut brain, &ctx);

        assert!(
            (brain.last_decision_time - 0.15).abs() < 1e-10,
            "brain should have recorded decision at 0.15; got {}",
            brain.last_decision_time
        );
    }

    #[test]
    fn step_normal_brain_no_decision_before_cooldown_elapsed() {
        let mut state = CreatureState::new();
        let mut brain = Brain::default();
        brain.last_decision_time = 0.0;

        let ctx = default_ctx(0.10);

        step(&mut state, &mut brain, &ctx);

        assert!(
            brain.last_decision_time < 0.10,
            "brain should not have decided before cooldown; last_decision_time={}",
            brain.last_decision_time
        );
    }

    #[test]
    fn step_dormant_brain_never_records_decision() {
        let mut state = CreatureState::new();
        let mut brain = Brain::dormant();
        let ctx = default_ctx(1000.0);

        step(&mut state, &mut brain, &ctx);

        assert_eq!(
            brain.last_decision_time, 0.0,
            "dormant brain must never record a decision"
        );
    }

    #[test]
    fn step_age_updated_before_brain_decision() {
        let mut state = CreatureState::new();
        state.age = 0.0;
        let mut brain = Brain::default();
        brain.last_decision_time = 0.0;

        let ctx = default_ctx(0.15);

        step(&mut state, &mut brain, &ctx);

        assert!(
            (state.age - AGE_INCREMENT_PER_TICK).abs() < 1e-7,
            "age must be updated in step"
        );
        assert!(
            (brain.last_decision_time - 0.15).abs() < 1e-10,
            "brain decision must have been recorded using updated age"
        );
    }

    #[test]
    fn step_waiting_does_not_drain_energy() {
        let mut state = CreatureState::new();
        state.behavior = BehaviorMode::Waiting;
        let energy_before = state.energy;
        let mut brain = Brain::default();
        let ctx = default_ctx(0.0);

        step(&mut state, &mut brain, &ctx);

        assert!(
            (state.energy - energy_before).abs() < 1e-7,
            "waiting should not drain energy; got {} expected {}",
            state.energy,
            energy_before
        );
    }
}
