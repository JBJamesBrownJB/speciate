# DNA Collector System - Genetic Engineering Gameplay

**Status:** ⏳ FUTURE FEATURE (not core simulation)

**Documentation:** `/docs/gameplay/dna-collector.md`

---

## Core Loop

Capture → Analyze → Manipulate → Clone

Players accelerate evolution through genetic engineering technology.

---

## Key Biological Principles

### DNA Degradation Post-Mortem

**Live capture:** 100% quality
**Fresh dead:** 60% quality (enzymatic degradation)
**Decomposed:** 20% quality (fragmented)

**Decay function:**
```rust
quality = base_quality × e^(-decay_rate × time_since_death)
```

Heat accelerates decay, cold preserves.

---

### Incremental Modification Limits

**5-15% per generation only**

**Biological rationale:**
- Large mutations are lethal (~99% harmful/neutral)
- Prevents "god mode" jumps
- Encourages gradual refinement

---

### Epistasis (Gene Interactions)

**Parameters are coupled, not independent:**

Size changes force metabolic recalculation:
```rust
metabolism ∝ size^0.75  // Kleiber's law
```

Speed modifications require structural changes (bone density, muscle ratio).

Vision improvements demand neural pathway adjustments.

**Cannot cherry-pick advantages without consequences.**

---

### Cloning Fidelity Degradation

**Clones of clones accumulate errors:**
```rust
fidelity = base_fidelity × (1 - clone_generation × degradation)
```

Somatic mutations, epigenetic drift, telomere shortening.

**Encourages returning to wild stock periodically.**

---

### Developmental Constraints

**Not all gene combinations are viable:**
- Head-to-body ratio limits (embryonically lethal)
- Organ scaling laws (heart must support body mass)
- Structural integrity (bone strength vs weight)

Failed combinations = lost biomass.

---

## Speciation Thresholds

**Gradient model, not binary:**

| Divergence | Status | Interbreeding |
|-----------|--------|---------------|
| 0-5% | Local variety | 100% |
| 5-15% | Subspecies | 80-100% (natural hybrids) |
| 15-30% | Species boundary | 20-80% (tech-assisted) |
| 30-50% | Genus boundary | 0-20% (extreme splicing) |
| 50%+ | Family boundary | 0% (impossible) |

**Speciation threshold:** 15-20% cumulative divergence

**Small variances (<5%) do NOT create new species**

---

## Hybrid Vigor vs Inbreeding Depression

```rust
fitness = 1.0 + (diversity_bonus × ln(unique_lineages))
          - (inbreeding_penalty × generations_cloned)
```

**Incentivizes:**
- Collecting diverse specimens ("trophy hunting")
- Refreshing gene pool from wild populations
- Avoiding repeated cloning of single lineage

---

## Cross-Breeding Mechanics

### Natural Cross-Breeding
Only works between closely related species (5-15% divergence):
- Same biome, overlapping territories
- Compatible reproductive systems
- Offspring may be sterile (mule pattern)

### Tech-Assisted Hybridization
"Hybridization Chamber" enables distant crosses:
- Higher failure rate for distant combinations
- Produces unique traits not found in either parent
- High biomass cost for failed attempts

---

## Environmental Gene Expression

**Same DNA produces different phenotypes:**
```rust
expressed_trait = gene_value × environment_modifier^0.3
```

**Examples:**
- Resource scarcity → smaller adult size
- Cold climate → thicker fur/blubber
- Predator presence → higher alertness

**Gameplay implication:** Must consider WHERE creatures will live, not just WHAT modifications.

---

## Cloning Bay Costs

### Biomass Requirements
```rust
biomass_cost = target_size × complexity_factor × (1 - efficiency)
```

Dead creatures provide raw building blocks.

### Energy Cost
```rust
energy_cost = biomass_required × complexity_factor × size^0.75
```

Scales with organism complexity.

### Development Time
```rust
maturation_time = base_time × size^0.25
```

Larger organisms take longer to mature.

**Strategic trade-off:** Small = quick but weak, Large = powerful but slow.

---

## Gameplay Motivation

### "NEW SPECIES DISCOVERED!" Reward

Unlocks at 15-20% divergence threshold:
- All player-created species are friendly/controllable
- Unique species unlock special abilities
- Permanent addition to DNA bank (pride of creation)
- Drives "trophy hunting" behavior

---

## Advanced Mechanics

### Progressive Understanding
- First capture: Sequence genome (raw data only)
- Observation time: Correlate genes to phenotypes
- Multiple specimens: Statistical confidence in gene-trait relationships
- Prevents instant "unlock all knowledge" exploit

### Horizontal Gene Transfer
- Gene vectors (engineered microorganisms)
- Transfer genes between unrelated species
- High-risk: Vectors may spread to wild populations
- High-reward: Novel trait combinations

---

## Design Principles

1. **Conservation of matter:** Biomass in = biomass out (minus inefficiency)
2. **Conservation of energy:** All processes cost energy ∝ complexity
3. **Incremental progress:** No instant "god creatures"
4. **Biological constraints:** Not all combinations viable
5. **Diversity incentive:** Hybrid vigor rewards collection, inbreeding punishes repetition
6. **Emergent complexity:** Advanced traits arise from primitive gene interactions
7. **Player-created species are friendly:** Incentivizes genetic experimentation

---

## Implementation Priority

**Future feature** (not core simulation).

All mechanics are biologically grounded and compatible with existing DNA-driven design principles.

**Full details:** `/docs/gameplay/dna-collector.md`
