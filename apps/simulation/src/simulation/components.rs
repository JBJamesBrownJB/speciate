
use bevy_ecs::prelude::*;
use bevy_reflect::Reflect;
use serde::{Deserialize, Serialize};


pub use super::core::components::{
    Acceleration, BoundaryConfig, Catatonic, DeltaTime, Position, Velocity,
};


pub use super::creatures::components::{

    BehaviorMode,
    CanAvoidObstacles,
    CanFlee,

    CanSeek,
    CanWander,
    CreatureState,
    HomePosition,

    CritId,
    FleeState,

    Target,
    WanderState,
};

#[derive(Component, Clone, Copy, Debug, Default, Serialize, Deserialize, Reflect)]
#[reflect(Component)]
pub struct Rotation {
    pub radians: f32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_creature_state_energy_management() {
        let mut state = CreatureState::new();
        let initial_energy = state.energy;

        state.consume_energy(10.0);
        assert_eq!(state.energy, initial_energy - 10.0);

        state.restore_energy(5.0);
        assert_eq!(state.energy, initial_energy - 5.0);
    }

    #[test]
    fn test_creature_state_exhaustion() {
        let mut state = CreatureState::new();


        state.consume_energy(75.0);
        assert!(state.is_low_energy());
        assert!(!state.is_exhausted());


        state.consume_energy(20.0);
        assert!(state.is_exhausted());
    }
}
