// Creature behavior constants - ALL tunable parameters in one place
//
// BIOLOGICAL REVIEW: Audited by zoologist-tom (Sprint 16)
// All values have been validated against empirical animal behavior research.
//
// USAGE KEY:
//   [ACTIVE]  - Currently used in simulation systems
//   [FUTURE]  - Defined for future DNA/allometric systems, not yet wired up
//   [LEGACY]  - Alias for backwards compatibility, migrate away from these
//
// Organization:
// 1. Physics & Movement (allometric scaling from reference creature)
// 2. Force Budget Multipliers
// 3. Perception
// 4. Personal Space & Avoidance
// 5. Time-to-Contact Deceleration
// 6. Seek Behavior
// 7. Wander Behavior
// 8. State & Transitions

use std::f32::consts::PI;

use crate::simulation::math::UnitInterval;

// =============================================================================
// PHYSICS & MOVEMENT (Reference Values for Allometric Scaling)
// =============================================================================
// These define a "reference creature" at 1m body length.
// Actual values scale allometrically with body size using biological formulas:
//   - Speed ∝ √(body_length)        [Froude number constraint]
//   - Acceleration ∝ 1/√(body_length)  [Square-cube law]
//   - Mass ∝ body_length³           [Volume scaling]
//   - Turn rate ∝ 1/body_length     [Angular momentum]
//
// BIOLOGICAL BASIS:
// 1m body length is medium dog/small deer/large cat - allows scaling both
// up (elephants, whales) and down (mice, insects) using standard allometry.

/// [FUTURE] Reference creature body length (meters).
/// VALIDATED: Excellent reference point for allometric scaling.
/// Used by: (reserved for DNA-based size variation)
pub const REFERENCE_BODY_LENGTH: f32 = 1.0;

/// [FUTURE] Mass at reference size (kg).
/// Real-world 1m creatures: Border Collie 15-22kg, German Shepherd 30-40kg,
/// Coyote 7-20kg, Kangaroo 25-35kg. 35kg implies stocky/muscular build.
/// NOTE: Could reduce to 25-30kg for a more average body plan.
/// Used by: (reserved for DNA-based mass calculation)
pub const REFERENCE_MASS: f32 = 35.0;

/// [FUTURE] Maximum sprint speed at reference size (m/s).
/// VALIDATED: 15 m/s = 54 km/h matches empirical data perfectly.
/// Real-world: Wolf 14-17 m/s, Deer 13-16 m/s, Lion 15-20 m/s, Large dog 12-18 m/s.
/// Allometric: Max speed ∝ mass^0.17 (Hirt et al., 2017) predicts 15-20 m/s for 35kg.
/// Used by: (reserved for DNA-based speed calculation)
pub const REFERENCE_MAX_SPEED: f32 = 15.0;

/// [FUTURE] Maximum acceleration at reference size (m/s²).
/// Real-world: Most quadrupeds 5-15 m/s², Dogs 6-9 m/s², Lions 7-10 m/s².
/// 6 m/s² is conservative - represents non-exceptional but capable animal.
/// NOTE: Could increase to 8 m/s² for more explosive burst acceleration.
/// Used by: (reserved for DNA-based acceleration calculation)
pub const REFERENCE_MAX_ACCEL: f32 = 6.0;

/// [FUTURE] Turn rate at reference size (degrees/second).
/// VALIDATED: Matches empirical quadruped data.
/// Real-world: Dogs/wolves at speed 90-180°/s, Cats 200-300°/s, Large ungulates 60-120°/s.
/// Turn rate scales as mass^(-0.33) - smaller creatures are more agile.
/// Used by: (reserved for DNA-based agility calculation)
pub const REFERENCE_TURN_RATE: f32 = 180.0;

// Convenience aliases - these ARE used by current systems
// TODO(DNA): Replace direct usage with allometric calculations

/// [ACTIVE] Default body length for spawned creatures.
/// Used by: BodySize::default(), creature spawning
pub const DEFAULT_BODY_LENGTH: f32 = REFERENCE_BODY_LENGTH;

/// [ACTIVE] Default mass for spawned creatures.
/// Used by: BodySize::default(), force calculations
pub const DEFAULT_MASS: f32 = REFERENCE_MASS;

/// [ACTIVE] Global speed cap for all creatures.
/// Used by: movement/systems.rs (velocity clamping)
pub const MAX_SPEED: f32 = REFERENCE_MAX_SPEED;

/// [ACTIVE] Default max acceleration.
/// Used by: BodySize::max_force() calculation
pub const MAX_ACCELERATION: f32 = REFERENCE_MAX_ACCEL;

/// [ACTIVE] Turn rate limit (degrees/second).
/// Used by: movement/systems.rs (turn rate limiting)
pub const MAX_TURN_RATE: f32 = REFERENCE_TURN_RATE;

/// [ACTIVE] Turn rate in radians (derived).
/// Used by: movement/systems.rs
pub const MAX_TURN_RATE_RAD: f32 = MAX_TURN_RATE * PI / 180.0;

// =============================================================================
// DRAG & DAMPING (Time-based, not frame-based)
// =============================================================================
// Drag coefficient for velocity damping. Applied as: v *= exp(-DRAG * dt)
// This is frame-rate independent, unlike per-frame multipliers.
//
// BIOLOGICAL BASIS:
// At dt=0.05s: v *= exp(-2.0 * 0.05) ≈ 0.905 (velocity decays ~13% per second)
// This is more aggressive than aerodynamic drag but approximates:
//   1. Ground friction during turns
//   2. Muscular energy dissipation
//   3. Postural instability at speed
//
// NOTE: 2.0 is high - creates "sticky" movement good for ground-based creatures.
// Consider 0.5-1.0 for more realistic coasting (swimming/flying creatures).

/// [ACTIVE] Velocity damping coefficient.
/// Used by: movement/systems.rs (v *= exp(-DRAG * dt))
pub const DRAG_COEFFICIENT: f32 = 2.0;

/// [ACTIVE] Threshold below which creature is considered stationary.
/// VALIDATED: 5 cm/s is imperceptible - shuffling in place or swaying.
/// Used by: movement/systems.rs (turn rate bypass for stopped creatures)
pub const STOPPED_THRESHOLD: f32 = 0.05;

/// [ACTIVE] Speed² threshold for locomotion noise filtering.
/// Filters sub-10cm/s fluctuations (postural adjustments, not intentional movement).
/// Used by: movement/systems.rs (noise application threshold)
pub const NOISE_SPEED_THRESHOLD_SQ: f32 = 0.01;

/// [FUTURE] Simulation time step (20 Hz tick rate).
/// At 20 Hz: creature at 15 m/s moves 75cm per tick.
/// Adequate for navigation/foraging; insufficient for detailed biomechanics.
/// NOTE: Actual dt comes from DeltaTime resource, not this constant.
/// Used by: (documentation reference only - actual dt is passed at runtime)
pub const DT: f32 = 0.05;

// =============================================================================
// FORCE BUDGET MULTIPLIERS (fraction of max_force)
// =============================================================================
// max_force = mass × MAX_ACCELERATION is the PHYSICAL LIMIT.
// These multipliers define what fraction each behavior can use.
//
// BIOLOGICAL BASIS (all VALIDATED):
// - Emergency: Adrenaline surge allows 100% muscular output briefly (fight-or-flight)
// - Pursuit: Aerobic threshold is ~75% of max effort; wolves cruise at 60-70%
// - Cruise: Migratory animals cruise at 30-50% of max; energy-efficient gaits
// - Wander: Foraging at 15-25% allows sensory processing while moving

/// [FUTURE] Emergency force (flee, brake, fight) - full muscular output.
/// VALIDATED: Fight-or-flight response triggers maximal output.
/// Used by: (reserved for flee behavior, not yet implemented)
pub const EMERGENCY_FORCE_MULT: UnitInterval = UnitInterval::new(1.0);

/// [FUTURE] Pursuit force for sustained chase.
/// VALIDATED: Matches aerobic threshold research (70-80% sustainable).
/// Used by: (reserved for predator chase behavior)
pub const PURSUIT_FORCE_MULT: UnitInterval = UnitInterval::new(0.7);

/// [FUTURE] Cruise force for directed travel (migration, commuting).
/// VALIDATED: Energy-efficient gaits operate at 35-45% of max.
/// Used by: (reserved for long-distance travel behavior)
pub const CRUISE_FORCE_MULT: UnitInterval = UnitInterval::new(0.4);

/// [ACTIVE] Wander force for exploration/foraging.
/// VALIDATED: Animals graze/forage at 15-25% to allow vigilance.
/// Used by: wander/systems.rs
pub const WANDER_FORCE_MULT: UnitInterval = UnitInterval::new(0.1);

/// [ACTIVE] Force multiplier for seek behavior (currently uses PURSUIT level).
/// Used by: seek/systems.rs
/// TODO: Should be CRUISE when searching, PURSUIT when actively chasing
pub const SEEK_FORCE_MULT: UnitInterval = PURSUIT_FORCE_MULT;

/// [LEGACY] Alias for emergency force - use EMERGENCY_FORCE_MULT instead.
/// Used by: (legacy code compatibility)
pub const BRAKE_FORCE_MULT: UnitInterval = EMERGENCY_FORCE_MULT;

// =============================================================================
// PERCEPTION
// =============================================================================
// BIOLOGICAL BASIS:
// Perception range scales with body size. Real-world vision varies enormously:
//   - Insects: 1-3 body lengths (very short)
//   - Fish: 5-20 body lengths
//   - Birds of prey: 100+ body lengths
//   - Terrestrial predators: 20-50 body lengths (open terrain)
//
// 10× is appropriate for dense vegetation / average sensory acuity.
// TODO(DNA): Make perception range DNA-driven with allometric scaling.

/// [ACTIVE] Maximum neighbors tracked simultaneously.
/// VALIDATED: Fish schools 4-7 (Partridge), Starling murmurations 6-7 (Ballerini),
/// Primates 3-5. Value of 8 is appropriate for moderate cognitive capacity.
/// Used by: perception/components.rs (Perception neighbor array size)
/// TODO(DNA): Make this a cognitive capacity gene.
pub const MAX_PERCEIVED_NEIGHBORS: usize = 7;

/// [ACTIVE] Perception range = body_length × this multiplier.
/// 10× gives 1m creature a 10m range - conservative, good for dense environments.
/// Used by: perception/components.rs (Perception::from_body_size)
/// TODO(DNA): Should become sensory acuity gene with allometric scaling.
pub const PERCEPTION_MULTIPLIER: f32 = 10.0;

//TODO:: Airbourne crits get massive FOV boost, for when we have flying crits

/// [ACTIVE] Neighbor selection strategy.
/// Used by: perception/systems.rs
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NeighborSortMode {
    /// Fast: Take first K neighbors found (pseudo-random order)
    PseudoRandom,
    /// Accurate: Collect all, sort by distance, keep K closest
    Topological,
}

pub const NEIGHBOR_SORT_MODE: NeighborSortMode = NeighborSortMode::Topological;

// Field of View - Creates predator/prey trade-off
//
// BIOLOGICAL BASIS (all VALIDATED):
// FOV_RANGE_EXPONENT creates elegant emergent design:
//   - Predators: Narrow FOV (60-120°), binocular depth perception, longer range
//   - Prey: Wide FOV (270-340°), early threat detection, shorter focused range
// This mirrors eagle (narrow/far) vs rabbit (wide/close) trade-off.
//
// With exponent 0.4:
//   - 45° FOV gets ~2.5× range bonus (specialist predator)
//   - 180° FOV gets 1× baseline (generalist)
//   - 340° FOV gets ~0.8× range (extreme prey adaptation)

/// [ACTIVE] Range compensation for narrow FOV.
/// VALIDATED: Elegant predator-prey emergent design.
/// Used by: perception/components.rs (Perception::calculate_range)
pub const FOV_RANGE_EXPONENT: f32 = 0.4;

/// [FUTURE] Minimum FOV - specialist predator (eagles ~45°, owls ~55°).
/// Used by: (reserved for DNA-based FOV limits)
pub const MIN_FOV_DEGREES: f32 = 45.0;

/// [FUTURE] Maximum FOV - extreme prey (horses ~350°, rabbits ~360°).
/// Used by: (reserved for DNA-based FOV limits)
pub const MAX_FOV_DEGREES: f32 = 340.0;

/// [ACTIVE] Default FOV - generalist omnivore (bears, dogs, wolves).
/// Used by: perception/components.rs (Perception::from_body_size default)
pub const DEFAULT_FOV_DEGREES: f32 = 180.0;

// =============================================================================
// PERSONAL SPACE & AVOIDANCE
// =============================================================================
// BIOLOGICAL BASIS:
// Real-world personal space varies by species and context:
//   - Fish schools: 0.5-2 body lengths (very tight)
//   - Bird flocks: 1-3 body lengths
//   - Mammals walking: 2-4 body lengths
//   - Mammals running: 3-6 body lengths
//
// 2× body_radius (= 1 body diameter) is appropriate for schooling fish,
// dense flocking birds, or herding mammals at slow speeds.
//
// NOTE: At high speeds, this is too tight. Consider velocity-dependent scaling:
//   effective_space = base_space × (1.0 + speed / max_speed)
// TODO(DNA): Make personal space a sociality gene.

/// [ACTIVE] Personal space = body_radius × this multiplier.
/// 2× radius = 1 body diameter - appropriate for social species at low speed.
/// Used by: perception/components.rs (AvoidanceBehavior::personal_space)
pub const PERSONAL_SPACE_MULTIPLIER: f32 = 2.0;

/// [ACTIVE] Seeking creatures tolerate closer proximity (tunnel vision during pursuit).
/// VALIDATED: Hunting animals override personal space concerns.
/// Used by: avoidance/systems.rs (reduced space for Seeking behavior)
pub const SEEKING_SPACE_REDUCTION: f32 = 0.5;

/// [ACTIVE] Emergency braking distance - apply max avoidance force within this range.
/// WARNING: Fixed 50cm doesn't scale with body size!
///   - 10cm creature: 50cm = 5 body lengths (appropriate)
///   - 1m creature: 50cm = 0.5 body lengths (too close!)
///   - 5m creature: 50cm = 0.1 body lengths (collision inevitable)
/// Used by: avoidance/systems.rs
/// TODO: Replace with EMERGENCY_BRAKE_BODY_LENGTHS = 1.0
pub const EMERGENCY_BRAKE_DISTANCE: f32 = 0.5;

// Energy-driven personal space modifiers
//
// BIOLOGICAL BASIS (VALIDATED):
// Starving animals tolerate extreme crowding at feeding sites.
// Vultures and scavengers pile onto carcasses regardless of personal space.
// Well-fed animals are more territorial and space-demanding.

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
/// Used by: perception/components.rs (AvoidanceBehavior::effective_personal_space)
pub static ENERGY_MODIFIER: EnergyModifierConstants = EnergyModifierConstants {
    min_modifier: 0.1,
    max_modifier: 1.0,
};

// =============================================================================
// TIME-TO-CONTACT (TTC) DECELERATION
// =============================================================================
// BIOLOGICAL BASIS (VALIDATED):
// Animals compute τ (tau) = distance / closing_velocity and begin decelerating
// when τ drops below species-specific thresholds.
// See: Lee (1976) "A Theory of Visual Control of Braking"
//
// This is how birds land, cats pounce, and humans catch balls - all use
// tau-based optical flow for timing deceleration.

/// [ACTIVE] Begin slowing when 2 seconds from target.
/// VALIDATED: At 15 m/s, this gives 30m approach corridor for smooth deceleration.
/// Used by: seek/systems.rs
pub const TTC_SLOW_THRESHOLD: f32 = 2.0;

/// [ACTIVE] Target zero velocity at 0.3 seconds from contact.
/// VALIDATED: 0.3s is typical reaction time for final positioning adjustments.
/// Used by: seek/systems.rs
pub const TTC_STOP_THRESHOLD: f32 = 0.3;

/// [ACTIVE] Minimum slow zone when TTC undefined (e.g., starting from stationary).
/// Ensures even slow-moving creatures have space to adjust.
/// Used by: seek/systems.rs
pub const MIN_SLOW_ZONE_BODY_LENGTHS: f32 = 3.0;

// =============================================================================
// SEEK BEHAVIOR
// =============================================================================
// Seeking: directed movement toward a target with TTC-based deceleration.
//
// WARNING: These fixed-meter values don't scale with body size!
// TODO: Convert to body-length fractions:
//   - POUNCE_BODY_LENGTHS = 1.0 (pounce from 1 body length away)
//   - ARRIVAL_BODY_LENGTHS = 0.5 (arrived when within 0.5 body lengths)

/// [ACTIVE] Distance for pounce trigger (meters).
/// WARNING: 10cm is too small for large creatures (1m wolf = 0.1 body lengths = overlapping!)
/// Used by: seek/systems.rs (final snap to target)
/// TODO: Replace with POUNCE_BODY_LENGTHS = 1.0
pub const POUNCE_THRESHOLD: f32 = 0.1;

/// [ACTIVE] Maximum speed during pounce approach (m/s).
/// NOTE: 2 m/s is walking pace. This represents controlled final approach (stalking),
/// NOT explosive pounce. Actual pounce speeds are 80-100% of max_speed.
/// Used by: seek/systems.rs
/// Consider renaming to STALK_APPROACH_SPEED for clarity.
pub const POUNCE_SPEED: f32 = 2.0;

/// [ACTIVE] Edge distance to apply braking (meters).
/// WARNING: Same scaling issue as POUNCE_THRESHOLD.
/// Used by: seek/systems.rs
/// TODO: Replace with ARRIVAL_BODY_LENGTHS = 0.5
pub const ARRIVAL_THRESHOLD: f32 = 0.1;

// =============================================================================
// WANDER BEHAVIOR
// =============================================================================
// Wander uses Reynolds steering: project a circle ahead, pick random point on it.
// This creates smooth, organic-looking exploration paths.
//
// WARNING: Fixed-meter values don't scale with body size!
// For a 10cm creature, WANDER_DISTANCE of 50m = 500 body lengths (absurdly far).
// TODO(DNA): Convert to body-length fractions and make DNA-driven.

/// [ACTIVE] Radius of wander target circle (meters).
/// Creates gradual direction changes. 10m means smooth, not jerky paths.
/// Used by: creatures/builder.rs (WanderState default)
/// TODO: Replace with WANDER_RADIUS_BODY_LENGTHS = 10.0
pub const WANDER_RADIUS: f32 = 10.0;

/// [ACTIVE] Distance ahead to project wander circle (meters).
/// WARNING: 50m for 1m creature = 50 body lengths (ok), but for 10cm = 500 (absurd).
/// Used by: creatures/builder.rs (WanderState default)
/// TODO: Replace with WANDER_DISTANCE_BODY_LENGTHS = 50.0
pub const WANDER_DISTANCE: f32 = 20.0;

/// [ACTIVE] Max angle change per tick (degrees).
/// VALIDATED: At 20 Hz, allows 90°/s maximum turn rate during wander.
/// Well below emergency turn rates (180°/s) - wandering should look relaxed.
/// Used by: creatures/builder.rs (WanderState default)
pub const ANGLE_CHANGE: f32 = 4.5;

// =============================================================================
// STATE & TRANSITIONS
// =============================================================================
// Energy thresholds based on biological hunger signals.
//
// BIOLOGICAL BASIS:
// - 30% reserves: Hormonal hunger signals trigger foraging priority
// - 10% reserves: Starvation mode - impaired decisions, reduced activity

/// [ACTIVE] Starting energy (arbitrary scale, ratios matter more than absolute value).
/// Used by: creatures/components/state.rs (CreatureState::default)
pub const DEFAULT_ENERGY: f32 = 100.0;

/// [ACTIVE] Hunger threshold - below this, prioritize foraging.
/// VALIDATED: 30% fat reserves trigger hormonal hunger signals in most animals.
/// Used by: transitions/systems.rs (behavior decisions)
pub const LOW_ENERGY_THRESHOLD: f32 = 30.0;

/// [ACTIVE] Exhaustion threshold - critical energy state.
/// VALIDATED: 10% reserves = starvation mode, impaired cognition.
/// Used by: transitions/systems.rs (panic threshold check)
pub const EXHAUSTED_THRESHOLD: f32 = 10.0;

/// [ACTIVE] Age increase per tick.
/// At 20 Hz: 0.001 × 20 = 0.02/sec, 1.2/min, 72/hour.
/// If age 100 = max lifespan, creatures live ~1.4 hours real-time.
/// Used by: transitions/systems.rs
pub const AGE_INCREMENT_PER_TICK: f32 = 0.001;

/// [ACTIVE] Energy drain per tick while wandering.
/// At 20 Hz: 0.2 energy/sec, ~6 minutes from 100 to 30 (hunger threshold).
/// Low drain is correct - wandering is energy-efficient.
/// Used by: transitions/systems.rs
/// TODO(DNA): Make this a metabolic rate gene.
pub const ENERGY_COST_WANDERING: f32 = 0.01;

/// [ACTIVE] Tick interval for timing calculations.
/// Used by: transitions/systems.rs (brain cooldown)
pub const TICK_INTERVAL_SECONDS: f64 = 0.05;

// Random target generation for cycling brain
//
// WARNING: Fixed-meter values don't scale with body size!
// A mouse wandering 50-200m = 500-2000 body lengths (absurdly far).
// TODO: Replace with body-length fractions:
//   RANDOM_TARGET_MIN_BODY_LENGTHS = 50.0
//   RANDOM_TARGET_MAX_BODY_LENGTHS = 200.0

/// [ACTIVE] Minimum distance for random wander targets.
/// Used by: transitions/systems.rs (cycling brain target generation)
pub const RANDOM_TARGET_MIN_DISTANCE: f32 = 50.0;

/// [ACTIVE] Maximum distance for random wander targets.
/// Used by: transitions/systems.rs (cycling brain target generation)
pub const RANDOM_TARGET_MAX_DISTANCE: f32 = 200.0;
