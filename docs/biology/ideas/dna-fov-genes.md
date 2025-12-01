# DNA-Based FOV Genes

## Concept

Make FOV angle an evolvable DNA gene with mutation and inheritance, allowing evolution to optimize FOV for ecological niches.

## Proposed Genes

### Primary: `fov_angle`
- Range: 45° to 340°
- Mutation rate: Low (structural eye changes are slow)
- Inheritance: Blend with small variance

### Secondary (Future)
- `visual_range_multiplier` - Independent range control (4-25x body length)
- `neural_speed` - Reaction time modifier (0.5-2.0x)

## Biological Rationale

### Eye Placement Trade-offs

**Frontal eyes (predators):**
- Binocular overlap for depth perception
- Narrow FOV (60-120°)
- Excellent distance judgment for strikes
- Large blind spot behind

**Lateral eyes (prey):**
- Near-panoramic coverage (270-340°)
- Early threat detection from any direction
- Poor depth perception (monocular vision)
- Small blind spot directly behind

### Real-World Examples

| Species | Range | FOV | Neural Speed | Strategy |
|---------|-------|-----|--------------|----------|
| Hawk | 22x | 90° | 1.0x | Extreme specialist |
| Rabbit | 6x | 300° | 1.8x | Reflexive prey |
| Owl | 14x | 120° | 0.7x | Patient ambush |
| Bison | 8x | 270° | 1.0x | Herd-dependent grazer |

### The Photoreceptor Budget

Retinal real estate is finite. Wide FOV spreads photoreceptors thin (lower acuity per degree). Narrow FOV concentrates them (higher acuity, longer effective range).

## Expected Emergent Behavior

- Predator lineages evolve narrower FOV over generations
- Prey lineages evolve wider FOV for threat detection
- Ambush predators might evolve extreme narrow FOV
- Herd animals might evolve moderate wide FOV (rely on group vigilance)

## Source

Zoologist-tom consultation, 2025-11-30
