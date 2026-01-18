# Phase 6: L1 Cone-to-Grid Intersection

**Status:** ✅ COMPLETE
**Completed:** 2026-01-18
**Prerequisite:** Phase 5 (L1 Ring Perception) complete
**Related:** `5-L1-fix.md`

---

## Implementation Summary

### Changes Made

**`apps/simulation/src/simulation/perception/systems.rs`** (lines 507-562):
- Replaced fixed 8-cell ring scan with cone-based scan
- Now checks ALL L1 cells within perception range (not just 8 neighbors)
- Applies FOV check using existing `is_in_fov()` function
- Removed extended L1 cells section (cone naturally includes cells at any distance)
- Removed unused `RING_OFFSETS` constant

**`apps/simulation/src/simulation/perception/tests.rs`** (3 test updates):
- Updated `test_l1_vision_records_discovered_cells` - expects > 0 cells (not >= 8)
- Updated `test_l1_vision_classifies_tiny_creatures_as_empty` - expects > 0 cells
- Updated `test_l1_scan_range_gate_skips_myopic_creatures` - expects > 0 cells

### Algorithm

```rust
if range >= L1_CELL_SIZE {
    let range_sq = range * range;
    let max_cell_dist = (range / L1_CELL_SIZE).ceil() as i32;

    for dx in -max_cell_dist..=max_cell_dist {
        for dy in -max_cell_dist..=max_cell_dist {
            if dx == 0 && dy == 0 { continue; }  // Skip own cell

            // Get cell center in world coords
            let (cell_center_x, cell_center_y) = l1_grid.cell_center_from_index(l1_idx);

            // Vector from creature to cell
            let to_cell_x = cell_center_x - creature_x;
            let to_cell_y = cell_center_y - creature_y;
            let dist_sq = to_cell_x * to_cell_x + to_cell_y * to_cell_y;

            // Range check
            if dist_sq > range_sq { continue; }

            // FOV check (reuses existing is_in_fov)
            let rough_dot = to_cell_x * facing_x + to_cell_y * facing_y;
            if !is_in_fov(rough_dot, dist_sq, cos_half_fov, cos_half_fov_sq) {
                continue;
            }

            // Cell is within cone - add to L1Vision
            l1_vision.push(...);
        }
    }
}
```

### Test Results

All 108 perception tests pass. Full test suite passes.

---

## Problem Statement

The L1 grid cells being perceived **completely ignore the FOV cone**. Cells behind the creature (outside the FOV) are included, and the pattern is just "all 8 surrounding L1 cells" regardless of facing direction.

### Visual Description (From Screenshot)

```
Current Behavior (NO FOV Culling):

                      ┌─┬─┬─┬─┬─┬─┐
                      │▓│▓│▓│▓│ │ │   ▓ = highlighted L1 cells
                      ├─┼─┼─┼─┼─┼─┤       (includes cells BEHIND creature)
    FOV cone ────────>│▓│▓│▓│●│ │ │   ● = creature facing LEFT
    (extends left)    ├─┼─┼─┼─┼─┼─┤
                      │▓│▓│▓│▓│ │ │   Problem: L1 cells to the RIGHT
                      └─┴─┴─┴─┴─┴─┘   of creature are highlighted even
                                       though they're BEHIND the FOV cone
```

The creature faces **west** (left), the FOV cone extends leftward, but the L1 grid highlights cells **to the east** (right/behind) that are completely outside the cone.

### Root Cause

The L1 ring scan was **intentionally designed with NO FOV culling**:

```rust
// systems.rs:515-517
// NO FOV CULLING: L1 is "area awareness" - creature should know about
// all surrounding cells including behind (escape routes, threats).
// This is simpler AND faster for most creatures.
```

**This design decision is incorrect.** L1 perception should respect the creature's FOV cone. A creature facing west with 120° FOV should NOT perceive L1 cells directly behind it (east).

### Observed Behavior

From the screenshot:
1. Creature facing **west** (~180°)
2. FOV cone (teal shaded area) extends **leftward**
3. L1 grid highlights include cells **to the right/behind** the creature
4. Pattern is essentially "the 8 surrounding L1 cells" - a rectangle, not a cone

**L1 perception currently ignores FOV entirely - it queries all 8 ring cells regardless of facing direction.**

---

## Desired State

L1 grid cells should be selected based on **whether any part of the cell intersects the actual FOV cone**. Cells behind the creature (outside the cone) should NOT be perceived.

### Visual Description

```
Desired Behavior (Cone-Respecting):

                      ┌─┬─┬─┬─┬─┬─┐
                      │▓│▓│▓│ │ │ │   Only cells within the FOV cone
                      ├─┼─┼─┼─┼─┼─┤   are highlighted
    FOV cone ────────>│▓│▓│▓│●│ │ │
    (extends left)    ├─┼─┼─┼─┼─┼─┤   Cells behind creature (right)
                      │▓│▓│▓│ │ │ │   are NOT perceived
                      └─┴─┴─┴─┴─┴─┘
                       \_______/
                       Only cells intersecting
                       the actual FOV cone
```

### Requirements

1. **Cone-Cell Intersection Test:** For each candidate L1 cell, check if the cell center (or nearest corner) falls within the FOV cone
2. **Respect FOV Angle:** A creature with 120° FOV should only perceive L1 cells within ±60° of facing direction
3. **Smooth Rotation:** As creature rotates, cells on the boundary smoothly enter/exit (no coarse snapping)
4. **Acceptable Margin:** Include cells that are "close enough" (within ~5-10° of cone edge) to avoid pop-in
5. **Performance:** Must remain fast - L1 scan runs for every creature with perception >= 60m

---

## Implementation Approaches

### Approach A: Per-Cell Dot Product Test

Replace pattern lookup with per-cell cone test:

```rust
// For each potential L1 cell in extended ring:
for (dx, dy) in L1_RING_OFFSETS {
    let cell_center_x = creature_x + dx as f32 * L1_CELL_SIZE;
    let cell_center_y = creature_y + dy as f32 * L1_CELL_SIZE;

    // Vector to cell center
    let to_cell_x = cell_center_x - creature_x;
    let to_cell_y = cell_center_y - creature_y;
    let dist = (to_cell_x * to_cell_x + to_cell_y * to_cell_y).sqrt();

    // Dot product with facing direction
    let dot = (to_cell_x * facing_x + to_cell_y * facing_y) / dist;

    // Check if within FOV (with safety margin for cell corners)
    if dot >= cos_half_fov_with_margin {
        // Cell is within cone - add to L1Vision
    }
}
```

**Pros:**
- Simple and correct
- Smooth rotation (no quantization)
- Easy to tune margin

**Cons:**
- Requires sqrt per cell (but only 8-24 cells per creature)
- No longer branchless (but L1 scan is already branchy)

### Approach B: Corner Test (Conservative)

Test all 4 corners of each cell - include cell if ANY corner is in cone:

```rust
for (dx, dy) in L1_RING_OFFSETS {
    let base_x = creature_x + dx as f32 * L1_CELL_SIZE;
    let base_y = creature_y + dy as f32 * L1_CELL_SIZE;

    let corners = [
        (base_x - HALF_CELL, base_y - HALF_CELL),  // SW
        (base_x + HALF_CELL, base_y - HALF_CELL),  // SE
        (base_x - HALF_CELL, base_y + HALF_CELL),  // NW
        (base_x + HALF_CELL, base_y + HALF_CELL),  // NE
    ];

    for (cx, cy) in corners {
        if is_point_in_cone(cx, cy, creature_x, creature_y, facing_x, facing_y, cos_half_fov) {
            // At least one corner in cone - include cell
            break;
        }
    }
}
```

**Pros:**
- More accurate than center-only
- Conservative (won't miss edge cells)

**Cons:**
- 4x more tests per cell
- May include cells that barely clip the cone

### Approach C: Hybrid - Precomputed Wide Patterns + Runtime Refinement

Keep octant lookup for rough culling, then refine:

```rust
// Step 1: Use existing octant pattern to get candidate cells (wide pattern)
let pattern = get_cell_pattern_by_octant(fov_bucket, octant);

// Step 2: Refine candidates with actual cone test
for cell in pattern.set_bits() {
    if is_cell_in_cone(cell, creature_pos, facing, cos_half_fov) {
        // Add to L1Vision
    }
}
```

**Pros:**
- Avoids testing cells obviously outside cone
- Backward compatible with existing infrastructure

**Cons:**
- Still has pattern quantization as first filter
- More complex

---

## Recommended Approach

**Approach A (Per-Cell Dot Product)** for the initial implementation:

1. **Simplest correct solution** - easy to verify, reuses existing `is_in_fov()` math
2. **Performance acceptable** - only 8-24 cells per creature
3. **Sqrt-free comparison** - use squared comparison (already done in `is_in_fov`):
   ```rust
   // Avoid sqrt by comparing dot² against cos²_half_fov * dist²
   let dot = to_cell_x * facing_x + to_cell_y * facing_y;
   let dist_sq = to_cell_x * to_cell_x + to_cell_y * to_cell_y;
   if is_in_fov(dot, dist_sq, cos_half_fov, cos_half_fov_sq) {
       // Cell center is in cone - include it
   }
   ```
4. **Safety margin** - use a slightly expanded `cos_half_fov` (~5-10° wider) to include cells that partially overlap the cone edge

### Code Location

Modify `apps/simulation/src/simulation/perception/systems.rs` in the L1 RING SCAN section (~line 507-598).

Current code:
```rust
// NO FOV CULLING: L1 is "area awareness" - creature should know about
// all surrounding cells including behind (escape routes, threats).
for (dx, dy) in RING_OFFSETS {
    // ... adds ALL 8 ring cells regardless of facing direction
}
```

New code should:
1. For each ring cell, compute direction vector from creature to cell center
2. Test if cell center is within FOV cone using `is_in_fov()`
3. Only add cells that pass the FOV test to L1Vision
4. Use a slightly expanded margin to include edge cells

---

## Performance Analysis

### Current Cost (No FOV Culling)

```
Per creature with perception >= 60m:
  - 8 ring offset iterations (RING_OFFSETS)
  - 8 bounds checks (get_cell_index_by_coords)
  - 8 biosignature lookups
  - 8 classify_l1_cell calls
  - 8 direction vector calculations (already computed)
  - 8 L1VisionEntry pushes

At 100K creatures (assume 50% have perception >= 60m = 50K):
  - 50K × 8 = 400K L1 cell iterations per tick
  - No cone tests currently
```

### Proposed Cost (With FOV Culling)

```
Per creature:
  - 8 ring offset iterations (unchanged)
  - 8 direction dot products: dot = dx*fx + dy*fy (2 muls, 1 add each = 24 ops)
  - 8 squared magnitude: dist_sq = dx² + dy² (2 muls, 1 add each = 24 ops)
  - 8 FOV comparisons: dot² vs cos²_half_fov × dist_sq (3 muls, 1 cmp = 32 ops)
  - ~3-5 cells pass FOV test → reduced downstream work

Total added: ~80 arithmetic ops per creature (trivial on modern CPUs)
Savings: ~40-60% fewer biosignature lookups, classifications, and L1Vision pushes

Net: LIKELY FASTER due to reduced downstream work
```

### Benchmark Estimate

| Operation | Cost | Count (no cull) | Count (with cull) |
|-----------|------|-----------------|-------------------|
| Dot product | ~3 cycles | 0 | 8 |
| Bounds check | ~5 cycles | 8 | 8 |
| Biosig lookup | ~10 cycles | 8 | ~4 |
| classify_l1_cell | ~20 cycles | 8 | ~4 |
| L1Vision push | ~15 cycles | 8 | ~4 |

**Estimated per-creature cost:**
- Current: 8 × (5+10+20+15) = 400 cycles
- Proposed: 8×3 + 8×5 + 4×(10+20+15) = 24 + 40 + 180 = 244 cycles

**~40% reduction in L1 scan cost** (fewer downstream operations outweigh cone test cost)

---

## L2 Grid for Extended Range

### Existing Infrastructure

The L2 grid already exists but is **not used for perception**:

```
Grid Hierarchy:
  L0: 20m cells  - entity scanning (3×3 neighborhood)
  L1: 60m cells  - area awareness (currently 8-cell ring, 60-180m)
  L2: 180m cells - strategic layer (exists but unused for perception)
```

### Potential L2 Perception (Future Work)

For creatures with very long perception range (180m+), we could add L2 perception:

```
Range Bands:
  0-20m:    L0 entity scan (3×3 = 9 cells)
  60-180m:  L1 cone scan (up to 8 cells with FOV culling)  <- THIS FIX
  180-540m: L2 cone scan (up to 8 cells with FOV culling)  <- FUTURE
```

**Benefits of L2:**
- Giant creatures could perceive threats/prey at 300m+
- Fewer cells to test (L2 cells are 180m, so 8 cells covers huge area)
- Already has BioSignature aggregation (total_mass, max_size, creature_count)

**Implementation would be similar:**
```rust
// After L1 ring scan, if perception range >= L2_CELL_SIZE (180m):
if range >= L2_CELL_SIZE {
    for (dx, dy) in L2_RING_OFFSETS {
        // Same cone test logic, but using L2 grid
        let to_cell_x = dx as f32 * L2_CELL_SIZE + base_offset_x;
        let to_cell_y = dy as f32 * L2_CELL_SIZE + base_offset_y;
        // ... FOV test, add to L2Vision if in cone
    }
}
```

**Deferred:** Focus on L1 FOV culling first. L2 can be added later for apex predators.

---

## Open Questions

1. **Extended L1 cells** - The narrow/wide FOV tiers add extra cells in specific directions (front for predators, sides for prey). Should these also use cone intersection, or are they intentionally direction-specific extensions?

2. **Wide FOV handling** - For creatures with >180° FOV, the cone test math changes (cos_half_fov becomes negative). Ensure the existing `is_in_fov()` logic handles this correctly for L1.

3. **L2 perception** - Should we add L2 perception for creatures with 180m+ range in the same PR, or defer to a follow-up?

---

## Testing Plan

1. **Visual verification** - Select creature in debug mode, rotate slowly, verify L1 cells track the actual cone (no cells behind creature when facing forward)
2. **FOV boundary test** - Creature with 120° FOV should perceive ~3-4 L1 cells in front, 0 behind
3. **Wide FOV test** - Creature with 270° FOV should perceive 6-7 cells (all except directly behind)
4. **Rotation smoothness** - Rotate creature 1° at a time, verify cells enter/exit smoothly
5. **Performance benchmark** - Compare tick time before/after at 10K, 50K, 100K creatures
6. **Regression** - Ensure seek/flee behaviors still work (they depend on L1Vision data)

---

## CORRECTED Understanding (v2)

The goal is simple: **Any L1 cell within the FOV cone gets perceived.**

### Not This (Pattern-Based):
```
Fixed 8 ring cells → pattern lookup → which of the 8 to include
```

### THIS (Cone-Based):
```
FOV cone (angle + range) → which L1 cells fall inside → perceive those
```

### Visual Example

```
Creature with 150m range, 90° FOV, facing EAST:

    L1 Grid (60m cells):
    ┌───┬───┬───┬───┬───┐
    │   │   │   │   │   │
    ├───┼───┼───┼───┼───┤
    │   │   │ ▓ │ ▓ │   │  ← These 2 cells are within
    ├───┼───┼───┼───┼───┤      the 150m range AND
    │   │   │ ● │ ▓ │ ▓ │  ← within the 90° FOV cone
    ├───┼───┼───┼───┼───┤
    │   │   │ ▓ │ ▓ │   │  ← 6 total cells perceived
    ├───┼───┼───┼───┼───┤
    │   │   │   │   │   │
    └───┴───┴───┴───┴───┘

    ● = creature
    ▓ = L1 cells within FOV cone (perceived)
```

### The Algorithm

```
1. Determine search radius: max_cells = ceil(perception_range / L1_CELL_SIZE)
2. For each L1 cell within that radius (excluding self):
   a. Compute vector from creature to cell center
   b. Compute distance squared
   c. If distance > perception_range → skip (outside range)
   d. Compute dot product with facing direction
   e. If angle outside FOV → skip (outside cone angle)
   f. Otherwise → add to L1Vision
```

### What You Will See

**Small creature (range ~50m, narrow FOV):**
```
    ┌───┬───┬───┐
    │   │   │   │
    ├───┼───┼───┤
    │   │ ● │ ▓ │→   Only 1 L1 cell (barely reaches)
    ├───┼───┼───┤
    │   │   │   │
    └───┴───┴───┘
```

**Medium creature (range ~120m, 90° FOV):**
```
    ┌───┬───┬───┬───┐
    │   │   │ ▓ │   │
    ├───┼───┼───┼───┤
    │   │ ● │ ▓ │ ▓ │→   4-5 L1 cells in the cone
    ├───┼───┼───┼───┤
    │   │   │ ▓ │   │
    └───┴───┴───┴───┘
```

**Large creature (range ~200m, wide 180° FOV):**
```
    ┌───┬───┬───┬───┬───┐
    │   │ ▓ │ ▓ │ ▓ │   │
    ├───┼───┼───┼───┼───┤
    │   │ ▓ │ ● │ ▓ │ ▓ │→   Many L1 cells - wide cone, long range
    ├───┼───┼───┼───┼───┤
    │   │ ▓ │ ▓ │ ▓ │   │
    └───┴───┴───┴───┴───┘
```

**The gray L1 cells will match the teal FOV cone shape because they ARE the cone.**

### Key Difference from Current Code

**Current:** Always 8 ring cells, no FOV check, no range check
**New:** Variable number of cells - exactly those whose centers fall inside the FOV cone

---

## My Understanding of the Problem

### What I See In Your Screenshots

```
Screenshot 1 (creature facing WEST with narrow FOV):

    FOV cone (teal) →  ◄━━━━━━━●      Gray L1 cells (wrong):
                               ↑      ▓▓▓▓
                          creature    ▓▓▓▓  ← extends EAST (behind!)
                                      ▓▓▓▓

    The teal cone points LEFT (west)
    But gray cells extend RIGHT (east) - COMPLETELY WRONG DIRECTION
```

```
Screenshot 2 & 3 (creature with various FOVs):

    The gray L1 cells form patterns that DO NOT MATCH the teal FOV cone.
    - Gray cells appear where the cone DOESN'T reach
    - Gray cells are missing where the cone DOES reach
    - The patterns are "all over the place"
```

### Root Cause

The L1 ring scan has **NO FOV CHECK AT ALL**. It literally adds all 8 surrounding cells:

```rust
// Current code (line 529-559):
for (dx, dy) in RING_OFFSETS {  // All 8 neighbors
    // ... NO FOV CHECK ...
    l1_vision.push(...);  // Just adds it blindly
}
```

The comment even says "NO FOV CULLING" - it was intentionally designed this way, but it's wrong.

---

## The Solution

Add the **same FOV check** that entity perception uses. For each L1 cell, check if the direction to that cell falls within the creature's FOV cone.

### Exact Code Change

```rust
// BEFORE (no FOV check):
for (dx, dy) in RING_OFFSETS {
    let l1_dx = base_offset_x + dx as f32 * L1_CELL_SIZE;
    let l1_dy = base_offset_y + dy as f32 * L1_CELL_SIZE;
    // ... immediately adds to l1_vision
}

// AFTER (with FOV check):
for (dx, dy) in RING_OFFSETS {
    let l1_dx = base_offset_x + dx as f32 * L1_CELL_SIZE;
    let l1_dy = base_offset_y + dy as f32 * L1_CELL_SIZE;
    let dist_sq = l1_dx * l1_dx + l1_dy * l1_dy;

    // FOV CHECK - same as entity perception
    let rough_dot = l1_dx * facing_x + l1_dy * facing_y;
    if !is_in_fov(rough_dot, dist_sq, cos_half_fov, cos_half_fov_sq) {
        continue;  // Skip cells outside FOV
    }

    // ... only adds cells that pass FOV check
}
```

The `is_in_fov()` function already exists and works correctly for entity perception. I'm just using it for L1 cells too.

---

## What You Will See After The Fix

### Example 1: Narrow FOV (90°) facing EAST

```
    BEFORE:                          AFTER:

    ▓▓▓                              · · ▓
    ▓●▓  ← all 8 cells               · ● ▓ →  ← only cells in front
    ▓▓▓                              · · ▓

    (● = creature, ▓ = gray L1 cell, → = facing direction)
```

### Example 2: Wide FOV (180°) facing EAST

```
    BEFORE:                          AFTER:

    ▓▓▓                              · ▓ ▓
    ▓●▓  ← all 8 cells               · ● ▓ →  ← front half only
    ▓▓▓                              · ▓ ▓
```

### Example 3: Creature facing SOUTHEAST

```
    BEFORE:                          AFTER:

    ▓▓▓                              · · ▓
    ▓●▓  ← all 8 cells               · ● ▓ ↘  ← cells match diagonal cone
    ▓▓▓                              · ▓ ▓
```

### The Key Visual Test

**The gray L1 cells should fall WITHIN the teal FOV cone.**

- If the cone points east → gray cells to the east
- If the cone points southwest → gray cells to the southwest
- If the cone is narrow → fewer gray cells
- If the cone is wide → more gray cells

The gray L1 pattern should roughly trace the shape of the teal FOV cone.

---

## Files To Modify

1. `apps/simulation/src/simulation/perception/systems.rs`
   - L1 RING SCAN section (~line 529-559): Add FOV check
   - EXTENDED L1 CELLS section (~line 564-595): Add FOV check

That's it - just add `is_in_fov()` calls before pushing to `l1_vision`.
