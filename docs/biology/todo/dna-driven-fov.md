# Vision System - DNA-Driven Perception

**Status:** ⏳ PARTIAL - Basic FOV implemented, DNA genes pending

**Current Implementation:**
- ✅ FOV cone filtering (implemented in Phase A)
- ✅ FOV-range trade-off formula
- ✅ Perception range scales by body size
- ❌ DNA vision genes (visual_range_multiplier, visual_arc, neural_speed)
- ❌ Stochastic vision timing
- ❌ Metabolic costs

See `docs/biology/done/fov-perception.md` for implemented FOV mechanics.

---

## Core Principle

**"You cannot maximize range, FOV, acuity, speed, AND low-light sensitivity simultaneously. Evolution produces specialists."**

Vision systems are expensive: retina is brain tissue, visual processing consumes 20-25% of metabolic output.

---

## Trophic Gating — Decision (2026-06-27)

**Decision: Do NOT hard-gate FOV range by trophic position. Use soft coupling via emergent cost.**

The instinct to reserve narrow FOV for carnivores and wide FOV for herbivores is biologically well-motivated but mechanistically wrong. **Trophic role does not cause FOV — eye placement does.** The real causal axis is binocular overlap (frontal eyes → depth perception) vs panoramic coverage (lateral eyes → rear vigilance). Trophic role is a strong *correlate* of that axis, not the mechanism.

Counter-examples that a hard clamp would forbid:
- **Tarsier** — insectivore but heavily preyed upon; narrow frontal vision for 3D arboreal leaping. Prey with narrow FOV.
- **Dragonfly** — apex aerial predator with ~360° compound vision. Predator with panoramic FOV.
- **Praying mantis** — predator with wide panoramic field + narrow binocular wedge for the strike.
- **Pig** — omnivore with ~310° FOV.

**The correct model:** Keep `fov_gene` full-range (160°–340°), derive two *opposing costs* from it, and let trophic position and FOV co-evolve under selection pressure. The predator-narrow / prey-wide correlation emerges as a *simulation output*, not a constraint baked into the gene.

### Emergent Costs (not config values — pure geometry)

**1. Frontal blind spot grows with width**
Above ~300° total FOV, open a small frontal blind cone (rabbits and horses both have this):
```
frontal_blind_angle = max(0, (fov_angle - 300°) × 0.5)
```
A wide-FOV predator literally loses sight of prey at strike range → misses → starves. This is why predators drift narrow under selection pressure — *without being told to*.

**2. Binocular overlap (depth / strike accuracy) scales inversely with total FOV**
```
binocular_overlap_angle = max(0, 180° - fov_angle × 0.4)
```
Narrow FOV → high binocular overlap → accurate 3D judgment for striking and arboreal navigation.
Wide FOV → near-zero overlap → can detect anything, but cannot accurately judge closing distance.
Hook this into capture/strike success probability in the perception/combat system.

**3. Rear blind arc = ambush vulnerability**
```
rear_blind_angle = 360° - fov_angle
```
Narrow FOV → large rear blind arc → high surprise risk. This pushes prey toward width naturally.

With these three costs live, run:
- A predator lineage with wide FOV → misses strikes → culled → lineage drifts narrow
- A prey lineage with narrow FOV → gets ambushed from behind → culled → lineage drifts wide
- But a dragonfly-style wide-FOV ambush predator or a tarsier-style narrow-FOV arboreal prey remains *reachable* if the rest of the genome supports it

**Optional soft bias (not a clamp):** A mutation distribution mean can be nudged toward the trophic-appropriate band (carnivore prior = lower FOV mean; herbivore prior = higher FOV mean) without closing off the tails. This gives faster convergence while preserving the full space of viable archetypes.

### Floor correction
`MIN_FOV_DEGREES = 45.0` in `perception/constants.rs` is biologically wrong if interpreted as total anatomical FOV (no vertebrate reaches 45° total). The sim's FOV is an *attentional cone* (directional perception field), so 45° is defensible as an extreme specialist value — but treat anything below 90° as an exotic rare adaptation requiring high metabolic cost, not a common predator trait. Realistic carnivore predators cluster at 160–220°.

---

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
- Neighbor tracking (up to 7 neighbors)
- Perception range scales by body size: `body_length × 10.0`
- Spatial grid for efficient neighbor queries

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

## Implementation Phases

### Phase 1: DNA Vision Genes

**Outcome:** Three genes replace hardcoded values

**Tasks:**
- Add `visual_range_multiplier` gene (4.0-25.0, default 10.0)
- Add `visual_arc` gene (π/3 to 2π radians, default π)
- Add `neural_speed` gene (0.5-2.0, default 1.0)
- Wide FOV (>π) applies 0.7× range penalty
- Large creatures capped at lower max range

### Phase 2: VisionTiming Component

**Outcome:** Size-based reaction times determine update frequency per creature

**Tasks:**
- Create `VisionTiming` component with cooldown tracking
- Small creatures (0.5m): 68ms reaction time (~15 updates/sec)
- Large creatures (5m): 500ms reaction time (~2 updates/sec)
- Modified by `neural_speed` gene
- Uses manual cooldown pattern (like brain system)

### Phase 3: Field of View (FOV)

**Outcome:** Directional perception with blind spots

**Tasks:**
- Stationary creatures see 360° (no facing direction)
- Moving creatures use velocity vector as facing direction
- Dot product check filters entities outside FOV cone
- Simple implementation (complex blind spots deferred)

### Phase 4: Stochastic Vision Integration

**Outcome:** Only ~10% of creatures update perception per tick

**Tasks:**
- No round-robin scheduling - natural stagger from spawn time variation
- Brain reads potentially stale perception (biologically realistic)
- Automatic Poisson distribution from individual reaction times
- Compatible with spatial grid queries

---

## Performance Target

**Combined optimization:**
- Spatial grid: 833× fewer comparisons (algorithmic win)
- Stochastic vision: 10× fewer updates (frequency win)
- Combined: ~8,000× reduction from baseline O(N²) all-creatures-every-tick

**Target:** 200K creatures @ <45ms tick (with spatial grid)

---

## Success Criteria

- [ ] VisionTiming component with size-based reaction times (68ms-500ms)
- [ ] FOV filters blind spots correctly (stationary = 360°, moving = FOV cone)
- [ ] DNA genes (visual_range_multiplier, visual_arc, neural_speed) functional
- [ ] ~10% creatures update per tick at steady state
- [ ] Four archetypes (hawk, rabbit, owl, bison) spawn with correct phenotypes
- [ ] Wide FOV (>180°) applies 0.7× range penalty
- [ ] 200K creatures @ <45ms tick (with spatial grid)
- [ ] All existing tests pass (zero behavioral regression)

---

## Source

Zoologist-tom consultation, 2025-11-30
