//! Shared state types for IPC
//!
//! This module contains serializable state types used by stdio and
//! other IPC backends for communicating simulation state.

pub mod snapshot_queue;

// Re-export for convenience
pub use snapshot_queue::{CreatureSnapshot, GameState, SharedSnapshotQueue, SnapshotQueue};
