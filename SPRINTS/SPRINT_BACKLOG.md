# Sprint 16: Stable Export Ordering + Viewport Culling

**Status:** In Progress
**Branch:** `feat/sprint-16-viewport-culling`

## Goal

1. **Phase 1:** Fix ghost-crits bug by ensuring stable creature export ordering
2. **Phase 2:** Reduce GPU workload with shader-based viewport culling

---

## Phase 1: Stable Export Ordering (Ghost-Crits Fix)

### Problem

Frontend interpolation tracks creatures by array INDEX, not by creature ID. When ECS query iteration order changes (due to spawning/despawning), interpolation state gets misapplied causing ghosting/strobing.

See: `docs/testing/bugs/ghost-crits.md`

### Solution

Sort exported creatures by CritId before writing to buffer. O(n log n) sort per tick, but ensures index stability.

### Ticket 1.1: Sort by CritId in export_positions()

**File:** `apps/simulation/src/ipc/bridge/bevy_app.rs`

```rust
// Collect query results into Vec
let mut entities: Vec<_> = query.iter(world).collect();

// Sort by CritId for stable ordering
entities.sort_unstable_by_key(|(id, _, _)| id.0);

// Write to buffer in sorted order
for (i, (id, pos, rot)) in entities.iter().take(export_count).enumerate() {
    // ...
}
```

### Ticket 1.2: Test Stable Ordering

- Spawn 1000 creatures rapidly
- Verify no ghosting/strobing
- Verify interpolation remains smooth

### Success Criteria (Phase 1)

1. Ghost-crits bug is fixed
2. Rapid spawning causes no visual artifacts
3. Interpolation remains smooth during spawn/despawn

---

## Phase 2: Shader-Based Viewport Culling

### Goal

Reduce GPU workload by not rendering creatures outside the viewport. The simulation still sends all creatures over IPC (which is not a bottleneck), but the GPU discards off-screen fragments.

### Why Shader-Based (Not Backend Culling)

Backend culling was attempted and abandoned:
- Viewport filtering changed which creatures were in the buffer
- Even with stable ordering, creatures entering/leaving caused interpolation discontinuities
- Shader culling keeps all creatures in buffer (indices stable) but skips rendering off-screen ones

### Ticket 2.1: Add Viewport Uniforms to Shader

**File:** `apps/portal/src/rendering/InterpolatedCreatureRenderer.ts`

Add uniforms for viewport bounds:
```typescript
uViewportMin: { value: new Float32Array([minX, minY]) },
uViewportMax: { value: new Float32Array([maxX, maxY]) },
```

### Ticket 2.2: Discard in Fragment Shader

**File:** `apps/portal/src/rendering/InterpolatedCreatureRenderer.ts`

Add early discard for fragments outside viewport:
```glsl
uniform vec2 uViewportMin;
uniform vec2 uViewportMax;

// In fragment shader, after world position is known
if (vWorldPos.x < uViewportMin.x || vWorldPos.x > uViewportMax.x ||
    vWorldPos.y < uViewportMin.y || vWorldPos.y > uViewportMax.y) {
    discard;
}
```

### Ticket 2.3: Pass World Position to Fragment Shader

**File:** `apps/portal/src/rendering/InterpolatedCreatureRenderer.ts`

Ensure interpolated world position is available in fragment shader via varying.

### Ticket 2.4: Update Uniforms from Camera

**File:** `apps/portal/src/main.ts`

Update viewport uniforms each frame from camera bounds:
```typescript
const bounds = camera.getWorldBounds(viewportWidth, viewportHeight);
creatureRenderer.setViewportBounds(bounds.minX, bounds.minY, bounds.maxX, bounds.maxY);
```

### Ticket 2.5: Tests

- Manual test: zoom in, verify off-screen creatures not rendered
- Performance test: measure GPU time reduction at high zoom

### Success Criteria (Phase 2)

1. Zooming in reduces GPU workload (fewer pixels rendered)
2. No visual artifacts (no strobing, ghosting, teleporting)
3. Interpolation remains smooth
4. Smooth performance at 100K+ creatures

---

## Non-Goals (Abandoned)

- ~~Backend viewport filtering~~ - Breaks interpolation (creatures enter/leave buffer)
- ~~Spatial grid for culling~~ - Synchronization issues
- ~~LOD rendering~~ - Over-engineered
- ~~IPC bandwidth reduction~~ - Not actually a bottleneck
