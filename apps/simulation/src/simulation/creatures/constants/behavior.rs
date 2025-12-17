//! Behavior constants
//!
//! Force budget multipliers, avoidance, seek, and wander parameters.

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
