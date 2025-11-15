//! Movement and steering force constants
//!
//! See `/workspace/docs/biology/biology-notes.md` for biological rationale.

// ============================================================================
// MOVEMENT PHYSICS CONSTANTS
// ============================================================================

/// Default body length for creatures (meters)
///
/// **Value:** 1.0 m (wolf-sized creature)
///
/// **DNA Range (Future DNA system):** 0.5m - 10m
/// - 0.5-2m: Small creatures (rodents, foxes, wolves)
/// - 2-5m: Large creatures (bears, lions, horses)
/// - 5-10m: Megafauna (elephants, extinct predators)
///
/// All other parameters scale from body length using allometric formulas.
///
/// TODO: Migrate to DNA gene: `body_length`
pub const DEFAULT_BODY_LENGTH: f32 = 1.0;

/// Default mass for 1m creature (kg)
///
/// **Value:** 65 kg (wolf mass)
/// **Formula:** `mass = 65.0 × body_length³`
///
/// **Examples:**
/// - 0.5m: 8.1 kg (fox)
/// - 1.0m: 65 kg (wolf)
/// - 3.0m: 1,755 kg (bear)
/// - 10m: 65,000 kg (elephant)
///
/// **Rationale:** Cubic scaling (mass ∝ volume)
///
/// TODO: Migrate to DNA-derived calculation
pub const DEFAULT_MASS: f32 = 65.0;

/// Maximum speed for creatures (meters/second)
///
/// **Value:** 5.0 m/s (18 km/h - wolf trot)
/// **Formula (Kleiber's Law):** `top_speed = 5.0 × body_length^0.25`
///
/// **Examples:**
/// - 0.5m: 4.2 m/s (15 km/h) - small predator sprint
/// - 1.0m: 5.0 m/s (18 km/h) - wolf trot
/// - 3.0m: 6.6 m/s (24 km/h) - bear charge
/// - 10m: 8.9 m/s (32 km/h) - elephant sprint
///
/// **Previous values were unrealistic:**
/// - Old: 50 m/s = 112 mph (cheetah at full sprint!)
/// - Old: 20 m/s = 45 mph (still too fast for most creatures)
///
/// **Physics validation:** At dt=0.05s, max movement = 5.0 × 0.05 = 0.25m per frame
/// This is safely below collision threshold (0.5m), preventing physics tunneling.
///
/// TODO: Migrate to DNA-derived calculation
pub const MAX_SPEED: f32 = 50.0;

/// Maximum acceleration (meters/second²)
///
/// **Value:** 8.0 m/s² (moderate acceleration)
/// **Formula:** `acceleration = 8.0 / body_length^0.67`
///
/// **Examples:**
/// - 0.5m: 13.5 m/s² (agile, quick start)
/// - 1.0m: 8.0 m/s² (baseline)
/// - 3.0m: 3.8 m/s² (lumbering start)
/// - 10m: 1.7 m/s² (slow ramp-up)
///
/// **Rationale:** Smaller creatures have higher power-to-weight ratio
///
/// TODO: Migrate to DNA-derived calculation
pub const MAX_ACCELERATION: f32 = 5.0;

/// Maximum turn rate (degrees per second)
///
/// **Value:** 180°/s (π radians/s - can turn around in 2 seconds)
/// **Formula:** `turn_rate = 180° / body_length^1.33`
///
/// **Examples:**
/// - 0.5m: 428°/s (extremely agile)
/// - 1.0m: 180°/s (baseline turn)
/// - 3.0m: 37°/s (wide turns)
/// - 10m: 8.4°/s (very sluggish)
///
/// **Rationale:** Large creatures have wide turning circles (inertia + limb mechanics)
///
/// TODO: Migrate to DNA-derived calculation
pub const MAX_TURN_RATE: f32 = 180.0;

/// Velocity damping factor (per frame at 20 Hz)
///
/// **Value:** 0.98 (2% velocity loss per frame)
///
/// **Effect:** Mimics air resistance + ground friction
/// - After 1 second (20 frames): velocity drops to 67% of initial
/// - After 2 seconds: velocity drops to 45% of initial
/// - Creatures naturally "coast to a stop" without continuous thrust
///
/// **Rationale:**
/// - Too high (0.99): Creatures slide like on ice
/// - Too low (0.92): Too much resistance, creatures barely move
/// - 0.98: Balanced air/ground resistance
///
/// **Note:** Original zoologist recommendation was 0.92, but this proved
/// too aggressive when combined with seek force of 10N. Adjusted to 0.98
/// for better gameplay while maintaining realistic deceleration.
///
/// **Biological impact:** Creates continuous energy cost for movement.
/// Fast creatures burn energy rapidly just maintaining speed.
///
/// Applied in `integrate_motion_system` after force integration.
pub const VELOCITY_DAMPING: f32 = 0.95;

/// Simulation time step (seconds)
///
/// **Value:** 0.05s (20 Hz update rate)
///
/// **Physics validation:**
/// - Max movement per frame: MAX_SPEED × DT = 5.0 × 0.05 = 0.25m
/// - Minimum collision threshold: 0.5m (half smallest creature)
/// - Safety margin: 2× (prevents tunneling)
///
/// Standard for game physics, balances performance with accuracy.
pub const DT: f32 = 0.05;

/// Slow zone multiplier (× personal_space)
///
/// **Value:** 3.0× personal space
///
/// **Example:** 2.5m personal_space → 7.5m slow zone
///
/// **Behavior:** Creature begins decelerating when entering this zone.
/// Linear deceleration from full speed → 0 over the distance.
///
/// **Rationale:** Animals "coast to a stop" like real creatures (no instant braking).
///
/// **Physics:** At MAX_SPEED 30 m/s, creature travels 1.5m per frame (dt=0.05s).
/// Slow zone must be >> 1.5m to allow gradual deceleration and prevent overshoot.
///
/// TODO: Consider DNA-driven for different approach strategies
pub const SLOW_ZONE_MULTIPLIER: f32 = 30.0;

// Locomotion noise base magnitude (Newtons)
//
// **Value:** 0.5 N (~5% of seek force)
//
// **Effect:** Adds lateral (side-to-side) wobble to movement, simulating:
// - Motor control variability (neuromuscular noise)
// - Terrain micro-irregularities (pebbles, grass, slopes)
// - Decision-making fluctuations (temporal smoothing lag)
//
// **Scaling:**
// - Faster creatures: More wobble (inertial instability)
// - Smaller creatures: More wobble per unit speed (surface area effects)
//
// **Formula:** `noise = BASE × (speed/MAX_SPEED)² × (1/sqrt(body_length))`
//
// **Biological rationale:**
// - Animals don't move in perfect straight lines
// - Fixes collinear obstacle edge case (adds lateral component)
// - Magnitude: 2-10% of typical locomotor forces (biologically realistic)
//
// NOTE: Locomotion noise magnitude is now configured via `MovementConfig` in `config.rs`
// TODO: Migrate to DNA gene: `motor_precision` (Future DNA system)

// ============================================================================
// STEERING FORCE CONSTANTS
// ============================================================================

/// Steering force constants for creature behaviors
///
/// Forces are in Newtons (N) and follow Newtonian physics (F = ma).
/// Larger creatures have more mass, so same force = less acceleration.
///
/// # Force Hierarchy (Biological Priority)
/// 1. **Panic:** 50N - Emergency collision prevention (survival)
/// 2. **Avoidance:** 15N - Collision prevention (high priority)
/// 3. **Seeking:** 10N - Goal pursuit (moderate priority)
/// 4. **Wander:** 5N - Exploration (low priority)
/// 5. **Flee:** 20N - Threat response (high priority, future)
///
/// This hierarchy ensures creatures avoid collisions while still pursuing goals.
#[derive(Debug, Clone, Copy)]
pub struct SteeringConstants {
    /// Maximum force for seeking behavior (goal-directed movement)
    ///
    /// **Value:** 10.0 N
    /// **Rationale:** Moderate priority - creatures should move toward goals
    /// but not at the expense of collision avoidance.
    ///
    /// TODO: Migrate to DNA (Future DNA system)
    pub seek_force: f32,

    /// Base force for obstacle avoidance (separation)
    ///
    /// **Value:** 15.0 N
    /// **Rationale:** Higher than seeking to prevent collisions.
    /// Uses inverse square scaling: actual force = base × (personal_space / distance)²
    ///
    /// TODO: Migrate to DNA (Future DNA system)
    pub avoidance_force: f32,

    /// Maximum panic force when collision is imminent
    ///
    /// **Value:** 50.0 N
    /// **Rationale:** Emergency evasion. Capped to prevent physics instability.
    /// Activated when distance < panic_threshold (50% of personal_space).
    ///
    /// TODO: Migrate to DNA (Future DNA system)
    pub panic_force: f32,

    /// Force for random wandering behavior
    ///
    /// **Value:** 5.0 N
    /// **Rationale:** Low priority exploration. Should not interfere with
    /// goal-directed or avoidance behaviors.
    ///
    /// TODO: Migrate to DNA (Future DNA system)
    pub wander_force: f32,

    /// Force for fleeing from threats (future implementation)
    ///
    /// **Value:** 20.0 N
    /// **Rationale:** Higher than seeking, but allows creatures to still
    /// navigate around obstacles while fleeing.
    ///
    /// TODO: Implement flee system (Sprint 7+)
    /// TODO: Migrate to DNA (Future DNA system)
    pub flee_force: f32,
}

impl Default for SteeringConstants {
    fn default() -> Self {
        Self {
            seek_force: 10.0,
            avoidance_force: 35.0,
            panic_force: 90.0,
            wander_force: 5.0,
            flee_force: 20.0,
        }
    }
}

/// Perception constants for spatial awareness
///
/// These define how creatures detect and react to their environment.
///
/// # Biological Scaling
/// Both parameters scale with body size:
/// - `perception_range = body_length × perception_multiplier`
/// - `personal_space = body_length + spacing_multiplier`
///
/// See docs/biology/biology-notes.md for full biological rationale and DNA integration plans.
#[derive(Debug, Clone, Copy)]
pub struct PerceptionConstants {
    /// Perception range multiplier (× body length)
    ///
    /// **Value:** 10.0× body length
    /// **Example:** 1m creature detects others within 10m
    ///
    /// **Range for DNA (Future DNA system):**
    /// - Min: 3.0× (ambush predator, short-range)
    /// - Max: 20.0× (vigilant prey, long-range)
    /// - Default: 10.0× (active forager)
    ///
    /// **Energy cost:** sqrt(perception_range / body_length) - high perception drains energy
    ///
    /// TODO: Migrate to DNA gene: `perception_multiplier`
    pub perception_multiplier: f32,

    /// Personal space buffer distance (meters, additive)
    ///
    /// **Value:** 1.5m buffer added to body length
    /// **Example:** 1m creature maintains 2.5m spacing (1.0 + 1.5)
    ///
    /// **Range for DNA (Future DNA system):**
    /// - Min: 0.5m (colonial/tolerant species)
    /// - Max: 3.0m (territorial species)
    /// - Default: 1.5m (solitary animal)
    ///
    /// **Behavioral impact:**
    /// - Low buffer: Dense groups, schooling, herding
    /// - High buffer: Territorial, solitary, aggressive
    ///
    /// TODO: Migrate to DNA gene: `spacing_buffer`
    pub personal_space: f32,

    /// Panic threshold (fraction of personal space)
    ///
    /// **Value:** 0.5 (50% of personal_space)
    /// **Example:** 2.5m personal_space → 1.25m panic threshold
    ///
    /// **Rationale:** When another creature is within 50% of comfort zone,
    /// collision is imminent - trigger maximum evasive force.
    ///
    /// TODO: Consider making this DNA-driven for different risk tolerances
    pub panic_threshold_ratio: f32,
}

impl Default for PerceptionConstants {
    fn default() -> Self {
        Self {
            perception_multiplier: 10.0,
            personal_space: 1.5,
            panic_threshold_ratio: 0.5,
        }
    }
}

// ============================================================================
// TERRITORY & WANDERING BEHAVIOR CONSTANTS
// ============================================================================

/// Territory and wandering behavior constants
///
/// These define how creatures patrol their home territories using the "elastic tether" model:
/// exploration freedom near home, increasing urgency to return when far.
///
/// # Biological Rationale
///
/// From movement ecology research (docs/biology/biology-notes.md, zoologist consultation 2025-11-08):
/// - Animals don't wander randomly - they patrol territories with soft boundaries
/// - "Elastic tether" model: exploration freedom near home, urgency when far
/// - Sigmoid transition creates smooth behavioral shift (not hard threshold)
/// - Composite movement strategies are the norm in territorial species
///
/// # Force Blending Strategy
///
/// - **Near home (0-10m):** 90% wandering, 10% homeward (free exploration)
/// - **Mid-range (10-20m):** 50% wandering, 50% homeward (balanced patrol)
/// - **Far from home (20-30m):** 10% wandering, 90% homeward (emergency return)
///
/// See docs/biology/biology-notes.md for full ecological rationale and DNA integration plans.
#[derive(Debug, Clone, Copy)]
pub struct TerritoryConstants {
    /// Territory core radius (meters)
    ///
    /// **Value:** 10.0 m
    /// **Rationale:** Low home bias within this zone (90% wandering, 10% homeward)
    ///
    /// **Range for DNA (Future DNA system):**
    /// - Min: 5.0 m (small territory, high-density habitat)
    /// - Max: 50.0 m (large territory, abundant resources)
    /// - Default: 10.0 m (typical forager)
    ///
    /// TODO: Migrate to DNA gene: `comfort_radius_multiplier` (metabolic needs scale territory)
    pub comfort_radius: f32,

    /// Blend center distance (meters)
    ///
    /// **Value:** 20.0 m
    /// **Rationale:** 50% blend point, patrol boundary
    ///
    /// **Range for DNA (Future DNA system):**
    /// - Min: 10.0 m (tight patrol)
    /// - Max: 100.0 m (wide-ranging explorer)
    /// - Default: 20.0 m (balanced patrol)
    ///
    /// TODO: Migrate to DNA gene: `territory_size`
    pub blend_center: f32,

    /// Maximum wander distance (meters)
    ///
    /// **Value:** 30.0 m
    /// **Rationale:** Hard limit for excursions (emergency return beyond this)
    ///
    /// **Range for DNA (Future DNA system):**
    /// - Min: 15.0 m (stay-at-home species)
    /// - Max: 150.0 m (nomadic species)
    /// - Default: 30.0 m (typical range)
    ///
    /// TODO: Migrate to DNA gene: `max_exploration_range`
    pub max_wander_distance: f32,

    /// Homeward seeking force magnitude (Newtons)
    ///
    /// **Value:** 50.0 N
    /// **Rationale:** Strong homeward pull when far from territory (higher priority than general seeking)
    ///
    /// **Note:** This is distinct from `STEERING.seek_force` (10.0 N).
    /// Returning home is higher priority than chasing food/goals.
    ///
    /// TODO: Migrate to DNA gene: `territory_attachment` (homing instinct strength)
    pub homeward_force: f32,

    /// Sigmoid steepness for blend curve
    ///
    /// **Value:** 1.5
    /// **Rationale:** Biologically realistic "elastic tether" behavior
    ///
    /// **Effect:**
    /// - Low k (0.1-0.5): Gradual transition over wide range
    /// - High k (1.0-3.0): Sharp transition near center
    /// - k=1.5: Smooth but definite transition (natural animal behavior)
    ///
    /// TODO: Migrate to DNA gene: `stress_response` (personality: bold vs cautious)
    pub sigmoid_steepness: f32,
}

impl Default for TerritoryConstants {
    fn default() -> Self {
        Self {
            comfort_radius: 10.0,
            blend_center: 20.0,
            max_wander_distance: 30.0,
            homeward_force: 50.0,
            sigmoid_steepness: 1.5,
        }
    }
}

// ============================================================================
// SEEKING & ARRIVAL BEHAVIOR CONSTANTS
// ============================================================================

/// Seeking and arrival behavior constants
///
/// These control how creatures approach targets with smooth deceleration and precise arrival.
///
/// # Arrival Strategy
///
/// **Exponential deceleration** provides "land on a dime" behavior:
/// - Maintains speed far from target (max reaction time)
/// - Sharp deceleration near target (prevents overshoot)
/// - Snap-to-target "pounce" when close and slow (prevents creeping)
///
/// # Arrival Zones
///
/// With `SLOW_ZONE_MULTIPLIER = 30.0`:
/// - **Slow zone:** 15.0m (begin exponential deceleration)
/// - **Pounce zone:** 0.5m at speed < 5.5 m/s (snap to target)
/// - **Emergency brake:** < 0.5m (hard counter-force)
///
/// See seek.rs for full algorithmic details and arrival mathematics.
#[derive(Debug, Clone, Copy)]
pub struct SeekingConstants {
    /// Maximum seeking force (Newtons)
    ///
    /// **Value:** 50.0 N
    /// **Rationale:** Strong goal-directed pursuit
    ///
    /// **Note:** This is distinct from `STEERING.seek_force` (10.0 N).
    /// `STEERING.seek_force` is for general force hierarchy documentation.
    /// This value is the actual implementation force.
    ///
    /// **Range for DNA (Future DNA system):**
    /// - Min: 10.0 N (weak seeker, easily distracted)
    /// - Max: 100.0 N (relentless pursuer)
    /// - Default: 50.0 N (determined forager)
    ///
    /// TODO: Migrate to DNA gene: `dna.express_gene("strength")`
    pub max_force: f32,

    /// Emergency brake force (Newtons)
    ///
    /// **Value:** 70.0 N (1.4× max_force)
    /// **Rationale:** Hard counter-force when within arrival radius (prevent overshoot)
    ///
    /// **Range for DNA (Future DNA system):**
    /// - Formula: `MAX_SEEK_FORCE * 1.4` (40% stronger than pursuit)
    ///
    /// TODO: Migrate to DNA gene: derived from `strength`
    pub brake_force: f32,

    /// Pounce distance threshold (meters)
    ///
    /// **Value:** 0.5 m
    /// **Rationale:** Snap-to-target when this close (prevents creeping)
    ///
    /// **Range for DNA (Future DNA system):**
    /// - Formula: `body_size * 0.5` (scales with creature size)
    ///
    /// TODO: Migrate to DNA gene: body-size-relative
    pub pounce_distance: f32,

    /// Pounce speed threshold (m/s)
    ///
    /// **Value:** 5.5 m/s
    /// **Rationale:** Only pounce if moving slowly (prevents high-speed snap)
    ///
    /// **Range for DNA (Future DNA system):**
    /// - Min: 2.0 m/s (cautious, slow approach)
    /// - Max: 10.0 m/s (aggressive, fast snap)
    /// - Default: 5.5 m/s (balanced)
    ///
    /// TODO: Migrate to DNA gene: `dna.express_gene("precision")`
    pub pounce_speed: f32,

    /// Target arrival tolerance (meters)
    ///
    /// **Value:** 0.5 m
    /// **Rationale:** Stop when edge reaches target (accounts for body radius)
    ///
    /// **Range for DNA (Future DNA system):**
    /// - Min: 0.1 m (precise, right on target)
    /// - Max: 2.0 m (sloppy, "close enough")
    /// - Default: 0.5 m (typical animal precision)
    ///
    /// TODO: Migrate to DNA gene: `arrival_precision`
    pub arrival_tolerance: f32,

    /// Exponential decay factor for slow zone deceleration
    ///
    /// **Value:** 1.5
    /// **Rationale:** Controls deceleration curve shape
    ///
    /// **Effect:**
    /// - Low k (0.5-1.0): Gentle deceleration, early slowdown
    /// - High k (2.0-3.0): Sharp deceleration, late braking
    /// - k=1.5: Balanced (maintains speed, then brakes hard)
    ///
    /// **Math:** `desired_speed = max_speed × e^(k×ratio) / e^k`
    ///
    /// TODO: Consider DNA-driven for different approach strategies
    pub slow_zone_decay: f32,
}

impl Default for SeekingConstants {
    fn default() -> Self {
        Self {
            max_force: 50.0,
            brake_force: 70.0,
            pounce_distance: 0.5,
            pounce_speed: 5.5,
            arrival_tolerance: 0.5,
            slow_zone_decay: 1.5,
        }
    }
}

/// Global instance of steering constants
///
/// Easy to access, easy to modify for tuning.
/// Will be replaced by DNA-driven parameters in Future DNA system.
pub static STEERING: SteeringConstants = SteeringConstants {
    seek_force: 10.0,
    avoidance_force: 35.0,
    panic_force: 90.0,
    wander_force: 5.0,
    flee_force: 20.0,
};

/// Global instance of perception constants
///
/// Easy to access, easy to modify for tuning.
/// Will be replaced by DNA-driven parameters in Future DNA system.
pub static PERCEPTION: PerceptionConstants = PerceptionConstants {
    perception_multiplier: 10.0,
    personal_space: 1.5,
    panic_threshold_ratio: 0.5,
};

/// Global instance of territory constants
///
/// Easy to access, easy to modify for tuning.
/// Will be replaced by DNA-driven parameters in Future DNA system.
pub static TERRITORY: TerritoryConstants = TerritoryConstants {
    comfort_radius: 10.0,
    blend_center: 20.0,
    max_wander_distance: 30.0,
    homeward_force: 50.0,
    sigmoid_steepness: 1.5,
};

/// Global instance of seeking constants
///
/// Easy to access, easy to modify for tuning.
/// Will be replaced by DNA-driven parameters in Future DNA system.
pub static SEEKING: SeekingConstants = SeekingConstants {
    max_force: 50.0,
    brake_force: 70.0,
    pounce_distance: 0.5,
    pounce_speed: 5.5,
    arrival_tolerance: 0.5,
    slow_zone_decay: 1.5,
};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_force_hierarchy() {
        let steering = SteeringConstants::default();

        // Panic > Avoidance > Seeking > Wander (biological priority)
        assert!(steering.panic_force > steering.avoidance_force);
        assert!(steering.avoidance_force > steering.seek_force);
        assert!(steering.seek_force > steering.wander_force);
    }

    #[test]
    fn test_perception_scaling() {
        let perception = PerceptionConstants::default();

        // For 1m creature:
        let body_size = 1.0;
        let perception_range = body_size * perception.perception_multiplier;
        let personal_space = body_size + perception.personal_space;
        let panic_threshold = personal_space * perception.panic_threshold_ratio;

        assert_eq!(perception_range, 10.0); // 10m detection range
        assert_eq!(personal_space, 2.5); // 2.5m personal space (1.0 + 1.5)
        assert_eq!(panic_threshold, 1.25); // 1.25m panic distance

        // Panic threshold should be less than personal space
        assert!(panic_threshold < personal_space);
    }

    #[test]
    fn test_constants_are_positive() {
        let steering = SteeringConstants::default();
        let perception = PerceptionConstants::default();

        assert!(steering.seek_force > 0.0);
        assert!(steering.avoidance_force > 0.0);
        assert!(steering.panic_force > 0.0);
        assert!(steering.wander_force > 0.0);
        assert!(steering.flee_force > 0.0);

        assert!(perception.perception_multiplier > 0.0);
        assert!(perception.personal_space > 0.0);
        assert!(perception.panic_threshold_ratio > 0.0);
        assert!(perception.panic_threshold_ratio < 1.0);
    }

    #[test]
    fn test_territory_constants_valid() {
        let territory = TerritoryConstants::default();

        // All values must be positive
        assert!(territory.comfort_radius > 0.0);
        assert!(territory.blend_center > 0.0);
        assert!(territory.max_wander_distance > 0.0);
        assert!(territory.homeward_force > 0.0);
        assert!(territory.sigmoid_steepness > 0.0);

        // Zone ordering: comfort < blend < max_wander
        assert!(territory.comfort_radius < territory.blend_center,
            "Comfort radius ({}) should be less than blend center ({})",
            territory.comfort_radius, territory.blend_center);
        assert!(territory.blend_center < territory.max_wander_distance,
            "Blend center ({}) should be less than max wander distance ({})",
            territory.blend_center, territory.max_wander_distance);

        // Sigmoid steepness should be reasonable (0.1 - 5.0)
        assert!(territory.sigmoid_steepness >= 0.1 && territory.sigmoid_steepness <= 5.0,
            "Sigmoid steepness ({}) should be between 0.1 and 5.0",
            territory.sigmoid_steepness);
    }

    #[test]
    fn test_territory_global_instance() {
        // Verify global TERRITORY instance is accessible and valid
        assert!(TERRITORY.comfort_radius > 0.0);
        assert!(TERRITORY.blend_center > TERRITORY.comfort_radius);
        assert!(TERRITORY.max_wander_distance > TERRITORY.blend_center);
    }

    #[test]
    fn test_seeking_constants_valid() {
        let seeking = SeekingConstants::default();

        // All values must be positive
        assert!(seeking.max_force > 0.0);
        assert!(seeking.brake_force > 0.0);
        assert!(seeking.pounce_distance > 0.0);
        assert!(seeking.pounce_speed > 0.0);
        assert!(seeking.arrival_tolerance > 0.0);
        assert!(seeking.slow_zone_decay > 0.0);

        // Brake force should be stronger than max force
        assert!(seeking.brake_force > seeking.max_force,
            "Brake force ({}) should exceed max force ({}) for emergency stopping",
            seeking.brake_force, seeking.max_force);

        // Pounce distance should be small (< 5m for precise arrival)
        assert!(seeking.pounce_distance < 5.0,
            "Pounce distance ({}) should be small for precise arrival",
            seeking.pounce_distance);

        // Arrival tolerance should be reasonable (< 5m)
        assert!(seeking.arrival_tolerance < 5.0,
            "Arrival tolerance ({}) should be small for target precision",
            seeking.arrival_tolerance);

        // Decay factor should be reasonable (0.5 - 5.0)
        assert!(seeking.slow_zone_decay >= 0.5 && seeking.slow_zone_decay <= 5.0,
            "Slow zone decay ({}) should be between 0.5 and 5.0",
            seeking.slow_zone_decay);
    }

    #[test]
    fn test_seeking_global_instance() {
        // Verify global SEEKING instance is accessible and valid
        assert!(SEEKING.max_force > 0.0);
        assert!(SEEKING.brake_force > SEEKING.max_force);
        assert!(SEEKING.pounce_distance > 0.0);
    }

    #[test]
    fn test_force_magnitudes_relative() {
        // Territory homeward force should be strong (comparable to panic, stronger than seek)
        assert!(TERRITORY.homeward_force > STEERING.seek_force,
            "Homeward force ({}) should be stronger than general seeking ({})",
            TERRITORY.homeward_force, STEERING.seek_force);

        // Seeking max_force should be strong (comparable to territory homeward)
        assert!(SEEKING.max_force >= STEERING.seek_force,
            "Seeking max_force ({}) should be at least as strong as general seek ({})",
            SEEKING.max_force, STEERING.seek_force);
    }
}
