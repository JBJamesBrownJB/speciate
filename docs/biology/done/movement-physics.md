# Movement Constants - Scientific Rationale

This document preserves the biological and physical rationale for movement constants extracted from code comments during the "Death to Comments" cleanup (2025-11-15).

**Status:** These values are currently hardcoded. All will migrate to DNA-driven parameters in future sprints.

---

## Allometric Scaling Laws

### Kleiber's Law (Speed Scaling)

**Formula:** `top_speed = base_speed × body_length^0.25`

**Biological basis:** Metabolic rate scales with body mass^0.75, influencing sustainable speed.

**Examples:**
- 0.5m creature: 4.2 m/s (15 km/h) - small predator sprint
- 1.0m creature: 5.0 m/s (18 km/h) - wolf trot
- 3.0m creature: 6.6 m/s (24 km/h) - bear charge
- 10m creature: 8.9 m/s (32 km/h) - elephant sprint

**Historical note:** Original MAX_SPEED was 50 m/s (112 mph, cheetah full sprint) - unrealistic for typical creatures.

### Mass Scaling (Cubic Law)

**Formula:** `mass = base_mass × body_length³`

**Biological basis:** Mass proportional to volume (density assumed constant).

**Examples:**
- 0.5m: 8.1 kg (fox)
- 1.0m: 65 kg (wolf)
- 3.0m: 1,755 kg (bear)
- 10m: 65,000 kg (elephant)

### Acceleration Scaling (Power-to-Weight Ratio)

**Formula:** `acceleration = base_accel / body_length^0.67`

**Biological basis:** Smaller creatures have higher power-to-weight ratio, enabling quicker starts.

**Examples:**
- 0.5m: 13.5 m/s² (agile, quick start)
- 1.0m: 8.0 m/s² (baseline)
- 3.0m: 3.8 m/s² (lumbering start)
- 10m: 1.7 m/s² (slow ramp-up)

### Turn Rate Scaling (Angular Inertia)

**Formula:** `turn_rate = 180° / body_length^1.33`

**Biological basis:** Large creatures have wide turning circles due to inertia and limb mechanics.

**Examples:**
- 0.5m: 428°/s (extremely agile)
- 1.0m: 180°/s (baseline turn)
- 3.0m: 37°/s (wide turns)
- 10m: 8.4°/s (very sluggish)

---

## Reynolds Steering Behaviors

### Algorithm Source

**Reference:** "The Nature of Code" by Dan Shiffman (Reynolds steering behaviors)

### Wandering Algorithm

**Method:** Circle projection method
- Project circle ahead of creature
- Randomly select point on circle perimeter
- Steer toward selected point
- Creates smooth, organic wandering paths

**Implementation:** See `apps/simulation/src/simulation/creatures/behaviors/wander.rs`

### Force Accumulation Pattern

**Principle:** Systems ADD forces to acceleration, physics integrates.

**Benefits:**
- Natural force blending (seek + avoid = emergent path)
- Extensible (add new behaviors without modifying existing ones)
- Biologically realistic (multiple sensory inputs → single motor output)

**Force Hierarchy (Biological Priority):**

See `apps/simulation/src/simulation/movement/constants.rs` for current force magnitudes:
1. **Panic** - Emergency collision prevention (survival)
2. **Avoidance** - Collision prevention (high priority)
3. **Seeking** - Goal pursuit (moderate priority)
4. **Wander** - Exploration (low priority)
5. **Flee** - Threat response (high priority, future)

---

## Movement Ecology Research

### Elastic Tether Model

**Source:** Movement ecology research (zoologist consultation 2025-11-08)

**See also:** `docs/biology/done/wandering-behavior.md` for complete territory wandering documentation

**Biological principle:** Animals don't wander randomly - they patrol territories with soft boundaries.

**Behavior:**
- Near home: Free exploration, low homeward bias
- Far from home: Strong urgency to return

**Mathematical model:** Sigmoid blending curve
- Smooth behavioral transitions (not hard thresholds)
- Near home (0-10m): 90% wandering, 10% homeward
- Mid-range (10-20m): 50% wandering, 50% homeward
- Far from home (20-30m): 10% wandering, 90% homeward

**Parameters:**
- `comfort_radius`: 10m (territory core, low home bias)
- `blend_center`: 20m (50% blend point, patrol boundary)
- `max_wander_distance`: 30m (hard limit for excursions)
- `sigmoid_steepness`: 1.5 (smooth but definite transition)

---

## Physics Validation

### Frame Rate Constraints

**Simulation tick:** 20 Hz (dt = 0.05s)

**Physics validation:**
- Max movement per frame: `MAX_SPEED × dt = 5.0 × 0.05 = 0.25m`
- Minimum collision threshold: 0.5m (half smallest creature)
- Safety margin: 2× (prevents physics tunneling)

**Tunneling prevention:** Ensures creatures can't pass through obstacles by moving too far in a single frame.

### Velocity Damping

**Value:** 0.98 (2% velocity loss per frame)

**Effect:** Mimics air resistance + ground friction
- After 1 second (20 frames): velocity drops to 67% of initial
- After 2 seconds: velocity drops to 45% of initial
- Creatures naturally "coast to a stop" without continuous thrust

**Tuning notes:**
- Too high (0.99): Creatures slide like on ice
- Too low (0.92): Too much resistance, creatures barely move
- 0.98: Balanced air/ground resistance

**Historical note:** Original zoologist recommendation was 0.92, but this proved too aggressive when combined with seek force of 10N. Adjusted to 0.98 for better gameplay.

**Biological impact:** Creates continuous energy cost for movement - fast creatures burn energy rapidly just maintaining speed.

### Perlin Noise Locomotion

**Status:** ✅ Implemented
**Location:** `apps/simulation/src/simulation/movement/systems.rs:66-79`, `movement/noise.rs`

**What it does:** Adds smooth, organic lateral jitter to moving creatures. Instead of traveling in perfectly straight lines (robotic), creatures gently weave side-to-side as they move (lifelike).

**Biological rationale:**
- Animals don't move in perfectly straight lines (muscle micro-adjustments, terrain irregularities, balance corrections)
- Smaller animals have more erratic movement (higher frequency body adjustments)
- Faster movement amplifies natural jitter (less control at high speed)

**How it works:**
- Applied **perpendicular** to velocity vector (lateral drift, not forward/backward)
- Only applies when speed > 0.01 m/s (stationary creatures don't jitter)
- Uses deterministic Perlin noise (same entity = same jitter pattern over time)
- Independent X/Y noise streams (prevents diagonal bias)

**Scaling factors:**
- **Speed scaling:** `noise_magnitude ∝ (speed / MAX_SPEED)²` - Faster movement = more jitter
- **Size scaling:** `noise_magnitude ∝ 1/√body_length` - Smaller creatures = more jitter
- **Base magnitude:** 99.5 (configurable via `MovementConfig.locomotion_noise_base`)
- **Time scale:** 0.01 (configurable via `MovementConfig.noise_time_scale`)

**Examples:**
- Small creature (0.5m), full speed: High-frequency weaving (visible jitter)
- Large creature (5m), full speed: Gentle sway (barely noticeable)
- Any creature, slow speed: Minimal drift (precise low-speed control)

**Why Perlin noise?**
- Smooth continuous variation (not random jumps)
- Deterministic (same seed = same pattern, enables replay/debugging)
- Organic appearance (matches natural rhythms)

**Trade-offs:**
- Increases path length slightly (meandering vs straight)
- Makes precise positioning harder (can't hit exact coordinates)
- **Future:** Will interact with terrain (smooth ground = less jitter, rough terrain = more)

---

## Arrival & Deceleration Algorithms

### Exponential Deceleration (Seeking)

**Strategy:** "Land on a dime" behavior
- Maintains speed far from target (max reaction time)
- Sharp deceleration near target (prevents overshoot)
- Snap-to-target "pounce" when close and slow (prevents creeping)

**Math:** `desired_speed = max_speed × e^(k×ratio) / e^k`

**Parameters:**
- `slow_zone_multiplier`: 30.0× personal_space (begin deceleration)
- `slow_zone_decay`: 1.5 (exponential decay factor)
- `pounce_distance`: 0.5m (snap threshold)
- `pounce_speed`: 5.5 m/s (max speed for snap)
- `arrival_tolerance`: 0.5m (stop when edge reaches target)

**Decay factor tuning:**
- Low k (0.5-1.0): Gentle deceleration, early slowdown
- High k (2.0-3.0): Sharp deceleration, late braking
- k=1.5: Balanced (maintains speed, then brakes hard)

### Emergency Braking

**Trigger:** Distance < arrival_tolerance

**Force:** 70N (1.4× max seeking force)

**Rationale:** Hard counter-force when overshooting prevents perpetual circling.

---

## Perception & Personal Space

### Perception Range

**Formula:** `perception_range = body_length × perception_multiplier`

**Current value:** 10.0× body length
**Example:** 1m creature detects others within 10m

**DNA range (future):**
- Min: 3.0× (ambush predator, short-range)
- Max: 20.0× (vigilant prey, long-range)
- Default: 10.0× (active forager)

**Energy cost:** `sqrt(perception_range / body_length)` - high perception drains energy

### Personal Space

**Formula:** `personal_space = body_length + spacing_buffer`

**Current value:** 1.5m buffer
**Example:** 1m creature maintains 2.5m spacing (1.0 + 1.5)

**DNA range (future):**
- Min: 0.5m (colonial/tolerant species)
- Max: 3.0m (territorial species)
- Default: 1.5m (solitary animal)

**Behavioral impact:**
- Low buffer: Dense groups, schooling, herding
- High buffer: Territorial, solitary, aggressive

### Panic Threshold

**Formula:** `panic_threshold = personal_space × panic_threshold_ratio`

**Current value:** 0.5 (50% of personal_space)
**Example:** 2.5m personal_space → 1.25m panic threshold

**Rationale:** When another creature is within 50% of comfort zone, collision is imminent - trigger maximum evasive force.

### State-Dependent Personal Space (Energy Modulation)

**Status:** ✅ Implemented

**Formula:** `effective_personal_space = base_personal_space × (0.4 + 0.6 × energy_fraction)`

**Energy effects:**
- 100% energy: 1.0× modifier (full personal space maintained)
- 50% energy: 0.7× modifier (30% reduction, mild hunger)
- 0% energy: 0.4× modifier (60% reduction, starvation)

**Biological basis:**
- Ghrelin (hunger hormone): Reduces territorial aggression by 40-60% in mammals
- Cortisol (stress): Dampens amygdala threat response to proximity
- Prefrontal override: Goal-directed behavior suppresses avoidance when resources critical

**Real-world examples:**
- **Vultures:** 50-100m soaring spacing → body-contact feeding (200+ birds in 20m²)
- **Wolves:** 2-5m travel spacing → shoulder-to-shoulder at kills
- **Wildebeest:** 5-10m grazing spacing → trampling density at water sources

**Implementation:**
- Location: `apps/simulation/src/simulation/perception/components.rs:84-91`
- Method: `AvoidanceBehavior::effective_personal_space(energy_fraction: f32)`
- Applies to ALL behaviors (seeking, wandering, catatonic) for biological realism

**Trade-offs:**
- **Cost:** Crowding increases disease transmission, injury risk, stress metabolism
- **Benefit:** Access to contested high-value resources (food, water, mates)
- **Niche:** Creates "cautious" (maintain space when hungry) vs "bold" (collapse space) archetypes

**Future DNA integration:**
- Gene: `energy_sensitivity` (0.2-1.0)
- Low sensitivity (0.2): Maintains boundaries even when starving (cautious, risk-averse)
- High sensitivity (1.0): Collapses personal space when hungry (bold, risk-tolerant)
- Formula: `modifier = 1.0 - (energy_sensitivity × (1.0 - energy_fraction))`

---

## DNA Migration Plan

All constants documented here are flagged for migration to DNA-driven parameters. See `/docs/technical-debt.md` for full inventory.

**Migration priority:**
1. **Phase 1:** Body size genes (length, mass)
2. **Phase 2:** Locomotion genes (speed, agility, turning)
3. **Phase 3:** Behavior genes (perception, territory size, aggression)

**Gene expression API (planned):**
```rust
// Future DNA-driven parameters
let max_speed = dna.express_gene("agility");
let perception_range = body_length * dna.express_gene("perception_multiplier");
let personal_space = body_length + dna.express_gene("spacing_buffer");
```

---

## References

- "The Nature of Code" by Dan Shiffman (Reynolds steering behaviors)
- Movement ecology research (elastic tether model) - See `docs/biology/done/wandering-behavior.md`
- Kleiber's Law (metabolic scaling)
- `/docs/architecture/behavior-engine.md` - Force accumulation architecture

**See also:** `docs/biology/done/wandering-behavior.md`, `docs/biology/done/brain-decision-timing.md`, `docs/biology/ideas/collision-physics.md`

---

**Last Updated:** 2025-11-29
