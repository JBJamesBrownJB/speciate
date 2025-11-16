# Performance Optimization Catalog

**Status**: Mix of Implemented and Planned
**Last Updated**: 2025-11-14
**Context**: Electron standalone desktop game (Phase 1)

Catalog of performance optimization strategies for the Speciate simulation system running in the Electron desktop application.

---

## Current Architecture

**Platform:** Electron desktop application (Windows/Mac/Linux)
**Backend:** Rust/Bevy ECS simulation subprocess
  - 30Hz Physics + Collision (dual-tick architecture)
  - 20Hz AI + Perception
  - 200m bucket grid with FxHash (O(N) queries)
**Frontend:** TypeScript/PixiJS renderer (90 FPS interpolated)
**IPC:** stdio MessagePack frames (30 Hz streaming)
**Target Scale:** 150,000-200,000 creatures @ 90 FPS rendering

**See:** `docs/architecture/dual-tick-simulation.md` for complete architecture.

---

## Implemented Optimizations

### Frontend Viewport Culling (Grid Rendering)

**Status:** ✅ Implemented

**Problem:** Rendering full-world grid at all zoom levels was causing performance issues.

**Solution:** Viewport-based culling for grid rendering.

**Implementation Details:**
- Grid only renders lines visible in current viewport bounds
- Grid visibility threshold: Only renders when zoom >= 20 px/m
- Fixed 1m grid spacing for consistent spatial reference
- Uses Camera and Viewport classes to calculate visible world bounds
- Padding added (2× grid spacing) to ensure smooth rendering at viewport edges

**Performance Impact:**
- Rendering cost scales with viewport size, not world size
- At zoomed-out views (< 20 px/m), grid completely disabled
- Eliminates rendering of 2000km × 2000km grid lines when only 100m × 100m viewport visible

**Files:**
- `apps/portal/src/rendering/GridRenderer.ts` - Viewport culling implementation
- `apps/portal/src/domain/Camera.ts` - Camera coordinate system
- `apps/portal/src/domain/Viewport.ts` - Visible bounds calculation
- Tests: 74 tests covering grid rendering, camera, and viewport

**Future Work:**
- Apply same viewport culling strategy to creature sprites when scale increases (10K+ creatures)
- Spatial indexing (quadtree/grid) for creature queries

---

## Simulation Performance Ideas

### ECS Query Optimization

**Current:** Systems iterate all entities even when unchanged.

**Optimizations:**
1. **Query Filters:** Use `Changed<>` and `With<>` to skip static entities
2. **Memory Layout:** Add `#[repr(C, align(16))]` for cache locality and SIMD
3. **Parallel Queries:** Use `par_iter()` for multi-core systems (maintain determinism for replay/save)

**Expected Impact:** 25-30% simulation throughput improvement

**Example:**
```rust
// Before: Iterates ALL creatures every frame
fn movement_system(query: Query<(&mut Position, &Velocity)>) {
    for (mut pos, vel) in query.iter_mut() {
        pos.x += vel.x * dt;
        pos.y += vel.y * dt;
    }
}

// After: Only iterates creatures that moved
fn movement_system(query: Query<(&mut Position, &Velocity), Changed<Velocity>>) {
    for (mut pos, vel) in query.iter_mut() {
        pos.x += vel.x * dt;
        pos.y += vel.y * dt;
    }
}
```

---

### Snapshot Queue Tuning

**Current:** 10-frame buffer (111ms @ 90 FPS)

**Optimization Areas:**
1. **Adaptive Capacity:** Increase buffer during frontend lag, decrease when stable
2. **Compression:** Use MessagePack instead of JSON for snapshot serialization
3. **Selective Snapshots:** Only include creatures in viewport bounds (requires camera state in Rust)

**Expected Impact:**
- 40-60% reduction in IPC payload size (viewport culling)
- Smoother rendering during temporary frontend stalls

---

### Creature Sprite Pooling

**Status:** Partially implemented

**Problem:** Creating/destroying PixiJS sprites every frame causes GC pressure at high creature counts.

**Solution:** Reuse sprite pool, hide unused sprites instead of destroying.

**Implementation:**
- Maintain pool of pre-allocated sprites
- Show/hide sprites based on visible creatures
- Reset sprite properties instead of recreating

**Expected Impact:**
- 70-80% fewer allocations
- More stable frame times at 10K+ creatures
- Reduced GC pauses

---

### Save/Load Optimization

**Current:** Full world serialization on every save

**Optimizations:**
1. **Incremental Saves:** Only serialize changed entities since last save
2. **Background Serialization:** Run save in separate thread (Electron main process async)
3. **Compression:** Use gzip/zstd for save file compression
4. **Chunked Loading:** Load world in chunks (creatures by region) for faster startup

**Expected Impact:**
- Save time: 5-10s → 500ms-1s (incremental)
- Load time: 3-5s → 1-2s (chunked + decompression)
- Save file size: 50-70% reduction (compression)

---

## Rendering Performance Ideas

### Spatial Indexing for Culling

**Problem:** Viewport culling requires iterating ALL creatures to find visible ones.

**Solution:** Spatial grid/quadtree for O(log n) viewport queries.

**Implementation:**
```typescript
// Current: O(n) - checks every creature
const visible = creatures.filter(c => viewport.contains(c.x, c.y));

// Optimized: O(log n) - only checks nearby cells
const cells = spatialGrid.getCellsInViewport(viewport);
const visible = cells.flatMap(cell => cell.creatures);
```

**Expected Impact:**
- 10K creatures: 10ms → 1ms viewport query
- 50K creatures: 50ms → 2-3ms viewport query
- Enables 100K+ creature scale

---

### PixiJS Batching & Instancing

**Current:** Each creature is individual sprite draw call

**Optimizations:**
1. **Sprite Batching:** Group creatures by texture (Pixi auto-batches same texture)
2. **Mesh Instancing:** Use ParticleContainer for static sprites
3. **LOD System:** Switch to point sprites at far zoom (< 5 px/m)

**Expected Impact:**
- Draw calls: 10K → 50-100 (batching)
- GPU memory: 30% reduction (instancing)
- FPS: +20-30% at 10K+ creatures

---

## Serialization Format Comparison

**For save files and IPC payloads:**

| Format | Size | Speed | Use Case |
|--------|------|-------|----------|
| **JSON** | 10.0 MB | Baseline | Debug, human-readable |
| **MessagePack** | 2.8-3.5 MB | ~2x faster | **Recommended for IPC** |
| **bincode** | 2.4-2.8 MB | ~3-5x faster | **Recommended for save files** (Rust-only) |
| **Protobuf** | 2.5-3.2 MB | ~2.5x faster | Schema-based, future-proofing |

**Current Choice:**
- IPC: MessagePack (fast + compact)
- Save files: Can use bincode (Rust ↔ Rust only, no browser needed)

---

## Monitoring & Benchmarking

**Performance Metrics to Track:**
```bash
# Rust simulation benchmarks
cargo bench --bench systems         # ECS system performance
cargo flamegraph --bin speciate     # CPU profiling

# Frontend profiling (Chrome DevTools)
- FPS stability (target: locked 90 FPS with interpolation)
- Frame budget breakdown (JS vs GPU vs idle)
- Memory usage (GC pauses < 5ms)
```

**Benchmark Targets:**
- Entity iteration: < 100μs per 1000 creatures
- Snapshot serialization: < 2ms per frame @ 10K creatures
- IPC round-trip: < 5ms (Rust → PixiJS)
- Save file write: < 1s @ 10K creatures
- Save file load: < 2s @ 10K creatures

---

## Key Insights

1. **Viewport culling** provides the biggest win for rendering (70-90% reduction in draw calls)
2. **ECS query filters** are low-hanging fruit (25-30% simulation speedup)
3. **Sprite pooling** essential for stable frame times at 10K+ creatures
4. **Spatial indexing** required to scale beyond 50K creatures
5. **Save/load optimization** critical for user experience (fast startup)

---

## Priority Order

**Phase 1 (Current - 150K creatures):**
- ✅ Viewport culling (grid)
- ✅ Lock-free snapshot queue
- ✅ Dual-tick architecture (30Hz physics, 20Hz AI)
- ✅ 200m bucket grid with FxHash
- Sprite pooling

**Phase 2 (Target - 200K creatures):**
- ECS query optimization (Changed<>, With<>)
- Viewport culling (creatures)
- Save/load optimization
- Parallel queries (par_iter)

**Phase 3 (Stretch - 1M creatures):**
- LOD simulation (near: full, far: statistical)
- Spatial indexing refinements
- PixiJS batching & instancing
- GPU compute shaders
