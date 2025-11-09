//! Creature-specific ECS components
//!
//! Following the hybrid ECS pattern:
//! - Capability markers (CanSeek, CanFlee) - permanent, zero-sized
//! - State components (CreatureState) - mutable, contains BehaviorMode enum
//! - Data components (Target, WanderState) - just coordinates/configuration

pub mod capabilities;
pub mod identity;
pub mod perception;
pub mod state;

pub use capabilities::*;
pub use identity::*;
pub use perception::*;
pub use state::*;
