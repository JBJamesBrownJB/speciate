# Perception System Specification

**Status:** Implemented
**Location:** `apps/simulation/src/simulation/perception/`

## Overview

The perception system handles creature awareness of nearby entities. Each tick, creatures detect neighbors within their perception range, which drives avoidance steering and future behavior decisions.

## Components

### Perception

```rust
pub struct Perception {
    pub range: f32,           // Detection range in meters
    pub nearby: Vec<Entity>,  // Detected entities (cleared each tick)
}
```

| Method | Formula | Description |
|--------|---------|-------------|
| `from_body_size(size)` | `size × 10.0` | Range scales with body length |
| `default_range()` | `10.0` | Default perception range |
| `has_neighbors()` | - | Returns true if nearby is non-empty |
| `neighbor_count()` | - | Returns number of detected neighbors |
| `clear()` | - | Clears the nearby list |
| `add_neighbor(entity)` | - | Adds entity to nearby list |

### AvoidanceBehavior

```rust
pub struct AvoidanceBehavior {
    pub personal_space: f32,  // Comfort zone radius in meters
    pub max_force: f32,       // Maximum avoidance steering force
}
```

| Method | Formula | Description |
|--------|---------|-------------|
| `from_body_size(size)` | `size + 1.5` | Personal space scales with body |
| `panic_threshold()` | `personal_space × 0.5` | Emergency response distance |

## Constants

From `movement/constants.rs`:

| Constant | Value | Description |
|----------|-------|-------------|
| `perception_multiplier` | 10.0 | Perception range = body × this |
| `personal_space` | 1.5 | Base spacing buffer (meters) |
| `panic_threshold_ratio` | 0.5 | Panic at 50% of personal space |

### Example Values (body_size = 1.0)

| Parameter | Value |
|-----------|-------|
| Perception range | 10.0m |
| Personal space | 2.5m |
| Panic threshold | 1.25m |

## System: `update_perception_system`

**Algorithm:** O(N²) brute force

```
1. Collect all (Entity, Position, BodySize) into Vec
2. For each creature with Perception:
   a. Clear previous nearby list
   b. Skip if behavior is Catatonic (inactive)
   c. For each other creature:
      - Calculate center-to-center distance
      - Calculate edge-to-edge distance: center_dist - (self_radius + other_radius)
      - If edge_dist ≤ perception.range: add to nearby list
```

### Distance Calculation

Uses **edge-to-edge distance** (not center-to-center):

```
edge_distance = sqrt((x2-x1)² + (y2-y1)²) - radius_self - radius_other
```

This means two creatures touching (edge_distance = 0) are detected regardless of their body sizes.

### Catatonic Filtering

Creatures with `BehaviorMode::Catatonic` skip perception updates:
- They don't perceive neighbors
- They ARE perceived by active creatures

## Integration

### System Order

1. `update_perception_system` - Populates `Perception.nearby`
2. `behavior_transition_system` - Brain reads perception (possibly stale)
3. `avoidance_system` - Uses `Perception.nearby` for steering forces

### Brain Independence

Brain runs on its own dynamic cooldown, independent of perception updates. Brain reads `Perception.nearby` whenever it decides, which may contain:
- Fresh data (perception updated this tick)
- Stale data (last update was N ticks ago, if stochastic vision)

## Performance

### Current Baseline (5K creatures)

| Metric | Value |
|--------|-------|
| Perception time | 34ms (67% of tick) |
| Comparisons | ~25 million (N²) |
| Algorithm | O(N²) brute force |

### Scaling Problem

| Creatures | Comparisons | Est. Time |
|-----------|-------------|-----------|
| 5K | 25M | 34ms |
| 10K | 100M | ~136ms |
| 20K | 400M | ~544ms |

## Known Issues

1. **O(N²) scaling** - Dominates tick time at scale
2. **Vec allocations** - `nearby: Vec<Entity>` allocates each tick
3. **No spatial indexing** - No grid/quadtree acceleration
4. **All-or-nothing** - Every active creature updates every tick

## Future Optimizations (Sprint 15 Phase 2)

| Phase | Optimization | Expected Gain |
|-------|--------------|---------------|
| 2A | Split queries (active/inactive) | +200% capacity |
| 2B | Pre-allocated Vec, SIMD distance | +25% capacity |
| 2C | Parallel iteration | +60% capacity |
| 2D | Stochastic vision (10% per tick) | +100-300% capacity |

## DNA Integration (Future)

Parameters to be DNA-encoded:
- `perception_multiplier` (5.0-20.0 range) - visual acuity
- `personal_space` (0.5-3.0 range) - social distance preference
- Vision update frequency (stochastic rate)
