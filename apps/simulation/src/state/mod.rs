//! World state loading and initialization
//!
//! Handles loading persisted state from TOML files on server startup.

pub mod loader;
pub mod snapshot;

pub use loader::*;

// snapshot is for internal use only
