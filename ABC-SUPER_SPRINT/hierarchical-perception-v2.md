# Hierarchical Perception v2: Multi-Level FOV Patterns

**Status**: DESIGN REVIEW

## Cell Polling Patterns

### Base 3×3 Grid (All Levels)

Every level uses the same 3×3 grid pattern, indexed by the creature's current cell:

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

### Extended Cells by Octant

The extended cells rotate with the creature's facing direction:

```
NARROW (Predator) - Front Extension:

Octant 0 (E):   (2,0), (3,0)      ─→
Octant 1 (NE):  (2,2), (3,3)      ╱
Octant 2 (N):   (0,2), (0,3)      │
Octant 3 (NW): (-2,2),(-3,3)      ╲
Octant 4 (W):  (-2,0),(-3,0)      ←─
Octant 5 (SW):(-2,-2),(-3,-3)     ╱
Octant 6 (S):  (0,-2),(0,-3)      │
Octant 7 (SE): (2,-2),(3,-3)      ╲


WIDE (Prey) - Side Extension:

Octant 0 (E):   (0,+2), (0,-2)    ↑ ↓ (perpendicular to E)
Octant 2 (N):   (+2,0), (-2,0)    ← → (perpendicular to N)
Octant 4 (W):   (0,+2), (0,-2)    ↑ ↓ (perpendicular to W)
Octant 6 (S):   (+2,0), (-2,0)    ← → (perpendicular to S)
(diagonals use rotated perpendiculars)
```

## Multi-Level Grid System

### Grid Hierarchy

```
L2 (90m cells)
┌─────────────────────────────────────────────────────────────────┐
│                           L2 Cell                               │
│  ┌───────────────────┬───────────────────┬───────────────────┐  │
│  │      L1 Cell      │      L1 Cell      │      L1 Cell      │  │
│  │  ┌─────┬─────┬─────┐  ┌─────┬─────┬─────┐  ...             │  │
│  │  │ L0  │ L0  │ L0  │  │ L0  │ L0  │ L0  │                  │  │
│  │  ├─────┼─────┼─────┤  ├─────┼─────┼─────┤                  │  │
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

### Level Specifications

```
┌────────┬────────────┬────────────────┬────────────────────────┐
│ Level  │ Cell Size  │ 3×3 Reach      │ +Extended Reach        │
├────────┼────────────┼────────────────┼────────────────────────┤
│ L0     │ 10m        │ ~30m           │ ~50m                   │
│ L1     │ 30m        │ ~90m           │ ~150m                  │
│ L2     │ 90m        │ ~270m          │ ~450m                  │
└────────┴────────────┴────────────────┴────────────────────────┘
```

## Execution Flow

### Phase 1: Level Selection

```
perception_range = creature.perception.range

         ┌──────────────────────────────────────────────────────┐
         │              Level Selection Logic                   │
         └──────────────────────────────────────────────────────┘
                                │
                                ▼
                  ┌─────────────────────────────┐
                  │ perception_range > 90m ?    │
                  └─────────────┬───────────────┘
                        yes     │     no
                   ┌────────────┴────────────┐
                   ▼                         ▼
           ┌──────────────┐         ┌──────────────────────┐
           │ scan_l2 = ✓  │         │ scan_l2 = ✗          │
           └──────────────┘         └──────────────────────┘
                   │                         │
                   ▼                         ▼
                  ┌─────────────────────────────┐
                  │ perception_range > 30m ?    │
                  └─────────────┬───────────────┘
                        yes     │     no
                   ┌────────────┴────────────┐
                   ▼                         ▼
           ┌──────────────┐         ┌──────────────────────┐
           │ scan_l1 = ✓  │         │ scan_l1 = ✗          │
           └──────────────┘         └──────────────────────┘
                   │                         │
                   └─────────────┬───────────┘
                                 ▼
                   ┌──────────────────────────┐
                   │ scan_l0 = ✓ (always)     │
                   └──────────────────────────┘
```

### Phase 2: L2 Scan (if scan_l2)

```
For each L2 cell in FOV pattern:
┌────────────────────────────────────────────────────────────────┐
│                                                                 │
│   1. Get L2 cell indices from pattern                          │
│      pattern = get_cell_pattern(fov_rad, fx, fy)               │
│      extra = get_extra_cells(fov_tier, fx, fy)                 │
│                                                                 │
│   2. For each L2 cell:                                         │
│      ┌───────────────────────────────────────────────────────┐ │
│      │ L2 BioSignature = aggregate of 9 child L1 biosigs     │ │
│      │                                                        │ │
│      │ classification = classify_l2(biosig, my_mass)         │ │
│      │                                                        │ │
│      │ if classification == Empty:                           │ │
│      │    └──> mark L2 cell as EMPTY (for L1 early-exit)     │ │
│      │         SKIP: 9 L1 children + 81 L0 grandchildren     │ │
│      │ else:                                                  │ │
│      │    └──> record in L2Vision (strategic awareness)       │ │
│      └───────────────────────────────────────────────────────┘ │
│                                                                 │
└────────────────────────────────────────────────────────────────┘
```

### Phase 3: L1 Scan (if scan_l1)

```
For each L1 cell in FOV pattern:
┌────────────────────────────────────────────────────────────────┐
│                                                                 │
│   1. Same pattern functions as L2, just at L1 scale            │
│                                                                 │
│   2. For each L1 cell:                                         │
│      ┌───────────────────────────────────────────────────────┐ │
│      │ EARLY-EXIT CHECK (if we scanned L2):                  │ │
│      │    parent_l2 = l1_to_l2(l1_cell_idx)                  │ │
│      │    if l2_is_empty(parent_l2):                         │ │
│      │       └──> SKIP this L1 cell entirely                 │ │
│      │                                                        │ │
│      │ L1 BioSignature = aggregate of entities in L1 cell    │ │
│      │                                                        │ │
│      │ classification = classify_l1(biosig, my_mass, my_size)│ │
│      │                                                        │ │
│      │ if classification == Empty:                           │ │
│      │    └──> mark L1 cell as EMPTY (for L0 early-exit)     │ │
│      │         SKIP: 9 L0 children                           │ │
│      │ else:                                                  │ │
│      │    └──> record in L1Vision (mid-range awareness)       │ │
│      └───────────────────────────────────────────────────────┘ │
│                                                                 │
└────────────────────────────────────────────────────────────────┘
```

### Phase 4: L0 Scan (always)

```
For each L0 cell in FOV pattern:
┌────────────────────────────────────────────────────────────────┐
│                                                                 │
│   1. Same pattern functions as L1/L2, just at L0 scale         │
│                                                                 │
│   2. For each L0 cell:                                         │
│      ┌───────────────────────────────────────────────────────┐ │
│      │ EARLY-EXIT CHECK (if we scanned L1):                  │ │
│      │    parent_l1 = l0_to_l1(l0_cell_idx)                  │ │
│      │    if l1_is_empty(parent_l1):                         │ │
│      │       └──> SKIP this L0 cell entirely                 │ │
│      │                                                        │ │
│      │ If NOT skipping:                                       │ │
│      │    for each entity in L0 cell:                        │ │
│      │       - FOV check (is entity in my FOV cone?)         │ │
│      │       - Size domination (can I perceive this size?)   │ │
│      │       - Add to neighbor candidates                     │ │
│      │                                                        │ │
│      │ After all L0 cells:                                    │ │
│      │    Select K=7 closest neighbors → NeighborCache       │ │
│      └───────────────────────────────────────────────────────┘ │
│                                                                 │
└────────────────────────────────────────────────────────────────┘
```

## Early-Exit Cascade Savings

### Example: Giant Creature (100m perception) in Sparse Area

```
Without hierarchical perception:
┌─────────────────────────────────────────────────────────────┐
│  L0 cells to scan: ~11 × 11 = ~121 cells                    │
│  Entity iterations: 121 × avg_entities_per_cell             │
└─────────────────────────────────────────────────────────────┘

With hierarchical perception:
┌─────────────────────────────────────────────────────────────┐
│  L2 scan: 11 cells (cheap biosig check)                     │
│     → 6 marked Empty (sparse area)                          │
│     → 5 non-Empty                                           │
│                                                              │
│  L1 scan: 11 cells per L2 = 55 theoretical                  │
│     → 6 L2 Empty × 9 L1 children = 54 L1 SKIPPED           │
│     → Only ~11 L1 actually checked                          │
│     → 7 marked Empty                                         │
│     → 4 non-Empty                                           │
│                                                              │
│  L0 scan: 9 cells per L1 neighbor = ~99 theoretical         │
│     → 7 L1 Empty × 9 L0 children = 63 L0 SKIPPED           │
│     → Only ~36 L0 actually checked                          │
│                                                              │
│  SAVINGS: 121 → 36 L0 cells (70% reduction)                 │
└─────────────────────────────────────────────────────────────┘
```

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

### Size Domination (Empty Classification)

At any level, a cell is "Empty" if:
```
effective_mass = biosig.total_mass - (masses I can't perceive due to size domination)

threshold = my_mass × PERCEPTION_THRESHOLD_FRACTION (0.05)

if effective_mass < threshold:
    classification = Empty  // Nothing here I can perceive!
```

## Implementation Checklist

### Step 1: Add L2 Grid Infrastructure
- [ ] Add `L2_CELL_SIZE = 90.0`, `L2_GRID_SIZE = 12` constants
- [ ] Add `l2_biosigs: Vec<BioSignature>` to `CoarseGrid`
- [ ] Add `l1_to_l2(l1_idx)` and `l2_to_l1s(l2_idx)` mapping functions
- [ ] Update entity insert/remove to maintain L2 biosigs

### Step 2: Add Pattern Iteration Helper
- [ ] Create `cells_from_pattern(base_idx, pattern, extra, grid_width)` iterator
- [ ] Returns cell indices matching FOV pattern
- [ ] Works identically at L0, L1, L2 (parameterized by grid geometry)

### Step 3: Restructure Perception System
- [ ] Add level selection logic (scan_l2, scan_l1 flags)
- [ ] Add L2 scan phase with biosig classification
- [ ] Add L1 scan phase with L2 early-exit check
- [ ] Modify L0 scan to use L1 early-exit from hierarchical scan

### Step 4: Early-Exit Tracking
- [ ] Add `l2_empty_mask: u64` for L1 early-exit
- [ ] Add `l1_empty_mask: u64` for L0 early-exit
- [ ] Map cell indices to mask bits efficiently

### Step 5: Add L2Vision Component (if needed)
- [ ] Decide if L2Vision is needed for strategic awareness
- [ ] Or: just use L2 for early-exit, L1Vision for strategic data

### Step 6: Tests
- [ ] Test pattern produces correct cells at each level
- [ ] Test early-exit cascade skips correct cells
- [ ] Test L1Vision/L2Vision populated correctly
- [ ] Test neighbors still found (avoidance MUST work!)
- [ ] Benchmark: verify performance improvement
