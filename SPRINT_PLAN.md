# Performance Optimization Roadmap

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
