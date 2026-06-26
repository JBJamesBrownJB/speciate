# Open Bugs — Scale / Simulation

Active bugs discovered during testing. Fix before merge to main when flagged URGENT.

---

## BUG-002 — Selection highlight not centred on creature; offset worse for small crits

**Discovered:** 2026-06-26  
**Status:** Open  
**Area:** `apps/portal/src/rendering/overlays/SelectionHighlight.ts` + `apps/portal/src/main.ts`  
**Priority:** Low (cosmetic; not affecting simulation correctness)

**Observed behaviour:**  
The yellow selection outline circle is visually offset from the selected creature.
The offset is most severe for small creatures — the smaller the crit, the further
the circle appears from its body. Large crits look roughly correct.

**Root cause (diagnosed):**  
Two coordinate streams are in play at the same time:

1. **Render position** — `InterpolatedCreatureRenderer` draws each creature at its
   *interpolated* world position (smoothly ahead of the last IPC snapshot by one
   render-frame's worth of movement).
2. **Highlight position** — `selectionHighlight.updatePosition(selected.x, selected.y)`
   every render frame uses `selectedCreature.x/y` from `SelectionManager`, which is
   updated by `updateSelectedFromBuffer` from the *raw IPC snapshot* (one tick behind
   the interpolated render).

The gap between these two positions is proportional to the creature's velocity × lag.
Small creatures have higher max speeds (allometric scaling) AND their body is small, so
even a modest position lag of 1–2 frames represents several body-lengths of offset.
Large creatures move slower per body-length, so the same absolute offset is invisible.

**Fix direction:**  
`selectionHighlight.updatePosition` should use the *interpolated* position of the
selected creature, not the raw IPC snapshot position. The interpolation buffer already
holds the current visual position — expose a `getInterpolatedPosition(id)` lookup from
`InterpolationBufferManager` and use that in the render loop instead of `selected.x/y`.

**Test to write (before fixing):**  
Unit-test `SelectionHighlight` with a mock that advances the creature position by one
frame of velocity between the IPC snapshot and the highlight update; assert the
highlight position matches the interpolated position, not the snapshot.

---

## BUG-001 — L1 grid biomass stats attributed to wrong cell [RESOLVED]

**Discovered:** 2026-06-26  
**Status:** Fixed — merged 2026-06-26 (fix/l1-cell-index-rca → main)  
**Area:** `apps/simulation/src/simulation/spatial/` — L1 aggregation  

**Observed behaviour:**  
A large creature visually positioned inside grid cell A is reported in the biomass
stats for the *adjacent* cell B. Cell A shows no large-creature signal; cell B
(where the creature is not visible) registers it. Confirmed by visual inspection
of the dev-ui grid overlay at 1M population.

**Likely root cause:**  
L1 aggregation is reading a creature's *registered L0 cell* (assigned at last
`grid_rebuild`) rather than its *current world position* when bucketing into L1
cells. If a creature crosses an L0 cell boundary between a `grid_rebuild` and the
subsequent `l1_aggregation`, the L1 slot it writes to is one cell behind its
visual position.

**Where to look:**  
- `apps/simulation/src/simulation/spatial/l1_aggregation.rs` — how each creature
  maps to an L1 cell index (position-derived vs. cached cell index?)
- `apps/simulation/src/simulation/spatial/grid.rs` — whether the stored cell index
  is updated atomically with position or lags by one tick

**Fix direction:**  
L1 aggregation should derive the L1 cell from the creature's *current `Position`
component*, not from any cached cell index. Alternatively, ensure `grid_rebuild`
and `l1_aggregation` run in the same tick with no movement in between (currently
they may not — verify Bevy schedule order).

**Test to write (before fixing):**  
Spawn a creature at a known position straddling an L1 cell boundary, run one tick,
assert that `l1_aggregation` reports the creature in the cell containing its
current `Position`, not its prior registered cell.
