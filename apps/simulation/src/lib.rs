//! Speciate - Server-authoritative AI life simulation
//!
//! This library provides the core ECS-based simulation engine for the Speciate project.

pub mod simulation;
pub mod network;
pub mod state;

// Re-export commonly used types
pub use simulation::{components::*, systems::*};
pub use network::{AppState, ws_handler};
