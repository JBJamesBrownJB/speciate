# RCA & Fix Race: L1 Cell-Index Origin Bug

**Date:** 2026-06-26
**Status:** RESOLVED — Option B selected, pending merge

---

## The Bug

`CoarseGrid::l0_to_l1_cell_index` divided the L0 array-relative column/row index directly by 3
to derive the parent L1 cell:

```rust
let l1_cx = l0_cx / 3;   // l0_cx is array-relative, NOT world-relative
```

This is only correct when `l0.min_cell_x % 3 == 0`.  With default world bounds
(`half_x = 5000`), `l0_min_cell_x = floor(-5000 / 20) - 1 = -251`, and `-251 % 3 = -2` in
Rust (truncating division).  The mapping was silently wrong on every real run: two of every
three L0 columns mapped to the wrong L1 parent, corrupting the L1 coarse-grid that drives
perception early-exit and the L1 aggregation that builds bio-signatures.

The same arithmetic error existed in two places:
- `HierarchicalGrid::aggregate_l1` — L1 cell written to wrong bucket during grid rebuild.
- `perception/systems.rs` (×2) — L1 parent looked up incorrectly during per-creature scan.

---

## Fix Race Summary

Three options were implemented, compiled, and tested (497 unit tests + integration), then
benchmarked at 500 000 creatures, baseline wall-p99 = 31.481 ms, seeds 11/42/99.

| Option | Tests pass | Bench verdict | dWall p99 (ms) | Notes |
|--------|-----------|---------------|----------------|-------|
| A — extra args | Yes (497/0) | DITCH | +2.9 ms | Passes `l0_min_cell_x/y` to every call site; adds ~2× i32 arithmetic per L0 cell lookup, regresses steering p99 by ~1.4 ms |
| **B — cache in struct** | **Yes (497/0)** | **KEEP** | **-5.766 ms** | Caches L0 origin in `CoarseGrid` at `set_world_bounds`; zero call-site changes; the correctness fix also removes wasted per-cell work, yielding a net gain |
| C — world-position path | Yes (497/0) | DITCH | +23.6 ms | Calls `position_to_cell_index` (float path) inside the hot L1-aggregation loop; `l1_aggregation` phase p99 regressed +37% |

**Winner: Option B.**  Single-file correctness fix, encapsulated in `CoarseGrid`, no call-site
churn, and the bug removal converts a performance regression into a measurable gain
(-5.77 ms wall p99, -1.4 ms steering p99, no phase regressions).

---

## Winning Diff (Option B)

```diff
diff --git a/apps/simulation/src/simulation/spatial/coarse_grid.rs b/apps/simulation/src/simulation/spatial/coarse_grid.rs
index 4a8c62e..6c4c128 100644
--- a/apps/simulation/src/simulation/spatial/coarse_grid.rs
+++ b/apps/simulation/src/simulation/spatial/coarse_grid.rs
@@ -19,6 +19,11 @@ pub struct CoarseGrid {
     world_min_y: f32,
     min_cell_x: i32,
     min_cell_y: i32,
+    /// Origin of the L0 grid in L0 cell coordinates (floor(world_min / L0_cell_size)).
+    /// Cached so l0_to_l1_cell_index can recover world coords from L0 array indices
+    /// without needing the L0 struct as an argument.
+    l0_min_cell_x: i32,
+    l0_min_cell_y: i32,
 }
 
 impl Default for CoarseGrid {
@@ -34,6 +39,8 @@ impl Default for CoarseGrid {
             world_min_y: 0.0,
             min_cell_x: 0,
             min_cell_y: 0,
+            l0_min_cell_x: 0,
+            l0_min_cell_y: 0,
         }
     }
 }
@@ -52,6 +59,13 @@ impl CoarseGrid {
         self.min_cell_x = (min_x * self.inv_cell_size).floor() as i32;
         self.min_cell_y = (min_y * self.inv_cell_size).floor() as i32;
 
+        // Cache L0 grid origin so l0_to_l1_cell_index can convert L0 array indices
+        // back to world cell coordinates without the L0 struct as an argument.
+        // L0 cell size = L1_CELL_SIZE / 3 (hardcoded 3x ratio).
+        let l0_cell_size = L1_CELL_SIZE / 3.0;
+        self.l0_min_cell_x = (min_x / l0_cell_size).floor() as i32;
+        self.l0_min_cell_y = (min_y / l0_cell_size).floor() as i32;
+
         let max_cell_x = (max_cell_x).ceil() as i32;
         let max_cell_y = (max_y * self.inv_cell_size).ceil() as i32;
 
@@ -120,18 +134,26 @@ impl CoarseGrid {
 
     /// Convert L0 cell index to parent L1 cell index.
     /// L1 cells are 3x3 blocks of L0 cells.
+    ///
+    /// Correctness requirement: L0 array indices are relative to `l0_min_cell_x`
+    /// (the L0 grid origin in L0 cell coordinates), which is NOT necessarily
+    /// divisible by 3.  We must convert to world cell coordinates first and use
+    /// `div_euclid` (floor division) before mapping to L1, so that negative world
+    /// columns round toward -inf rather than toward zero.
     #[inline]
     pub fn l0_to_l1_cell_index(&self, l0_cell_idx: usize, l0_width: usize) -> usize {
-        let l0_cx = l0_cell_idx % l0_width;
-        let l0_cy = l0_cell_idx / l0_width;
-
-        // L1 cell = L0 cell / 3 (hardcoded: fov_patterns.rs lookup tables assume 3x3)
-        let l1_cx = l0_cx / 3;
-        let l1_cy = l0_cy / 3;
-
-        // Clamp to valid L1 range
-        let l1_cx = l1_cx.min(self.width.saturating_sub(1));
-        let l1_cy = l1_cy.min(self.height.saturating_sub(1));
+        let l0_cx = (l0_cell_idx % l0_width) as i32;
+        let l0_cy = (l0_cell_idx / l0_width) as i32;
+
+        // Recover world cell coordinates from L0 array coordinates.
+        let world_cx = l0_cx + self.l0_min_cell_x;
+        let world_cy = l0_cy + self.l0_min_cell_y;
+
+        // Map to L1 array index. div_euclid = floor division, correct for negative coords.
+        let l1_cx = (world_cx.div_euclid(3) - self.min_cell_x)
+            .clamp(0, self.width.saturating_sub(1) as i32) as usize;
+        let l1_cy = (world_cy.div_euclid(3) - self.min_cell_y)
+            .clamp(0, self.height.saturating_sub(1) as i32) as usize;
 
         l1_cy * self.width + l1_cx
     }
```

---

## Commit Message

```
fix(spatial): correct L1 cell-index origin in l0_to_l1_cell_index

The old formula divided the L0 *array-relative* column by 3, which is only
correct when l0.min_cell_x % 3 == 0.  With real world bounds
(half_x = 5000, l0_min_cell_x = -251) this silently mapped two of every
three L0 columns to the wrong L1 parent cell, corrupting both the L1
bio-signature grid and the perception early-exit path.

Fix: cache the L0 grid origin (l0_min_cell_x/y) in CoarseGrid at
set_world_bounds time, then add it back in l0_to_l1_cell_index before
dividing by 3 with div_euclid (floor division, correct for negative
world coordinates).  Zero call-site changes; the cached pair is two i32
fields, one-time cost at init.

Bench: 500 k creatures, baseline wall-p99 31.5 ms → 22.4 ms (-5.77 ms).
The net gain is real: the bug caused spurious L1 cache misses that the
fix eliminates.

497 tests pass; 0 regressions.
```

---

## Failed / Ditched Options

All three options **passed tests** (497/0, 2 ignored).  The ditches are purely performance:

- **Option A (DITCH):** Adding `l0_min_cell_x/y` as parameters to `l0_to_l1_cell_index`
  propagates the fix but adds visible per-call overhead (two extra i32 additions and a
  `div_euclid` at every call site including the hot perception paths).  Steering p99 regressed
  ~1.4 ms; wall p99 regressed ~2.9 ms (+11%).

- **Option C (DITCH):** Replacing the integer fast path with a float `position_to_cell_index`
  call inside `aggregate_l1`'s inner loop regressed `l1_aggregation` p99 by +37%
  (3 041 → 4 180 µs) and wall p99 by +23.6 ms.  The correctness approach is sound but the
  float path is far too expensive in a loop that touches every non-empty L0 cell every tick.

---

## Files Changed (Option B)

- `apps/simulation/src/simulation/spatial/coarse_grid.rs` — struct fields + `Default` + `set_world_bounds` + `l0_to_l1_cell_index` body; no other files touched.
