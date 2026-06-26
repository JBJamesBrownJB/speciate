# Open Bugs — Scale / Simulation

Active bugs discovered during testing. Fix before merge to main when flagged URGENT.

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
