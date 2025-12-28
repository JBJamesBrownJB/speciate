# Population Genetics Algorithm

## Problem / Opportunity

Individual-based simulation has a hard performance ceiling. Even with optimizations, simulating thousands of creatures making decisions at 1000x+ speed is computationally impossible in real-time.

But evolution doesn't require tracking individuals - it's fundamentally about **changes in allele frequencies over generations**. Population genetics has modeled this mathematically since the 1920s.

This algorithm enables:
- **Hyper-speed evolution** (10,000x+) without CPU explosion
- **Seamless speed transitions** (real sim ↔ statistical model)
- **Reusable engine** for multiple gameplay features (Time Bubble, Fast-Forward, etc.)

## Proposed Solution

A standalone evolution engine that models populations statistically rather than individually.

### Core Model

**Population State** (what we track):
```
- population_size: N
- allele_frequencies: { trait_gene: frequency, ... }
- trait_distributions: { body_size: (mean, variance), speed: (mean, variance), ... }
- generation_count: G
- fitness_landscape: environment-dependent selection pressures
```

**Per-Generation Update** (the algorithm):

1. **Selection** - fitter alleles increase in frequency
   - Fitness determined by environment (food availability, predation pressure, etc.)
   - Frequency shifts proportional to relative fitness

2. **Genetic Drift** - random sampling in finite populations
   - Small populations: large random swings
   - Large populations: stable frequencies
   - Models founder effects, bottlenecks

3. **Mutation** - rare random changes
   - Low probability per allele per generation
   - Source of new variation

4. **Population Dynamics** - birth/death rates
   - Carrying capacity limits
   - Boom-bust cycles from resource availability

**Output:** Evolved gene pool after N generations, ready to instantiate real creatures.

### Seamless Speed Transitions

The key insight: **blend between real simulation and statistical model based on speed**.

| Speed Range | Mode | Visual | What's Happening |
|-------------|------|--------|------------------|
| 1x - 100x | Real Simulation | See creatures moving fast | Actual entities, physics, behavior |
| 100x - 500x | Hybrid Zone | Motion blur, trails | Real sim but rendering simplified |
| 500x+ | Population Genetics | Shimmering blur, stats overlay | Statistical model, no individuals |

**Transition In (speeding up past threshold):**
1. Capture current population's DNA into gene pool statistics
2. Despawn individual entities (or hide/freeze them)
3. Switch to statistical updates
4. Visual: blur effect, particle swirl, stats overlay appears

**Transition Out (slowing down below threshold):**
1. Sample N creatures from evolved gene pool distribution
2. Spawn real entities with sampled DNA at random positions within region
3. Resume real simulation (still fast, but decelerating)
4. Visual: blur clears, creatures "crystallize" out of the swirl

**Player Experience:**
Speed up → creatures blur → swirling particles with stats → stats changing rapidly → slow down → creatures emerge → see evolved population

### Reusability

This algorithm is a **service** that multiple features can call:

| Feature | How It Uses Pop Gen Algorithm |
|---------|-------------------------------|
| **Time Bubble** | Bubble beyond threshold X switches to pop gen mode |
| **Fast-Forward Game Start** | Seed 5K creatures, pop gen for 1000 generations, reseed world |
| **Extinction Recovery** | Director seeds refugia, runs pop gen briefly, spawns survivors |
| **Hibernation Caves** | Creatures enter cave, pop gen runs while player away, emerge evolved |

### Algorithm Details

**Selection Equation:**
```
p' = p × w / w̄
where:
  p = current allele frequency
  w = fitness of allele carriers
  w̄ = mean population fitness
  p' = next generation frequency
```

**Genetic Drift (Wright-Fisher):**
```
p' ~ Binomial(2N, p) / 2N
where:
  N = population size
  p = current frequency
  Smaller N = more drift
```

**Mutation:**
```
p' = p × (1 - μ) + (1 - p) × μ
where:
  μ = mutation rate (~0.001 per gene per generation)
```

**Population Growth:**
```
N' = N × r × (1 - N/K)
where:
  r = growth rate (from mean reproduction genes)
  K = carrying capacity (from environment)
```

## Golden Zone

**This IS the Golden Zone** - the entire concept is an optimization that creates biological behavior:

| Optimization | Biological Accuracy |
|--------------|---------------------|
| O(1) computation regardless of population | Models real population genetics |
| Skip individual behavior simulation | Evolution IS statistical, not individual |
| Aggregate trait tracking | How biologists actually study evolution |
| Sampling on exit | Represents genetic variation in population |

The statistical model isn't a shortcut - it's actually a more accurate representation of evolutionary dynamics than individual-based simulation at large scales.

## Trade-offs

| Benefit | Cost |
|---------|------|
| Unlimited speed (10,000x+) | No individual creature identity preserved |
| O(1) computation | Can't watch individual behaviors |
| Biologically accurate for large N | Less accurate for tiny populations (N < 30) |
| Reusable across features | Requires transition logic for seamless blending |
| Elegant visual (blur → stats → emerge) | Players don't see "the action" during hyper-speed |

### When NOT to Use

- Population < 30 (drift equations become inaccurate)
- Player wants to watch specific individuals
- Behavioral evolution matters (learned behaviors, not just genetics)
- Short time spans (< 10 generations - just use real sim)

## Expert Input

**Zoologist Consultation (2025-12-28):**

Population genetics models (Wright-Fisher, Hardy-Weinberg) are the standard tool for modeling evolution at population scale. They're well-validated across decades of research.

Key phenomena that emerge correctly:
- Genetic drift (small populations diverge randomly)
- Selection response (adaptation to environment)
- Founder effects (rare alleles can dominate after bottleneck)
- Mutation-selection balance

Accuracy: Excellent for N > 50, good for N > 30, questionable below that.

Recommendation: For tiny populations, use simplified individual sim or warn player about "unstable genetics."

## Dependencies

- DNA system with discrete genes/alleles (already exists)
- Environment model for fitness landscape (food distribution, predation pressure)
- Creature spawning system (to instantiate from gene pool)
- Visual effects for blur transition (particle system, stats overlay)

## Related Ideas

- `docs/gameplay/ideas/time-bubble.md` - Uses pop gen for speeds beyond threshold
- `docs/gameplay/ideas/fast-forward-game-start.md` - Uses pop gen for rapid world seeding
- `docs/biology/ideas/game-director.md` - Could use pop gen for extinction recovery
- `docs/biology/ideas/dna-driven-design.md` - DNA system that pop gen operates on

## Open Questions

- What's the optimal threshold speed for transition? (100x? 500x?)
- How smooth should the visual transition be? (instant vs 2-second blend)
- Should players see generation counter during pop gen mode?
- How do we handle multiple species in same region? (separate gene pools)
- What fitness landscape parameters matter most? (food, predation, temperature?)
- Should there be a "fast forward N generations" button that uses this?

---
*Captured: 2025-12-28*
