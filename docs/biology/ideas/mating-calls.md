# Mating Calls: Vocal Reproduction Signaling

**Status:** Idea (DNA-driven signal type)

**Depends on:** `signal-broadcasting.md` (foundation architecture)

## Problem / Opportunity

Reproduction requires finding a compatible mate. Currently creatures would need to visually locate mates, which is inefficient and doesn't match real animal behavior. Mating calls allow creatures to advertise reproductive availability over distance.

## Proposed Solution

### Emission: DNA-Driven Trait

Unlike seismic impacts (physics-derived), mating calls require a DNA trait:

**Vocal Apparatus Gene:**
- Encodes ability to emit vocal signals
- Affects call strength (loud vs quiet)
- Affects call frequency/character (species recognition)
- Energy cost to produce (trade-off)

**Emission Triggers:**
- Creature must be in mating state (reproductive readiness)
- Must have vocal apparatus trait expressed
- Costs energy per emission (prevents spamming)

### Reception: Species-Specific

Creatures should primarily respond to calls from their own species:
- Signal includes implicit species signature (derived from emitter DNA)
- Receivers have species-match preference in brain processing
- Cross-species calls may still be detectable but not attractive

### Propagation

Vocal signals behave differently from seismic:
- **Medium:** Air (not ground) - no ground contact required
- **Range:** Moderate (species-dependent via DNA)
- **Decay:** 5-10 ticks half-life (echoes, reverberations)
- **Obstacles:** Future: could be blocked by terrain features

### Behavioral Response

When a receptive creature detects a compatible mating call:
1. Interrupt current behavior (if priority allows)
2. Orient toward call direction
3. Enter seeking state toward call source
4. Compete with other responders (if multiple arrive)

## Golden Zone Opportunities

| Optimization | Free Biological Behavior |
|--------------|--------------------------|
| Only emit when in mating state | Seasonal breeding concentration |
| Energy cost per call | Costly signaling (honest advertising) |
| Louder calls propagate further | Sexual selection for strong callers |
| Only process calls when receptive | Non-breeding creatures ignore noise |

**Costly Signaling Golden Zone:**
- Louder calls = more energy spent = only healthy creatures can afford it
- Creates honest signal of fitness without explicit "fitness" stat
- Natural sexual selection emerges from physics

## Trade-offs

**Energy cost:** Calling is expensive. Weak creatures cannot afford prolonged calling, creating natural honest signaling.

**Predator attraction:** Mating calls are audible to predators too. Calling advertises your location to everyone, not just potential mates.

**Competition:** Multiple callers in an area create noise that makes localization harder. First-mover advantage vs late-caller drowning out.

## Expert Input

### Zoologist (zoologist-tom)

Real-world mating call dynamics:
- **Frogs:** Males call, females approach silently (asymmetric)
- **Birds:** Dawn chorus creates intense competition (timing matters)
- **Insects:** Species-specific frequencies prevent cross-species attraction
- **Elk:** Bugling costs significant energy and attracts rivals

**Key insight:** Mating calls create "hotspots" where multiple creatures converge. This enables emergent lek behavior (display grounds) without explicit programming.

## Dependencies

- `signal-broadcasting.md` architecture (required)
- DNA system for vocal apparatus trait
- Mating/reproduction system (creature states)
- Energy system (to pay call cost)

## Related Ideas

- `signal-broadcasting.md` - Foundation architecture
- `flocking-calls.md` - Similar vocal mechanism, different purpose
- `chemical-scent.md` - Alternative mating signal (pheromones)

## Open Questions

- Should males and females have different calling behavior (asymmetric signaling)?
- Can creatures "fake" fitness through loud calls despite being weak (deception)?
- Should call character be heritable (sexual selection on call traits)?
- How do calls interact with energy-vigilance trade-offs?

---
*Captured: 2025-12-28*
