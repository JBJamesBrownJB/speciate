# Field of View (FOV) Perception

**Status:** ✅ Implemented (Phase A)
**Location:** `apps/simulation/src/simulation/perception/`

## What It Does

Replaces 360-degree circular perception with a directional FOV cone. Creatures have a facing direction (from `Rotation` component) and can only perceive entities within their field of view.

## The Core Tradeoff

**Narrow FOV = longer range** (more photoreceptors per degree)
**Wide FOV = shorter range but better coverage**

Formula: `perception_range = base_range * (180 / fov_angle)^0.4`

| FOV Angle | Range Multiplier | Typical Role |
|-----------|------------------|--------------|
| 90° | 1.35x | Ambush predator |
| 120° | 1.26x | Apex predator |
| 180° | 1.0x | Generalist (default) |
| 270° | 0.91x | Large prey |
| 320° | 0.73x | Small prey (near-panoramic) |

## Key Parameters

See `perception/constants.rs`:
- `FOV_RANGE_EXPONENT` = 0.4
- `MIN_FOV_DEGREES` = 45.0
- `MAX_FOV_DEGREES` = 340.0
- `DEFAULT_FOV_DEGREES` = 180.0
- `PERCEPTION_MULTIPLIER` = 10.0

## FOV Check (Optimized)

Uses sqrt-free dot product comparison:
1. Early-exit if target behind (`rough_dot <= 0`)
2. Squared comparison: `rough_dot² >= cos_half_fov_sq × dist²`

See `perception/systems.rs:63-76` for implementation.

## Debug Visualization

The perception overlay renders a filled wedge (pie slice):
- Arc spans `rotation ± fov_angle/2`
- Semi-transparent cyan fill
- Neighbor connection lines

## Future Work

See `ideas/`:
- `dna-fov-genes.md` - FOV as evolvable DNA gene
- `stochastic-vision.md` - Reaction-time-gated perception
