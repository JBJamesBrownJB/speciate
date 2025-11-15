//! Shared state types for IPC
//!
//! This module contains serializable state types used by stdio and
//! other IPC backends for communicating simulation state.

pub mod snapshot_queue;

// Command system (dev tools)
#[cfg(feature = "dev-tools")]
pub mod commands;
#[cfg(feature = "dev-tools")]
pub mod stdin_reader;
#[cfg(feature = "dev-tools")]
pub mod command_executor;

// Re-export for convenience
pub use snapshot_queue::{CreatureSnapshot, GameState, SharedSnapshotQueue, SnapshotQueue};

#[cfg(feature = "dev-tools")]
pub use commands::Command;
#[cfg(feature = "dev-tools")]
pub use command_executor::CommandReceiver;
#[cfg(feature = "dev-tools")]
pub use stdin_reader::spawn_stdin_reader_thread;

// Re-export command executor system
#[cfg(feature = "dev-tools")]
pub use command_executor::command_executor_system;
