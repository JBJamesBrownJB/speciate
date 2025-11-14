# Cargo Test Fix - Module Import Issue

## Problem

Compilation failed with:
```
error[E0432]: unresolved import `crate::snapshot_queue`
  --> src/simulation/core/simulation.rs:17:12
   |
17 | use crate::snapshot_queue::SharedSnapshotQueue;
   |            ^^^^^^^^^^^^^^
```

## Root Cause

The package has both a library (`lib.rs`) and a binary (`main.rs`). The binary was redeclaring modules that already exist in the library:

```rust
// main.rs (WRONG)
mod simulation;  // Shadows library's simulation module
mod snapshots;   // Shadows library's snapshots module
mod state;       // Shadows library's state module
```

When the binary redeclares these modules, they're compiled in the binary's context where `crate::snapshot_queue` refers to the binary's root (main.rs), not the library's root (lib.rs).

## Solution

Removed module redeclarations from `main.rs` and imported from the library instead:

```rust
// main.rs (CORRECT)
mod config;  // Only binary-specific module

// Import from library
use speciate::{Simulation, SimulationBuilder};
use speciate::simulation::core::timing::TickTimer;
use speciate::snapshots::{SnapshotType, SnapshotWorker, WorldSnapshot};
use speciate::state::SimStateFile;
```

## Files Changed

1. `/workspace/apps/simulation/src/main.rs`:
   - Removed `mod simulation;`, `mod snapshots;`, `mod state;`
   - Changed imports to use `speciate::` prefix

## Test Status

**Library compiles:** Yes
**Tests pass:** All tests passing (128 lib + 7 integration + 14 doc = 149 total)

## Additional Fix

Made `SnapshotQueue` resource optional in `snapshot_system` to support tests that don't need Tauri IPC:

```rust
// Before
queue: Res<SharedSnapshotQueue>,

// After
queue: Option<Res<SharedSnapshotQueue>>,

// Early return if no queue
let Some(queue) = queue else {
    return;
};
```

This allows tests to run without providing the SnapshotQueue resource while still supporting Tauri integration when needed.
