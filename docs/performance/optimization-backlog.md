# Performance Optimization Backlog
**Target:** 150K-200K creatures @ 90 FPS rendering
---
## IPC Optimizations
### Zero-Copy Serialization (FlatBuffers/Cap'n Proto)
**Problem:** MessagePack requires Node.js to allocate memory for buffer read, then allocate objects to decode (copy-and-parse).
**Solution:** Migrate to FlatBuffers or Cap'n Proto for zero-copy reads where binary payload IS the in-memory object.
**Benefits:** Access `creatures[0].x` without decoding entire frame. Reduces Electron-side deserialization overhead.
**Trade-offs:** Requires schema definition file (`.fbs`), code generation step in build. Violates Phase 1 "schema-free" simplicity.
**Timeline:** Later towards release day on steam (when schema stabilizes toward release). MessagePack serialization (3ms) is NOT current bottleneck—IO blocking is.
**Consultant Recommendation:** Stick with MessagePack for Phase 1. Optimize right bottleneck first (background writer thread).
---
## Simulation Optimizations
### ECS Query Filters
**Problem:** Systems iterate ALL entities every frame, even unchanged ones.
**Solution:** Use Bevy `Changed<>` and `With<>` filters to skip static entities.
**Notes:** 25-30% throughput improvement. Maintain determinism for replay/save.
---
### Parallel ECS Queries
**Problem:** AI and physics systems run single-threaded despite multi-core CPUs.
**Solution:** Use `par_iter()` for independent entity processing.
**Notes:** Requires careful synchronization to maintain deterministic simulation.
---
### Memory Layout Optimization
**Problem:** Cache misses from poorly aligned component data.
**Solution:** Add `#[repr(C, align(16))]` for SIMD-friendly cache locality.
**Notes:** Low-level optimization, measure before implementing.
---
### Size-Based Reaction Latency
**Problem:** All creatures react at same 20Hz AI tick rate, ignoring biological size constraints.
**Solution:** Reaction delay derived from body length: 100ms (≤1m) to 1000ms (20m creatures). Creatures commit to decisions for their reaction time.
**Notes:** Enables size-based behavior diversity. Large creatures slower but deliberate. No god-tier builds. Future sprint after dual-tick.
---
### Frontend -> Sim Spatial Indexing communication
**Problem:** We send and render all crits, even if they are not in view of camera
**Solution:** Sim sends camera viewbox to sim, in world coordinates and sim only sends data for crits within view
**Notes:** 10K creatures: 10ms→1ms. Required for 100K+ scale.
---
### Zoom LOD sim payload
**Problem:** We send unecessary info as we zoom out, such as rotation.
**Solution:** Frontend notifies sim when zoom changes and sim reduces payload by removing things like rotation, size, maybe even reduces precision of x,y to just int or something.
**Notes:** 
---
### LOD Rendering
**Problem:** Full sprite detail wasted when zoomed out.
**Solution:** Switch to point sprites at far zoom (< 5 px/m).
**Notes:** Reduces GPU memory 30%. Pairs well with spatial indexing.
---
## Persistence Optimizations
### Incremental Saves
**Problem:** Full world serialization on every save (5-10s at scale).
**Solution:** Only serialize changed entities since last save.
**Notes:** Save time: 5-10s→500ms. Requires dirty entity tracking.
---
### Background Serialization
**Problem:** Save operation blocks main thread, freezes game.
**Solution:** Run save in separate thread (async write).
**Notes:** Non-blocking saves. Electron main process handles async I/O.
---
### Save File Compression
**Problem:** Large save files (10MB+ at scale).
**Solution:** Use gzip/zstd compression on save files.
**Notes:** 50-70% size reduction. bincode already compact for Rust-only saves.
---
### Chunked Loading
**Problem:** Slow startup loading entire world at once.
**Solution:** Load world in chunks (creatures by region).
**Notes:** Load time: 3-5s→1-2s. Progressive loading UX.
---
### Object Pooling ("Ghost" Pool)
**Problem:** Dying and spawning cits cause memory allocator churn.
**Solution:**Recycle dead entities instead of spawn/despawn to prevent memory allocator churn.
**Notes:** Leak Prevention: Automatic cleanup of interpolation history buffers (PreviousPositions) upon entity death