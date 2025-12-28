# Wandering Behavior - Territory & Elastic Tether

**Status:** ✅ 90% IMPLEMENTED (DNA genes pending)

**Code:** `apps/simulation/src/simulation/creatures/steering/wander.rs`

---

## Core Concept

Animals use **composite movement**: exploration (random wander) + goal-directed navigation (homeward pull). No hard territory boundaries—sigmoid probability curve creates "elastic tether" effect.

**Implementation:** Hybrid force blending system combines Reynolds wandering with homeward seeking force.

---

## Key Parameters

### HomePosition Component
**Status:** ✅ Implemented

Spawn location = permanent territory center. Creature probabilistically returns when far from home.

### Comfort Radius
**Current:** 10.0m (hardcoded)
**Formula (future DNA):** `comfort_radius = body_length × 25.0 × dna.territory_multiplier`

Core territory where creature feels "at home" (low homeward bias).

### Home Bias (Sigmoid Curve)
**Status:** ✅ Implemented

```rust
home_bias = 1.0 / (1.0 + e^(-steepness × (distance - blend_center) / comfort_radius))
```

**Probabilities:**
- 5m from home: <20% bias (free exploration)
- 20m from home: 50% bias (balanced)
- 35m from home: >80% bias (strong pull)

**Constants:** See `apps/simulation/src/simulation/movement/constants.rs` (TERRITORY struct)
- `blend_center` - Distance where wander/homeward forces are 50/50
- `sigmoid_steepness` - Curve sharpness
- `max_wander_distance` - Hard limit for excursions

### Force Blending
**Status:** ✅ Implemented

```rust
blended_force = (1 - home_bias) × wander_force + home_bias × homeward_force
```

- See `constants.rs` STEERING.wander_force (gentle exploration)
- See `constants.rs` TERRITORY.homeward_force (strong pull when needed)

---

## Biological Realism

**Real animal territory behavior:**
1. Core area (25-40% of range) - comfort radius
2. Home range (100%) - rare excursions beyond
3. Occasional sallies - exploratory forays
4. Gradient navigation - no hard boundaries

**Why sigmoid?**
- Matches neural decision-making (accumulating evidence → probabilistic action)
- Prevents robotic "snap back" at exact boundary
- Smooth biological response, not discrete threshold

**GPS collar studies:** Real wolves show this exact pattern (Gautestad & Mysterud 2005, Nathan et al. 2008)

---

## DNA Genes (Not Yet Implemented)

### territory_multiplier: f32 (0.5-2.0)
Size of territory relative to body length.

**Trade-offs:**
- Large territory: More resources BUT higher patrol energy cost, harder to defend
- Small territory: Energy efficient BUT vulnerable to local depletion

### exploration_bias: f32 (0.0-1.0)
How bold/cautious the creature is.

**Trade-offs:**
- High bias: Discovers distant opportunities BUT leaves territory vulnerable
- Low bias: Stays safe BUT misses opportunities

### stress_territory_modifier: f32 (0.5-2.0)
Expand/contract territory under stress.

**Triggers:**
- Starvation → expand (seek new resources)
- Mating season → expand (seek mates)
- Injury/low energy → contract (conserve energy)
- Fleeing → ignore home bias temporarily

---

## Emergent Archetypes

**Homebody** (territory_multiplier: 0.5, exploration_bias: 0.0)
- 12.5m comfort radius
- Tightly patrols small area
- Energy-efficient, vulnerable to local depletion

**Wide Ranger** (territory_multiplier: 2.0, exploration_bias: 1.0)
- 50m comfort radius
- Large exploratory range
- Finds distant resources, high energy cost

**Opportunist** (territory_multiplier: 1.0, exploration_bias: 0.5)
- 25m comfort radius
- Balanced strategy, adapts to conditions

---

## Implementation Status

### ✅ Implemented
- HomePosition component
- Sigmoid home bias calculation
- Hybrid force blending (wander + homeward)
- Comfort radius (hardcoded 10.0m)
- Territory wandering system

### ❌ Not Implemented
- DNA genes (territory_multiplier, exploration_bias, stress_territory_modifier)
- Size-based scaling formulas
- Stress response modification
- Spatial memory

**Location:** `apps/simulation/src/simulation/movement/constants.rs:49-68`
