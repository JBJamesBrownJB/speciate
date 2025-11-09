//! Creature behavior domain
//!
//! Contains all creature-specific logic: capabilities, behavioral states,
//! steering behaviors, and spawning. Systems in this module ADD forces
//! to acceleration (force accumulation pattern).

pub mod behaviors;
pub mod builder;
pub mod components;
pub mod events;
pub mod spawner;
pub mod systems;

pub use behaviors::*;
pub use builder::*;
pub use components::*;
pub use events::*;
pub use spawner::*;
pub use systems::*;
