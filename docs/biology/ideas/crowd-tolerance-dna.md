# DNA-Driven Crowding Tolerance

## Concept

Species-specific responses to crowding, encoded in DNA.

## Proposed DNA Traits

- `crowding_tolerance: f32` (0.0 = solitary, 1.0 = obligate schooler)
- `neighbor_limit: u8` (species-specific MAX_PERCEIVED_NEIGHBORS, 3-12)
- `social_delegation: f32` (0.0 = self-reliant, 1.0 = rely on group for threat detection)

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

## Source

Zoologist-tom consultation, 2025-11-30
