//! Physics & Movement constants
//!
//! Reference values for allometric scaling and movement physics.
//!
//! BIOLOGICAL BASIS:
//! 1m body length is medium dog/small deer/large cat - allows scaling both
//! up (elephants, whales) and down (mice, insects) using standard allometry.

use std::f32::consts::PI;

// =============================================================================
// REFERENCE VALUES (for allometric scaling)
// =============================================================================

/// [FUTURE] Reference creature body length (meters).
/// VALIDATED: Excellent reference point for allometric scaling.
pub const REFERENCE_BODY_LENGTH: f32 = 1.0;

/// [FUTURE] Mass at reference size (kg).
/// Real-world 1m creatures: Border Collie 15-22kg, German Shepherd 30-40kg,
/// Coyote 7-20kg, Kangaroo 25-35kg. 35kg implies stocky/muscular build.
pub const REFERENCE_MASS: f32 = 35.0;

/// [FUTURE] Maximum sprint speed at reference size (m/s).
/// VALIDATED: 15 m/s = 54 km/h matches empirical data perfectly.
/// Real-world: Wolf 14-17 m/s, Deer 13-16 m/s, Lion 15-20 m/s.
pub const REFERENCE_MAX_SPEED: f32 = 20.0;

/// [FUTURE] Maximum acceleration at reference size (m/s²).
/// Real-world: Most quadrupeds 5-15 m/s², Dogs 6-9 m/s², Lions 7-10 m/s².
pub const REFERENCE_MAX_ACCEL: f32 = 10.0;

/// [FUTURE] Turn rate at reference size (degrees/second).
/// VALIDATED: Matches empirical quadruped data.
/// Real-world: Dogs/wolves at speed 90-180°/s, Cats 200-300°/s.
pub const REFERENCE_TURN_RATE: f32 = 180.0;

// =============================================================================
// ACTIVE VALUES (used by current systems)
// =============================================================================

/// [ACTIVE] Default body length for spawned creatures.
pub const DEFAULT_BODY_LENGTH: f32 = REFERENCE_BODY_LENGTH;

/// [ACTIVE] Default mass for spawned creatures.
pub const DEFAULT_MASS: f32 = REFERENCE_MASS;

/// [ACTIVE] Global speed cap for all creatures.
pub const MAX_SPEED: f32 = REFERENCE_MAX_SPEED;

/// [ACTIVE] Default max acceleration.
pub const MAX_ACCELERATION: f32 = REFERENCE_MAX_ACCEL;

/// [ACTIVE] Turn rate limit (degrees/second).
pub const MAX_TURN_RATE: f32 = REFERENCE_TURN_RATE;

/// [ACTIVE] Turn rate in radians (derived).
pub const MAX_TURN_RATE_RAD: f32 = MAX_TURN_RATE * PI / 180.0;

// =============================================================================
// DRAG & DAMPING
// =============================================================================

/// [ACTIVE] Velocity damping coefficient.
/// Applied as: v *= exp(-DRAG * dt)
pub const DRAG_COEFFICIENT: f32 = 2.0;

/// [ACTIVE] Threshold below which creature is considered stationary.
/// VALIDATED: 5 cm/s is imperceptible.
pub const STOPPED_THRESHOLD: f32 = 0.05;

/// [ACTIVE] Speed² threshold for locomotion noise filtering.
pub const NOISE_SPEED_THRESHOLD_SQ: f32 = 0.01;

/// [FUTURE] Simulation time step (20 Hz tick rate).
/// NOTE: Actual dt comes from DeltaTime resource.
pub const DT: f32 = 0.05;
