//! Core simulation infrastructure
//!
//! This module contains the fundamental simulation orchestration,
//! timing, and shared spatial components used across all domains.

pub mod components;
pub mod simulation;
pub mod timing;
pub mod world_bounds;

// Re-export commonly used types
pub use components::*;
pub use simulation::{Simulation, SimulationBuilder};
pub use timing::*;
pub use world_bounds::WorldBounds;
