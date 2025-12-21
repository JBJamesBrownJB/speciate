# Performance Optimization Roadmap

## Phase 0: Remove UpdateSlice (Prep)
**Goal:** Clean slate - remove failed tick-skipping approach

Remove `UpdateSlice` component and its usage from all systems (7 files):
- `components/update_slice.rs` - DELETE
- `components/mod.rs` - remove export
- `builder.rs` - remove from CritBundle
- `perception/systems.rs` - remove slice filtering
- `behaviors/transitions/systems.rs` - remove slice filtering
- `simulation.rs` - remove any registration
- `snapshot.rs` - remove from persistence

Systems will run every tick (baseline) until Phase 1 adds new frequency control.

---

## Phase 1: System Frequency Control
**Goal:** Dynamic per-system Hz control with zero overhead at full rate

Uses spatial grid cell bucketing (`cell_index % divisor`) to throttle cognitive systems.

**Details:** [`docs/performance/todo/system-update-frequency.md`](docs/performance/todo/system-update-frequency.md)

---

## Phase 2: Dual Spatial Grid (Sensory Mipmapping)
**Goal:** Decouple perception cost from simulation scale

Introduces L0 (fine, 20m) and L1 (coarse, 100m) grids for hierarchical perception.

**Details:** [`docs/performance/todo/dual-spatial-grid.md`](docs/performance/todo/dual-spatial-grid.md)

---

## Phase 3: L1 Bucketing Migration
**Goal:** Leverage coarse grid for even cheaper frequency control

Migrate system-frequency bucketing from L0 (10m cells) to L1 (100m cells) = 25x fewer cells.

**Details:** See "Future: Dual-Grid Integration" in system-update-frequency.md
