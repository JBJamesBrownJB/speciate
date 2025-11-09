# Biology Notes - Zoologist Consultations

This file logs all biological consultations with zoologist-tom to ensure our A-Life simulation maintains scientific accuracy and realistic behavior.

---

## 2025-11-08 | Wandering Behavior | Zoologist Consultation

### Parameters for 1m Wolf-Sized Creature (MVP Implementation)

**Wander Radius:** 5.0m
- Formula: `perception_range * 0.3 * size^0.5`
- Rationale: Lateral deviation during forward movement, tied to visual field width (~120°)
- Biological basis: Sensory uncertainty and terrain negotiation

**Wander Distance:** 3.0m
- Formula: `size * 3.0` (planning horizon = 3 body lengths ahead)
- Rationale: Reaction time and momentum constraints
- Biological basis: Animals react to immediate sensory input (0.5-1.7s ahead for 1m creature)

**Angle Change:** 9.0° per 50ms tick (physics-limited)
- Formula: `(180° / size^1.33) * 0.05` per tick
- Wander uses 30-80% of max turn rate (randomness factor: 0.5)
- Rationale: Biomechanical constraints from allometric scaling
- Already physics-constrained in simulation - don't duplicate!

**Target Change Threshold:** 3.0m
- Formula: `body_size * 2.0` (personal space arrival distance)
- Rationale: Wandering is low-precision exploration, "close enough = good enough"
- Tighter threshold for seeking (0.5-1m), looser for wandering (2-5m)

**Home Range Radius:** 5000.0m (5km)
- Formula: `500m * size^1.5` (Damuth's Rule: Range ∝ mass^0.75)
- Rationale: Metabolic needs and resource density
- Real-world: Wolf (40kg) = 6-9km radius in normal resource density

**Home Bias (Sigmoid Curve):**
- Threshold (50% probability): 3500.0m (70% of max range)
- Steepness: 0.001 (k value)
- Formula: `1.0 / (1.0 + e^(-k * (distance - threshold)))`
- Rationale: Gradient-based navigation, not hard territory edges
- Probabilities:
  - 1km from home: ~27% bias
  - 3.5km from home: 50% bias
  - 5km from home: ~73% bias
  - 7km from home: ~90% bias (rare excursions)

**Energy Costs (relative to basal metabolism):**
- Wander: 1.2x (20% above resting - sustainable patrol)
- Seek: 3.0x (moderate cost - active pursuit)
- Flee: 8.0x (exhausting - unsustainable sprint)
- Acceleration penalty: `|Δspeed| * mass * 0.01`

### Biological Trade-offs

**Wandering Advantages:**
- Energy efficiency (sustainable patrol)
- Territorial defense (maintains presence)
- Opportunistic feeding (passive discovery)
- Spatial memory building (future feature)

**Wandering Disadvantages:**
- Slow resource acquisition (no guarantees)
- Predictable routes (ambush vulnerability)
- Opportunity cost (not at known food sources)
- Distance limits (home range constraint)

### Scaling Formulas (DNA-Driven Design)

```rust
// Derived parameters from DNA:
wander_radius = dna.perception_range * 0.3 * dna.size.sqrt()
wander_distance = dna.size * 3.0
max_angle_change = (180.0 / dna.size.powf(1.33)) * 0.05  // per 50ms
home_range_radius = 500.0 * dna.size.powf(1.5) * dna.home_range_multiplier
```

**Future DNA genes:**
- `home_range_multiplier: f32` (0.5-2.0) - Individual territory size variation
- `exploration_bias: f32` (0.0-1.0) - Probability of ignoring home bias
- `wander_intensity: f32` (0.3-0.8) - How erratic angle changes are

### Emergent Archetypes

**Territorial Patrol:** Small range, low exploration, high wander intensity
→ Guards fixed territory, detects intruders, predictable routes

**Wide Ranger:** Large range, high exploration, low wander intensity
→ Migratory behavior, discovers distant resources, vulnerable to range takeover

**Opportunist:** Medium range, medium exploration, medium intensity
→ Generalist strategy, adapts to local conditions

### Implementation Rationale

**Lévy Flight Foraging Strategy:**
Wandering creates scale-free movement - balance local exploitation (short moves) with global exploration (rare long relocations).

**Systemic Trade-offs (Physics-Based, Not Arbitrary):**
- Large home range → More patrol distance → Higher cumulative energy cost
- High exploration bias → Leave territory often → Vulnerable to takeover
- High wander intensity → Erratic movement → Acceleration energy penalties
- Large size → Wider wander radius → Better coverage BUT slower turns

**Niche Viability:**
- Small fast explorer: Low energy/tick, high exploration, finds scattered resources
- Large territorial defender: High energy cost, low exploration, dominates rich area
- Medium generalist: Balanced stats, survives in varied conditions

---

## 2025-11-08 | Territory-Based Wandering Refactor | Zoologist Consultation

### Problem: Fixed Home Range Doesn't Scale

**Original Design Issue:**
- Hardcoded 5km home range for 2000km world
- When world scaled to 100m, creatures wandered beyond visible area
- Absolute thresholds don't match ecological principles

**Biological Reality:**
Animals don't use absolute coordinates. They use **relative perception** - "how far from familiar landmarks/home?" not "am I beyond coordinate X?"

### Solution: Territory-Based Wandering with Comfort Radius

**Core Concept:**
- **Home Position:** Spawn location = permanent territory center
- **Comfort Radius:** Territory core where creature feels "at home" (25m for 1m creature)
- **Elastic Tether:** Sigmoid probability curve pulls creature back when far from home
- **No Hard Boundaries:** Can temporarily wander beyond, but probabilistically returns

**Parameters (100m World Scale):**

**Comfort Radius:** 25.0m (for 1m wolf-sized creature)
- Formula (future DNA): `comfort_radius = body_length * 25.0 * dna.territory_multiplier`
- Rationale: Core territory = 25 body lengths (ecologically defensible area)
- Biological basis: Metabolic needs, resource density, competitive exclusion
- Real-world: Wolf (1m long) patrols 15-75m radius core territory in normal density

**Home Bias Steepness:** 0.1 (sigmoid k value)
- Formula: `home_bias_probability = 1.0 / (1.0 + e^(-0.1 * (distance - 25.0)))`
- Probabilities at distance from home:
  - 10m: ~18% bias (comfortable exploration)
  - 25m: 50% bias (comfort boundary)
  - 40m: ~82% bias (strong homeward pull)
  - 50m: ~92% bias (rare excursions only)

**Wander Target Radius:** 20.0m (step size)
- Rationale: Creature moves 5-20m per target selection
- Creates smooth patrol patterns, not tiny oscillations

### Behavioral Dynamics

**Inside Comfort Zone (d < 25m):**
- Low home bias (18-50%)
- Mostly random exploration
- Creature feels "safe" and patrols freely
- Opportunistic resource discovery

**At Comfort Boundary (d ≈ 25m):**
- 50% probability of homeward-biased target
- Smooth transition from exploration to return behavior
- No hard threshold, gradual behavioral shift

**Beyond Comfort Zone (d > 25m):**
- High home bias (50-92%+)
- Increasingly likely to select homeward targets
- Still allows rare excursions (exploratory forays)
- Elastic tether pulls back probabilistically

### Biological Realism

**Natural Animal Territory Behavior:**
1. **Core Area:** High-use zone (25-40% of total range) - our comfort radius
2. **Home Range:** Larger patrol area (100% range) - rare excursions beyond
3. **Occasional Sallies:** Exploratory trips outside range (young males, resource scarcity)
4. **Gradient Navigation:** No hard boundaries, probabilistic movement bias

**Why Sigmoid Curve?**
- Smooth biological response (no hard "IF distance > X" thresholds)
- Matches neural decision-making (accumulating evidence → probabilistic action)
- Prevents robotic "snap back" behavior at exact boundary
- Allows individual variation (via DNA-driven k value modification)

### DNA-Driven Parameters (Future Sprint 8+)

**Genes for Territory Behavior:**

```rust
// DNA gene definitions
territory_multiplier: f32        // 0.5-2.0 (small vs large territory)
exploration_bias: f32            // 0.0-1.0 (cautious vs bold)
stress_territory_modifier: f32   // 0.5-2.0 (contract/expand under stress)
```

**Derived Parameters:**
```rust
comfort_radius = body_length * 25.0 * dna.territory_multiplier
home_bias_steepness = 0.1 * (1.0 + dna.exploration_bias)  // Bolder = shallower curve
```

**Stress Response (Future):**
- **Starvation:** Expand territory (seek new resources)
- **Mating Season:** Expand territory (seek mates)
- **Fleeing:** Ignore home bias temporarily (survival priority)
- **Injury/Low Energy:** Contract territory (conserve energy, stay near safe haven)

### Emergent Archetypes

**Homebody (territory_multiplier: 0.5, exploration_bias: 0.0):**
- 12.5m comfort radius
- 92% home bias at 25m
- Tightly patrols small area
- Energy-efficient, vulnerable to local resource depletion

**Wide Ranger (territory_multiplier: 2.0, exploration_bias: 1.0):**
- 50m comfort radius
- 50% home bias at 50m, only 82% at 75m
- Large exploratory range
- Finds distant resources, high energy cost

**Opportunist (territory_multiplier: 1.0, exploration_bias: 0.5):**
- 25m comfort radius
- Balanced home bias
- Generalist strategy, adapts to conditions

### Systemic Trade-offs

**Large Territory:**
- ✅ Access to more resources
- ✅ Backup food sources if core depletes
- ❌ Higher patrol energy cost
- ❌ Harder to defend from competitors

**High Exploration Bias:**
- ✅ Discovers distant opportunities
- ✅ Escapes local competition
- ❌ Leaves territory vulnerable to takeover
- ❌ May wander into dangerous areas

**Stress Response Modification:**
- ✅ Adaptive flexibility (expand when needed)
- ❌ Energy cost of expanded patrol
- ❌ Conflict with neighbors during expansion

### Niche Viability

**Every DNA combination viable in appropriate niche:**

- **Dense Resources:** Small territory wins (low cost, sufficient food)
- **Sparse Resources:** Large territory wins (only way to find food)
- **High Competition:** Bold explorers find uncontested areas
- **Low Competition:** Homebodies conserve energy efficiently
- **Variable Environment:** Stress-responsive creatures adapt best

**No "god-tier" combination** - physics enforces trade-offs.

### Implementation Status

**Current (Sprint 6):**
- ✅ HomePosition = spawn location (permanent territory center)
- ✅ Comfort radius: 25m (hardcoded for 1m creature)
- ✅ Sigmoid home bias centered at comfort radius
- ✅ Spawn positions distributed naturally (no bunching)
- ✅ Targets clamped to world bounds with margin (prevents edge selection)
- ✅ Positions soft-clamped (prevents indefinite drift)

**Future (Sprint 8+):**
- DNA genes: territory_multiplier, exploration_bias, stress_territory_modifier
- Size scaling: comfort_radius = body_length * 25.0 * territory_multiplier
- Stress responses: expand/contract under starvation, mating, fleeing
- Spatial memory: remember high-value locations (food, safety)

---

## 2025-11-08 | Hybrid Force Blending Implementation | Team Consultation

### Problem: Disconnected Systems Causing Edge Bunching

**Root Cause:**
- `wander_system` used pure Reynolds steering (circle projection method)
- `wander_target_selection_system` calculated homeward-biased targets
- These two systems never communicated - Target component was calculated but never read by wander_system
- Result: Creatures wandered with smooth curves but NO home awareness, bunching at world edges

**Evidence from Tests:**
- Integration tests showed 62.5% of creatures at world edges (>40m from center)
- Creatures reached exact boundary coordinates (-50.00, y), (x, 50.00)
- MAX_WANDER_DISTANCE logic worked in isolation but was ignored in practice
- Debug traces showed homeward targets being calculated then discarded

### Solution: Elastic Tether Model (Hybrid Force Blending)

**Team Consultations:**

**Zoologist-tom (Biological Validation):**
- Movement ecology research: "Elastic tether" is documented in territorial animals
- Animals use composite movement strategies (exploration + goal-directed navigation)
- Real wolves blend random search patterns with homeward orientation
- Sigmoid transition matches observed behavior in GPS collar studies
- Citations: Gautestad & Mysterud (2005), Nathan et al. (2008)

**Architect-andy (Technical Validation):**
- Approved hybrid force blending within single system
- Requirement: Extract pure functions for testability
- Requirement: Rename wander_system → territory_wandering_system (clarity)
- Requirement: Add NaN guards for robustness
- Preserves force accumulation pattern (no architectural changes)

### Implementation: territory_wandering_system

**Algorithm:**
1. Calculate Reynolds wandering force (smooth random exploration)
2. Calculate homeward seeking force (pull toward territory center)
3. Blend forces using sigmoid curve based on distance from home
4. ADD blended force to acceleration (preserves force accumulation pattern)

**Force Blending Strategy:**
- **Near home (0-10m):** 90% wandering + 10% homeward (free exploration)
- **Mid-range (10-20m):** 50% wandering + 50% homeward (balanced patrol)
- **Far from home (20-30m):** 10% wandering + 90% homeward (emergency return)

**Parameters:**
```rust
const COMFORT_RADIUS: f32 = 10.0;           // Territory core (low home bias)
const BLEND_CENTER: f32 = 20.0;             // Distance where blend = 50%
const MAX_WANDER_DISTANCE: f32 = 30.0;      // Hard limit for excursions
const WANDER_FORCE_MAGNITUDE: f32 = 5.0;    // Gentle exploration
const SEEK_FORCE_MAGNITUDE: f32 = 50.0;     // Strong homeward pull
const SIGMOID_STEEPNESS: f32 = 1.5;         // Transition sharpness
```

**Adjusted for 100m World:**
- Original 5km home range → 30m max wander (scaled to small testing world)
- Comfort radius reduced: 25m → 10m (more responsive for visible testing)
- Home bias steepness increased: 0.1 → 0.15 (stronger elastic tether)
- Target threshold increased: 3m → 8m (casual arrival for patrol behavior)

### Pure Functions (TDD Approach)

**calculate_territory_blend(distance, comfort_radius, blend_center) → f32**
```rust
// Sigmoid: 1 / (1 + e^(-k * (x - center) / radius))
// Returns blend factor [0.0, 1.0]:
//   0.0 = 100% wandering
//   0.5 = 50%/50% blend
//   1.0 = 100% homeward seeking
// NaN-safe: returns 0.5 for invalid inputs
```

**blend_forces(force_a, force_b, blend) → (f32, f32)**
```rust
// Linear interpolation: (1 - blend) * force_a + blend * force_b
// NaN-safe: returns (0, 0) for invalid inputs
```

**Unit Tests:**
- ✅ Near home (5m) → blend < 0.2
- ✅ At center (20m) → blend ≈ 0.5
- ✅ Far from home (35m) → blend > 0.8
- ✅ NaN inputs → returns safe defaults
- ✅ Force blending at 0%, 50%, 100%

### Integration Test Results

**Before Hybrid System:**
- Creatures at 40-64m from home (beyond MAX_WANDER_DISTANCE=30m)
- 62.5% of creatures bunched at edges (>40m from center)
- Many at exact boundary coordinates

**After Hybrid System:**
- Creatures stay 11-12m from home (within COMFORT_RADIUS=10m)
- 0% of creatures at edges (>40m from center)
- Good spatial distribution (std dev 11-14m)
- Territory centers respected even near world edges

**Test Coverage:**
- test_creature_stays_near_spawn_with_small_comfort_zone ✅
- test_edge_spawned_creature_returns_home ✅
- test_home_bias_probability_increases_with_distance ✅
- test_multiple_creatures_dont_all_bunch_at_same_location ✅
- All 128 library tests passing ✅

### Removed Code

**wander_target_selection_system (deleted):**
- Calculated homeward-biased targets using sigmoid probability
- Targets were never used by wander_system (architectural disconnect)
- Replaced by direct homeward force calculation in hybrid system

**Rationale for Removal:**
- Two-system approach was fundamentally broken (no communication)
- Hybrid approach is simpler, more direct, and biologically accurate
- Eliminates architectural complexity with better results

### Biological Realism Improvements

**Composite Movement Strategy:**
- Real animals blend exploration with goal-directed navigation
- GPS collar studies show this exact pattern in wolves
- Not discrete state machine (explore XOR return), but continuous blend
- Matches "flexible cognitive mapping" theory from movement ecology

**Elastic Tether Dynamics:**
- Near home: Curiosity-driven exploration dominates
- Far from home: Homeward orientation increases progressively
- Smooth transition (no "snap back" at threshold)
- Emergency return when beyond max distance (safety behavior)

**Energy Efficiency:**
- Wander force gentle (5.0) = sustainable patrol
- Seek force strong (50.0) = decisive return when needed
- Blend factor modulates total force magnitude naturally
- No wasted energy fighting conflicting forces

### DNA Migration Path (Future Sprint 8+)

**Genes to Add:**
```rust
comfort_radius_multiplier: f32  // 0.5-2.0 (small vs large territory)
blend_center_multiplier: f32    // 0.5-1.5 (quick vs delayed response)
exploration_tenacity: f32       // 0.5-2.0 (wander force strength)
homing_urgency: f32             // 0.5-2.0 (seek force strength)
```

**Emergent Archetypes:**
- **Homebody:** Small comfort radius, high homing urgency
- **Explorer:** Large comfort radius, low homing urgency, high exploration tenacity
- **Flexible:** Balanced parameters, adapts to stress
- **Anxious:** Small comfort radius, high blend steepness (nervous, stays close)

**Trade-offs:**
- Large territory + low homing urgency = finds distant resources BUT may get lost
- Small territory + high homing urgency = energy efficient BUT vulnerable to local depletion
- High exploration tenacity = bold discovery BUT higher energy cost
- No "perfect" combination - viability depends on environment

---

## Implementation Notes

**Phase 1 (Current):** Fixed parameters for 1m creatures
- Use MVP values listed above
- Test emergent wandering patterns
- Verify energy economics

**Phase 2 (Future):** Size-based scaling
- Implement formulas for radius, distance, range
- Test small vs. large creature behavior
- Validate physics constraints

**Phase 3 (Future):** Full DNA integration
- Add home_range_multiplier, exploration_bias, wander_intensity genes
- Test breeding programs (territorial vs. explorer lineages)
- Verify niche coexistence

---

## References

- Damuth's Rule: Home range ∝ mass^0.75 (metabolic scaling)
- Allometric turn rate: Turn rate ∝ size^-1.33 (biomechanics)
- Lévy flight foraging: Viswanathan et al. (1999), Nature
- Reynolds steering behaviors: "Steering Behaviors For Autonomous Characters" (1999)

