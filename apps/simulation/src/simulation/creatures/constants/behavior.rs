//! Behavior constants
//!
//! Force budget multipliers and wander parameters.

use crate::simulation::math::UnitInterval;

// =============================================================================
// FORCE BUDGET MULTIPLIERS
// =============================================================================
// max_force = mass × MAX_ACCELERATION is the PHYSICAL LIMIT.
// These multipliers define what fraction each behavior can use.

/// [ACTIVE] Wander force for exploration/foraging.
/// VALIDATED: Animals graze/forage at 15-25% to allow vigilance.
/// Low force allows avoidance and other survival behaviors to dominate when needed.
pub const WANDER_FORCE_MULT: UnitInterval = UnitInterval::new(0.25);

/// [ACTIVE] Force multiplier for seek behavior (pursuit level: 70%).
/// VALIDATED: Matches aerobic threshold research (70-80% sustainable).
pub const SEEK_FORCE_MULT: UnitInterval = UnitInterval::new(0.7);

// =============================================================================
// AVOIDANCE (TTC-BASED)
// =============================================================================

/// Critical time-to-collision threshold (seconds).
/// When TTC < this value, avoidance urgency reaches maximum.
/// 2.0 seconds gives creatures enough reaction time to steer around obstacles.
pub const CRITICAL_TTC_SECONDS: f32 = 2.0;

// =============================================================================
// WANDER BEHAVIOR
// =============================================================================

/// [ACTIVE] Radius of wander target circle (meters).
/// Creates gradual direction changes.
pub const WANDER_RADIUS: f32 = 10.0;

/// [ACTIVE] Distance ahead to project wander circle (meters).
pub const WANDER_DISTANCE: f32 = 20.0;

/// [ACTIVE] Max angle change per tick (degrees).
/// VALIDATED: At 20 Hz, allows 90°/s maximum turn rate during wander.
pub const ANGLE_CHANGE: f32 = 4.5;
