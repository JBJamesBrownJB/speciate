# Special Traits System - Unlockable Discrete Features

## Problem / Opportunity

Players need "prize creatures" to hunt for - rare finds that reward exploration and drive engagement. Continuous trait sliders (sensory budget, size) don't create the same excitement as discovering a creature with a visually distinct, powerful ability.

## Proposed Solution

**Special traits** are binary features distinct from base traits:

- **Binary:** A creature has tusks or doesn't (no "slightly tusked")
- **Rare:** Emerge through mutation, controlled by game-director
- **Visually distinct:** Player can spot them
- **Unlockable:** Once discovered, available for player breeding programs

### Trait Categories

| Category | Examples | General Trade-off |
|----------|----------|-------------------|
| **Defense** | Scales/armor, spines, thick hide, shell | Mass increase, speed penalty |
| **Offense** | Tusks, horns, claws, venom glands | Energy cost, specialized diet/delivery |
| **Escape** | Ink sack, autotomy (drop tail), speed burst | Resource depletion, recovery period |
| **Camouflage** | Chromatophores (color change), transparency | Processing cost, structural weakness |
| **Social** | Bioluminescence, pheromone glands, display structures | Reveals position, energy cost |
| **Special Senses** | Echolocation, electroreception, infrared pits | See Idea 3: Special Senses |

### Trait Examples with Trade-offs

**Defense:**
- **Armor/Scales:** Damage reduction, but +30% mass, reduced speed, molting vulnerability
- **Shell:** Near-immunity to crushing, but massive weight, can't right self if flipped
- **Spines:** Predator deterrent, but snag vegetation, hydrodynamic drag

**Offense:**
- **Venom:** One-hit subdual, but 5-10% metabolic cost, requires precise delivery
- **Tusks:** Combat + digging, but continuous growth cost, infection risk if broken
- **Horns:** Combat + display, but skull reinforcement mass, neck muscle requirement

**Escape:**
- **Ink Sack:** Instant visual cover, but limited uses, useless in clear water
- **Autotomy:** Escape from grip, but tissue loss, weeks to regenerate
- **Speed Burst:** Explosive acceleration, but lactate buildup, recovery immobility

**Camouflage:**
- **Chromatophores:** Active color change, but slower movement during match, limited palette
- **Transparency:** Passive invisibility, but no UV protection, organs visible to parasites

**Social:**
- **Bioluminescence:** Mate attraction + lure prey, but reveals position, ATP cost per flash
- **Pheromones:** Silent long-range communication, but scent trail attracts predators

### Trait Incompatibilities

**Hard Incompatibilities (Mutually Exclusive):**

| Trait A | Trait B | Why |
|---------|---------|-----|
| Transparency | Bioluminescence | Light organs visible as glowing blobs |
| Transparency | Chromatophores | Pigment cells block transparency |
| Heavy armor | Speed burst | Mass ratio prevents burst |
| Thick hide | Chromatophores | Color change requires thin flexible skin |
| Autotomy | Armor (on limb) | Can't have fracture planes in rigid structures |
| Echolocation | Stealth movement | Echolocation broadcasts position |

**Soft Incompatibilities (Stacking Penalties):**
- Multiple defense traits compound weight costs
- Display structures contradict camouflage strategy
- Venom + crushing jaws (delivery system damage)

## Golden Zone

### 1. Armored Creatures Skip Damage Calculations

Predators can't hurt heavily armored prey below size threshold - skip attack calculations entirely.

**Biology:** Predators learn to avoid armored prey (handling time too high).

### 2. Venomous Creatures Skip Extended Combat

Single successful hit ends encounter - no extended damage loop.

**Biology:** Venom ends fights quickly. Snakes strike and wait.

### 3. Camouflaged + Stationary = Skip Perception

Creatures with active camouflage matching environment AND stationary are not added to perception candidate lists.

**Biology:** Stationary octopus on matching substrate is genuinely invisible.

### 4. Bioluminescence as Beacon

Bioluminescent creatures register as "beacons" - nearby creatures receive notification instead of searching.

**Biology:** Light sources detected passively, not actively searched.

### 5. Echolocation Simplified Perception

Spherical range check instead of vision cone + occlusion raycast.

**Biology:** Echolocation penetrates obstacles, uniform range, simpler than vision.

## Trade-offs

### Costs
- **Significant content:** Each trait needs visual representation, behavior integration
- **Balance complexity:** Stacking rules, incompatibilities need careful design
- **Rarity tuning:** Too rare = frustrating; too common = not special

### Benefits
- **Core engagement loop:** Hunt for rare creatures drives exploration
- **Strategic breeding:** Meaningful choices in genetic programs
- **Visual diversity:** Creatures look distinct and memorable
- **Performance wins:** Golden Zone optimizations create emergent biology

## Expert Input

**Zoologist-tom consultation (2025-12-28):**

- **Genuinely binary traits** in nature: venom, bioluminescence, electric organs, echolocation, ink sacks, autotomy
- **Traits that seem binary but are continuous:** camouflage patterns, scales, claws, thick hide - treat as threshold unlocks
- **Rarity by genetic complexity:** Single gene = common, 2-3 genes = uncommon, novel pathway = rare, organ system = very rare
- **Suggested additions:** Electroreception (counters camouflage), infrared pits (thermal vision), regeneration, hibernation, parthenogenesis, mimicry, filter feeding

**Rarity Tiers Suggested:**

| Tier | Rate | Traits |
|------|------|--------|
| Common | 1-5% | Enhanced coloration, thicker hide |
| Uncommon | 0.1-1% | Spines, basic horns, autotomy |
| Rare | 0.01-0.1% | Venom, ink sack, chromatophores |
| Legendary | <0.01% | Electric organs, echolocation, transparency |

## Dependencies

- **Basic DNA system** (exists) - gene expression framework
- **Visual creature rendering** (exists) - must support trait visuals
- **Game director** (idea) - controls spawn rates for rare traits
- **Capture/collection system** (future) - player acquisition mechanic

## Related Ideas

- `ink-sacks.md` - Detailed design for ink sack trait
- `dna-driven-design.md` - DNA architecture and gene expression
- `sensory-simplex.md` - Base senses (special senses are separate)
- `motion-detection.md` - Camouflage counter (stationary = invisible)
- `taming-system.md` - Related player mechanic for creature collection
- `game-director.md` - Controls rare trait spawn rates

## Open Questions

- **Trait stacking limits:** Unlimited? Slot-based (3-5 max)? Body-part exclusive? Energy cost scaling?
- **Unlock methods:** Capture alive? DNA sample? Research/observation? Multiple tech trees?
- **Visual representation:** How to render each trait distinctly? Procedural or asset-based?
- **DNA encoding:** Binary genes? Threshold on continuous genes? Prerequisite chains?
- **Inheritance:** Dominant/recessive? Always pass if parent has it? Mutation chance to lose?

## Implementation Priority (Suggested)

| Phase | Traits | Rationale |
|-------|--------|-----------|
| **Phase 1** | Armor, Venom, Autotomy | Simple mechanics, immediate impact |
| **Phase 2** | Bioluminescence, Chromatophores, Ink | Visual distinctiveness, counters |
| **Phase 3** | Echolocation, Electroreception | Alternative perception modes |
| **Phase 4** | Social traits, Mimicry | Requires multi-creature coordination |

---

*Captured: 2025-12-28*
*Zoologist consultation: ae94a6c*
