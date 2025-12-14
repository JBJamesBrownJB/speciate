//! Behavior constants
//!
//! Force budget multipliers, avoidance, seek, and wander parameters.

use crate::simulation::math::UnitInterval;

// =============================================================================
// FORCE BUDGET MULTIPLIERS
// =============================================================================
// max_force = mass × MAX_ACCELERATION is the PHYSICAL LIMIT.
// These multipliers define what fraction each behavior can use.

/// [FUTURE] Emergency force (flee, brake, fight) - full muscular output.
/// VALIDATED: Fight-or-flight response triggers maximal output.
pub const EMERGENCY_FORCE_MULT: UnitInterval = UnitInterval::new(1.0);

/// [FUTURE] Pursuit force for sustained chase.
/// VALIDATED: Matches aerobic threshold research (70-80% sustainable).
pub const PURSUIT_FORCE_MULT: UnitInterval = UnitInterval::new(0.7);

/// [FUTURE] Cruise force for directed travel (migration, commuting).
/// VALIDATED: Energy-efficient gaits operate at 35-45% of max.
pub const CRUISE_FORCE_MULT: UnitInterval = UnitInterval::new(0.4);

/// [ACTIVE] Wander force for exploration/foraging.
/// VALIDATED: Animals graze/forage at 15-25% to allow vigilance.
/// Low force allows avoidance and other survival behaviors to dominate when needed.
pub const WANDER_FORCE_MULT: UnitInterval = UnitInterval::new(0.25);

/// [ACTIVE] Force multiplier for seek behavior (currently uses PURSUIT level).
pub const SEEK_FORCE_MULT: UnitInterval = PURSUIT_FORCE_MULT;

/// [LEGACY] Alias for emergency force - use EMERGENCY_FORCE_MULT instead.
pub const BRAKE_FORCE_MULT: UnitInterval = EMERGENCY_FORCE_MULT;

// =============================================================================
// PERSONAL SPACE & AVOIDANCE
// =============================================================================

/// [ACTIVE] Personal space = body_radius × this multiplier.
/// 2× radius = 1 body diameter - appropriate for social species at low speed.
pub const PERSONAL_SPACE_MULTIPLIER: f32 = 2.0;

/// [ACTIVE] Seeking creatures tolerate closer proximity (tunnel vision during pursuit).
/// VALIDATED: Hunting animals override personal space concerns.
pub const SEEKING_SPACE_REDUCTION: f32 = 0.5;

/// [ACTIVE] Emergency braking distance - apply max avoidance force within this range.
/// WARNING: Fixed 50cm doesn't scale with body size!
pub const EMERGENCY_BRAKE_DISTANCE: f32 = 0.5;

/// Energy-driven personal space modifier constants.
#[derive(Debug, Clone, Copy)]
pub struct EnergyModifierConstants {
    pub min_modifier: f32,
    pub max_modifier: f32,
}

impl Default for EnergyModifierConstants {
    fn default() -> Self {
        Self {
            min_modifier: 0.4,
            max_modifier: 1.0,
        }
    }
}

/// [ACTIVE] Energy-based personal space scaling.
/// Starving (0%): 10% of normal space - desperate crowding tolerated.
/// Full (100%): 100% of normal space - territorial behavior.
pub static ENERGY_MODIFIER: EnergyModifierConstants = EnergyModifierConstants {
    min_modifier: 0.1,
    max_modifier: 1.0,
};

// =============================================================================
// TIME-TO-CONTACT (TTC) DECELERATION
// =============================================================================

/// [ACTIVE] Begin slowing when 2 seconds from target.
/// VALIDATED: At 15 m/s, this gives 30m approach corridor for smooth deceleration.
pub const TTC_SLOW_THRESHOLD: f32 = 2.0;

/// [ACTIVE] Target zero velocity at 0.3 seconds from contact.
/// VALIDATED: 0.3s is typical reaction time for final positioning adjustments.
pub const TTC_STOP_THRESHOLD: f32 = 0.3;

/// [ACTIVE] Pre-computed: TTC_SLOW_THRESHOLD - TTC_STOP_THRESHOLD
/// Used in seek system for interpolation. Avoids runtime subtraction.
pub const TTC_RANGE: f32 = TTC_SLOW_THRESHOLD - TTC_STOP_THRESHOLD;

/// [ACTIVE] Pre-computed: 1.0 / TTC_RANGE
/// Converts division to multiplication in hot path.
pub const TTC_RANGE_INV: f32 = 1.0 / TTC_RANGE;

/// [ACTIVE] Minimum slow zone when TTC undefined (e.g., starting from stationary).
pub const MIN_SLOW_ZONE_BODY_LENGTHS: f32 = 3.0;

// =============================================================================
// SEEK BEHAVIOR
// =============================================================================

/// [ACTIVE] Distance for pounce trigger (meters).
/// WARNING: 10cm is too small for large creatures!
pub const POUNCE_THRESHOLD: f32 = 0.1;

/// [ACTIVE] Maximum speed during pounce approach (m/s).
/// NOTE: 2 m/s is walking pace - controlled final approach (stalking).
pub const POUNCE_SPEED: f32 = 2.0;

/// [ACTIVE] Edge distance to apply braking (meters).
pub const ARRIVAL_THRESHOLD: f32 = 0.1;

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
