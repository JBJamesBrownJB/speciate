# Movement System Specification

**Status:** Implemented
**Location:** `apps/simulation/src/simulation/movement/systems.rs`
**Sprint:** Sprint 15 - ECS Optimizations

## Overview

The movement system integrates forces into velocity and position using Euler integration with Rayon parallelization. Sprint 15 achieved 6.3x speedup through multi-core execution, enabling 20K+ creature capacity.

## Components

### Core Components

```rust
pub struct Position { pub x: f32, pub y: f32 }
pub struct Velocity { pub vx: f32, pub vy: f32 }
pub struct Acceleration { pub ax: f32, pub ay: f32 }
pub struct BodySize { pub length: f32, pub inv_sqrt_length: f32 }
```

## Constants

| Constant | Value | Description |
|----------|-------|-------------|
| `MAX_SPEED` | 50.0 | Maximum creature velocity (m/s) |
| `VELOCITY_DAMPING` | 0.98 | Per-frame damping factor (drag) |
| `STOPPED_THRESHOLD` | 0.01 | Velocity magnitude below which creature is considered stopped |

## Algorithm

### Euler Integration (Rayon Parallelized)

The movement system implements explicit Euler integration with these steps:

1. **Force Accumulation** (multiple systems)
   - Steering behaviors ADD to `Acceleration` component
   - Wander, seek, flee, avoidance all contribute forces

2. **Parallel Velocity Integration** (Sprint 15 optimization)
   - Collect all entities into Vec for Rayon processing
   - `par_iter_mut()` updates velocity for all creatures simultaneously
   - Formula: `velocity += acceleration * dt`
   - Apply velocity damping: `velocity *= VELOCITY_DAMPING`

3. **Locomotion Noise** (procedural animation)
   - Perlin noise applied perpendicular to velocity
   - Magnitude scales with speed² and inverse body size
   - Creates organic, non-linear movement

4. **Speed Clamping**
   - Clamp to MAX_SPEED: `if speed² > MAX_SPEED² { velocity *= scale }`
   - Preserves direction, only limits magnitude

5. **Acceleration Reset**
   - Set `acceleration = 0` after integration
   - Clean slate for next frame's force accumulation

6. **Position Integration**
   - Formula: `position += velocity * dt`

7. **Parallel Boundary Enforcement** (reuse collected Vec)
   - Clamp position to world bounds
   - Reflect velocity at boundaries: `if x < min_x { vx = max(vx, 0) }`

### Rayon Parallelization Architecture (Sprint 15)

**Pattern: Collect → Parallel → Write-back**

```rust
// Collect entities into Vec
let mut entities: Vec<_> = query.iter_mut().collect();

// Parallel physics integration (uses all 16 cores)
entities.par_iter_mut().for_each(|(entity, size, position, velocity, acceleration, creature_state)| {
    // Physics logic runs in parallel
    // Each core processes ~625 creatures (at 10K total)
});

// Parallel boundary enforcement (reuse Vec)
entities.par_iter_mut().for_each(|(..., position, velocity, ...)| {
    // Boundary clamping runs in parallel
});
```

**Key Design Decisions:**
- Manual Vec collection (Bevy's `par_iter_mut()` didn't engage Rayon properly)
- Two parallel loops reuse same Vec (efficient)
- Automatic write-back through mutable references
- No explicit synchronization needed

## Performance

### Sprint 15 Metrics (10K creatures)

| Metric | Before | After | Improvement |
|--------|--------|-------|-------------|
| Movement time | 25.9ms | 4.1ms | **6.3x faster** |
| Total tick time | 47.7ms | 28.7ms | 40% faster |
| CPU cores (avg) | 9.3 | 16 | All cores engaged |
| IPC | 3.0 | 4.25 | Better instruction throughput |
| Capacity validated | 10K | 20K | 2x scaling confirmed |

**Bottleneck:** Perception system (20ms) is now the primary bottleneck, not movement.

### Optimization History

| Optimization | Description | Gain |
|-------------|-------------|------|
| OPT-1 | Split velocity/acceleration queries | +1.5ms |
| OPT-2 | Body size cache (inv_sqrt_length) | +0.9ms |
| OPT-3 | Early exit for catatonic creatures | +0.8ms |
| OPT-4 | Speed² comparison (avoid sqrt) | +0.7ms |
| **OPT-Rayon** | **Rayon parallelization** | **+21.8ms (6.3x)** |

**See:** `SPRINT_DOCS/SPRINT_BACKLOG.md` for full optimization log.

## Integration

### System Order

```
1. Behavior Systems (parallel where possible)
   ├── wander_system
   ├── seek_system
   ├── flee_system
   └── avoidance_system

2. integrate_motion_system (MUST run after all behaviors)
   ├── Rayon parallel velocity integration
   ├── Speed clamping
   ├── Acceleration reset
   ├── Position integration
   └── Rayon parallel boundary enforcement

3. Visual Systems
   └── rotation_system (updates creature facing direction)
```

**Critical:** Movement integration MUST run after ALL force-generating systems. Adding forces to `Acceleration` has no effect if run after integration (acceleration is zeroed).

### Catatonic Creature Handling

Catatonic creatures have special logic:
- Zero acceleration immediately
- Apply damping to velocity
- Stop completely when below `STOPPED_THRESHOLD`
- Skip noise and other active movement logic

## Future Work

### Planned Optimizations (Deferred to Sprint 16+)

1. **Vec2 SIMD Migration** (Phase 2B)
   - Replace `(f32, f32)` with `glam::Vec2` for SIMD operations
   - Expected: +15-20% speedup
   - Blocked: Requires serialization format change

2. **Spatial Grid Integration**
   - Localized movement updates (only process active grid cells)
   - Expected: Better cache coherency at 50K+ creatures

### DNA Integration

Constants to be DNA-encoded in future sprint:
- `max_speed` (10-100 m/s range, size-dependent)
- `velocity_damping` (0.95-0.99 range, affects inertia/agility trade-off)
- `locomotion_noise_base` (0.0-1.0 range, movement "sloppiness")

**See:** `/workspace/docs/biology/dna-driven-design.md`
