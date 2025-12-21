# Sprint: Simple Drive Simplex (Phase B)

## Outcome

Remove behavior state machine. Everything becomes drive-based.

**Depends on:** Phase A (Dual Grid) - L1 BioSignatures

---

## Architecture: No Behavior States

**Remove:** `BehaviorMode` enum (Catatonic, Wandering, Seeking, Fleeing)

**Replace with:** Continuous drives that produce emergent behavior

---

## The Drive System

Two layers, always active:

### Layer 1: L1 Navigation Drive
Uses coarse grid to decide **where to go**:
- **Repulsion** from L1 cells with large crits (`max_size` > self)
- **Attraction** to L1 cells with lower `total_mass` (emptier areas)

### Layer 2: L0 Avoidance
Uses fine grid neighbors for **immediate collision avoidance**:
- Lateral steering around nearby crits
- Unchanged from current system

---

## Emergent Behaviors (No Explicit Code)

| Situation | What Happens | Why |
|-----------|--------------|-----|
| Empty L1 cell, no neighbors | Crit rests (no drive, no avoidance) | No gradients to follow |
| Crowded L1 cell | Crit drifts toward emptier cells | Attraction to low mass |
| Large crit nearby (L1) | Small crit moves away | Repulsion from max_size |
| Crit approaches another (L0) | Lateral dodge | Layer 2 avoidance kicks in |
| Path blocked by others | Navigates around | Avoidance + drive combined |
| All L1 cells equally populated | Rests in equilibrium | No gradient to follow |

**Equilibrium note:** When all nearby cells have equal mass, the crit rests. This is correct - it's found its place among equals. Perfect equilibrium is rare; as others move, gradients shift and the crit responds.

**Key insight:** "Resting", "wandering", "fleeing" are not states - they're descriptions of what the drives produce.

---

## What This Replaces

| Old | New |
|-----|-----|
| `BehaviorMode::Catatonic` | No drive gradient → naturally still |
| `BehaviorMode::Wandering` | Attraction to empty space → natural dispersal |
| `BehaviorMode::Fleeing` | Repulsion from large crits → moves away |
| Random direction changes | Gradient following → purposeful movement |

---

## Baseline: All Crits Are Loners

Default behavior without DNA complexity:
- Spread out to find space
- Avoid larger crits
- Rest when alone

**Future DNA additions:**
- Gregariousness → prefer crowds
- Schooling → match neighbor velocity
- Aggression → approach instead of flee

---

## Implementation Steps (High Level)

1. **Add DriveState component** - stores combined drive direction
2. **Create L1 drive system** - computes repulsion + attraction from L1 grid
3. **Integrate with steering** - drive feeds into acceleration alongside avoidance
4. **Remove BehaviorMode** - delete enum and state transition logic
5. **Remove wandering system** - L1 drive replaces it

---

## Validation

- [ ] Crits naturally disperse across the world
- [ ] Small crits avoid areas with large crits
- [ ] Crits rest when in empty areas (no jittering)
- [ ] Layer 2 avoidance still prevents collisions
- [ ] No BehaviorMode enum in codebase
- [ ] Wandering system removed

---

## Files to Modify

| Area | Change |
|------|--------|
| `creatures/components/` | Add DriveState, remove BehaviorMode |
| `creatures/behaviors/` | Add L1 drive system, remove wander |
| `creatures/steering/` | Integrate drive into acceleration |
| `core/simulation.rs` | Update system registration |
