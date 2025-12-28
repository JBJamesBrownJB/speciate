# Realistic Size Distribution

**Status:** TODO
**Consulted:** zoologist-tom
**Golden Zone:** Yes - rarity emerges from biology, not arbitrary game balance

## Problem

Current DNA randomization treats all sizes as equally likely. In nature, giant creatures are rare while medium-sized creatures dominate.

## Solution: Log-Normal Distribution

Nature uses **log-normal distribution** because body size results from multiplicative growth processes.

### The Formula

```rust
size = e^(μ + σ × Z)

// Parameters for 1m median with realistic spread:
μ = 0.0    // ln(1.0) = median at 1m
σ = 0.7    // controls spread (higher = more variation)
Z = standard normal random (mean 0, std 1)
```

### Expected Distribution (μ=0, σ=0.7)

| Percentile | Size | Category |
|------------|------|----------|
| 5th | 0.32m | Tiny |
| 25th | 0.62m | Small |
| 50th | 1.0m | Medium |
| 75th | 1.6m | Large |
| 95th | 3.2m | Giant |
| 99th | 5.2m | Mega |

### Percentage Breakdown

| Size Range | Percentage |
|------------|------------|
| 0.1m - 0.5m | ~22% |
| 0.5m - 1.0m | ~28% |
| 1.0m - 2.0m | ~28% |
| 2.0m - 4.0m | ~16% |
| 4.0m+ | ~6% |

## Biological Rationale

### Why Giants Are Rare

1. **Metabolic constraints (Kleiber's Law):** Large organisms face exponentially increasing energy costs
2. **Ecological pyramid:** Energy transfer is ~10% efficient per trophic level; apex predators need enormous territories
3. **Reproductive strategy:** Large creatures have fewer offspring, slower population growth
4. **Evolutionary time:** Large body size takes longer to evolve, higher extinction risk

## Implementation

### Architecture

Create a single central spawning module:

```
apps/simulation/src/simulation/spawner.rs
```

This file handles ALL creature spawning and contains the biologically-sound randomization logic.

### Core Function

```rust
use rand_distr::{LogNormal, Distribution};

pub fn spawn_random(
    rng: &mut impl Rng,
    min_size: f32,
    max_size: f32,
) -> CreatureDna {
    let size = generate_size_log_normal(rng, 0.0, 0.7, min_size, max_size);
    // ... generate other DNA traits based on size ...
}

fn generate_size_log_normal(
    rng: &mut impl Rng,
    mu: f32,
    sigma: f32,
    min_size: f32,
    max_size: f32,
) -> f32 {
    let log_normal = LogNormal::new(mu, sigma).unwrap();

    // Rejection sampling for hard bounds
    loop {
        let size = log_normal.sample(rng);
        if size >= min_size && size <= max_size {
            return size;
        }
    }
}
```

### Dependencies

Add to `Cargo.toml`:
```toml
rand_distr = "0.4"
```

## Future Enhancements

### Niche-Specific Distributions

| Niche | μ | σ | Rationale |
|-------|---|---|-----------|
| Predator | 0.3 | 0.6 | Need to overpower prey |
| Prey | -0.2 | 0.8 | Many small + some too-big-to-hunt |
| Scavenger | 0.0 | 0.5 | Mobility + stomach capacity balance |
| Filter Feeder | 0.5 | 0.9 | No pursuit cost, abundant food |

### Biome Modifiers

| Biome | μ Modifier | Rationale |
|-------|-----------|-----------|
| Island | -0.3 | Island dwarfism (limited resources) |
| Open ocean | +0.3 | Gigantism (3D space, buoyancy) |
| Dense forest | -0.2 | Maneuverability advantage |
| Open plains | +0.1 | Size = safety from predators |

### Size-Correlated Traits

```rust
// Larger creatures live longer
lifespan = base_lifespan * size.powf(0.25)

// Larger creatures have fewer offspring
offspring_count = base_offspring / size.powf(0.5)
```

## Golden Zone Benefit

This distribution creates emergent rarity without explicit rarity mechanics:
- Giants being rare is a **consequence of biology**, not a game balance number
- Players intuitively understand "big creatures are special" because nature works that way
- Performance win: fewer giants = fewer expensive large-entity calculations
