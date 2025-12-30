# Hierarchical Perception Design

## Problem Statement

Current perception is **bottom-up** (L0 → L1 as byproduct):
- L0 scan: Fixed 9 cells regardless of perception range
- L1 discovered: Only parents of queried L0 cells (max 4)
- Result: Large creatures can't see strategically beyond ~30m

We need **top-down** perception (L1 → L0 drill-down):
- L1 scan: All cells within perception range (~35 cells for 100m range)
- L0 drill-down: Only where L1 indicates something interesting
- Result: Strategic awareness + tactical neighbors in one efficient pass

---

## Current Data Flow (BROKEN)

```
┌─────────────────────────────────────────────────────────────────────┐
│                         CURRENT APPROACH                             │
├─────────────────────────────────────────────────────────────────────┤
│                                                                      │
│   ┌─────────────┐                                                   │
│   │  L0 Scan    │  Fixed 9 cells (3×3)                              │
│   │  (10m grid) │  ~30m diagonal reach                              │
│   └──────┬──────┘                                                   │
│          │                                                          │
│          ├─────────────────┐                                        │
│          │                 │                                        │
│          ▼                 ▼                                        │
│   ┌─────────────┐   ┌─────────────┐                                │
│   │ NeighborCache│   │  L1Vision   │  Only L1 parents of            │
│   │ K closest    │   │  Max 4 cells │  queried L0 cells!            │
│   └─────────────┘   └─────────────┘                                │
│                                                                      │
│   PROBLEM: 100m perception range → only sees 4 L1 cells (30m×30m)  │
│                                                                      │
└─────────────────────────────────────────────────────────────────────┘
```

---

## Option A: Top-Down Hierarchical (RECOMMENDED)

Start with L1 scan (strategic), drill down to L0 (tactical) where needed.

```
┌─────────────────────────────────────────────────────────────────────┐
│                    TOP-DOWN HIERARCHICAL SCAN                        │
├─────────────────────────────────────────────────────────────────────┤
│                                                                      │
│  ┌─────────────────────────────────────────────────────────────┐   │
│  │ PHASE 1: L1 STRATEGIC SCAN                                   │   │
│  │                                                              │   │
│  │  Input: creature position, facing, perception.range, FOV    │   │
│  │                                                              │   │
│  │  ┌────────────────────────────────────────────────────────┐ │   │
│  │  │  1. Compute L1 cells in range                          │ │   │
│  │  │     • L1_radius = ceil(perception.range / L1_CELL_SIZE)│ │   │
│  │  │     • For 100m range: ceil(100/30) = 4 → ~49 cells max │ │   │
│  │  ├────────────────────────────────────────────────────────┤ │   │
│  │  │  2. FOV cull L1 cells                                  │ │   │
│  │  │     • Check if L1 cell center is in FOV                │ │   │
│  │  │     • ~50% reduction → ~25 cells                       │ │   │
│  │  ├────────────────────────────────────────────────────────┤ │   │
│  │  │  3. Classify each L1 cell                              │ │   │
│  │  │     • Empty:   total_mass == 0                         │ │   │
│  │  │     • Threat:  max_size > my_size (predator present)   │ │   │
│  │  │     • Prey:    max_size < my_size (food present)       │ │   │
│  │  │     • Crowded: creature_count > threshold              │ │   │
│  │  ├────────────────────────────────────────────────────────┤ │   │
│  │  │  4. Store in L1Vision                                  │ │   │
│  │  │     • All non-Empty cells with classification          │ │   │
│  │  │     • Direction vector from creature to cell center    │ │   │
│  │  └────────────────────────────────────────────────────────┘ │   │
│  │                                                              │   │
│  │  Output: L1Vision populated (~25 cells with classifications)│   │
│  └──────────────────────────────────────────────────────────────┘   │
│                              │                                      │
│                              │ L1 classifications inform L0 scan    │
│                              ▼                                      │
│  ┌─────────────────────────────────────────────────────────────┐   │
│  │ PHASE 2: L0 TACTICAL DRILL-DOWN                              │   │
│  │                                                              │   │
│  │  Input: creature's 9 adjacent L0 cells + L1 classifications │   │
│  │                                                              │   │
│  │  ┌────────────────────────────────────────────────────────┐ │   │
│  │  │  For each L0 cell (sorted by distance):                │ │   │
│  │  │                                                        │ │   │
│  │  │  1. Lookup parent L1 classification (from Phase 1)     │ │   │
│  │  │                                                        │ │   │
│  │  │  2. EARLY EXIT opportunities:                          │ │   │
│  │  │     ┌──────────────────────────────────────────────┐  │ │   │
│  │  │     │ L1 Empty     → SKIP entire L0 cell           │  │ │   │
│  │  │     │   (includes size domination - giants skip    │  │ │   │
│  │  │     │    cells with only mice!)                    │  │ │   │
│  │  │     │ K neighbors  → SKIP non-adjacent cells       │  │ │   │
│  │  │     │ FOV culled   → SKIP (existing logic)         │  │ │   │
│  │  │     └──────────────────────────────────────────────┘  │ │   │
│  │  │                                                        │ │   │
│  │  │  3. Scan L0 cell entities (if not skipped)            │ │   │
│  │  │     • FOV check each entity                           │ │   │
│  │  │     • Size domination filter                          │ │   │
│  │  │     • Add to neighbor candidates                      │ │   │
│  │  └────────────────────────────────────────────────────────┘ │   │
│  │                                                              │   │
│  │  Output: NeighborCache with K closest neighbors              │   │
│  └──────────────────────────────────────────────────────────────┘   │
│                                                                      │
└─────────────────────────────────────────────────────────────────────┘
```

### Early Exit Matrix

| L1 Classification | L0 Action | Rationale |
|-------------------|-----------|-----------|
| **Empty** | **SKIP** L0 scan | No perceivable mass (size domination built-in!) |
| **Threat** | **SCAN** L0 | Need neighbors for avoidance while fleeing |
| **Prey** | **SCAN** L0 | Need neighbors for avoidance while hunting |
| **Crowded** | **SCAN** L0 | Need neighbors for avoidance |

**Key insight:** `classify_l1_cell()` already accounts for size domination!
```rust
let threshold = my_mass * PERCEPTION_THRESHOLD_FRACTION;
if effective_count == 0 || effective_mass < threshold {
    return L1Classification::Empty;  // Nothing I CAN perceive
}
```
A giant sees an L1 cell full of mice as **Empty** because `mice_total_mass < giant_threshold`.

### Performance Estimate

```
L1 scan: 25 cells × 10 ops = 250 ops per creature
L0 scan: 9 cells × ~20% skip = ~7 cells × 50 ops = 350 ops
Total: ~600 ops vs current ~450 ops (+33%)

BUT: L1Vision now has 25 cells instead of 4 (6× more strategic awareness)
AND: L0 scans fewer cells in sparse areas (wins back the cost)
```

---

## Option B: Zoned Perception (Biological Model)

Inspired by real vision: fovea (detail) + peripheral (motion/threat detection).

```
┌─────────────────────────────────────────────────────────────────────┐
│                       ZONED PERCEPTION MODEL                         │
├─────────────────────────────────────────────────────────────────────┤
│                                                                      │
│                        ┌───────────────────┐                        │
│                        │   Creature FOV    │                        │
│                        │   Perception Cone │                        │
│                        └─────────┬─────────┘                        │
│                                  │                                  │
│         ┌────────────────────────┼────────────────────────┐        │
│         │                        │                        │        │
│         ▼                        ▼                        ▼        │
│  ┌─────────────┐         ┌─────────────┐         ┌─────────────┐  │
│  │ NEAR ZONE   │         │ MID ZONE    │         │ FAR ZONE    │  │
│  │ 0 - 30m     │         │ 30m - 60m   │         │ 60m - range │  │
│  │             │         │             │         │             │  │
│  │ • L0 scan   │         │ • L1 scan   │         │ • L1 scan   │  │
│  │ • All detail│         │ • Classify  │         │ • Classify  │  │
│  │ • Neighbors │         │ • No L0     │         │ • No L0     │  │
│  │ • Avoidance │         │ • Direction │         │ • Direction │  │
│  └─────────────┘         └─────────────┘         └─────────────┘  │
│         │                        │                        │        │
│         ▼                        ▼                        ▼        │
│  ┌─────────────┐         ┌─────────────────────────────────────┐  │
│  │NeighborCache│         │           L1Vision                  │  │
│  │ K entities  │         │  Classified cells with directions   │  │
│  └─────────────┘         └─────────────────────────────────────┘  │
│                                                                      │
└─────────────────────────────────────────────────────────────────────┘
```

### Zone Breakdown

| Zone | Range | Grid Level | Output | Purpose |
|------|-------|------------|--------|---------|
| **Near** | 0-30m | L0 (9 cells) | NeighborCache | Collision avoidance, immediate threats |
| **Mid** | 30-60m | L1 (~8 cells) | L1Vision | Tactical planning (where to go next) |
| **Far** | 60m-range | L1 (~20 cells) | L1Vision | Strategic navigation (flee/hunt direction) |

### Benefits

- **Biological accuracy**: Real animals have detailed near vision, blurry far vision
- **Natural early-exit**: Far threats don't need entity-level detail
- **Cache-friendly**: Near zone is hot, far zone is cold

---

## Option C: Adaptive Resolution (Size-Based)

Different creatures need different perception strategies.

```
┌─────────────────────────────────────────────────────────────────────┐
│                    SIZE-BASED ADAPTIVE PERCEPTION                    │
├─────────────────────────────────────────────────────────────────────┤
│                                                                      │
│  ┌──────────────────┐  ┌──────────────────┐  ┌──────────────────┐  │
│  │  SMALL CREATURE  │  │ MEDIUM CREATURE  │  │  LARGE CREATURE  │  │
│  │  (< 1m)          │  │ (1m - 3m)        │  │ (> 3m)           │  │
│  ├──────────────────┤  ├──────────────────┤  ├──────────────────┤  │
│  │                  │  │                  │  │                  │  │
│  │ Range: ~20m      │  │ Range: ~50m      │  │ Range: ~100m     │  │
│  │                  │  │                  │  │                  │  │
│  │ L1: 1-2 cells    │  │ L1: ~6 cells     │  │ L1: ~25 cells    │  │
│  │ L0: 9 cells      │  │ L0: 9 cells      │  │ L0: 9 cells      │  │
│  │                  │  │                  │  │                  │  │
│  │ Strategy:        │  │ Strategy:        │  │ Strategy:        │  │
│  │ • All L0         │  │ • L1 for threats │  │ • L1 dominant    │  │
│  │ • L1 minimal     │  │ • L0 for detail  │  │ • L0 for nearby  │  │
│  │                  │  │                  │  │                  │  │
│  │ [L0 dominant]    │  │ [Balanced]       │  │ [L1 dominant]    │  │
│  └──────────────────┘  └──────────────────┘  └──────────────────┘  │
│                                                                      │
│  INSIGHT: Small creatures already limited to ~1 L1 cell anyway!     │
│           Large creatures NEED L1 for long-range strategic vision.  │
│                                                                      │
└─────────────────────────────────────────────────────────────────────┘
```

---

## Recommended Approach: Option A + C Hybrid

Combine top-down hierarchical with size-aware optimizations.

```
┌─────────────────────────────────────────────────────────────────────┐
│              UNIFIED HIERARCHICAL PERCEPTION SYSTEM                  │
├─────────────────────────────────────────────────────────────────────┤
│                                                                      │
│  INPUT: creature (pos, facing, size, perception.range, fov)         │
│                                                                      │
│  ┌───────────────────────────────────────────────────────────────┐  │
│  │ STEP 1: DETERMINE L1 SCAN SCOPE                                │  │
│  │                                                                │  │
│  │  l1_radius = ceil(perception.range / L1_CELL_SIZE)            │  │
│  │                                                                │  │
│  │  // Small creature optimization: skip L1 if radius <= 1        │  │
│  │  if l1_radius <= 1 {                                          │  │
│  │      // Just use current approach (L0 scan only)              │  │
│  │      goto STEP_3_L0_ONLY                                      │  │
│  │  }                                                            │  │
│  └───────────────────────────────────────────────────────────────┘  │
│                              │                                      │
│                              ▼                                      │
│  ┌───────────────────────────────────────────────────────────────┐  │
│  │ STEP 2: L1 STRATEGIC SCAN                                      │  │
│  │                                                                │  │
│  │  for l1_cell in l1_cells_in_fov(pos, facing, l1_radius, fov): │  │
│  │      biosig = l1_grid.get_biosignature(l1_cell)               │  │
│  │      classification = classify(biosig, my_mass, my_size)      │  │
│  │                                                                │  │
│  │      if classification != Empty:                              │  │
│  │          l1_vision.push(L1VisionEntry {                       │  │
│  │              cell_idx,                                        │  │
│  │              classification,                                  │  │
│  │              direction: normalize(cell_center - pos),         │  │
│  │          })                                                   │  │
│  │                                                                │  │
│  │  // Build L1 classification lookup for L0 phase               │  │
│  │  l1_lookup: HashMap<L1CellIdx, Classification>                │  │
│  └───────────────────────────────────────────────────────────────┘  │
│                              │                                      │
│                              ▼                                      │
│  ┌───────────────────────────────────────────────────────────────┐  │
│  │ STEP 3: L0 TACTICAL SCAN (with L1 early-exit)                  │  │
│  │                                                                │  │
│  │  for l0_cell in l0_cells_3x3(pos, facing, fov_pattern):       │  │
│  │                                                                │  │
│  │      // EARLY EXIT: Check L1 parent classification            │  │
│  │      parent_l1 = l0_to_l1(l0_cell)                            │  │
│  │      classification = l1_lookup.get(parent_l1)                │  │
│  │                                                                │  │
│  │      // Only skip Empty - all others need neighbors for       │  │
│  │      // avoidance (even while fleeing!)                       │  │
│  │      if classification == Empty {                             │  │
│  │          continue;  // Nothing I can perceive (size domination)│  │
│  │      }                                                        │  │
│  │                                                                │  │
│  │      // Existing entity scanning logic...                     │  │
│  │      for entity in l0_cell.entities:                          │  │
│  │          if in_fov && not_size_dominated:                     │  │
│  │              candidates.push(entity)                          │  │
│  │                                                                │  │
│  │  // Select K closest                                          │  │
│  │  neighbor_cache = select_k_closest(candidates)                │  │
│  └───────────────────────────────────────────────────────────────┘  │
│                                                                      │
│  OUTPUT:                                                            │
│    • L1Vision: Strategic awareness (direction to threats/prey/empty)│
│    • NeighborCache: Tactical neighbors (for avoidance/interaction)  │
│                                                                      │
└─────────────────────────────────────────────────────────────────────┘
```

---

## Visual: Before vs After

```
                    BEFORE                              AFTER
                    ──────                              ─────

         L1 cells visible: 4                 L1 cells visible: ~25
         (parents of 9 L0 cells)             (full perception range)

             ┌─┬─┬─┐                              ┌─┬─┬─┬─┬─┬─┬─┐
             │ │ │ │                              │ │ │░│░│░│ │ │
             ├─┼─┼─┤                              ├─┼─┼─┼─┼─┼─┼─┤
             │ │●│ │  ← 3×3 L0 cells             │ │░│░│░│░│░│ │
             ├─┼─┼─┤    = max 4 L1               ├─┼─┼─┼─┼─┼─┼─┤
             │ │ │ │    parents                  │░│░│░│●│░│░│░│  ← Full FOV
             └─┴─┴─┘                              ├─┼─┼─┼─┼─┼─┼─┤    coverage
                                                  │ │░│░│░│░│░│ │
       "Where are threats?"                       ├─┼─┼─┼─┼─┼─┼─┤
       "I can only see ~30m"                      │ │ │░│░│░│ │ │
                                                  └─┴─┴─┴─┴─┴─┴─┘

                                             "I see threats at 100m!"
```

---

## Implementation Complexity

| Aspect | Option A | Option B | Option C | Hybrid |
|--------|----------|----------|----------|--------|
| Code changes | Medium | Medium | High | Medium |
| Performance | Good | Good | Varies | Best |
| Biological accuracy | Good | Best | Good | Good |
| Early-exit opportunities | Many | Some | Many | Many |
| Memory overhead | Low | Low | Medium | Low |

---

## L1 Cell Query Function (New)

Need to add a function to query L1 cells within a radius + FOV:

```rust
/// Query L1 cells within perception range, with FOV culling.
/// Returns cell indices sorted by distance.
pub fn collect_l1_cells_in_fov(
    &self,
    x: f32,
    y: f32,
    perception_range: f32,
    facing_x: f32,
    facing_y: f32,
    cos_half_fov: f32,
    output: &mut Vec<(f32, usize)>,  // (distance_sq, cell_idx)
) {
    output.clear();

    let l1_radius = (perception_range / L1_CELL_SIZE).ceil() as i32;
    let (my_l1_cx, my_l1_cy) = self.world_to_cell(x, y);

    for dy in -l1_radius..=l1_radius {
        for dx in -l1_radius..=l1_radius {
            let l1_cx = my_l1_cx + dx;
            let l1_cy = my_l1_cy + dy;

            // Bounds check
            let Some(cell_idx) = self.get_cell_index(l1_cx, l1_cy) else { continue };

            // Distance check
            let (center_x, center_y) = self.cell_center(l1_cx, l1_cy);
            let cell_dx = center_x - x;
            let cell_dy = center_y - y;
            let dist_sq = cell_dx * cell_dx + cell_dy * cell_dy;

            if dist_sq > perception_range * perception_range { continue }

            // FOV check
            let dot = cell_dx * facing_x + cell_dy * facing_y;
            if !is_in_fov(dot, dist_sq, cos_half_fov, cos_half_fov_sq) { continue }

            output.push((dist_sq, cell_idx));
        }
    }

    // Sort by distance for ordered processing
    output.sort_unstable_by(|a, b| a.0.total_cmp(&b.0));
}
```

---

## Questions to Resolve

1. **Skip Threat L0 cells?**
   - Pro: Creature is fleeing, doesn't need neighbor detail
   - Con: Might miss an escape route between threats

2. **L1Vision capacity?**
   - Current: 4 entries (MAX_L1_VISION)
   - Needed: ~48 entries for large creatures
   - Memory: 48 × 16 bytes = 768 bytes per creature

3. **L1 scan frequency?**
   - L1 could run at lower frequency than L0 (strategic data is less time-sensitive)
   - e.g., L1 at ÷4, L0 at ÷2

---

## Next Steps

1. [ ] Decide on approach (recommend: Hybrid A+C)
2. [ ] Add `collect_l1_cells_in_fov()` to CoarseGrid
3. [ ] Restructure perception system into phases
4. [ ] Update L1Vision capacity
5. [ ] Add tests for hierarchical scan
6. [ ] Benchmark performance vs current approach
