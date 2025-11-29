# Wander System Specification

**Status:** Implemented
**Location:** `apps/simulation/src/simulation/creatures/behaviors/wander.rs`
**Sprint:** Sprint 15 - ECS Optimizations

## Overview

The wander system enables autonomous exploration within a home territory. It blends random wandering (free roaming) with homeward attraction (territory binding) using sigmoid interpolation. This creates natural, organic movement patterns that keep creatures near their spawn point while allowing exploration.

## Components

### WanderState

```rust
pub struct WanderState {
    pub angle: f32,          // Current wander direction (radians)
    pub change_timer: f32,   // Time until next direction change
}
```

### Related Components

- `HomePosition` - Creature's spawn point (territory center)
- `Position` - Current location
- `Velocity` - Used to calculate forward direction
- `Acceleration` - Wander and homeward forces are added here

## Constants

| Constant | Value | Description |
|----------|-------|-------------|
| `TERRITORY.comfort_radius` | 30.0 | Distance from home before homeward pull starts |
| `TERRITORY.blend_center` | 25.0 | Midpoint of blend transition |
| `TERRITORY.sigmoid_steepness` | 3.0 | Controls blend sharpness (higher = faster transition) |
| `STEERING.wander_force` | 8.0 | Base wander steering force |
| `STEERING.homeward_force` | 12.0 | Base homeward attraction force |

## Algorithm

### Territory Blending (Sigmoid Interpolation)

The core innovation of the wander system is smooth blending between two behaviors:

1. **Wander Force** (exploration) - Random direction changes for free roaming
2. **Homeward Force** (homing instinct) - Attraction toward HomePosition

**Blend Formula:**

```rust
// Sigmoid blend between wander (free roam) and homeward (territory bound)
// Steepness controls how quickly the blend transitions
normalized = (distance_from_home - blend_center) / comfort_radius
sigmoid = 1.0 / (1.0 + (-sigmoid_steepness * normalized).exp())
sigmoid = clamp(sigmoid, 0.0, 1.0)

// Apply blend
final_force = wander_force * (1.0 - sigmoid) + homeward_force * sigmoid
```

**Behavior by Distance:**

| Distance from Home | Sigmoid Value | Behavior |
|-------------------|---------------|----------|
| 0-20 units | ~0.0 | Pure wander (100% exploration) |
| 25 units (blend_center) | 0.5 | 50/50 mix |
| 30+ units (comfort_radius) | ~1.0 | Pure homeward (100% attraction) |

**Why Sigmoid?**
- Smooth, continuous transition (no sudden jerks)
- Biologically realistic (animals gradually prioritize home as they wander farther)
- Adjustable steepness parameter controls how "decisive" the transition is

### Wander Force Generation

1. **Random Direction Updates**
   ```rust
   if change_timer <= 0.0 {
       angle = random(-TERRITORY.wander_angle_change, +TERRITORY.wander_angle_change)
       change_timer = TERRITORY.wander_change_interval
   }
   ```

2. **Calculate Wander Direction**
   ```rust
   // Project forward from current velocity
   forward_x = velocity.vx
   forward_y = velocity.vy

   // Add random perturbation
   wander_x = forward_x * cos(angle) - forward_y * sin(angle)
   wander_y = forward_x * sin(angle) + forward_y * cos(angle)

   // Normalize and scale
   wander_force = normalize(wander_x, wander_y) * STEERING.wander_force
   ```

3. **Apply Blend**
   ```rust
   wander_contribution = wander_force * (1.0 - sigmoid)
   ```

### Homeward Force Generation

1. **Calculate Direction to Home**
   ```rust
   to_home_x = home_position.x - current_position.x
   to_home_y = home_position.y - current_position.y
   distance_from_home = sqrt(to_home_x² + to_home_y²)
   ```

2. **Urgency Scaling**
   ```rust
   // Homeward force increases with distance (linear urgency)
   urgency = distance_from_home / comfort_radius
   urgency = clamp(urgency, 0.0, 2.0)  // Cap at 2x for very far distances

   homeward_direction = normalize(to_home_x, to_home_y)
   homeward_force = homeward_direction * STEERING.homeward_force * urgency
   ```

3. **Apply Blend**
   ```rust
   homeward_contribution = homeward_force * sigmoid
   ```

### Final Force Combination

```rust
total_wander_force = wander_contribution + homeward_contribution
acceleration += total_wander_force
```

**Result:** Creatures explore freely near home, but are gently pulled back as they roam too far. The farther they go, the stronger the pull, creating a natural "elastic band" effect.

## Biological Realism

### Territory Formation

Real animals establish territories with:
- **Core area** - Safe zone near den/nest (pure exploration)
- **Home range** - Familiar area (mixed exploration + caution)
- **Edge** - Boundary they rarely cross (strong homing instinct)

Wander system models this with:
- Core: 0-20 units (comfort_radius - blend_center spread)
- Home range: 20-30 units (blend transition zone)
- Edge: 30+ units (pure homeward pull)

### Energy-Distance Trade-off

**Future DNA Integration:**
- Comfort radius scales with energy reserves (high energy = larger territory)
- Low energy creatures stay close to home (conserve energy)
- Homeward force strength based on "boldness" gene

## Performance

| Metric | Value | Notes |
|--------|-------|-------|
| Wander time | 0.58ms | @ 10K creatures |
| Complexity | O(N) | Linear in creature count |
| Bottleneck | None | Wander is not a performance concern |

## Integration

### System Order

```
1. territory_wandering_system
   └── Calculates wander + homeward forces, writes to Acceleration

2. integrate_motion_system
   └── Integrates forces into velocity/position
```

### Interaction with Other Behaviors

Wander forces **ADD** to acceleration alongside:
- Seek forces (when targeting food/mate)
- Flee forces (when escaping threat)
- Avoidance forces (obstacle avoidance)

**Behavior Hierarchy:**
- **Panic/Flee** - Overrides everything (survival)
- **Seek** - High priority (hunger, reproduction)
- **Wander** - Baseline behavior (exploration)

**Example:** A wandering creature that spots food will:
1. Wander force pushes random exploration
2. Seek force pulls toward food
3. If food is close, seek dominates
4. If food is far, wander continues with slight bias toward food

### Catatonic Creatures

Catatonic creatures skip wander system entirely:
- Query filter: `With<CanWander>`
- Catatonic creatures lack this marker (or it's handled via behavior mode check)
- See: `movement-spec.md` for catatonic handling

## Future Work

### Planned Enhancements

1. **DNA Integration**
   - `comfort_radius` (10-50 units, personality/energy-dependent)
   - `homeward_force` (5-20, boldness vs caution)
   - `sigmoid_steepness` (1-5, decisiveness of territory adherence)
   - `wander_force` (3-15, curiosity vs laziness)

2. **Dynamic Territory**
   - Territory size shrinks when low energy (conserve energy)
   - Territory size grows when well-fed (confident exploration)
   - Multiple "home" positions (migration patterns)

3. **Social Territory**
   - Shared territories for family groups
   - Territory defense (aggression when intruders enter)
   - Territory overlap resolution

### Alternative Approaches Considered

**Rejected: Hard Boundary**
```rust
if distance > comfort_radius {
    force = toward_home  // Binary switch
}
```
- **Problem:** Creatures "bounce" at boundary (unrealistic)
- **Sigmoid is superior:** Smooth, gradual pull creates natural movement

**Rejected: Linear Blend**
```rust
blend = min(distance / comfort_radius, 1.0)  // Linear ramp
```
- **Problem:** Constant gradient, no "decision point"
- **Sigmoid is superior:** S-curve creates clear core/edge/transition zones

## Constants Reference

All wander constants are defined in `apps/simulation/src/simulation/movement/constants.rs`:

```rust
pub const TERRITORY: TerritoryConstants = TerritoryConstants {
    comfort_radius: 30.0,
    blend_center: 25.0,
    sigmoid_steepness: 3.0,
    wander_angle_change: 0.3,    // Max angle delta per update (radians)
    wander_change_interval: 2.0, // Seconds between direction changes
};

pub const STEERING: SteeringConstants = SteeringConstants {
    wander_force: 8.0,
    homeward_force: 12.0,
    // ... other steering forces
};
```

**Future:** These constants will be replaced by per-creature DNA expression.

## Visual Debugging

**Recommended debug visualizations:**
1. Draw `comfort_radius` circle around HomePosition (green)
2. Draw `blend_center` circle (yellow)
3. Draw wander force vector (blue arrow)
4. Draw homeward force vector (red arrow)
5. Color creature by sigmoid value (0.0=blue, 0.5=purple, 1.0=red)

**See:** `/workspace/docs/debugging/visual-debug.md` (if it exists)
