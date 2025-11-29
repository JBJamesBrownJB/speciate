use bevy_ecs::prelude::*;
use bevy_reflect::Reflect;
use serde::{Deserialize, Serialize};

const BASE_COOLDOWN_MS: f32 = 150.0;
const AGE_SENSITIVITY: f32 = 2.0;
const MAX_AGE: f32 = 100.0; // Normalized age calculation
const MAX_ENERGY: f32 = 100.0;
const PANIC_THRESHOLD: f32 = 2.0; // body_size multiplier

#[derive(Clone, Copy, Debug, PartialEq, Default, Serialize, Deserialize, Reflect)]
#[repr(u8)]
pub enum BrainMode {
    #[default]
    Normal = 0,
    Cycling = 1,
    Dormant = 2,
}

impl BrainMode {
    pub fn makes_decisions(&self) -> bool {
        !matches!(self, BrainMode::Dormant)
    }
}

#[derive(Component, Clone, Copy, Debug, Serialize, Deserialize, Reflect)]
#[reflect(Component)]
pub struct Brain {
    pub mode: BrainMode,
    #[serde(skip)]
    #[reflect(ignore)]
    pub last_decision_time: f64,
}

impl Default for Brain {
    fn default() -> Self {
        Self {
            mode: BrainMode::Normal,
            last_decision_time: 0.0,
        }
    }
}

impl Brain {
    pub fn with_mode(mode: BrainMode) -> Self {
        Self {
            mode,
            ..Default::default()
        }
    }

    pub fn cycling() -> Self {
        Self::with_mode(BrainMode::Cycling)
    }

    pub fn dormant() -> Self {
        Self::with_mode(BrainMode::Dormant)
    }

    pub fn effective_cooldown_ms(&self, age: f32, energy: f32) -> f32 {
        let age_normalized = (age / MAX_AGE).clamp(0.0, 1.0);
        let energy_normalized = (energy / MAX_ENERGY).clamp(0.0, 1.0);

        let age_factor = 1.0 + age_normalized.powf(2.5) * AGE_SENSITIVITY;
        let energy_factor = 1.0 + (1.0 - energy_normalized).powf(2.0) * 1.5;

        BASE_COOLDOWN_MS * age_factor * energy_factor
    }

    pub fn can_decide(&self, current_time: f64, age: f32, energy: f32) -> bool {
        if !self.mode.makes_decisions() {
            return false;
        }
        let cooldown_sec = self.effective_cooldown_ms(age, energy) as f64 / 1000.0;
        (current_time - self.last_decision_time) >= cooldown_sec
    }

    pub fn record_decision(&mut self, current_time: f64) {
        self.last_decision_time = current_time;
    }
}

pub fn should_panic(nearest_threat_dist: f32, body_size: f32, energy: f32) -> bool {
    if energy < 5.0 {
        return false; // "Giving up" - too weak to panic
    }
    nearest_threat_dist < (PANIC_THRESHOLD * body_size)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_brain_default_is_normal_mode() {
        let brain = Brain::default();
        assert_eq!(brain.mode, BrainMode::Normal);
        assert!(brain.mode.makes_decisions());
    }

    #[test]
    fn test_brain_dormant_does_not_make_decisions() {
        let brain = Brain::dormant();
        assert_eq!(brain.mode, BrainMode::Dormant);
        assert!(!brain.mode.makes_decisions());
    }

    #[test]
    fn test_brain_cycling_makes_decisions() {
        let brain = Brain::cycling();
        assert_eq!(brain.mode, BrainMode::Cycling);
        assert!(brain.mode.makes_decisions());
    }

    #[test]
    fn test_brain_respects_dynamic_cooldown() {
        let mut brain = Brain::default();
        brain.last_decision_time = 0.0;

        // Young (age=0), full energy (100) = base cooldown 150ms
        assert!(!brain.can_decide(0.10, 0.0, 100.0)); // 100ms < 150ms
        assert!(brain.can_decide(0.15, 0.0, 100.0)); // 150ms = 150ms
        assert!(brain.can_decide(0.20, 0.0, 100.0)); // 200ms > 150ms
    }

    #[test]
    fn test_brain_cooldown_increases_with_age() {
        let brain = Brain::default();

        let young_cooldown = brain.effective_cooldown_ms(0.0, 100.0);
        let old_cooldown = brain.effective_cooldown_ms(80.0, 100.0);

        assert!(old_cooldown > young_cooldown * 1.5, "Old creatures should think slower");
    }

    #[test]
    fn test_brain_cooldown_increases_with_low_energy() {
        let brain = Brain::default();

        let full_energy = brain.effective_cooldown_ms(0.0, 100.0);
        let half_energy = brain.effective_cooldown_ms(0.0, 50.0);
        let low_energy = brain.effective_cooldown_ms(0.0, 10.0);

        assert!(half_energy > full_energy, "Low energy should slow thinking");
        assert!(low_energy > half_energy, "Very low energy should slow more");
    }

    #[test]
    fn test_brain_record_decision_updates_time() {
        let mut brain = Brain::default();
        brain.record_decision(1.5);
        assert_eq!(brain.last_decision_time, 1.5);
    }

    #[test]
    fn test_dormant_brain_cannot_decide() {
        let brain = Brain::dormant();
        assert!(!brain.can_decide(100.0, 0.0, 100.0));
    }

    #[test]
    fn test_should_panic_within_threshold() {
        assert!(should_panic(1.5, 1.0, 100.0)); // 1.5 < 2.0 * 1.0
        assert!(!should_panic(2.5, 1.0, 100.0)); // 2.5 > 2.0 * 1.0
    }

    #[test]
    fn test_should_panic_disabled_when_exhausted() {
        assert!(!should_panic(0.5, 1.0, 4.0)); // energy < 5.0 = giving up
        assert!(should_panic(0.5, 1.0, 6.0)); // energy > 5.0 = can panic
    }

    #[test]
    fn test_last_decision_time_not_serialized() {
        use serde_json;

        let mut brain = Brain::cycling();
        brain.last_decision_time = 100.0; // Simulate brain that has been running

        // Serialize
        let json = serde_json::to_string(&brain).unwrap();

        // Deserialize
        let loaded_brain: Brain = serde_json::from_str(&json).unwrap();

        // last_decision_time should be reset to 0.0 (default)
        assert_eq!(loaded_brain.last_decision_time, 0.0,
            "last_decision_time should not be serialized - it must reset on reload to prevent cycling bugs");
        assert_eq!(loaded_brain.mode, BrainMode::Cycling, "mode should be preserved");
    }

    #[test]
    fn test_cycling_brain_works_after_simulated_reload() {
        let mut brain = Brain::cycling();
        brain.last_decision_time = 100.0; // Simulate old saved state

        // Simulate serialize/deserialize (last_decision_time should reset)
        let json = serde_json::to_string(&brain).unwrap();
        let reloaded_brain: Brain = serde_json::from_str(&json).unwrap();

        // After reload, brain should be able to make decisions immediately
        // (or after base cooldown, not waiting for time 100.0+ again!)
        assert!(reloaded_brain.can_decide(0.15, 0.0, 100.0),
            "Reloaded brain should be able to decide after base cooldown (150ms), not stuck waiting for time 100+");
    }
}
