# Sensory Simplex - Fixed Sensory Budget

## Problem / Opportunity

Creatures need differentiated sensory capabilities to create ecological diversity and strategic player choices in genetic breeding. Without trade-offs, "maxed-out everything" becomes the dominant strategy. Real animals cannot excel at all senses simultaneously due to biological constraints.

## Proposed Solution

All creatures have a **fixed sensory budget** (e.g., 3.0 points) distributed across base senses:

- **Vision** - Visual acuity, detection distance, field of view
- **Hearing** - Sound sensitivity, frequency range, directional precision
- **Smell** - Chemical detection, concentration threshold, tracking range

The total budget is capped. Better hearing means worse vision or smell. This creates:

- **Specialists:** Vision-dominant predators, hearing-dominant prey, scent-tracking nocturnal hunters
- **Generalists:** Balanced distribution across all three senses
- **Ecological niches:** Different terrains favor different sensory allocations

### Sensory Simplex Visualization

```
        VISION
           ▲
          /·\
         /   \       · = creature's sensory allocation
        /  ·  \      Position shows specialization
       /       \
      ▼─────────▼
   HEARING    SMELL
```

Dot position represents the creature's sensory distribution. Evolution and player genetic modification shift the dot but cannot expand the triangle.

### Weighted Costs (Biological Accuracy)

Not all senses cost the same:

- **Vision:** 1.5x cost (retina, optic nerve, visual cortex expensive)
- **Hearing:** 1.0x cost (moderate - cochlea, auditory cortex)
- **Smell:** 0.7x cost (cheap - olfactory bulb is small)

Creates strategic depth: vision is powerful but expensive; smell is cheap but limited.

### Sub-Modalities Within Each Sense

Each sense has internal trade-offs:

**Vision:**
- Acuity (resolution) ↔ Field of View (FOV)
- Color vision ↔ Night vision
- Motion detection ↔ Static detail

**Hearing:**
- Frequency range (ultrasonic ↔ infrasonic)
- Directional precision ↔ Sensitivity
- Near-field ↔ Far-field

**Smell:**
- Sensitivity (faint concentrations) ↔ Range (distant sources)
- Discrimination (similar chemicals) ↔ Breadth (many chemicals)

## Golden Zone

Multiple performance optimizations that create emergent biological behavior:

### 1. Environment-Driven Sense Skipping

| Optimization | Biological Behavior |
|--------------|---------------------|
| Skip vision at night | Nocturnal creatures don't use eyes |
| Skip smell in high wind | Scent dispersal prevents tracking |
| Skip hearing underwater | Airborne hearing doesn't work submerged |

**Performance:** Creatures with low sensory weight in ineffective conditions skip expensive perception calculations.

### 2. Attention Bottleneck

Real animals attend to one sense deeply at a time. Process dominant sense fully; others get reduced resolution.

**Performance:** Only one full-resolution sensory scan per creature per tick.
**Biology:** Matches attentional focus in animal brains.

### 3. Automatic Nocturnal Evolution

Light level affects vision effectiveness. Creatures active at night evolve toward hearing/smell; day-active creatures toward vision.

**Performance:** Night eliminates vision processing load.
**Biology:** No explicit "nocturnal gene" - emerges from sensory allocation + activity timing.

### 4. Sensory Niche Partitioning

Different allocations thrive in different environments:
- Vision specialists → open terrain, daylight
- Hearing specialists → forests, night
- Smell specialists → tracking, underground

**Performance:** Creatures don't compete for same perception calculations.
**Biology:** Automatic niche separation prevents extinction.

## Trade-offs

### Costs

- **Complexity:** Three-way trade-off harder to balance than binary choice
- **Player confusion:** Requires education on why "all maxed" isn't possible
- **Implementation timing:** Requires sound and scent systems to exist before value is clear

### Benefits

- **Strategic breeding:** Player choices have meaningful consequences
- **Evolutionary pressure:** Environments naturally select sensory specialists
- **Performance scaling:** Skip unused senses = cheaper perception
- **Ecological diversity:** Automatic niche creation

## Expert Input

**Zoologist-tom consultation (2025-12-28):**

- **Neural real estate allocation** is the primary biological mechanism (cortical volume is zero-sum)
- **Real-world specialists:** Eagles (vision 4-8x human), bats (echolocation, vestigial eyes), sharks (300M olfactory receptors, poor vision)
- **Environmental effectiveness modifiers** create biome-specific evolution pressure (fog reduces vision, wind disperses scent, vegetation amplifies sound)
- **Sensory synergy bonuses** for balanced allocations prevent extreme specialists always dominating
- **Weighted costs** more accurate than equal points (vision is metabolically expensive)

## Dependencies

- **Sound perception system** (future - mating calls, predator vocalizations)
- **Scent/chemical system** (future - pheromones, tracking)
- **DNA gene expression** (exists - basic DNA system complete)
- **Drive simplex** (Phase B - in progress) - consumes sensory input as drive contributions

## Related Ideas

- `dna-driven-design.md` - DNA architecture and gene expression patterns
- `fov-perception.md`, `dna-driven-fov.md` - Vision sub-modalities (acuity vs FOV trade-off)
- `chemical-scent.md` - Smell/scent tracking system
- `mating-calls.md`, `flocking-calls.md` - Hearing/sound systems
- `seismic-awareness.md` - Fourth sense candidate (seismic vibration)
- `drive-simplex.md` - Drive system architecture that integrates multi-sensory input
- `stress-tunnel-vision.md` - Dynamic vision narrowing (state modifier)
- `energy-vigilance.md` - Energy affecting perception range (state modifier)

## Open Questions

- **Should sensory budget SIZE vary by creature?** (Some have 3.0, others 4.0 but higher energy cost)
- **Environmental effectiveness multipliers:** How strongly should fog/wind/vegetation affect senses?
- **Synergy bonuses:** How much bonus for balanced allocations to prevent extreme specialists?
- **Player genetic modification:** Direct slider control or indirect selection through breeding?
- **Special senses:** Should echolocation/infrared/electroreception add to budget or replace base senses?

## Implementation Notes

**Timing:** After sound and scent systems exist (much later than ABC Super Sprint).

**Evolution mechanism:** Natural selection + player genetic modification technology.

**DNA structure:**
- Three genes: `vision_weight`, `hearing_weight`, `smell_weight`
- Normalized to sum to `SENSORY_BUDGET` with weighted costs
- Heritable through reproduction
- Sub-modality genes for acuity/FOV, frequency range, etc.

**Perception integration:**
- Skip senses below threshold (e.g., 0.3) for performance
- Apply environmental effectiveness modifiers
- Feed into drive simplex as sensory contributions

---

*Captured: 2025-12-28*
*Zoologist consultation: a698ae5*
