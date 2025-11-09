//! Speciate - Server-authoritative AI life simulation
//!
//! This library provides the core ECS-based simulation engine for the Speciate project.

pub mod config;
pub mod nats;
pub mod simulation;
pub mod snapshots;
pub mod state;

// Dev commands module (feature-gated, local testing only)
#[cfg(feature = "dev-commands")]
pub mod dev_commands;

// Re-export commonly used types from simulation module
pub use simulation::{components::*, core::*};

// Re-export systems from their domain modules
pub use simulation::creatures::behaviors::{
    behavior_transition_system, flee_system, seek_system, territory_wandering_system,
};
pub use simulation::movement::{
    integrate_motion_system, rotation_system,
};

// Re-export builder and spawner for backward compatibility
pub use simulation::creatures::builder::CritBuilder;
pub use simulation::creatures::spawner::{
    spawn_creature, spawn_initial_creatures, CreatureSpawnRequest,
};
