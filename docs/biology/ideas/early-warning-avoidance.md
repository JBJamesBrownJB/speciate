# Early Warning Avoidance - Biological Concept

**Status:** ⏳ PLANNED
**Expected Duration:** 3-4 hours

---

## Concept Overview

Animals don't wait until the last second to avoid obstacles - they start making subtle trajectory adjustments well before close encounters. This document describes the biological rationale for adding **early warning zones** to creature avoidance behavior.

**Current behavior:** Creatures only react when others enter personal_space (2.5m for 1m creature)
- Late course corrections (jerky movement)
- Unnecessary slowdowns (already too close)
- Unrealistic "last second" avoidance

**Proposed behavior:** Gentle trajectory tweaking starts at 3-5× personal_space (8-9m)
- Early, smooth course corrections
- Maintains speed while navigating
- Biologically realistic spacing

---

## Biological Examples

### Birds in Murmurations

**Spacing behavior:**
- Free flight: 7-10 body lengths spacing
- Start reacting: 7-10 body lengths away
- Close proximity: 1-2 body lengths (during tight turns)

**Key insight:** Smooth, continuous trajectory tweaking. Never sudden swerves except during predator attacks.

**Reference:** Ballerini et al. (2008) - "Interaction ruling animal collective behavior depends on topological rather than metric distance"

### Fish Schools

**Spacing behavior:**
- Normal swimming: 1-2 body lengths maintained
- React at: 3-4 body lengths
- Emergency: <0.5 body lengths triggers rapid acceleration

**Key insight:** Gradual repulsion gradient, not binary on/off.

**Reference:** Partridge & Pitcher (1980) - "The sensory basis of fish schools"

### Human Pedestrians

**Spacing behavior:**
- Walking speed: 1.4 m/s (typical)
- Start adjusting: 2-3 seconds before collision (~5-10m)
- Personal space: ~0.5m (intimate distance)

**Key insight:** Course corrections begin at 10-20 body lengths, not at personal space boundary.

**Reference:** Helbing & Molnár (1995) - "Social force model for pedestrian dynamics"

---

## Core Biological Principle

**"Animals optimize energy by avoiding unnecessary hard maneuvers."**

### Energy Cost Comparison

**Early gentle correction:**
- Small lateral force over long distance
- Maintains forward speed
- Low metabolic cost (minimal muscle activation)
- Example: 5N force over 5m = 25J energy

**Late hard swerve:**
- Large lateral force over short distance
- Requires deceleration + acceleration
- High metabolic cost (maximal muscle activation)
- Example: 90N force over 1m + speed recovery = 200J+ energy

**Ratio:** Late swerves cost 8× more energy than early corrections

### Sensory Processing Timeline

**Visual detection → Motor response timeline:**

1. **Visual detection** (retina processes light): 10-30ms
2. **Neural transmission** (eye → brain): 10-40ms (distance-dependent)
3. **Visual cortex processing** (identify object): 40-80ms
4. **Decision making** (evaluate threat): 50-150ms
5. **Motor cortex activation** (plan movement): 30-60ms
6. **Muscle activation** (execute movement): 20-50ms

**Total: 160-410ms** from detection to action

**Key insight:** By the time the brain processes "close proximity," the animal needs to already be mid-correction. Early detection is survival-critical.

---

## Three-Zone Behavioral Model

### Zone 1: Early Warning (3-5× personal_space)

**Distance:** 7.5-12.5m for 1m creature (2.5m personal_space)

**Behavior:**
- Gentle lateral force (10-15% of normal avoidance)
- Maintains forward speed
- Subtle course tweaks
- No stress response (low cortisol)

**Biological analog:** Peripheral vision detection, subconscious adjustment

### Zone 2: Personal Space (1× personal_space)

**Distance:** 2.5m for 1m creature

**Behavior:**
- Active avoidance force (inverse-square scaling)
- May slow down slightly
- Deliberate path adjustment
- Mild stress response (elevated cortisol)

**Biological analog:** Direct visual focus, conscious avoidance

### Zone 3: Panic (0.5× personal_space)

**Distance:** 1.25m for 1m creature

**Behavior:**
- Maximum evasive force (panic_force cap)
- Emergency swerve/brake
- High stress response (fight-or-flight)

**Biological analog:** Amygdala hijack, reflexive escape

---

## Speed Adaptation: Why Fast Creatures Need More Warning

### Stopping Distance Physics

**Formula:** `stopping_distance = (velocity²) / (2 × deceleration)`

**Examples (assuming 5 m/s² deceleration):**
- Slow (2 m/s): 0.4m stopping distance
- Medium (5 m/s): 2.5m stopping distance
- Fast (8 m/s): 6.4m stopping distance

**Key insight:** Fast creatures need proportionally more warning distance to make gentle corrections.

### Biological Evidence

**Cheetahs (fast runners):**
- Top speed: 28 m/s
- Course correction at: 50-100m (for obstacles)
- Stopping distance at full speed: ~150m

**Tortoises (slow walkers):**
- Top speed: 0.3 m/s
- Course correction at: 1-2m
- Stopping distance: <0.1m

**Ratio:** Warning distance scales with speed, not just body size.

---

## Energy Modulation: Hunger Overrides Caution

### Hormonal Mechanism

**Ghrelin (hunger hormone):**
- Reduces territorial aggression by 40-60% in mammals
- Dampens amygdala threat response
- Prioritizes resource acquisition over safety

**Effect on spacing:**
- Well-fed: Maintain full personal_space and early warning distance
- Hungry: Tolerate closer proximity (30-60% reduction)
- Starving: Collapse personal_space to minimum (60% reduction)

**Formula:** `effective_warning = base_warning × (0.4 + 0.6 × energy_fraction)`

### Real-World Examples

**Vultures at carcass:**
- Soaring spacing: 50-100m between birds
- Feeding spacing: Body contact (200+ birds in 20m²)
- **Reduction:** 95% personal_space collapse

**Wolves at kill:**
- Travel spacing: 2-5m between pack members
- Feeding spacing: Shoulder-to-shoulder
- **Reduction:** 80% personal_space collapse

**Wildebeest at water:**
- Grazing spacing: 5-10m
- Watering hole: Trampling density (high injury risk)
- **Reduction:** 90% personal_space collapse

**Trade-off:** Access to critical resources vs increased disease/injury risk

---

## Gradient vs Binary Zones

### Why Gradual Force Ramp?

**Binary approach (current):**
```
Distance > 2.5m:  0N force
Distance ≤ 2.5m:  35N force (sudden jump)
```
**Problem:** Creatures don't anticipate, causes jerky movement

**Gradient approach (proposed):**
```
Distance > 8m:     0N force
Distance 2.5-8m:   5.25N force (gentle)
Distance ≤ 2.5m:   35N force (active)
Distance ≤ 1.25m:  90N force (panic)
```
**Benefit:** Smooth behavioral transitions, realistic spacing

### Neural Basis

**Neurons encode gradients, not thresholds:**
- Looming visual stimuli → gradual neural firing rate increase
- Not binary "threat detected" signal
- Motor cortex receives continuous magnitude signal

**Result:** Real animals produce smooth force curves, not step functions.

---

## Ecological Implications

### Niche Differentiation

**Cautious species (high early_warning_sensitivity):**
- Wide berth around others (2-3× normal early warning)
- Smooth, predictable paths
- **Trade-off:** Longer travel times, less aggressive resource competition
- **Ecological role:** Stable populations in low-density environments

**Aggressive species (low early_warning_sensitivity):**
- Narrow berth (0.5× normal early warning)
- Direct paths, push through crowds
- **Trade-off:** Higher collision risk, stress metabolism
- **Ecological role:** Dominant in high-resource environments

### Emergent Flock Behavior

**With early warning:**
- Flocks maintain stable spacing
- Smooth coordinated movement
- Low energy cost (minimal corrections)

**Without early warning:**
- Flocks oscillate (compress → expand cycles)
- Jerky individual movement
- High energy cost (constant hard corrections)

---

## Performance Considerations

### Computational Cost

**Per-creature overhead:**
- One additional distance check: `O(1)`
- One `velocity.length()` call: `O(1)`
- One `max_speed` calculation: `O(1)` (can be cached)

**Per-neighbor overhead:**
- One comparison: `if edge_distance <= early_warning_distance`

**Total overhead:** <5% on avoidance system

### Optimization Opportunities

**Caching:**
- `max_speed` calculated once at spawn (from body_size)
- `speed_ratio` updated once per tick (from velocity)

**Early exit:**
- Stationary creatures (speed ≈ 0) skip speed extension

**Expected impact:** ~0.1ms @ 10K creatures

---

## Future: Trajectory Prediction

**Current:** React to all nearby entities equally

**Advanced:** Only react if on collision course

**Algorithm:**
1. Calculate time-to-closest-approach (TCA) from velocity vectors
2. Predict minimum distance at TCA
3. If TCA < 2 seconds AND min_distance < personal_space: apply early force
4. Otherwise: ignore (will pass safely)

**Benefit:** Ignores creatures moving away or passing perpendicularly

**Cost:** Additional vector math (~15% more expensive)

**Biological analog:** Visual motion processing - only looming objects trigger avoidance

---

## References

### Scientific Literature

- Ballerini et al. (2008) - "Interaction ruling animal collective behavior"
- Partridge & Pitcher (1980) - "The sensory basis of fish schools"
- Helbing & Molnár (1995) - "Social force model for pedestrian dynamics"
- Couzin et al. (2002) - "Collective memory and spatial sorting in animal groups"
- Reynolds (1987) - "Flocks, herds and schools: A distributed behavioral model"
- LeDoux (1996) - "The Emotional Brain" (amygdala hijack)
- Hansson et al. (2014) - "Ghrelin influences novelty seeking behavior" (hunger-aggression)

### Implementation Resources

- Reynolds steering behaviors (Nature of Code, Dan Shiffman)
- Boids algorithm (Craig Reynolds, 1987)

---

## Implementation Phases

### Phase 1: Add Perception Constants

**Outcome:** Three new constants in `perception/constants.rs`

**Tasks:**
- Add `EARLY_WARNING_MULTIPLIER`: 3.0 (base zone is 3× personal_space)
- Add `SPEED_WARNING_FACTOR`: 0.5 (fast creatures get extra lead time)
- Add `EARLY_WARNING_FORCE_RATIO`: 0.15 (gentle nudge, 15% of AVOIDANCE_FORCE)

### Phase 2: Implement Speed-Adaptive Warning Distance

**Outcome:** Avoidance system calculates dynamic early warning zone based on current speed

**Tasks:**
- Formula: `base_warning + (speed_ratio × personal_space × factor)`
- Slow creatures (2 m/s): 8.0m early warning
- Fast creatures (8 m/s): 9.5m early warning
- Provides stopping distance safety margin

### Phase 3: Add Early Warning Force Application

**Outcome:** New force zone applied before existing personal_space logic

**Tasks:**
- Mutually exclusive zones (early OR personal OR panic, not multiple)
- Weak force (5.25N for 35N avoidance base) for gentle nudging
- Energy modulation applies to early warning (hungry creatures still push through)

### Phase 4: Testing & Validation

**Outcome:** Four test scenarios validating smooth navigation

**Tasks:**
- Static crowd: Gentle spacing emerges over time
- High-speed approach: Fast creatures react earlier than slow
- Dense navigation: Smooth weaving vs jerky swerves
- Energy variation: Hungry creatures tolerate closer early encounters

---

## Implementation Files

**Constants:**
- `apps/simulation/src/simulation/perception/constants.rs`

**Logic:**
- `apps/simulation/src/simulation/creatures/behaviors/avoidance.rs`

**Documentation:**
- `docs/biology/done/avoidance-behavior.md` - Add "Early Warning Zone" section
- `docs/biology/done/perception-system.md` - Update constants table

---

## Success Criteria

- [ ] Constants added to `perception/constants.rs` with tests
- [ ] Speed-adaptive warning distance calculation implemented
- [ ] Early warning force applied correctly (before personal_space check)
- [ ] Static crowd test: Gentle spacing emerges (no jerky movements)
- [ ] High-speed test: Fast creatures react earlier than slow creatures
- [ ] Dense navigation test: Smooth weaving through crowds
- [ ] Energy test: Hungry creatures tolerate closer early proximity
- [ ] All existing avoidance tests still pass (no behavioral regression)
- [ ] Documentation updated

---

## See Also

- `docs/biology/done/avoidance-behavior.md` - Current avoidance system
- `docs/biology/done/perception-system.md` - Perception and personal space
- `docs/biology/done/movement-physics.md` - Force accumulation architecture

---

**Last Updated:** 2025-12-11
