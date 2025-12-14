# Fused Behavior Systems (Pre-Release Optimization)

**Status:** Deferred until pre-release optimization phase
**Expected Gain:** 2-4ms per tick (~10-15% improvement)
**Trade-off:** Reduced code maintainability

## Current Architecture

Behavior systems run separately:

```
Perception → Seek → Wander → Avoidance → Movement
```

Each system iterates 200K creatures independently, causing repeated memory access.

## The Optimization

Fuse seek, wander, and avoidance into a single parallel pass:

```rust
entities.par_iter_mut().for_each(|(pos, vel, accel, state, ...)| {
    // All behaviors computed while data is hot in L1 cache
    let seek_force = compute_seek(pos, target, ...);
    let wander_force = compute_wander(pos, home, wander_state, ...);
    let avoid_force = compute_avoidance(pos, neighbor_cache, ...);

    accel.ax += seek_force.x + wander_force.x + avoid_force.x;
    accel.ay += seek_force.y + wander_force.y + avoid_force.y;
});
```

## Why It Helps

| Metric | Separate Systems | Fused System |
|--------|-----------------|--------------|
| Entity iterations | 600K (3 × 200K) | 200K |
| Cache behavior | Data evicted between systems | Data stays hot |
| Rayon dispatches | 3 | 1 |
| Memory bandwidth | 3× component reads | 1× component reads |

## Why We Defer It

**During development:**
- Separate systems are easier to debug
- Behavior changes are isolated
- Tests target specific behaviors
- Faster iteration on individual systems

**At release:**
- Behavior logic is stable
- Performance is critical
- Worth the maintainability cost

## Implementation Notes

When implementing, extract behavior logic into pure functions:

```rust
// Keep these testable and reusable
fn calculate_seek_force(pos: &Position, target: &Target, ...) -> (f32, f32);
fn calculate_wander_force(pos: &Position, state: &WanderState, ...) -> (f32, f32);
fn calculate_avoidance_force(pos: &Position, cache: &NeighborCache, ...) -> (f32, f32);

// Fused system calls all three
pub fn fused_behavior_system(...) {
    entities.par_iter_mut().for_each(|(...) | {
        let (sx, sy) = calculate_seek_force(...);
        let (wx, wy) = calculate_wander_force(...);
        let (ax, ay) = calculate_avoidance_force(...);
        accel.ax += sx + wx + ax;
        accel.ay += sy + wy + ay;
    });
}
```

This preserves testability while gaining the performance benefit.

## When To Do This

- [ ] All behavior systems are feature-complete
- [ ] No active iteration on steering behaviors
- [ ] Targeting release candidate build
- [ ] Profile confirms behavior systems are a bottleneck
