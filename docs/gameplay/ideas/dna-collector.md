# DNA Collector System

## Core Loop

Capture → Analyze → Manipulate → Clone

Evolution happens naturally in the game, but players can accelerate it through genetic engineering technology.

---

## Technology Progression

1. **Traps/Weapons** - Capture live creatures
2. **DNA Bank** - Store and analyze genetic profiles
3. **DNA Manipulator** - Modify genes within biological constraints
4. **Cloning Bay** - Produce modified creatures from biomass

---

## DNA Collection

### Live Capture (100% quality)
- Full genome sequencing
- Requires observation time to understand gene functions
- Multiple specimens needed for statistical confidence in gene-trait correlations

### Fresh Dead (60% quality)
- Enzymatic degradation begins
- Partial genome with gaps
- Limited manipulation options

### Decomposed (20% quality)
- Heavily fragmented DNA
- Minimal research value
- Cannot be cloned

**DNA Quality Decay:**
```
quality = base_quality × e^(-decay_rate × time_since_death)
```
Decay accelerates in heat, slows in cold.

---

## DNA Manipulation

### Parameter Modification Limits

**Incremental Changes Only:** 5-15% per generation
- Reflects biological reality: most large mutations are lethal
- Prevents "god mode" jumps to perfect designs
- Encourages gradual refinement

### Epistasis (Gene Interactions)

**Parameters are coupled, not independent:**
- Size changes force metabolic recalculation (Kleiber's law: metabolism ∝ mass^0.75)
- Speed modifications require structural changes (bone density, muscle ratio)
- Vision improvements demand neural pathway adjustments

Cannot cherry-pick advantages without biological consequences.

### Developmental Constraints

**Not all gene combinations are viable:**
- Head-to-body ratio limits (embryonically lethal if violated)
- Organ scaling laws (heart size must support body mass)
- Structural integrity (bone strength vs weight)

Failed combinations result in non-viable embryos (lost biomass).

---

## Cloning Bay

### Resource Requirements

**Biomass:** Dead creatures provide raw building blocks
```
biomass_cost = target_size × complexity_factor × (1 - efficiency)
```

**Energy Cost:** Scales with organism complexity
```
energy_cost = biomass_required × complexity_factor × size^0.75
```

### Development Time

Larger organisms take longer to mature:
```
maturation_time = base_time × size^0.25
```

**Strategic trade-off:** Small creatures are quick but weak; large creatures are powerful but slow to produce.

### Cloning Fidelity

**Clones accumulate errors:**
```
fidelity = base_fidelity × (1 - clone_generation × degradation)
```

Cloning clones reduces viability (somatic mutations, epigenetic drift). Encourages returning to wild stock periodically.

---

## Hybridization & Cross-Breeding

### Natural Cross-Breeding

**Only works between closely related species (5-15% divergence):**
- Same biome, overlapping territories
- Compatible reproductive systems
- Offspring may be sterile (mule pattern)

### Tech-Assisted Hybridization

**Advanced gene-splicing tech enables distant crosses:**
- "Hybridization Chamber" - Combines DNA from unrelated species
- Higher failure rate for distant combinations
- Produces unique traits not found in either parent
- Visual: Load giant predator in one cage, tiny herbivore in another, use genetic "fusion ray"

**Risk/Reward:**
- High biomass cost for failed attempts
- Successful hybrids unlock novel trait combinations
- Unpredictable outcomes (emergent gene interactions)

---

## Genetic Diversity Mechanics

### Hybrid Vigor vs Inbreeding Depression

**Track genetic diversity in your stock:**
```
fitness = 1.0 + (diversity_bonus × ln(unique_lineages))
          - (inbreeding_penalty × generations_cloned)
```

**Gameplay implications:**
- Diverse gene pool → healthier, more adaptable creatures
- Repeated cloning → accumulates deleterious recessive alleles
- Incentivizes "trophy hunting" for diverse specimens

---

## Speciation

### Cumulative Divergence Threshold

Speciation is a **gradient**, not a binary switch:

| Divergence | % Difference | Status | Interbreeding |
|-----------|-------------|---------|---------------|
| 0-5% | Local variety | Same population | 100% |
| 5-15% | Subspecies | Natural hybrids possible | 80-100% |
| 15-30% | Species boundary | Tech-assisted hybrids only | 20-80% |
| 30-50% | Genus boundary | Extreme gene-splicing required | 0-20% |
| 50%+ | Family boundary | Cannot combine | 0% |

**Speciation threshold:** 15-20% cumulative divergence
**Small DNA variances (<5%) do NOT create new species**

### Speciation Rewards

**"NEW SPECIES DISCOVERED!" notification:**
- All player-created species are friendly/controllable
- Unique species unlock special abilities
- Permanent addition to DNA bank (pride of creation)
- **Gameplay drive:** Hunting for that winning "new species" feeling

Unlocks unique species tag and branching evolutionary paths.

---

## Environmental Gene Expression

**Same DNA produces different phenotypes in different environments:**
```rust
expressed_trait = gene_value × environment_modifier^0.3
```

**Examples:**
- Resource scarcity → smaller adult size despite identical DNA
- Cold climate → thicker fur/blubber expression
- Predator presence → higher alertness behavior

**Gameplay:** Must consider WHERE modified creatures will live, not just WHAT modifications to make.

---

## Advanced Mechanics (Optional)

### Horizontal Gene Transfer
- Engineer gene vectors (microorganisms)
- Transfer genes between unrelated species
- Risk: Vectors may spread uncontrollably to wild populations

### Progressive Understanding
- First capture: Sequence genome (raw data)
- Observation: Correlate genes to phenotypes
- Research: Unlock manipulation precision

### Manipulator Tiers

| Tier | Precision | Max Change/Gen | Viability | Epistasis-Aware |
|------|-----------|---------------|-----------|-----------------|
| Basic | 5% | 5% | 50% | No |
| Advanced | 1% | 15% | 85% | Partial |
| Precision | 0.1% | 20% | 99% | Full |

---

## Resources Summary

**DNA Profiles:** Genetic blueprints (decay over time if not preserved)
**Biomass:** Raw matter for cloning (from dead creatures)
**Energy:** Powers cloning process (scales with complexity)
**Research Time:** Understanding gene functions (requires observation)

---

## Design Principles

1. **Conservation of matter:** Biomass in = biomass out (minus inefficiency)
2. **Conservation of energy:** All processes cost energy proportional to complexity
3. **Incremental progress:** No instant "god creatures"
4. **Biological constraints:** Not all combinations are viable
5. **Diversity incentive:** Hybrid vigor rewards collection, inbreeding punishes repetition
6. **Emergent complexity:** Advanced traits arise from primitive gene interactions
7. **Player-created species are friendly:** Incentivizes genetic experimentation
