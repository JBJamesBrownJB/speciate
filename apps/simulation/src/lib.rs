//! Speciate simulation library
//!
//! This crate provides a high-performance ECS-based simulation for autonomous creatures.
//!
//! ## Module Organization
//!
//! - `simulation` - Core simulation logic (ECS, physics, behaviors)
//! - `persistence` - Save state management
//! - `ipc` - Inter-process communication (NAPI, snapshots)
//! - `config` - Configuration structs
//! - `state` - State management
//! - `napi_addon` - Native Node.js addon (NAPI-RS bridge)
//! - `trials` - Trial loading system (dev-tools feature)
//! - `instrumentation` - Performance metrics (dev-tools feature)
//!
//! ## Re-export Pattern
//!
//! This module re-exports commonly used types for convenience. All exports are explicit
//! (no glob re-exports) to provide a clear, discoverable API surface.

pub mod config;
pub mod simulation;
pub mod persistence;
pub mod ipc;
pub mod state;
#[cfg(feature = "napi")]
pub mod napi_addon;

#[cfg(feature = "napi")]
pub use napi_addon::*;

#[cfg(feature = "dev-tools")]
pub mod trials;

pub mod instrumentation;

#[cfg(feature = "dev-tools")]
#[macro_export]
macro_rules! time_system {
    ($timings:expr, $name:expr) => {
        let _guard = $timings.time($name);
    };
}

#[cfg(not(feature = "dev-tools"))]
#[macro_export]
macro_rules! time_system {
    ($timings:expr, $name:expr) => {};
}


// Core simulation types
pub use simulation::core::{Simulation, SimulationBuilder};
pub use simulation::core::components::{
    Acceleration, BodySize, BoundaryConfig, DeltaTime,
    PhysicsTick, Position, Velocity,
};

// Creature components and capabilities
pub use simulation::creatures::components::{
    BehaviorMode, Brain, BrainMode, CanAvoidObstacles, CanFlee, CanSeek, CanWander,
    CreatureState, CritId, FleeState, HomePosition, Target, WanderState,
};
pub use simulation::components::Rotation;

// Perception
pub use simulation::perception::{AvoidanceBehavior, Perception};

// Creature spawning
pub use simulation::creatures::builder::CritBuilder;
pub use simulation::creatures::spawner::{
    spawn_creature, CreatureSpawnRequest,
};

// Systems (for custom schedules)
pub use simulation::creatures::behaviors::{
    behavior_transition_system, flee_system, seek_system, territory_wandering_system,
};

// Math utilities
pub use simulation::math;
pub use simulation::movement::{
    integrate_motion_system, rotation_system,
};

// IPC types
pub use ipc::{CreatureSnapshot, GameState, SharedSnapshotQueue, SnapshotQueue};

// NAPI exports

