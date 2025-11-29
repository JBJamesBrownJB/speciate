# Sprint 18: DNA-Driven Vision & Stochastic Perception

**Theme:** Replace hardcoded vision with DNA-driven variation, enabling ecological specialists (hawks vs rabbits) and 90% perception time reduction through stochastic updates

**Goal:** Implement three vision genes (range, FOV, neural speed) with biological trade-offs, plus reaction-time-gated perception updates to create realistic vision diversity and achieve 200K+ creature capacity.

**Prerequisites:** Sprint 16 complete (spatial grid working)

**Expected Duration:** 3 days

**Target Performance:** 200K creatures @ <45ms tick with spatial grid + stochastic vision

---

## High-Level Phases

### Phase 1: DNA Vision Genes
**Outcome:** Three genes replace hardcoded values - `visual_range_multiplier` (4-25), `visual_arc` (60°-360°), `neural_speed` (0.5-2.0)

**Key Decisions:**
- Hawks: long range (22×) + narrow FOV (90°) + fast reactions (1.3×)
- Rabbits: short range (6×) + wide FOV (300°) + very fast reactions (1.8×)
- Wide FOV (>180°) applies 0.7× range penalty (peripheral vision trade-off)
- Large creatures capped at lower max range (elephants can't have hawk eyes)

### Phase 2: VisionTiming Component
**Outcome:** Size-based reaction times (68ms-500ms) determine update frequency per creature

**Key Decisions:**
- Small creatures (0.5m): 68ms reaction time (~15 updates/sec)
- Large creatures (5m): 500ms reaction time (~2 updates/sec)
- Modified by `neural_speed` gene (fast thinkers react quicker)
- Uses manual cooldown pattern (like brain system)

### Phase 3: Field of View (FOV)
**Outcome:** Directional perception with blind spots - moving creatures only see forward hemisphere

**Key Decisions:**
- Stationary creatures see 360° (no facing direction)
- Moving creatures use velocity vector as facing direction
- Dot product check filters entities outside FOV cone
- Simple implementation (complex blind spots deferred)

### Phase 4: Stochastic Vision Integration
**Outcome:** Only ~10% of creatures update perception per tick, 90% use stale data

**Key Decisions:**
- No round-robin scheduling - updates naturally stagger from spawn time variation
- Brain reads potentially stale perception (biologically realistic - sensory lag exists)
- Automatic Poisson distribution from individual reaction times
- Compatible with spatial grid queries (Phase 3 operates on subset)

---

## Guidance Notes

### Biological Context (Zoologist-tom Consultation)

**Fundamental Law:** "You cannot maximize range, FOV, acuity, speed, AND low-light sensitivity simultaneously. Evolution produces specialists."

**Vision is expensive:** Retina is brain tissue. Visual processing consumes 20-25% of metabolic output in visual species.

**Trade-offs:**
- High range concentrates photoreceptors → limits FOV effectiveness → costs metabolism
- Wide FOV distributes photoreceptors thinly → reduces effective range (0.7× penalty)
- Fast neural processing → burns energy during active vision → prone to false positives

**Creature Archetypes:**
- **Hawk:** Range 22×, FOV 90°, Speed 1.3 → Apex predator, vulnerable while feeding
- **Rabbit:** Range 6×, FOV 300°, Speed 1.8 → Detects threats everywhere, prone to panic
- **Owl:** Range 14×, FOV 120°, Speed 0.7 → Patient ambush hunter, slow integration time
- **Bison:** Range 8×, FOV 270°, Speed 1.0 → Relies on herd early warning

### Technical Context

**Why Stochastic After Spatial Grid?**
- Spatial grid: 833× fewer comparisons (algorithmic win)
- Stochastic vision: 10× fewer updates (frequency win)
- Combined: ~8,000× reduction from baseline O(N²) all-creatures-every-tick

**Automatic Staggering:**
Creatures spawn at different ticks with different reaction times → natural Poisson distribution, no spiky CPU load.

### Gameplay Impact

**Ecological Niches Emerge:**
- Open terrain: Hawks dominate (extreme range)
- Dense cover: Rabbits thrive (wide FOV detects flanking)
- Night hunts: Owls excel (low-light adapted, patient)

**Player Strategy:**
- Domesticate fast-reacting creatures for early warning systems
- Breed long-range vision for scouting/surveillance roles
- Exploit predator blind spots (approach from behind)

**Metabolism Costs:**
- High range: +0.5% base metabolism per point above 10
- Fast neural speed: +3% active metabolism, +1% base per 0.1 above 1.0
- Wide FOV: +1% biomass birth cost per 30° above 180°

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
