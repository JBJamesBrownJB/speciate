# Mini-Win: MiMalloc + Zero-Allocation Polling

**Date:** 2025-12-07
**Commit:** `b238bba`
**Branch:** `feat/sprint-16-lod-ai-framework`

## The Discovery

While investigating a memory leak in the NAPI buffer polling system, we accidentally achieved a **2x performance improvement** across the board.

## What We Changed

### 1. MiMalloc Global Allocator

```rust
// apps/simulation/src/lib.rs
use mimalloc::MiMalloc;

#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;
```

Replaced glibc's default malloc with Microsoft's high-performance allocator.

### 2. Zero-Allocation Buffer Filling

**Before:** Rust allocated a new `Float32Array` every poll (60Hz = 60 allocations/second per buffer)
```javascript
const fullBuffer = simulationEngine.getBuffer();  // Allocates in Rust, V8 never GC'd properly
```

**After:** JavaScript owns persistent buffers, Rust just fills them
```javascript
// Created once at startup
creatureBuffer = new Float32Array(250000 * 4);  // 4MB, reused forever

// Every poll: zero allocations
const count = simulationEngine.fillBuffer(creatureBuffer);
```

### 3. Two-Pass ECS Iteration

**Before:** Allocated a temporary Vec every tick
```rust
let creatures: Vec<_> = query.iter(world).collect();
```

**After:** Count first, then iterate directly to buffer
```rust
let entity_count = query.iter(world).count();
for (i, (id, pos, rot)) in query.iter(world).take(export_count).enumerate() {
    write_slice[i] = id.0 as f32;
    // ...
}
```

## Why 2x Speedup?

**MiMalloc is the dominant factor.** The simulation is allocation-heavy:

| Allocation Source | Frequency | MiMalloc Benefit |
|-------------------|-----------|------------------|
| Bevy ECS query iterators | Every system run | Thread-local heaps |
| Rayon parallel work-stealing | Every parallel loop | No malloc lock contention |
| JSON telemetry serialization | Every 30 ticks | Faster small string allocs |
| Vec operations in systems | Constantly | Better cache locality |

The system allocator (glibc malloc) has significant **locking overhead** in multi-threaded contexts. With Rayon engaging all 16 CPU cores for movement systems, threads were contending for the global malloc lock.

MiMalloc's design gives each thread its own heap segment, eliminating this contention entirely.

## Performance Evidence

From the commit message and session observations:
- Tick times dropped ~50% at equivalent creature counts
- Memory usage stabilized (leak fixed)
- No increase in CPU usage despite faster simulation

## Key Takeaways

1. **Allocator choice matters enormously** for multi-threaded ECS workloads
2. **V8's GC doesn't handle NAPI-allocated TypedArrays well** - let JS own the buffers
3. **Avoid temporary collections** when you can iterate directly to output buffers

## Files Changed

- `apps/simulation/Cargo.toml` - Added `mimalloc = "0.1"`
- `apps/simulation/src/lib.rs` - Set global allocator
- `apps/simulation/src/napi_addon/simulation_engine.rs` - Added `fillBuffer()`, `fillPerceptionDebug()`
- `apps/simulation/src/ipc/bridge/bevy_app.rs` - Two-pass iteration
- `apps/portal/electron/napi-main.cjs` - Persistent buffer management
