# Seismic Impacts: Ground Vibrations from Large Creatures

**Status:** Idea (First signal type implementation)

**Depends on:** `signal-broadcasting.md` (foundation architecture)

## Problem / Opportunity

Large creatures moving at speed create ground vibrations that smaller creatures can detect. This enables prey to sense approaching predators through the ground, even when out of visual range or field of view.

**This is the recommended first implementation** because:
- Physics-derived (size x velocity) - no new DNA traits required
- Clear biological grounding (elephants, spiders, sand scorpions)
- Strong Golden Zone opportunities
- Foundation for all other signal types

## Proposed Solution

### Emission: Physics-Derived (No DNA Required)

Signal strength calculated during movement phase:

```
seismic_strength = size * velocity_magnitude * SEISMIC_COEFFICIENT
```

**Thresholds:**
- If `seismic_strength < MIN_SIGNAL_THRESHOLD`: skip emission (optimization + biology: small/slow creatures are quiet)
- If `velocity_magnitude < MOVEMENT_THRESHOLD`: skip emission entirely (stationary = silent)

**Propagation:**
- Emit to surrounding L0 cells (9-cell neighborhood)
- Log aggregate to L1 cells within propagation range
- Larger creatures (low frequency) propagate further than small creatures

### Reception: DNA-Gated Ability

Creatures need seismic sensing trait to detect ground vibrations:
- Requires ground contact (airborne/jumping creatures receive nothing)
- Self-noise penalty: moving creatures have reduced reception quality
- Sensory trade-off: high seismic sensitivity may reduce visual acuity

### Decay

Seismic signals decay rapidly (2-5 ticks half-life):
- Mechanical waves dissipate quickly in substrate
- Creates "real-time" sensing, not trailing
- Contrast with chemical signals which persist much longer

## Golden Zone Opportunities

| Optimization | Free Biological Behavior |
|--------------|--------------------------|
| Skip emission for stationary creatures | Ambush predators are seismically invisible |
| Skip emission for small creatures | Mice walk past elephants undetected |
| Freeze to sense better | Prey freeze response = skip movement processing |
| Self-noise penalty | Must stop to listen accurately |
| Size-based propagation distance | Giant footsteps felt from afar |

**Freeze Response Golden Zone:**
A creature that stops moving to improve seismic reception:
1. Skips movement physics calculations (performance win)
2. Gains reception quality boost (gameplay win)
3. Exhibits natural prey behavior (biological accuracy)

## Trade-offs

**Ground contact requirement:** Flying creatures, swimming creatures, and jumping creatures cannot sense seismic signals. This creates distinct ecological niches.

**Self-noise:** Large creatures are inherently worse at seismic reception because their own movement generates noise that masks incoming signals.

**Attention cost:** Processing seismic signals competes with visual attention. Creatures focused on seismic input may miss visual threats.

## Expert Input

### Zoologist (zoologist-tom)

Real-world seismic sensing species:

| Species | Mechanism | Range | Key Insight |
|---------|-----------|-------|-------------|
| Elephants | Pacinian corpuscles in feet | 10-30 km | Low-frequency waves travel far |
| Spiders | Slit sensilla in leg joints | Web diameter (0.1-2m) | Extremely precise localization |
| Golden Moles | Hypertrophied ear bones | 10-20m | Primary sense (eyes vestigial) |
| Sand Scorpions | Basitarsal slit sensilla | 0.5m | Can distinguish prey from predator |

**Biological behaviors to model:**
- Elephants "freeze" to triangulate source direction
- Spiders pluck web to confirm signal before approaching
- Golden moles are nearly blind (sensory trade-off)

### ECS (ecs-emma)

Implementation approach:
- Emission during movement phase (spatial work already done)
- Write to L0 cells, not directly to creatures (parallel safe)
- Sequential distribution from cells to creature signal buffers
- Signal buffer already covers seismic type tag

## Dependencies

- `signal-broadcasting.md` architecture must be implemented first
- Movement system (already exists)
- Spatial grid L0/L1 (already exists)

## Related Ideas

- `signal-broadcasting.md` - Foundation architecture (required)
- `mating-calls.md` - Vocal signals (similar propagation, different trigger)
- `size-domination.md` - Large creatures already have advantages; seismic adds stealth cost

## Open Questions

- Should substrate type affect propagation? (rock vs sand vs water)
- Should seismic reception be a discrete trait or continuous (some species better than others)?
- How does this interact with burrowing creatures? Do they sense better underground?

---
*Captured: 2025-12-28*
*Recommended as first signal type implementation*
