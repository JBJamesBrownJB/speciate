# Ghost Crits Bug

**Status:** ✅ Fixed (Sprint 16)

## Symptoms

When rapidly spawning trials which have a mix of seeking and catatonic crits we get a weird ghosting effect where as you press spawn it looks like many ghost crits flash into view and quickly disappear every time you spawn, like they 'strobe' in and out of existence.

## Root Cause

**Frontend interpolation tracks creatures by array INDEX, not by creature ID.**

### How Interpolation Works

`InterpolationBufferManager` stores position history per-index:
```
[startX₀, startY₀, endX₀, endY₀, startRot₀, endRot₀, size₀, startX₁, ...]
```

Each tick:
1. Old END positions → new START positions (swap in-place by index)
2. New simulation data → END positions
3. GPU interpolates: `lerp(START, END, alpha)`

**Critical assumption:** Creature at index N remains the same creature across ticks.

### Why Spawning Breaks This

Backend export uses Bevy ECS query iteration:
```rust
// apps/simulation/src/ipc/bridge/bevy_app.rs:250
for (i, (id, pos, rot)) in query.iter(world).take(export_count).enumerate() {
```

**Bevy queries do NOT guarantee stable iteration order.** When entities spawn/despawn, archetype tables reorganize and query order changes unpredictably.

**Tick 1:** Backend exports `[CreatureA, CreatureB, CreatureC]` (indices 0, 1, 2)
- Frontend stores interpolation state: A at index 0, B at index 1, C at index 2

**Tick 2:** User spawns new creatures, ECS reorders → `[CreatureD, CreatureA, CreatureC, CreatureB]`
- Index 0 now has CreatureD, but buffer still has A's interpolation START position
- CreatureD inherits CreatureA's START position → **teleports from A's location**
- **Result:** Ghost crits strobing in and out

### Files Involved

**Interpolation (affected):**
- `apps/portal/src/rendering/InterpolationBufferManager.ts` - Index-based tracking

**Export (causing reordering):**
- `apps/simulation/src/ipc/bridge/bevy_app.rs:250` - Iterates ECS query, order not guaranteed

## Fix

Sort exported creatures by CritId before writing to buffer. This ensures stable ordering regardless of ECS archetype changes.

**Location:** `apps/simulation/src/ipc/bridge/bevy_app.rs:222-265`

**Implementation:** Parallel sort using Rayon (`par_sort_unstable_by_key`)

**Performance:** Benchmarked at 1.35ms for 400K creatures (3% of 45ms tick budget)

```rust
// Collect and sort by CritId for stable ordering
let mut entities: Vec<_> = query.iter(world).collect();
entities.par_sort_unstable_by_key(|(id, _, _)| id.0);
```

## Related Issues

This same root cause was discovered during viewport culling implementation (Sprint 16). Backend viewport filtering caused creatures to enter/leave the buffer, triggering the same index mismatch problem.
