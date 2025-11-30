# Energy-Vigilance Relationship

## Concept

Low energy INCREASES scanning behavior, not reduces it. Hungry animals are more vigilant for food opportunities.

## Biological Basis

This is the INVERSE of what intuition suggests:

- Hungry predators scan more frequently for prey
- Starving herbivores search wider areas for food
- Energy-depleted animals cannot afford to miss opportunities
- Satiated animals can relax vigilance (already fed)

## Proposed Implementation

```
hunger_factor = 1.0 - (current_energy / max_energy)
scanning_frequency = base_frequency * (1.0 + 0.5 * hunger_factor)
```

At 0% energy: 50% more frequent perception updates
At 100% energy: baseline frequency

## Alternative: Perception Range Increase

Instead of frequency, could increase range:
```
effective_range = base_range * (1.0 + 0.3 * hunger_factor)
```

Hungry creatures scan further for resources.

## Trade-off Considerations

Increased vigilance should have costs:
- More energy spent on perception? (probably negligible)
- Reduced focus on current task? (interrupt wandering more easily)
- Increased stress from heightened awareness?

## Contrast with Crowding

- Crowding reduces range (too much input, focus on immediate)
- Hunger increases range (searching for resources)
- These can combine: hungry creature in crowd = conflicting pressures

## Source

Zoologist-tom consultation, 2025-11-30
