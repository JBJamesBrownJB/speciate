//! Dev commands for local testing
//!
//! This module provides development-only commands that can be sent via NATS
//! to control the simulation for testing purposes. This code is completely
//! removed in production builds via the `dev-commands` feature flag.
//!
//! **SECURITY:** This module is for LOCAL DEVELOPMENT ONLY. It should NEVER
//! be enabled in production builds.

#[cfg(feature = "dev-commands")]
pub mod commands;
#[cfg(feature = "dev-commands")]
pub mod listener;
#[cfg(feature = "dev-commands")]
pub mod systems;

#[cfg(feature = "dev-commands")]
pub use commands::DevCommand;
#[cfg(feature = "dev-commands")]
pub use listener::DevCommandListener;
#[cfg(feature = "dev-commands")]
pub use systems::{process_dev_commands_system, DevSpawnIdCounter};
