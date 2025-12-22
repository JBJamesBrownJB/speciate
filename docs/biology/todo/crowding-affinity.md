# Crowding Affinity DNA Gene

## Status: DEFERRED

Identified during Phase A planning. DNA-driven social behavior spectrum.

---

## Concept

A DNA gene that controls how creatures respond to CROWDED L1 cells (cells with visible mass but no threat/prey).

```rust
pub crowding_affinity: f32,  // -1.0 (avoid crowds) to +1.0 (seek crowds)
```

## Biological Basis

Animals exist on a spectrum from solitary to highly social:

| Value | Strategy | Real Examples |
|-------|----------|---------------|
| -0.8 to -0.3 | Solitary | Tigers, bears, octopi, wolverines |
| -0.3 to +0.3 | Neutral | Most mid-sized mammals, deer |
| +0.3 to +0.8 | Social | Wolves, elephants, herding ungulates |
| +0.8 to +1.0 | Swarm | Schooling fish, starlings, locusts |

## Implementation

When L1 cell is classified as CROWDED:

```rust
let crowding_drive = if l1_cell.classification == L1Classification::Crowded {
    cell_direction * creature.dna.crowding_affinity
} else {
    Vec2::ZERO
};
// Negative affinity = repulsion from crowd
// Positive affinity = attraction to crowd
```

## Emergent Behaviors

### Solitary Creatures (negative affinity)
- Territorial spacing
- Predators avoid each other's hunting grounds
- Even distribution across landscape

### Social Creatures (positive affinity)
- Herd formation
- Safety in numbers
- Coordinated escape (stampede)
- Schooling/flocking patterns

### Mixed Population
- Predators spread out (solitary)
- Prey clumps together (social)
- Creates interesting spatial dynamics
- Predators "prowl the edges" of herds

## Entertainment Value: Very High

Players observe:
- Different "species" with recognizable social structures
- Herds forming and moving as units
- Lone predators stalking herd edges
- Territorial disputes between solitary creatures

## DNA Integration

### Gene Definition

```rust
// In DNA component
pub struct DNA {
    // ... existing genes ...
    pub crowding_affinity: f32,  // -1.0 to +1.0
}
```

### Expression

Could be derived from other genes:
- High aggression + large size → solitary (predator spacing)
- Low aggression + small size → social (safety in numbers)
- Or encoded directly for more control

### Mutation

```rust
// During reproduction
child.crowding_affinity = lerp(
    parent_a.crowding_affinity,
    parent_b.crowding_affinity,
    0.5
) + random_mutation(-0.1, 0.1);
```

## Interaction with Other Systems

### Phase B Drives
Crowding affinity feeds directly into drive calculation:
- CROWDED cell + positive affinity = attraction drive
- CROWDED cell + negative affinity = repulsion drive

### Predator-Prey Dynamics
- Predators evolve solitary (avoid competition)
- Prey evolves social (safety in numbers)
- Creates natural predator-prey spatial patterns

### Carrying Capacity
- Social creatures clump → local resource depletion
- Solitary creatures spread → even resource usage
- Affects population dynamics

## Dependencies

- L1 cell classification (Phase A)
- Drive system (Phase B)
- DNA component with gene expression

## Related

- See Phase B (`ABC-SUPER_SPRINT/2-simple-drive-simplex.md`) for drive system
- See `docs/biology/ideas/dna-driven-design.md` for DNA architecture
