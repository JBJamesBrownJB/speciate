//! Fused steering behavior system.
//!
//! This module consolidates all steering behaviors (wander, seek, avoidance, flee)
//! into a single system for improved performance. Instead of 4 separate queries and
//! iterations, we have 1 query and 1 iteration with all steering forces accumulated.
//!
//! Architecture:
//! - Pure functions calculate individual forces (testable in isolation)
//! - `update_steering_system` fuses all behaviors into single parallel iteration
//! - Uses `accumulate_steering` from math module for additive force blending

mod avoidance;
mod flee;
mod seek;
mod system;
mod wander;

pub use avoidance::{
    calculate_avoidance, AvoidanceConfig, AvoidanceInput, AvoidanceOutput, Neighbor,
};
pub use flee::calculate_flee_force;
pub use seek::{calculate_arrival, ArrivalParams, ArrivalResult};
pub use system::update_steering_system;
pub use wander::{calculate_wander, WanderParams, WanderResult};
