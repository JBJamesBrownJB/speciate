# Rayon Parallelization

**Status:** Done (Sprint 15)
**Location:** `apps/simulation/src/simulation/movement/systems.rs:35-113`

## What It Does

Movement systems use Rayon for multi-core parallel execution instead of single-threaded iteration.

## Why It Exists

Single-threaded ECS iteration was a bottleneck at scale. Rayon enables true multi-core parallel processing of independent entity physics.

## Performance Results

- **6.3x speedup** at 10K creatures (25.9ms → 4.1ms)
- All 16 CPU cores engaged
- IPC (Instructions Per Cycle): 4.25
- Validated at 20K creatures with determinism tests

## Implementation Pattern

```rust
// Collect entities into Vec for Rayon
let mut entities: Vec<_> = query.iter_mut().collect();

// Parallel physics integration (uses all CPU cores)
entities.par_iter_mut().for_each(|(entity, size, position, velocity, ...)| {
    // Physics logic runs in parallel
});

// Parallel boundary enforcement (reuse Vec)
entities.par_iter_mut().for_each(|(position, velocity, ...)| {
    // Boundary clamping in parallel
});
```

## Key Insights

- **Manual Vec collection required:** Bevy's native `par_iter_mut` doesn't engage Rayon in NAPI context
- **Two parallel loops:** Reuse same Vec for efficiency
- **Automatic write-back:** Through mutable references, no explicit sync needed

## Original Backlog Entry

This implements "Parallel ECS Queries" from the optimization backlog:
- Problem: AI and physics systems run single-threaded despite multi-core CPUs
- Solution: Use `par_iter()` for independent entity processing
- Note: Requires careful synchronization to maintain deterministic simulation
