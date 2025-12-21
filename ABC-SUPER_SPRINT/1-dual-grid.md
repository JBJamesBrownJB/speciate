# Sprint: Dual Spatial Grid (Phase A)

## Status: IN PROGRESS - BUG IDENTIFIED

Phase A infrastructure is implemented, but **perception threshold (size domination) is NOT working as expected**.

---

## Known Bug: L1 Perception Threshold Not Filtering Targets

**Observed behavior:** A 5m crit (threshold = 219kg) perceives a 0.5m crit (mass = 4.375kg) even when they are 2+ L1 cells apart.

**Expected behavior:** The 5m crit should NOT perceive the 0.5m crit because 4.375kg << 219kg threshold.

**Root cause (suspected):** The L1 early-exit check (Step 6) only checks the **perceiver's current L1 cell**, not the target's cell. Since the perceiver's own mass (4375kg) always exceeds its threshold (219kg), perception runs fully and detects ALL creatures in range regardless of their mass.

**Fix needed:** The L1 threshold check must filter targets per-cell during the perception scan, not just as an early-exit for the perceiver's cell.

**Test trials created:** `apps/simulation/specs/behavior/l1-*.toml` (6 files for visual verification)

---

## Outcome

Infrastructure-only. No behavior changes. Foundation for Phase C (frequency control) and Phase B (Drive Simplex).

### Deliverables

1. **L1 Coarse Grid** - 30m cells (3×3 L0) with aggregated BioSignatures ✓
2. **L1 Aggregation System** - Reduces L0 → L1 every tick ✓
3. **Portal Visualization** - G key cycles: Off → L0 → L1 ✓
4. **L1 Cell Query** - Hover-based cell info panel (replaces heatmap streaming) ✓
5. **Validation** - Confirm 9 L0 cells covers max avoidance range ✓

### Non-Goals (deferred)

- Crits using L1 for steering (Phase B: Drive Simplex)
- Frequency control bucketing (Phase C)
- 3-phase perception pipeline (Phase B)

---

## Architecture

### Grid Sizes

| Grid | Cell Size | Total Cells | Content |
|------|-----------|-------------|---------|
| L0 (Fine) | 10m | 1,000,000 | Entity IDs |
| L1 (Coarse) | 30m (3×3 L0) | ~111,000 | BioSignatures |

**Rationale:** Max crit size is 5m, so 10m L0 cells are fine. L1 = 3×3 L0 = 30m.

### L1 BioSignature (Minimal)

```rust
pub struct BioSignature {
    pub total_mass: f32,      // Sum of all creature mass in this L1 cell
    pub max_size: f32,        // Largest creature (for future threat assessment)
    pub creature_count: u16,  // Number of creatures (cheap, useful for empty detection)
}
```

**Future additions** (when needed): herbivore_mass, carnivore_mass, center_of_mass

### Perception Threshold (Size Domination)

**Emergent behavior:** Large crits don't see small ones, but small ones see large ones.

```rust
// Add to Perception component
pub perception_threshold: f32,  // = body_mass * 0.05 (5% of own mass)
```

**How it works:**
- Giant (mass 1000) → threshold = 50
- Mouse (mass 1) → below Giant's threshold → Giant doesn't "see" Mouse
- But Mouse sees Giant (Giant's mass >> Mouse's threshold)

**Result:** Mouse must get out of Giant's way. Giant walks straight through.

This creates asymmetric avoidance without explicit "size domination" code.

---

## Implementation

### Step 1: BioSignature Struct
**File:** `spatial/biosignature.rs` (new)

### Step 2: CoarseGrid (L1)
**File:** `spatial/coarse_grid.rs` (new)
- 100m cell size
- Stores `BioSignature` per cell
- Methods: `get_biosignature(cell_idx)`, `clear()`, `non_empty_cells()`

### Step 3: HierarchicalGrid Wrapper
**File:** `spatial/hierarchical.rs` (new)
```rust
pub struct HierarchicalGrid {
    pub l0: DoubleBufferedSpatialGrid,
    pub l1: CoarseGrid,
}
```

### Step 4: L1 Aggregation System
**File:** `spatial/systems.rs`

Runs every tick after L0 rebuild:
1. Clear all L1 cells
2. For each non-empty L0 cell, aggregate into parent L1 cell
3. Sum `total_mass`, track `max_size`, increment `creature_count`

**Design for Rayon:** Structure data for parallel aggregation where possible.

### Step 5: Add Perception Threshold
**File:** `perception/components.rs`

Add to Perception component:
```rust
pub perception_threshold: f32,  // = body_mass * 0.05
```

Initialize in `Perception::from_body_size()` based on creature mass.

### Step 6: Early-Exit + Size Domination
**File:** `perception/systems.rs`

Before L0 scan, check if creature's L1 cell has visible mass:
```rust
let l1_cell = grid.l1.get_cell(pos.x, pos.y);
let threshold = perception.threshold;  // body_mass * 0.05

if l1_cell.total_mass < threshold {
    // Cell is "invisible" to this creature (empty or only tiny crits)
    neighbor_cache.clear();
    return;
}
// Otherwise proceed with normal L0 scan
```

**Two benefits:**
1. **Perf:** Skip L0 scan when cell is empty
2. **Emergent:** Large crits skip cells with only small crits (size domination)

Benchmark with/without to validate perf improvement.

### Step 7: Telemetry IPC
Add L1 cell size and bounds to telemetry stream for portal.

### Step 8: Portal Overlay
**File:** `portal/src/rendering/overlays/SpatialGridOverlay.ts`

G key cycles: Off → L0 grid → L1 grid → Off

### Step 9: L1 Cell Query (Hover Info)
**Files:**
- `simulation/src/ipc/sim_command.rs` - L1CellInfo struct, QueryL1Cell command
- `simulation/src/ipc/bridge/bevy_app.rs` - calculate_l1_cell_info(), command handler
- `simulation/src/napi_addon/simulation_engine.rs` - query_l1_cell() NAPI method
- `portal/electron/napi-main.cjs` - IPC handler
- `portal/electron/preload.cjs` - queryL1Cell bridge
- `portal/src/rendering/overlays/SpatialGridOverlay.ts` - Hover detection, info panel

**What it does:**
- Mouse hover in L1 mode triggers throttled query (100ms)
- Backend returns: cell coords, creature count, total mass (kg), avg size (m), max size (m)
- Portal displays info panel with cell data
- Cell under mouse is highlighted

**Key calculations:**
- `max_size = bio.max_size * 2.0` (radius → length)
- `avg_size = (avg_mass / DEFAULT_MASS)^(1/3)` (mass → length)

**Replaced:** L1 heatmap streaming (caused FPS drops at high creature counts)

---

## Validation

- [x] L1 aggregation produces correct totals (sum of L0 masses)
- [x] Portal shows both grids (G key cycling works)
- [x] L1 hover query shows correct cell info (creatures, mass, sizes)
- [x] Performance: L1 aggregation < 1ms at 20K creatures
- [x] Benchmark: Early-exit optimization improves perception latency
- [x] Unit tests: L1CellInfo calculations (5 tests in bevy_app.rs)
- [ ] Size domination: Large crit walks through small crits (doesn't avoid) *(Phase B)*
- [ ] Size domination: Small crit avoids/flees from large crit *(Phase B)*
- [x] Confirm: 9 L0 cells (30m) covers max avoidance range for 5m crits

---

## Decisions Made

1. **L0 cell size:** Keep 10m (max crit = 5m, so 10m is fine)
2. **L1 cell size:** 30m (3×3 L0 cells)
3. **BioSignature:** Minimal - `total_mass` + `max_size` + `creature_count` (no herb/carni yet)
4. **Early-exit:** Yes, test L1 emptiness check before L0 scan for perf
5. **Rayon-ready:** Design grid structures for parallel aggregation

---

## Files Created/Modified

| File | Change |
|------|--------|
| `spatial/biosignature.rs` | NEW: BioSignature struct ✓ |
| `spatial/coarse_grid.rs` | NEW: L1 grid ✓ |
| `spatial/hierarchical.rs` | NEW: Wrapper ✓ |
| `spatial/systems.rs` | NEW: Aggregation system ✓ |
| `spatial/mod.rs` | Export new modules ✓ |
| `core/simulation.rs` | Replace grid resource ✓ |
| `napi_addon/telemetry.rs` | Add L1 data ✓ |
| `portal/.../SpatialGridOverlay.ts` | Grid cycling + L1 hover query ✓ |
| `ipc/sim_command.rs` | L1CellInfo struct, QueryL1Cell command ✓ |
| `ipc/bridge/bevy_app.rs` | calculate_l1_cell_info(), handler ✓ |
| `napi_addon/simulation_engine.rs` | query_l1_cell() NAPI method ✓ |
| `portal/electron/napi-main.cjs` | IPC handler ✓ |
| `portal/electron/preload.cjs` | queryL1Cell bridge ✓ |
| `portal/src/global.d.ts` | TypeScript types ✓ |
