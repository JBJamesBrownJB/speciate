//! Creature steering behaviors
//!
//! All behavior systems follow the force accumulation pattern:
//! they ADD to Acceleration, never replace it. This allows multiple
//! behaviors to blend naturally (emergent obstacle avoidance, etc.).
//!
//! System ordering: These run BEFORE physics integration.

pub mod avoidance;
pub mod flee;
pub mod seek;
pub mod transitions;
pub mod wander;

pub use avoidance::*;
pub use flee::*;
pub use seek::*;
pub use transitions::*;
pub use wander::*;
