//! Movement domain systems
//!
//! Pure motion integration - applies accumulated forces to velocity/position using Euler integration.
//! These systems run AFTER behavior systems have applied forces to Acceleration.
//!
//! Also includes rotation system which updates creature orientation based on velocity direction.

pub mod components;
pub mod constants;
pub mod noise;
pub mod rotation;
pub mod systems;

pub use constants::*;
pub use noise::*;
pub use rotation::*;
pub use systems::*;
