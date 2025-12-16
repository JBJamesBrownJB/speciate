# Fused Steering System

**Status:** Ready for implementation
**Expected Gain:** 2-4ms per tick (~10-15% improvement)
**Scope:** Steering behaviors only (wander, seek, avoidance, flee)

## What We're Fusing

Four steering systems that all share the same query pattern and output:

| System | Queries | Writes To |
|--------|---------|-----------|
| `territory_wandering_system` | Position, Velocity, BodySize, CreatureState | Acceleration |
| `seek_system` | Position, Velocity, BodySize, CreatureState | Acceleration |
| `avoidance_system` | Position, Velocity, BodySize, CreatureState | Acceleration |
| `flee_system` | Position, Velocity, BodySize, CreatureState | Acceleration |

**Why fuse these?** Same query, same output, same tick rate → 4x redundant iteration.

## What Stays Separate

| System | Why Separate |
|--------|--------------|
| `rebuild_spatial_grid` | Different output (spatial index) |
| `update_perception` | Different output (NeighborCache), runs before steering |
| `behavior_transition` | Different output (BehaviorMode), decides WHAT not HOW |
| `integrate_motion` | Runs after steering, reads Acceleration |
| `rotation` | Different query (Velocity → Rotation) |

## Before/After

```
BEFORE (4 queries, 4 iterations):
┌──────────┐   ┌──────────┐   ┌──────────┐   ┌──────────┐
│  Wander  │ → │   Seek   │ → │  Avoid   │ → │   Flee   │
└──────────┘   └──────────┘   └──────────┘   └──────────┘
   Query 1       Query 2        Query 3        Query 4

AFTER (1 query, 1 iteration):
┌─────────────────────────────────────────────────────────┐
│                   update_steering                        │
│  wander() + seek() + avoid() + flee() → Acceleration    │
└─────────────────────────────────────────────────────────┘
                        Query 1
```

## Folder Structure

```
creatures/
├── steering/
│   ├── mod.rs          # exports update_steering_system
│   ├── system.rs       # THE fused system (one query, calls pure fns)
│   ├── context.rs      # SteeringContext struct
│   ├── wander.rs       # calculate_wander_force() + #[cfg(test)]
│   ├── seek.rs         # calculate_seek_force() + #[cfg(test)]
│   ├── avoidance.rs    # calculate_avoidance_force() + #[cfg(test)]
│   └── flee.rs         # calculate_flee_force() + #[cfg(test)]
├── perception/         # SEPARATE - different concern
├── transitions/        # SEPARATE - state machine
└── movement/           # SEPARATE - physics integration
```

## Pure Function Pattern

Each behavior file exports a pure function:

```rust
pub fn calculate_force(ctx: &SteeringContext, state: &WanderState) -> Vec2
```

**SteeringContext** bundles common read-only data:
- position, velocity, body_size, creature_state
- world_bounds, dt

Tests live inline with `#[cfg(test)]`.

## Implementation Plan

### Phase 1: Restructure folders

1. Create `creatures/steering/` directory
2. Move existing pure functions (wander/steering.rs already exists)
3. Extract pure functions from seek, avoidance, flee systems
4. Verify tests pass

### Phase 2: Create fused system

1. Create `steering/system.rs` with `update_steering_system`
2. Single query, iterate once, call all pure functions
3. Use Rayon `par_iter_mut` for parallelization

### Phase 3: Wire up

1. Replace 4 individual systems with 1 fused system in plugin
2. Update system ordering in `simulation/core/simulation.rs`
3. Delete old `behaviors/*/systems.rs` files

### Phase 4: Benchmark

1. Compare at 10K, 50K, 100K, 200K creatures
2. Profile with `perf stat` for cache behavior

## Testing Strategy

**Unit tests:** Inline in each pure function file (wander.rs, seek.rs, etc.)
**Integration tests:** Existing TOML specs in `specs/behavior/`

## When to Implement

- [x] All steering behaviors feature-complete
- [x] Behavior logic stable
- [ ] Profile confirms steering is bottleneck
- [ ] Ready for optimization pass

## References

- Existing pure function pattern: `behaviors/wander/steering.rs`
- Rayon parallelization: `movement/systems.rs:35-113`
- System registration: `simulation/core/simulation.rs:75-110`
