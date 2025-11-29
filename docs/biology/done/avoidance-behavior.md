# Avoidance Behavior - Personal Space & Collision Prevention

**Status:** ✅ Implemented
**Location:** `apps/simulation/src/simulation/creatures/behaviors/avoidance.rs`

---

## What It Does

Creatures maintain personal space by generating repulsion forces when others get too close. Force strength follows inverse-square scaling (like electromagnetic repulsion) - the closer the neighbor, the exponentially stronger the push-away force.

**Result:** Natural spacing emerges without hard collision detection. Creatures flow around each other smoothly like birds in a murmuration or fish in a school.

---

## Why It Exists

### Biological Realism

**Real animals maintain spacing for:**
- Collision prevention (physical safety)
- Resource competition (feeding zones)
- Disease avoidance (epidemiological buffer)
- Comfort/stress management (social tolerance threshold)

**Examples:**
- Schooling fish: 1-2 body lengths spacing (predator confusion)
- Wolves traveling: 2-5m spacing (navigation freedom)
- Birds on wire: Wing-span + buffer (takeoff clearance)

### Electromagnetic Analogy

Inverse-square force matches real physics:
- Like charges repel with F ∝ 1/r²
- Closer = exponentially stronger (not linear)
- Prevents "soft collisions" (gradual overlap)
- Creates natural equilibrium distance

---

## Key Parameters

**Location:** `apps/simulation/src/simulation/movement/constants.rs`

### Personal Space
- See `constants.rs` PERCEPTION.personal_space for base buffer value
- Added to body radius to determine minimum spacing
- **Scaling:** Larger creatures need proportionally larger buffers

### Force Magnitudes
See `constants.rs` STEERING constants for current values:
- **avoidance_force** - Baseline repulsion force
- **panic_force** - Emergency override force (significantly stronger)

### Panic Threshold
- See `constants.rs` or brain component for panic threshold multiplier
- Trigger distance scales with body_size
- **Rationale:** Amygdala hijack - fight-or-flight overrides deliberative processing

---

## Energy-Modulated Personal Space

**Status:** ✅ Implemented

### Biological Principle

**Hungry creatures tolerate closer proximity to reach contested resources.**

**Formula:** `effective_space = base_space × (0.4 + 0.6 × energy_fraction)`

### Energy Effects

| Energy Level | Modifier | Spacing Change |
|--------------|----------|----------------|
| 100% (full) | 1.0× | Full personal space maintained |
| 50% (hungry) | 0.7× | 30% reduction, mild hunger tolerance |
| 0% (starving) | 0.4× | 60% reduction, desperation override |

### Biological Basis

**Hormonal mechanisms:**
- **Ghrelin (hunger hormone):** Reduces territorial aggression by 40-60% in mammals
- **Cortisol (stress):** Dampens amygdala threat response to proximity
- **Prefrontal override:** Goal-directed behavior suppresses avoidance when resources critical

**Real-world examples:**
- **Vultures:** 50-100m soaring spacing → body-contact feeding (200+ birds in 20m²)
- **Wolves:** 2-5m travel spacing → shoulder-to-shoulder at kills
- **Wildebeest:** 5-10m grazing spacing → trampling density at water sources

### Trade-offs

**Costs of crowding:**
- Disease transmission risk (respiratory, contact spread)
- Physical injury (trampling, aggressive competition)
- Stress metabolism (elevated cortisol → fat storage)

**Benefits of crowding:**
- Access to high-value resources (food, water, mates)
- Resource monopolization (exclude competitors through sheer presence)
- Social learning (observe successful foraging strategies)

---

## Panic Override System

### When Panic Triggers

**Distance threshold:** Neighbor within `body_size × 2.0`

**Bypass normal decision cooldown** - immediate reaction, no deliberation.

### Biological Rationale

**Amygdala hijack:**
- Thalamus → Amygdala (fast, crude threat assessment: 12-25ms)
- Bypasses cortex (slow, accurate processing: 100-300ms)
- Motor cortex activation before conscious awareness

**Survival advantage:** React to threat before brain "knows" it's a threat.

**Examples:**
- Startle response (predator lunges)
- Flinch reflex (falling object)
- Emergency swerve (car cutting in)

### Panic Disabling

**Trigger:** Energy < 5.0

**Rationale:** Too weak to run, "giving up" behavior (conserve last energy for basic metabolism).

---

## Implementation Details

### Inverse-Square Force Formula

**For each nearby creature:**
```
distance_ratio = personal_space / actual_distance
force_magnitude = base_force × distance_ratio²
```

**Force scaling examples:**
- Distance = personal_space → Force = 1.0× base (equilibrium)
- Distance = personal_space / 2 → Force = 4.0× base (close quarters)
- Distance = personal_space / 4 → Force = 16.0× base (collision imminent)

### Force Capping

**Maximum total force:** `max_force` parameter

**Why cap?** Prevents numerical instability at extreme proximity (divide-by-zero, runaway acceleration).

### Performance Optimization

**OPT-7 Early Exit:**
- Check squared distance BEFORE computing expensive `sqrt()`
- 80% of neighbors filtered with cheap comparison
- Only compute `sqrt()` for close neighbors (20% of cases)

**Performance gain:** 19% faster (3.2ms → 2.6ms @ 10K creatures)

---

## Integration with Other Systems

### Perception System
Avoidance uses `Perception::iter_neighbors()` to find nearby creatures. Only checks entities within perception range (typically 10× body length).

### Movement System
Avoidance forces **accumulate** into `Acceleration` component alongside:
- Seek forces (goal pursuit)
- Flee forces (threat escape)
- Wander forces (exploration)

**Result:** Natural path planning emerges from force blending.

**Example:** Creature seeking food weaves around obstacles automatically (seek + avoid = smooth navigation).

---

## Future Work

### DNA Integration (Planned)

**Gene: `personal_space_multiplier` (0.5-3.0)**
- Low (0.5): Colonial/tolerant species (schooling fish, herding ungulates)
- High (3.0): Territorial species (wolves, bears, big cats)

**Gene: `panic_threshold_multiplier` (1.5-3.0)**
- Low (1.5): Nervous, hair-trigger response (prey species)
- High (3.0): Calm, confident (apex predators)

**Gene: `energy_sensitivity` (0.2-1.0)**
- Low (0.2): Maintains boundaries even when starving (cautious, risk-averse)
- High (1.0): Collapses personal space when hungry (bold, risk-tolerant)

### Energy Costs

**Metabolic penalty for crowding:**
- Stress hormone production (cortisol)
- Immune system activation (disease exposure)
- Vigilance metabolism (constant threat monitoring)

**Formula (planned):** `stress_metabolism = base × (1.0 - effective_space / base_space)`

---

## Related Systems

- **Seeking:** Target-directed movement (combines with avoidance for path planning)
- **Fleeing:** Threat escape (uses avoidance to navigate around obstacles while fleeing)
- **Wandering:** Exploration (avoidance keeps wandering creatures from bunching up)

---

## References

- Electromagnetic inverse-square law (physics analog)
- Amygdala hijack (LeDoux, 1996 - fear response)
- Ghrelin-aggression link (Hansson et al., 2014 - hunger physiology)
- Movement ecology (Nathan et al., 2008 - spacing behaviors)

**See also:** `docs/biology/done/movement-physics.md`, `docs/biology/done/perception-system.md`, `docs/biology/done/target-radius-seeking.md`

---

**Last Updated:** 2025-11-29
