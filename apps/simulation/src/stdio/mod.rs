//! Stdio IPC backend for Electron integration
//!
//! This module provides hooks that emit MessagePack-encoded game state
//! to stdout using a length-prefixed protocol. Designed for Electron IPC communication.

mod hooks;

pub use hooks::StdioHooks;

// Re-export GameState types from ipc module (they're the same for all IPC backends)
pub use crate::ipc::snapshot_queue::{CreatureSnapshot, GameState};
