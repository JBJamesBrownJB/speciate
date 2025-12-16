Some crits suddenly have super speed and appear to breach physical constraints.

I suspect through testing that it is when any of the behaviours with the full 1.0 unit of force multiplier are invoked. 

So below, I suspect that when the emergency force is invoked, its takes a path in code which can lead to uncapped force.

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