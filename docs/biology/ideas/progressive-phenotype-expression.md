# Progressive Phenotype Expression - Generational Visual Evolution

## Problem / Opportunity

Players observing a simulation need visible feedback that evolution is *working*. Continuous trait sliders and behind-the-scenes fitness improvements are invisible. Players can't see that their ecosystem management is succeeding without explicit metrics.

**Opportunity:** Make evolutionary success *visible* through progressive physical elaboration - successful lineages literally look more interesting over time.

## Proposed Solution

**Generational Complexity Unlocks:** Each successful reproduction event increments a "lineage complexity" counter. As this counter crosses thresholds, creatures unlock:

### Tier 1: Superficial Details (Cosmetic)

Visual elaborations with no gameplay effect - pure cosmetic rewards:

| Detail | Description | Unlock Threshold |
|--------|-------------|------------------|
| Stripes | Body pattern variations | 5 generations |
| Spots | Scattered markings | 8 generations |
| Antennae | Small protrusions | 12 generations |
| Horns (decorative) | Non-functional display | 15 generations |
| Crests | Head/back ridges | 20 generations |
| Bioluminescent spots | Glowing accents | 25 generations |
| Elaborate tail | Extended/ornate tail | 30 generations |

**Key:** These are heritable - offspring inherit parent's visual complexity tier.

### Tier 2: Biome Adaptations (Functional)

Occasional functional traits tied to the biome where the lineage has thrived:

| Biome | Adaptation | Effect | Unlock Chance |
|-------|------------|--------|---------------|
| Desert | Sand camouflage | -30% visibility in desert | 5% per generation in biome |
| Swamp | Swamp camouflage | -30% visibility in swamp | 5% per generation in biome |
| Forest | Dappled coat | -25% visibility in forest | 5% per generation in biome |
| Tundra | White coat | -35% visibility in snow | 5% per generation in biome |
| Coral | Reef patterns | -25% visibility in coral | 5% per generation in biome |
| Cave | Pale coloration | -40% visibility in darkness | 5% per generation in biome |

**Key:** Biome adaptations are *localized* - creatures lose effectiveness when leaving their adapted biome.

### Tier 3: Advanced Adaptations (Rare, Functional)

Very rare unlocks for extremely successful lineages (50+ generations in same biome):

- **Countershading:** Reduces visibility from all angles
- **Disruptive coloration:** Breaks silhouette recognition
- **Thermal regulation markings:** Better energy efficiency in biome
- **Mimicry patterns:** Resembles dangerous species (predator deterrent)

## Progression Mechanics

### Lineage Complexity Counter

```
Reproduction event
       ↓
Parent lineage_complexity + 1 → Offspring lineage_complexity
       ↓
Check threshold crossings → Unlock visual tier
       ↓
Check biome residence → Roll for biome adaptation
```

### Inheritance Rules

- **Cosmetic tier:** Always inherited (max of both parents)
- **Biome adaptations:** 80% inheritance chance, can lose if offspring lives in different biome for 10+ generations
- **Cross-biome breeding:** Offspring has 50% chance of each parent's biome adaptation

### Visibility Reduction

Biome adaptations reduce the creature's "visibility score" used in perception calculations:

```rust
fn calculate_visibility(creature: &Creature, observer_position: Vec2) -> f32 {
    let base_visibility = creature.size * SIZE_VISIBILITY_FACTOR;

    // Check if creature has biome adaptation matching current biome
    if let Some(adaptation) = creature.biome_adaptation {
        if adaptation.biome == get_biome_at(creature.position) {
            return base_visibility * adaptation.visibility_modifier;
        }
    }

    base_visibility
}
```

## Visual Progression Examples

**Generation 0:** Basic blob shape, solid color
**Generation 5:** Faint stripe pattern emerges
**Generation 15:** Defined stripes + small horn nubs + (maybe) desert camouflage
**Generation 30:** Elaborate striping + prominent horns + crest + confirmed biome adaptation
**Generation 50+:** Maximally elaborated + rare advanced adaptation

## Player Experience

### Reward for Ecosystem Management

A well-managed ecosystem produces stable lineages that survive many generations:

- **Poor management:** Constant extinction, creatures stay basic/blobby
- **Good management:** Lineages flourish, world fills with elaborate, adapted creatures
- **The visual feedback:** "My world looks interesting" = "I'm playing well"

### Reward for Playtime

Natural progression over extended play:

- **Hour 1:** Simple creatures, minimal variation
- **Hour 10:** Some lineages showing stripes, first biome adaptations appearing
- **Hour 50:** Diverse population with highly elaborated, locally-adapted creatures

### Emergent Storytelling

Players can identify successful lineages visually:

- "That striped family has been in the desert for ages - they're almost invisible now"
- "The forest creatures all have dappled coats but the newcomers from the plains stick out"
- "That lineage has the most elaborate horns I've seen - they've been thriving"

## Golden Zone

### 1. Skip Perception for Adapted + Stationary

Creatures with biome camouflage AND stationary in their biome = skip adding to perception candidate list entirely.

**Biology:** Perfectly camouflaged stationary prey is invisible.
**Performance:** Reduces perception workload in stable ecosystems.

### 2. Visibility Threshold Culling

Creatures below visibility threshold at observer distance = skip individual perception check.

**Biology:** Small, camouflaged creatures at distance are genuinely invisible.
**Performance:** Fewer perception calculations for adapted populations.

### 3. Lineage Counter as Cheap Fitness Proxy

Lineage complexity counter is a O(1) lookup for "is this creature from a successful line?"

**Biology:** Successful lineages are better adapted (tautologically).
**Performance:** Game director can use counter for spawning decisions without fitness recalculation.

## Trade-offs

### Costs

- **Visual content:** Need procedural or asset variations for each cosmetic tier
- **Biome system dependency:** Requires biome detection at creature positions
- **Complexity tracking:** Additional component on each creature

### Benefits

- **Visible evolution:** Players SEE natural selection working
- **Organic reward loop:** Good play → pretty creatures → satisfaction
- **Emergent narrative:** Lineage history visible in creature appearance
- **Performance wins:** Adapted creatures = fewer perception calculations

## DNA Integration

### Lineage Complexity Gene

```rust
pub struct LineageGene {
    pub complexity_counter: u16,      // Increments each generation
    pub cosmetic_tier: u8,            // Current visual elaboration tier
    pub biome_adaptation: Option<BiomeAdaptation>,
}
```

### Biome Adaptation Structure

```rust
pub struct BiomeAdaptation {
    pub biome: BiomeType,
    pub visibility_modifier: f32,     // 0.6 = 40% reduction
    pub generations_in_biome: u16,    // Tracking for advanced adaptations
}
```

## Dependencies

- **Biome system** - Must know what biome a creature is in
- **Perception system** - Must integrate visibility modifiers
- **Visual rendering** - Must support procedural detail tiers
- **DNA system** - Must store lineage complexity

## Related Ideas

- `special-traits.md` - Binary trait unlocks (complementary system)
- `dna-driven-design.md` - DNA architecture
- `game-director.md` - Could boost complexity in struggling populations
- `procedural-gait-synthesis.md` - Visual elaboration rendering
- `motion-detection.md` - Camouflage interaction with movement

## Open Questions

- **Complexity cap:** Is there a maximum tier, or infinite elaboration?
- **Complexity decay:** Should lineages lose complexity after failed reproductions?
- **Visual generation:** Procedural shader vs. layered sprites vs. modular assets?
- **Multi-biome creatures:** How to handle creatures that migrate between biomes?
- **Predator adaptations:** Should predators get biome hunting bonuses instead of camouflage?

---

*Captured: 2026-01-14*
