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
pub const MAX_SPEED: f32 = 30.0;

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

/// Locomotion noise base magnitude (Newtons)
///
/// **Value:** 0.5 N (~5% of seek force)
///
/// **Effect:** Adds lateral (side-to-side) wobble to movement, simulating:
/// - Motor control variability (neuromuscular noise)
/// - Terrain micro-irregularities (pebbles, grass, slopes)
/// - Decision-making fluctuations (temporal smoothing lag)
///
/// **Scaling:**
/// - Faster creatures: More wobble (inertial instability)
/// - Smaller creatures: More wobble per unit speed (surface area effects)
///
/// **Formula:** `noise = BASE × (speed/MAX_SPEED)² × (1/sqrt(body_length))`
///
/// **Biological rationale:**
/// - Animals don't move in perfect straight lines
/// - Fixes collinear obstacle edge case (adds lateral component)
/// - Magnitude: 2-10% of typical locomotor forces (biologically realistic)
///
/// TODO: Migrate to DNA gene: `motor_precision` (Future DNA system)
pub const LOCOMOTION_NOISE_BASE: f32 = 90.5;

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
/// - `personal_space = body_length × spacing_multiplier`
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

    /// Personal space multiplier (× body length)
    ///
    /// **Value:** 2.5× body length
    /// **Example:** 1m creature maintains 2.5m spacing from others
    ///
    /// **Range for DNA (Future DNA system):**
    /// - Min: 1.5× (colonial/tolerant species)
    /// - Max: 4.0× (territorial species)
    /// - Default: 2.5× (solitary animal)
    ///
    /// **Behavioral impact:**
    /// - Low spacing: Dense groups, schooling, herding
    /// - High spacing: Territorial, solitary, aggressive
    ///
    /// TODO: Migrate to DNA gene: `spacing_multiplier`
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
        let personal_space = body_size * perception.personal_space;
        let panic_threshold = personal_space * perception.panic_threshold_ratio;

        assert_eq!(perception_range, 10.0); // 10m detection range
        assert_eq!(personal_space, 1.5); // 1.5m comfort zone
        assert_eq!(panic_threshold, 0.75); // 0.75m panic distance

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
}
