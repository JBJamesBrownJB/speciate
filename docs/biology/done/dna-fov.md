# Field of View (FOV) Perception

**Status:** Implemented
**Location:** `apps/simulation/src/simulation/perception/`

## What It Does

Replaces 360-degree circular perception with a directional FOV cone. Creatures now have a facing direction (from `Rotation` component) and can only perceive entities within their field of view. FOV angle determines both the arc width and the perception range through a biological tradeoff.

## The Core Tradeoff

**Narrow FOV = longer effective range** (more photoreceptors per degree)
**Wide FOV = shorter range but better coverage** (prey survival strategy)

Formula: `perception_range = base_range * (180 / fov_angle)^0.4`

| FOV Angle | Range Multiplier | Typical Role |
|-----------|------------------|--------------|
| 80-90     | 1.35-1.38x       | Ambush predator |
| 100-120   | 1.18-1.26x       | Apex predator |
| 160-180   | 1.0-1.04x        | Pursuit predator / Generalist |
| 220-240   | 0.87-0.91x       | Omnivore |
| 280-300   | 0.76-0.78x       | Large prey |
| 320-340   | 0.71-0.73x       | Small prey (near-panoramic) |

## Biological Bounds

- **Minimum:** 45 degrees (extreme specialist - mantis shrimp, owl focus)
- **Maximum:** 340 degrees (near-panoramic - true 360 unrealistic for mobile creatures)
- **Default:** 180 degrees (neutral baseline, no range penalty)

## Key Parameters

| Constant | Value | Location |
|----------|-------|----------|
| `FOV_RANGE_EXPONENT` | 0.4 | `perception/constants.rs:9` |
| `MIN_FOV_DEGREES` | 45.0 | `perception/constants.rs:10` |
| `MAX_FOV_DEGREES` | 340.0 | `perception/constants.rs:11` |
| `DEFAULT_FOV_DEGREES` | 180.0 | `perception/constants.rs:12` |
| `PERCEPTION_MULTIPLIER` | 10.0 | `perception/constants.rs:2` |

## Implementation

### Perception Component

The `Perception` component stores FOV in radians and derives range from FOV and body size:

- `fov_angle: f32` - Field of view in radians (stored internally)
- `range: f32` - Derived from `body_size * PERCEPTION_MULTIPLIER * fov_factor`

Constructors:
- `Perception::new(fov_degrees, body_size)` - Explicit FOV
- `Perception::from_body_size(body_size)` - Uses 180 default
- `Perception::from_body_size_with_fov(body_size, fov_degrees)` - Explicit FOV

### FOV Cone Check

The perception system uses angle normalization to check if targets are within the FOV cone:

1. Calculate angle to target: `atan2(dy, dx)`
2. Get relative angle: `normalize_angle(angle_to_target - facing_direction)`
3. Check: `|relative_angle| <= half_fov`

The `normalize_angle` function wraps angles to [-PI, PI] range.

### System Integration

The `update_perception_system` now requires `Rotation` in its query to determine facing direction. Entities are only added as neighbors if:
1. Within distance (center_distance <= perception_range + combined_radii)
2. Within FOV cone (relative_angle <= half_fov)

## CritBuilder Support

FOV can be configured via the builder pattern:

```rust
CritBuilder::new()
    .at(100.0, 50.0)
    .with_fov(120.0)  // Predator-like narrow FOV
    .build(id)
```

Default FOV is 180 degrees if not specified.

## Debug Visualization

The perception overlay now renders a filled wedge (pie slice) instead of a circle:
- Wedge centered on creature position
- Arc spans `rotation +/- fov_angle/2`
- Semi-transparent cyan fill with outline
- Neighbor connection lines still rendered

## Future Work

### DNA Gene System

Currently FOV is set at spawn time via CritBuilder. Future work will:
- Add `fov_angle` as a DNA gene with mutation/inheritance
- Allow evolution to optimize FOV for ecological niches

### Stochastic Vision

Not yet implemented - will add reaction-time-gated perception updates.

### Additional Vision Genes (Planned)

- `visual_range_multiplier` - Independent range control (4-25x)
- `neural_speed` - Reaction time modifier (0.5-2.0)

## Biological Rationale (Zoologist Consultation)

### Eye Placement Trade-offs

**Frontal eyes (predators):**
- Binocular overlap for depth perception
- Narrow FOV (60-120 degrees)
- Excellent distance judgment for strikes
- Large blind spot behind

**Lateral eyes (prey):**
- Near-panoramic coverage (270-340 degrees)
- Early threat detection from any direction
- Poor depth perception (monocular vision)
- Small blind spot directly behind

### Real-World Examples

- **Hawks:** 22x body length range, 90 FOV (extreme specialist)
- **Rabbits:** 6x range, 300 FOV, 1.8x neural speed (reflexive prey)
- **Owls:** 14x range, 120 FOV, 0.7x neural speed (patient ambush)
- **Bison:** 8x range, 270 FOV (herd-dependent grazer)

### The Photoreceptor Budget

Retinal real estate is finite. Wide FOV spreads photoreceptors thin (lower acuity per degree). Narrow FOV concentrates them (higher acuity, longer effective range). The 0.4 exponent models this biological constraint without being so harsh that wide-FOV creatures are blind.

## References

- `apps/simulation/src/simulation/perception/constants.rs` - FOV constants
- `apps/simulation/src/simulation/perception/components.rs` - Perception component
- `apps/simulation/src/simulation/perception/systems.rs` - FOV cone check
- `apps/simulation/src/simulation/creatures/builder.rs` - CritBuilder FOV support
- `apps/portal/src/rendering/PerceptionOverlay.ts` - Wedge visualization

---

**Last Updated:** 2025-11-30
