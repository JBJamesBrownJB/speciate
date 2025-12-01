# Vision System - DNA-Driven Perception

**Status:** ⏳ PLANNED (stochastic-vision)

**Current Implementation:** Hardcoded 10× perception range, 360° awareness, no FOV

---

## Core Principle

**"You cannot maximize range, FOV, acuity, speed, AND low-light sensitivity simultaneously. Evolution produces specialists."**

Vision systems are expensive: retina is brain tissue, visual processing consumes 20-25% of metabolic output.

---

## Three Core Vision Genes

### 1. visual_range_multiplier: f32 (4.0-25.0, default 10.0)

**Biological basis:** Eye size, foveal photoreceptor density, optical quality

**Trade-offs:**
- High range requires concentrated photoreceptors → reduces FOV effectiveness
- Metabolism cost: +0.5% base per point above 10
- Birth cost: +2% biomass per point above 10

**Formula:**
```rust
let fov_penalty = if dna.visual_arc > PI { 0.7 } else { 1.0 };
let size_bonus = size.powf(0.1);  // Higher vantage point
let max_multiplier = 30.0 / size.powf(0.3);  // Size cap
let clamped = dna.visual_range_multiplier.min(max_multiplier);
let perception_range = body_length × clamped × fov_penalty × size_bonus;
```

### 2. visual_arc: f32 (π/3 to 2π radians, default π)

**Biological basis:** Eye placement (lateral vs frontal), retinal extent

**Ranges:**
- 60-90° (π/3-π/2): Predator (binocular, frontal)
- 180-240° (π-4π/3): Generalist (mixed)
- 270-360° (3π/2-2π): Prey (lateral, near-omnidirectional)

**Trade-offs:**
- Wide arc (>π) applies 0.7× penalty to effective range (peripheral vision = lower acuity)
- Birth cost: +1% biomass per π/6 above π

**FOV Implementation:**
- Stationary creatures: 360° (no facing direction)
- Moving creatures: Use velocity vector as facing, apply FOV cone
- Dot product check filters entities outside cone

### 3. neural_speed: f32 (0.5-2.0, default 1.0)

**Biological basis:** Optic nerve myelination, reflexive vs deliberative processing

**Trade-offs:**
- Fast processing = high energy burn, prone to false positives
- Maintenance cost: +1% base metabolism per 0.1 above 1.0
- Active cost: +3% active metabolism per 0.1 above 1.0
- Birth cost: +1% biomass per 0.1 above 1.0

**Reaction time modifier:**
```rust
let base_ms = 68.0 + (size - 0.5) × 49.41;  // Size formula
let modified_ms = (base_ms / dna.neural_speed).clamp(30.0, 1000.0);
```

---

## Validated Archetypes

### Hawk (Aerial Apex Predator)
- `visual_range_multiplier`: 22.0 (8× human acuity)
- `visual_arc`: π/2 (90°, narrow binocular)
- `neural_speed`: 1.3 (fast deliberative)
- **Phenotype:** Extreme distance, poor peripheral, calculated pursuit
- **Costs:** +9% base metabolism, +27% birth biomass

### Rabbit (Prey Generalist)
- `visual_range_multiplier`: 6.0 (moderate)
- `visual_arc`: 5π/3 (300°, near-omnidirectional)
- `neural_speed`: 1.8 (extremely fast reflexes)
- **Phenotype:** Effective range 4.2× (FOV penalty), detects everywhere, prone to panic
- **Costs:** +8% base metabolism, +24% active metabolism, +12% birth biomass

### Owl (Nocturnal Ambush)
- `visual_range_multiplier`: 14.0 (good not exceptional)
- `visual_arc`: 2π/3 (120°, binocular frontal)
- `neural_speed`: 0.7 (slow integrative)
- **Phenotype:** Patient hunter, long integration time, deliberate strike
- **Costs:** +2% base metabolism, +8% birth biomass

### Bison (Social Grazer)
- `visual_range_multiplier`: 8.0 (decent, secondary to herd)
- `visual_arc`: 3π/2 (270°, wide lateral)
- `neural_speed`: 1.0 (baseline)
- **Phenotype:** Range penalized for wide arc, collective panic behavior
- **Costs:** +4% birth biomass

---

## Stochastic Vision (Planned)

**Problem:** 200K creatures updating perception every tick = CPU bottleneck

**Solution:** Reaction-time-gated updates

### VisionTiming Component

**Size-based reaction times:**
- Small creatures (0.5m): 68ms (~15 updates/sec)
- Large creatures (5m): 500ms (~2 updates/sec)
- Modified by `neural_speed` gene

**Implementation:**
```rust
struct VisionTiming {
    last_update: f64,
    cooldown_ms: f32,  // Derived from size + neural_speed
}
```

**Behavior:**
- Only ~10% of creatures update perception per tick
- Brain reads potentially stale data (biologically realistic - sensory lag exists)
- Natural Poisson distribution from spawn time variation (no spiky CPU load)

**Performance gain:** 10× fewer perception updates → 90% time reduction

---

## The Photoreceptor Budget

Retinal real estate is finite. Wide FOV spreads photoreceptors thin (lower acuity per degree). Narrow FOV concentrates them (higher acuity, longer effective range).

This is why `visual_arc > π` applies the 0.7× range penalty - peripheral vision trades resolution for coverage.

---

## Ecological Niches

**Open terrain:** Hawks dominate (extreme range)
**Dense cover:** Rabbits thrive (wide FOV detects flanking)
**Night hunts:** Owls excel (low-light adapted, patient)

**Player strategy:**
- Domesticate fast-reacting creatures for early warning
- Breed long-range vision for scouting/surveillance
- Exploit predator blind spots (approach from behind)

---

## Expected Emergent Evolution

Over generations, lineages should specialize:

- **Predator lineages** → narrower FOV, longer range (hunting efficiency)
- **Prey lineages** → wider FOV, faster neural speed (threat detection)
- **Ambush predators** → extreme narrow FOV, slow neural speed (patient strikes)
- **Herd animals** → moderate wide FOV (rely on group vigilance, less individual investment)

---

## Implementation Status

### ✅ Currently Implemented
- Basic Perception component with configurable range
- Neighbor tracking (up to 40 neighbors)
- Perception range scales by body size: `body_length × 10.0`

### ❌ Not Implemented (Planned)
- DNA vision genes (visual_range_multiplier, visual_arc, neural_speed)
- VisionTiming component
- FOV cone filtering
- Stochastic perception updates
- Metabolic costs for vision
- Archetype-based specialization

**Current values:** Hardcoded 10× multiplier, 360° awareness, every-tick updates

**Location:** `apps/simulation/src/simulation/perception/`

---

## Source

Zoologist-tom consultation, 2025-11-30
