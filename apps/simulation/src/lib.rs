//! Speciate - Server-authoritative AI life simulation
//!
//! This library provides the core ECS-based simulation engine for the Speciate project.

pub mod simulation;
pub mod state;
pub mod state_loader;
pub mod config;
pub mod spawner;
pub mod snapshot;
pub mod snapshot_worker;

// Re-export commonly used types
pub use simulation::{components::*, systems::*};
