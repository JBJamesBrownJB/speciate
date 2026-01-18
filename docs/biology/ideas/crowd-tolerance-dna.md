# DNA-Driven Crowding & Social Behavior

## Concept

Species-specific responses to crowding and social density, encoded in DNA. Combines stress tolerance (reactive) with spatial preference (proactive).

## Proposed DNA Traits

- `boldness: f32` (0.0 = seeks empty cells, 1.0 = seeks crowded cells) - **spatial preference**
- `crowding_tolerance: f32` (0.0 = panics at any density, 1.0 = calm in any density) - **stress threshold**
- `neighbor_limit: u8` (species-specific MAX_PERCEIVED_NEIGHBORS, 3-12)
- `social_delegation: f32` (0.0 = self-reliant, 1.0 = rely on group for threat detection)

### Boldness vs Crowding Tolerance (Distinct Concepts)

| Trait | Type | Controls |
|-------|------|----------|
| `boldness` | Proactive | WHERE creature chooses to go (wander target selection) |
| `crowding_tolerance` | Reactive | HOW creature responds once there (stress threshold) |

This creates four behavioral phenotypes:

| Boldness | Crowding Tolerance | Behavior |
|----------|-------------------|----------|
| High | High | Obligate schooler - seeks AND tolerates dense groups |
| High | Low | Social but stressed - seeks groups, gets overwhelmed, leaves, seeks again (unstable) |
| Low | High | Tolerant loner - avoids crowds but doesn't panic if crowded |
| Low | Low | True solitary - avoids crowds AND stressed if forced into them |

## Emergent Archetypes

| crowding_tolerance | social_delegation | Archetype |
|-------------------|-------------------|-----------|
| High | High | Schooling fish |
| High | Low | Herding prey (vigilance cycling) |
| Low | Low | Solitary predator |
| Low | High | N/A (biologically unrealistic) |

## Biological Rationale

- Schooling fish track 6-7 nearest neighbors
- Herding mammals rely on "many eyes" effect
- Solitary species actively avoid dense groups
- Pack predators maintain spacing during hunts

## Integration with Dynamic Range Reduction

The `crowding_tolerance` gene would modify the crowd pressure curve:
- High tolerance = less range reduction when crowded
- Low tolerance = more range reduction (stress response)

## Boldness-Driven Wander Target Selection

**Key insight:** Replace truly random wandering with density-preference-based movement.

When selecting wander targets, creatures evaluate L1 cell density and bias selection based on boldness:

```
cell_score = base_desirability
           + (boldness - 0.5) * normalized_density * social_weight
```

- `boldness = 0.5`: Neutral, density doesn't affect preference
- `boldness = 1.0`: Strong positive weight for high-density cells
- `boldness = 0.0`: Strong negative weight (prefers empty cells)

### Golden Zone Opportunities

| Optimization | Biological Behavior |
|--------------|---------------------|
| Timid creatures skip crowded L1 cells entirely | Shy animals show area avoidance, not individual avoidance |
| Bold creatures in groups skip individual threat detection | Social delegation - rely on group vigilance |
| Pre-filter candidate cells by density | Personality-based habitat selection |

### Emergent Behaviors to Expect

1. **Spatial segregation:** Bold creatures cluster, timid creatures scatter
2. **Aggregation waves:** Density fluctuations as bold creatures follow stale information
3. **Edge populations:** Timid creatures occupy periphery of resource patches
4. **Mating assortment:** Bold creatures encounter bold mates, creating heritable personality clusters

## Source

Zoologist-tom consultations, 2025-11-30 and 2025-12-29
