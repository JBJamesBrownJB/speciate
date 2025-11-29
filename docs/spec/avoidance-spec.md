# Avoidance System Specification

**Status:** Implemented
**Location:** `apps/simulation/src/simulation/creatures/behaviors/avoidance.rs`
**Sprint:** Sprint 15 - ECS Optimizations

## Overview

The avoidance system generates repulsion forces to prevent creatures from colliding. It uses inverse-square force scaling (like electromagnetic repulsion) with panic override for extreme threats. Sprint 15 added OPT-7 early exit optimization for 19% performance gain.

## Components

### AvoidanceBehavior

```rust
pub struct AvoidanceBehavior {
    pub personal_space: f32,  // Distance to maintain from others
    pub max_force: f32,       // Maximum repulsion force magnitude
}
```

### Related Components

- `Perception` - Contains nearby entities detected by vision system
- `BodySize` - Used to calculate effective collision radii
- `Acceleration` - Repulsion forces are added here

## Constants

| Constant | Value | Description |
|----------|-------|-------------|
| `STEERING.avoidance_force` | 15.0 | Base repulsion force magnitude |
| `STEERING.panic_force` | 50.0 | Maximum force when panicking |
| Personal space | 2.5 (typical) | Distance creatures try to maintain |
| Panic threshold multiplier | 2.0 × body_size | Distance that triggers panic |

## Algorithm

### Inverse-Square Repulsion Force

For each neighbor detected by perception:

1. **Distance Calculation**
   ```rust
   center_distance = sqrt((x1 - x2)² + (y1 - y2)²)
   edge_distance = center_distance - radius_self - radius_other
   ```

2. **Early Exit (OPT-7)** - Sprint 15 optimization
   ```rust
   // Check squared distance BEFORE sqrt (80% of neighbors filtered here)
   max_interaction_distance_sq = (personal_space + max_combined_radius)²
   if center_distance_sq > max_interaction_distance_sq {
       continue;  // Skip this neighbor
   }

   // Only compute sqrt for close neighbors (20% of cases)
   center_distance = sqrt(center_distance_sq)
   ```

   **Performance:** Reduced avoidance time by 19% (3.2ms → 2.6ms @ 10K creatures)

3. **Repulsion Force Formula**
   ```rust
   ratio = personal_space / safe_distance
   force_magnitude = avoidance_force * ratio²

   // Inverse-square scaling: closer = exponentially stronger
   // Examples:
   //   distance = personal_space → force = 1.0x base
   //   distance = personal_space/2 → force = 4.0x base (²)
   //   distance = personal_space/4 → force = 16.0x base (²)
   ```

4. **Panic Override**
   ```rust
   if safe_distance < panic_threshold {
       force_magnitude = min(force_magnitude, panic_force)
   }
   ```

   **Why cap at panic_force?** Prevents numerical instability at extremely close distances. Panic is "maximum alarm" not "infinite force."

5. **Force Direction**
   ```rust
   // Repel directly away from neighbor
   force_x = (away_x / center_distance) * force_magnitude
   force_y = (away_y / center_distance) * force_magnitude
   ```

6. **Force Accumulation**
   ```rust
   // Sum repulsion from all neighbors
   total_repulsion_x += force_x
   total_repulsion_y += force_y
   ```

7. **Maximum Force Limiting**
   ```rust
   // Clamp total force to max_force (prevents runaway acceleration)
   if total_magnitude > max_force {
       total_repulsion *= (max_force / total_magnitude)
   }

   acceleration += total_repulsion
   ```

### Personal Space Scaling

Personal space is a **per-creature parameter**, not global:
- Small creatures: `personal_space = 1.5` (tighter spacing tolerated)
- Large creatures: `personal_space = 3.0` (need more clearance)
- **Future:** DNA-encoded with size correlation

### Body Size Integration

The system accounts for actual creature dimensions:

```rust
self_radius = body_size.length / 2.0
other_radius = other_body_size.length / 2.0
edge_distance = center_distance - self_radius - other_radius
```

**This ensures:**
- Large creatures don't overlap small ones
- Force activates when body surfaces approach, not centers
- Realistic collision prevention

## Panic Override Logic

### When Panic Triggers

```rust
panic_threshold = 2.0 * body_size.length
if edge_distance < panic_threshold {
    // PANIC MODE
}
```

**Example:** A creature with `body_size = 1.0` panics when neighbor is within 2.0 units.

### Why Panic Exists

1. **Biological Realism** - Animals react more strongly to immediate threats
2. **Prevents Deadlock** - Without panic, creatures can get stuck pushing against each other with equal force
3. **Emergency Response** - Overrides normal force limits for critical situations

### Panic Disabling

Panic is disabled when `energy < 5.0`:
- Simulates "giving up" behavior (too weak to react)
- Prevents pointless thrashing when nearly dead
- Implemented in `brain.rs:should_panic()`

## Performance

### Sprint 15 Metrics (OPT-7 Early Exit)

| Metric | Before | After | Improvement |
|--------|--------|-------|-------------|
| Avoidance time | 3.2ms | 2.6ms | **19% faster** |
| Sqrt calls | 100% | ~20% | 80% reduction |

**How it works:** Most neighbors are far enough away that we can reject them with a cheap squared-distance check, avoiding the expensive `sqrt()` call.

### Complexity

- **Time:** O(N × M) where N = creatures, M = avg neighbors per creature
- **Bottleneck:** Nested loop over perception neighbors
- **Mitigation:** Perception system limits M to 20-30 via spatial filtering

## Integration

### System Order

```
1. perception_system
   └── Populates Perception.neighbors for each creature

2. avoidance_system (runs in parallel with other behaviors)
   └── Reads Perception, writes to Acceleration

3. integrate_motion_system
   └── Integrates accumulated forces into velocity/position
```

### Interaction with Other Behaviors

Avoidance forces **ADD** to acceleration alongside:
- Seek forces (attraction to target)
- Flee forces (escape from threat)
- Wander forces (exploration)

**Result:** Natural force blending creates emergent path planning.

**Example:** A creature seeking food will:
1. Steer toward food (seek force)
2. Avoid obstacles in path (avoidance force)
3. Final path is the vector sum (smooth navigation around obstacles)

### Why Panic Overrides Other Behaviors

Panic forces are processed in `avoidance_system`, but the panic *decision* happens in `brain.rs:should_panic()`. This allows:
- Behavior transition system to switch to `Fleeing` mode
- Avoidance to still apply strong repulsion
- Other behaviors (seek, wander) to be suppressed

## Future Work

### Planned Enhancements

1. **DNA Integration**
   - `personal_space` (1.0-4.0 range, correlated with body size)
   - `panic_threshold` (1.5-3.0 range, fight-or-flight temperament)
   - `avoidance_force` (10.0-30.0 range, aggressiveness of avoidance)

2. **Hierarchical Avoidance** (Spatial Grid Integration)
   - Only check avoidance against creatures in same/adjacent grid cells
   - Expected: O(N × M) → O(N × log M) with spatial partitioning

3. **Predictive Avoidance**
   - Account for neighbor velocity, not just position
   - Predict collision course and avoid preemptively
   - Requires velocity in Perception data

### Known Limitations

- **No collision resolution:** Avoidance tries to prevent overlap but doesn't fix it if it happens
- **No pushback:** Creatures can get "trapped" against walls by swarm pressure
- **Perception lag:** Uses stale neighbor positions (from last perception update)

**See:** `perception-spec.md` for vision update timing details.
