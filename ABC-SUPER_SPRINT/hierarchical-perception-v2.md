# Hierarchical Perception v2: Range-Band FOV Model

**Status**: 🔄 IN PROGRESS - Current Priority
**Builds On:** Phase A (Dual Grid) - L0/L1 infrastructure complete
**Enables:** 500K+ creatures via range-appropriate resolution

---

## ✅ Progress Log

### Phase 1: L2 Grid Infrastructure - COMPLETE (2025-12-30)

**Implemented:**
- `L2_CELL_SIZE = 180.0` constant (3× L1, following the hierarchy pattern)
- Extended `CoarseGrid` with L2 storage (`l2_cells`, `l2_prev_non_empty`, etc.)
- Added L2 methods: `clear_l2()`, `position_to_l2_cell_index()`, `l1_to_l2_cell_index()`
- Added `BioSignature::merge()` for proper L1→L2 aggregation
- Added `merge_to_l2_cell()` for aggregation
- Added `aggregate_l2()` to `HierarchicalGrid`
- Added `aggregate_l2_system()` with `l2_aggregation` timing instrumentation
- System registered in schedule: L0 rebuild → L1 agg → **L2 agg** → perception
- All 45 spatial tests pass

**Files Modified:**
- `src/simulation/spatial/constants.rs` - L2_CELL_SIZE constant
- `src/simulation/spatial/biosignature.rs` - merge() method
- `src/simulation/spatial/coarse_grid.rs` - L2 storage and methods
- `src/simulation/spatial/hierarchical.rs` - aggregate_l2()
- `src/simulation/spatial/systems.rs` - aggregate_l2_system()
- `src/simulation/spatial/mod.rs` - exports
- `src/simulation/core/simulation.rs` - schedule registration
- `src/instrumentation/mod.rs` - l2_aggregation_us timing

**Next:** Phase 2 - L2Vision component

---

## Core Concept: Range Bands (Not Nested Early-Exit)

Each grid level handles a **specific range band**, not nested areas:

```
                         ┌─────────────────────────────────────────────┐
                         │              L2 OUTER RING                  │
                         │    (180m+ range, strategic directions)      │
                         │        ┌─────────────────────────┐          │
                         │        │       L1 MID RING       │          │
                         │        │  (60-180m, cell-level)  │          │
                         │        │     ┌─────────────┐     │          │
                         │        │     │  L0 CORE    │     │          │
                         │        │     │  (0-60m,    │     │          │
                         │        │     │  entities)  │     │          │
                         │        │     └─────────────┘     │          │
                         │        └─────────────────────────┘          │
                         └─────────────────────────────────────────────┘
```

**Why range bands instead of nested early-exit?**
- You don't need entity-level detail at 200m
- "Threat in that direction" is sufficient for strategic navigation
- Each level has one clear job
- Matches biological perception (detail near, blur far)

---

## Level Specifications

| Level | Cell Size | Range Band | Returns | Purpose |
|-------|-----------|------------|---------|---------|
| **L0** | 20m | 0-60m | **Entities** → NeighborCache | Collision avoidance, immediate threats |
| **L1** | 60m | 60-180m | **Cell classifications** → L1Vision | Mid-range awareness, flee/approach decisions |
| **L2** | 180m | 180m+ | **Strategic directions** → L2Vision | Long-range navigation, herd detection |

### What Each Level Provides

```
L0 (Entity Detail, 0-60m):
  - Individual neighbor positions, velocities, sizes
  - Used by: Steering, TTC avoidance, targeting
  - Output: NeighborCache (max 7 entities)

L1 (Cell Awareness, 60-180m):
  - Classification per cell: Empty, Threat, Prey, Crowded
  - Direction to cell center
  - Used by: Drive system, flee/hunt decisions
  - Output: L1Vision (classifications + directions)

L2 (Strategic, 180m+):
  - Aggregate threat/prey mass in direction
  - Used by: Long-range navigation, migration
  - Output: L2Vision (strategic directions only)
```

---

## Grid Hierarchy (Physical Layout)

```
L2 (180m cells)
┌─────────────────────────────────────────────────────────────────┐
│                           L2 Cell                               │
│  ┌───────────────────┬───────────────────┬───────────────────┐  │
│  │   L1 Cell (60m)   │   L1 Cell (60m)   │   L1 Cell (60m)   │  │
│  │  ┌─────┬─────┬─────┐  ┌─────┬─────┬─────┐  ...             │  │
│  │  │ L0  │ L0  │ L0  │  │ L0  │ L0  │ L0  │                  │  │
│  │  ├─────┼─────┼─────┤  ├─────┼─────┼─────┤  (L0 = 20m)      │  │
│  │  │ L0  │ L0  │ L0  │  │ L0  │ L0  │ L0  │                  │  │
│  │  ├─────┼─────┼─────┤  ├─────┼─────┼─────┤                  │  │
│  │  │ L0  │ L0  │ L0  │  │ L0  │ L0  │ L0  │                  │  │
│  │  └─────┴─────┴─────┘  └─────┴─────┴─────┘                  │  │
│  └───────────────────┴───────────────────┴───────────────────┘  │
│                                                                  │
│  (3×3 L1 cells = 9 L1 cells per L2 cell)                       │
│  (3×3 L0 cells per L1 cell = 81 L0 cells per L2 cell)          │
└─────────────────────────────────────────────────────────────────┘
```

---

## Cell Polling Patterns

### Base 3×3 Grid (All Levels)

Every level uses the same 3×3 grid pattern, indexed by the creature's current cell at that level:

```
     -1    0    +1
    ┌────┬────┬────┐
 +1 │ NW │ N  │ NE │   dy = +1
    ├────┼────┼────┤
  0 │ W  │ ●  │ E  │   dy = 0  (● = creature's cell)
    ├────┼────┼────┤
 -1 │ SW │ S  │ SE │   dy = -1
    └────┴────┴────┘
      dx = -1  0  +1
```

### FOV-Based Culling (Octant + FOV Bucket)

The creature's facing direction quantizes to 8 octants:
```
         NW  N  NE
           ╲ │ ╱
        W ──●── E      Octants: E=0, NE=1, N=2, NW=3, W=4, SW=5, S=6, SE=7
           ╱ │ ╲
         SW  S  SE
```

FOV determines which cells to cull (rear cells behind creature):

```
NARROW FOV (<125°) - Cull 3 cells:         MEDIUM FOV (125-215°) - Cull 1 cell:

East-facing example:                        East-facing example:
    ┌────┬────┬────┐                           ┌────┬────┬────┐
    │ ░░ │ ✓  │ ✓  │                           │ ✓  │ ✓  │ ✓  │
    ├────┼────┼────┤                           ├────┼────┼────┤
    │ ░░ │ ● →│ ✓  │   ░░ = culled             │ ░░ │ ● →│ ✓  │
    ├────┼────┼────┤                           ├────┼────┼────┤
    │ ░░ │ ✓  │ ✓  │                           │ ✓  │ ✓  │ ✓  │
    └────┴────┴────┘                           └────┴────┴────┘

WIDE FOV (≥215°) - Cull 0 cells:

    ┌────┬────┬────┐
    │ ✓  │ ✓  │ ✓  │
    ├────┼────┼────┤
    │ ✓  │ ● →│ ✓  │   All 9 cells queried
    ├────┼────┼────┤
    │ ✓  │ ✓  │ ✓  │
    └────┴────┴────┘
```

### FOV-Tier Extended Cells (+2 beyond 3×3)

Based on creature archetype (predator vs prey):

```
NARROW FOV (Predator) - +2 cells FORWARD:

    ┌────┬────┬────┐
    │    │    │    │
    ├────┼────┼────┬────┬────┐
    │    │ ● →│    │ ▓▓ │ ▓▓ │   ▓▓ = extended cells at (2,0) and (3,0)
    ├────┼────┼────┴────┴────┘
    │    │    │    │
    └────┴────┴────┘
          └──────────────────→ Depth hunting zone


WIDE FOV (Prey) - +2 cells PERPENDICULAR:

              ┌────┐
              │ ▓▓ │   ▓▓ = extended cell at (0,+2)
    ┌────┬────┼────┼────┬────┐
    │    │    │    │    │    │
    ├────┼────┼────┼────┼────┤
    │    │    │ ● →│    │    │
    ├────┼────┼────┼────┼────┤
    │    │    │    │    │    │
    └────┴────┼────┼────┴────┘
              │ ▓▓ │   ▓▓ = extended cell at (0,-2)
              └────┘
              │
              ↓ Panoramic threat detection
```

---

## Execution Flow (Range-Band Model)

### Overview

```
Creature with perception_range = 250m

┌──────────────────────────────────────────────────────────────────┐
│  STEP 1: L0 SCAN (always, 0-60m)                                 │
│    - 3×3 L0 cells + FOV culling                                  │
│    - Entity iteration with size domination                       │
│    - Output: NeighborCache (max 7 entities)                      │
├──────────────────────────────────────────────────────────────────┤
│  STEP 2: L1 SCAN (if perception > 60m, covers 60-180m ring)      │
│    - 3×3 L1 cells + FOV culling                                  │
│    - EXCLUDE L1 cells already covered by L0 (center cell)        │
│    - BioSignature classification per cell                        │
│    - Output: L1Vision (classifications + directions)             │
├──────────────────────────────────────────────────────────────────┤
│  STEP 3: L2 SCAN (if perception > 180m, covers 180m+ ring)       │
│    - 3×3 L2 cells + FOV culling + extended cells                 │
│    - EXCLUDE L2 cells already covered by L1 (center cell)        │
│    - BioSignature classification per cell                        │
│    - Output: L2Vision (strategic directions)                     │
└──────────────────────────────────────────────────────────────────┘
```

### Step 1: L0 Scan (Always, 0-60m)

```
For each L0 cell in FOV pattern (3×3 around creature):
┌────────────────────────────────────────────────────────────────┐
│                                                                 │
│   For each entity in L0 cell:                                   │
│      - Skip if entity == self                                   │
│      - Skip if outside FOV cone                                 │
│      - Skip if entity.mass < my_threshold (size domination)     │
│      - Add to neighbor candidates                               │
│                                                                 │
│   After all L0 cells:                                           │
│      Select K=7 closest neighbors → NeighborCache               │
│                                                                 │
└────────────────────────────────────────────────────────────────┘
```

### Step 2: L1 Scan (60-180m Ring)

```
If perception_range > 60m:
┌────────────────────────────────────────────────────────────────┐
│                                                                 │
│   Get L1 cells in FOV pattern (3×3 around creature's L1 cell)   │
│   EXCLUDE: center L1 cell (already covered by L0 scan)          │
│                                                                 │
│   For each L1 cell in ring:                                     │
│      - Get BioSignature                                         │
│      - Classify: Empty, Threat, Prey, Crowded                   │
│      - Calculate direction to cell center                       │
│      - Store in L1Vision                                        │
│                                                                 │
└────────────────────────────────────────────────────────────────┘
```

### Step 3: L2 Scan (180m+ Ring)

```
If perception_range > 180m:
┌────────────────────────────────────────────────────────────────┐
│                                                                 │
│   Get L2 cells in FOV pattern (3×3 + extended cells)            │
│   EXCLUDE: center L2 cell (already covered by L1 scan)          │
│                                                                 │
│   For each L2 cell in ring:                                     │
│      - Get BioSignature (aggregate of child L1 cells)           │
│      - Classify: Empty, Threat, Prey, Crowded                   │
│      - Calculate direction to cell center                       │
│      - Store in L2Vision                                        │
│                                                                 │
└────────────────────────────────────────────────────────────────┘
```

---

## BioSignature Aggregation

### L1 BioSignature (existing)
```
L1 cell biosig = sum of entity biosigs in all 9 child L0 cells

For each entity in L1 cell:
  biosig.total_mass += entity.mass
  biosig.count += 1
  biosig.max_mass = max(biosig.max_mass, entity.mass)
  biosig.min_mass = min(biosig.min_mass, entity.mass)
```

### L2 BioSignature (new)
```
L2 cell biosig = sum of 9 child L1 biosigs

For each L1 child:
  l2_biosig.total_mass += l1_biosig.total_mass
  l2_biosig.count += l1_biosig.count
  l2_biosig.max_mass = max(l2_biosig.max_mass, l1_biosig.max_mass)
  l2_biosig.min_mass = min(l2_biosig.min_mass, l1_biosig.min_mass)
```

### Classification Logic (All Levels)

```rust
fn classify_cell(biosig: &BioSignature, my_mass: f32, my_size: f32) -> Classification {
    let threshold = my_mass * PERCEPTION_THRESHOLD_FRACTION; // 0.05

    if biosig.count == 0 || biosig.total_mass < threshold {
        Classification::Empty
    } else if biosig.max_size > my_size {
        Classification::Threat
    } else if biosig.max_size < my_size * PREY_SIZE_RATIO { // 0.3
        Classification::Prey
    } else {
        Classification::Crowded
    }
}
```

---

## Portal Overlay Updates

### Grid Visualization (G Key Cycling)

Extend current G key cycling to include L2:

```
Current:  Off → L0 → L1 → (back to Off)
New:      Off → L0 → L1 → L2 → (back to Off)
```

| Mode | What's Displayed |
|------|------------------|
| Off | No grid overlay |
| L0 | 20m grid lines, entity dots |
| L1 | 60m grid lines, cell heatmap (total_mass) |
| L2 | 180m grid lines, cell heatmap (total_mass) |

### L2 Grid Rendering

**File:** `apps/portal/src/rendering/overlays/SpatialGridOverlay.ts`

```typescript
// Add L2 grid mode
enum GridOverlayMode {
  Off = 0,
  L0 = 1,
  L1 = 2,
  L2 = 3,  // NEW
}

// L2 cell rendering (same pattern as L1, just larger)
if (mode === GridOverlayMode.L2) {
  const L2_CELL_SIZE = 180;
  // Draw 180m grid lines
  // Color cells by biosig.total_mass (heatmap)
}
```

### L1/L2 Vision Lines (Selected Creature)

When a creature is selected AND grid overlay is L1 or L2:

```
┌─────────────────────────────────────────────────────────────────┐
│  Selected creature draws lines to perceived cells:              │
│                                                                  │
│  L1 Mode (G pressed twice):                                     │
│    - Lines from creature → L1 cell centers (60-180m ring)       │
│    - Color by classification:                                    │
│        Red    = Threat                                          │
│        Orange = Prey                                            │
│        Yellow = Crowded                                         │
│        Green  = Empty                                           │
│                                                                  │
│  L2 Mode (G pressed three times):                               │
│    - Lines from creature → L2 cell centers (180m+ ring)         │
│    - Same color coding as L1                                    │
│    - Thicker/longer lines (strategic range)                     │
│                                                                  │
└─────────────────────────────────────────────────────────────────┘
```

### Hover Info Panel Updates

Extend existing L1 hover to support L2:

**L1 Hover (existing):**
```
L1 Cell (3, 5)
─────────────
Creatures: 12
Total Mass: 847.3
Max Size: 4.2m
```

**L2 Hover (new):**
```
L2 Cell (1, 2)
─────────────
Creatures: 89
Total Mass: 6,234.7
Max Size: 8.1m
L1 Children: 9
```

---

## IPC Updates

### New Buffer: L2 Grid Data

Similar to existing L1 buffer pattern:

**Rust (simulation_engine.rs):**
```rust
#[napi]
pub fn fill_l2_grid_buffer(&self, mut buffer: Float32Array) -> i32 {
    // For each L2 cell: [total_mass, max_size, creature_count]
    // Same pattern as L1 buffer
}
```

### Selected Creature Vision Data

Extend existing selected creature IPC to include L1Vision and L2Vision:

```rust
#[napi]
pub fn get_selected_creature_vision(&self) -> Option<CreatureVisionData> {
    // Returns:
    // - neighbors: Vec<NeighborData> (existing)
    // - l1_vision: Vec<L1VisionEntry> (cell_idx, classification, direction)
    // - l2_vision: Vec<L2VisionEntry> (cell_idx, classification, direction)
}
```

---

## Implementation Checklist

### Step 1: L2 Grid Infrastructure (Rust) ✅ COMPLETE
- [x] Add `L2_CELL_SIZE = 180.0` constant
- [x] Add `l2_cells: Vec<BioSignature>` to `CoarseGrid`
- [x] Add `l1_to_l2_cell_index()` mapping function
- [x] Add `aggregate_l2()` to reduce L1 → L2
- [x] Add `aggregate_l2_system()` with timing instrumentation
- [x] Register system in schedule (after L1 agg, before perception)
- [ ] Add `fill_l2_grid_buffer()` NAPI function (Phase 4)

### Step 2: Range-Band Perception (Rust)
- [ ] Add L2Vision component
- [ ] Refactor perception to use range-band model
- [ ] L0: 0-60m entity scan (existing, unchanged)
- [ ] L1: 60-180m ring scan (exclude center L1 cell)
- [ ] L2: 180m+ ring scan (exclude center L2 cell)
- [ ] Populate L1Vision component
- [ ] Populate L2Vision component
- [ ] Add `get_selected_creature_vision()` NAPI function

### Step 3: Portal Grid Overlay (TypeScript)
- [ ] Add L2 mode to `GridOverlayMode` enum
- [ ] Extend G key cycling: Off → L0 → L1 → L2 → Off
- [ ] Render L2 grid lines (180m spacing)
- [ ] Render L2 heatmap (same pattern as L1)
- [ ] Add L2 hover info panel

### Step 4: Portal Vision Lines (TypeScript)
- [ ] Add L1Vision line rendering (selected creature + L1 mode)
- [ ] Add L2Vision line rendering (selected creature + L2 mode)
- [ ] Color-code lines by classification
- [ ] Only render for selected creature (performance)

### Step 5: IPC Integration
- [ ] Add `onL2GridUpdate` handler in preload
- [ ] Add `onCreatureVisionUpdate` handler in preload
- [ ] Wire up main process to send L2 data
- [ ] Wire up main process to send vision data for selected creature

### Step 6: Tests
- [x] Unit tests: L2 biosig aggregation (hierarchical.rs)
- [ ] Unit tests: Range-band cell selection (exclude center)
- [ ] Unit tests: Classification at each level
- [ ] Integration: Neighbors still found (avoidance MUST work!)
- [ ] Integration: Vision lines render correctly
- [ ] Benchmark: Verify performance improvement at scale

---

## Validation Checklist

### Infrastructure
- [ ] L2 grid displays correctly (G key → L2 mode)
- [ ] L2 heatmap shows mass distribution
- [ ] L2 hover shows correct aggregate data

### Perception
- [ ] L0 still finds neighbors for steering (critical!)
- [ ] L1 ring excludes center cell (no double-counting)
- [ ] L2 ring excludes center cell (no double-counting)
- [ ] Classification correct at all levels

### Visualization
- [ ] Selected creature shows L1 vision lines (L1 mode)
- [ ] Selected creature shows L2 vision lines (L2 mode)
- [ ] Line colors match classification
- [ ] No performance regression with vision lines

### Performance
- [ ] 360K creatures stable
- [ ] L2 aggregation < 0.5ms
- [ ] No IPC bottleneck from new buffers
