
pub mod config;
pub mod simulation;
pub mod persistence;
pub mod ipc;
pub mod stdio;
pub mod state;
pub mod runner;


#[cfg(feature = "dev-tools")]
pub mod trials;


pub use simulation::{components::*, core::*};


pub use simulation::creatures::behaviors::{
    behavior_transition_system, flee_system, seek_system, territory_wandering_system,
};
pub use simulation::movement::{
    integrate_motion_system, rotation_system,
};


pub use simulation::creatures::builder::CritBuilder;
pub use simulation::creatures::spawner::{
    spawn_creature, spawn_initial_creatures, CreatureSpawnRequest,
};


pub use ipc::{CreatureSnapshot, GameState, SharedSnapshotQueue, SnapshotQueue};


pub use runner::{ConsoleHooks, NoOpHooks, RunnerConfig, RunnerHooks, SimulationRunner};


pub use stdio::StdioHooks;
