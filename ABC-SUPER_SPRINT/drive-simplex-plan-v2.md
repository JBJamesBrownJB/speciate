# Drive Simplex Implementation Plan v2

**Status:** PLANNING
**Previous Attempt:** See `drive-simplex-plan-FAILED.md` for lessons learned

---

## Goal

Replace discrete BehaviorMode enum with continuous drive-based behavior.

---

## v2 Phases

- Confirm L1 grid has good coordinates in world
- Crits percieve L1 cells: after a crit does its L0 scan, it proceeds to L1 scan and fills its 'Biome Store' with L1 cells based in its FOV. It may only scan one random L1 cell within its FOV for performance? we will try out a bunch of options here. This step will need us to decide how many, what pattern of L1 cells its scans as well as frontend visualisation of the crit perciving the L1 cell with a line like in L0 cells perception. We will add visual aids in --dev-tools overlay to show L1 cell scan hits just like we have for L0 cell scan polling etc..
- Crits categorise L1 cells based on what is there, we will start simple, like if it contains a crit 20% larger than itself, it categorises it as a threat. Add visual overlay aids, all behind -dev-tools flag on the portal so that when we select a crit, it colour codes L1 cells with its category (safe/threat)
- At this point maybe we swap out wandering state with drive simplex...? check at the time
- We will decide how to proceed at this point... it may be, that this just replaces 'wandering' with drive simplex, and we proceed with other things at this point...?

---

## L1 Border Repulsion Integration

**Goal:** Use L1 border cells to create natural avoidance zone at map edges.

### Implementation Steps

1. **Flag border cells at init** - Mark L1 cells at map edges as "border" category (static, compute once)
2. **Add border repulsion force** - Creatures in border cells receive constant outward force toward map center
3. **Integrate with L1 perception** - When crits scan L1 cells, border cells auto-categorize as "avoid" (red in dev overlay)
4. **Wander target filtering** - Drive simplex excludes border cells from wander target candidates

### Visual Overlay (--dev-tools)

- Border cells rendered with distinct color (e.g., orange/red gradient)
- When creature selected, show repulsion force vector if in border zone

### Golden Zone Opportunity

Predators feel weaker border repulsion than prey (scale by predator_score). Creates emergent "edge hunting" where predators can herd prey toward constrained edges - mirrors wolf/lion tactics.

**See:** `docs/biology/ideas/l1-border-repulsion.md` for full design.

---
