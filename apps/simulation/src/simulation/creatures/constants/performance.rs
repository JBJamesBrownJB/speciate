//! Performance tuning constants
//!
//! These constants control system skipping and load distribution.
//! Used across multiple systems (perception, behavior transitions, etc.)

/// Number of update slices for system skipping.
/// Creatures are distributed across slices; each tick only one slice runs.
/// 2 = 50% load per tick, 4 = 25% load per tick.
pub const UPDATE_SLICE_COUNT: u8 = 2;
