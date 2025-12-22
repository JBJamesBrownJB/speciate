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
- **Repulsion** from L1 cells classified as THREAT (`max_size` > self)
- **Attraction** to L1 cells classified as EMPTY (lower `total_mass`)
- **Hunt drive** toward L1 cells classified as PREY (`max_size` < self * 0.3)
- **Avoidance** of CROWDED cells (default behavior, DNA-driven variations deferred)

### Layer 2: L0 Avoidance
Uses fine grid neighbors for **immediate collision avoidance**:
- Lateral steering around nearby crits
- Unchanged from current system
- Only use minimum (9 surrounding cells) for neighbours assuming this works for crits of all sizes.

---

## Threat Variety (Nuanced Response)

Phase A provides simple THREAT classification. Phase B drive system adds **velocity-aware urgency**:

### Threat + Velocity Interpretation

```rust
// L1Perception provides: THREAT classification + cell direction
// L0 scan provides: actual threat entity velocity (if in range)

let threat_vector = threat_position - my_position;
let approach_speed = threat_velocity.dot(threat_vector.normalize());

let flee_urgency = if approach_speed > APPROACH_THRESHOLD {
    // THREAT_APPROACHING: charging toward me
    1.0  // Maximum urgency flee
} else if approach_speed < -APPROACH_THRESHOLD {
    // THREAT_RETREATING: moving away from me
    0.2  // Low urgency, maybe ignore
} else {
    // THREAT_STATIONARY: not moving or perpendicular
    0.5  // Monitor, gradual repositioning
};

// Apply to drive
flee_drive = threat_direction * -flee_urgency;
```

### Emergent Behaviors from Threat Variety

| Situation | flee_urgency | Behavior |
|-----------|--------------|----------|
| Predator charging | 1.0 | Explosive flee response |
| Predator resting | 0.2 | Cautious grazing nearby |
| Predator circling | 0.5 | Gradual repositioning |
| Predator turns away | drops to 0.2 | Relaxation, resume grazing |

**Entertainment value:** Creates cat-and-mouse dynamics. Prey relaxes when predator turns away, panics when it charges. Players observe tension building and releasing.

**Note:** The L1 classification itself doesn't change - drive system interprets THREAT + L0 velocity data.

---

## Emergent Behaviors (No Explicit Code)

| Situation | What Happens | Why |
|-----------|--------------|-----|
| Empty L1 cell, no neighbors | Crit rests (no drive, no avoidance) | No gradients to follow |
| Crowded L1 cell | Crit drifts toward emptier cells | Default avoidance of crowds |
| Large crit nearby (L1) | Small crit moves away | Repulsion from THREAT |
| Large crit charging | Small crit flees explosively | flee_urgency = 1.0 |
| Large crit resting | Small crit grazes cautiously nearby | flee_urgency = 0.2 |
| Prey-rich L1 cell | Predator drifts toward area | Attraction to PREY |
| Crit approaches another (L0) | Lateral dodge (prevents contact) | Layer 2 avoidance kicks in |
| Path blocked by others | Navigates around | Avoidance + drive combined |
| All L1 cells equally populated | Rests in equilibrium | No gradient to follow |

**Equilibrium note:** When all nearby cells have equal mass, the crit rests. This is correct - it's found its place among equals. Perfect equilibrium is rare; as others move, gradients shift and the crit responds.

**Key insight:** "Resting", "wandering", "fleeing", "hunting" are not states - they're descriptions of what the drives produce.

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
- Spread out to find space (avoid CROWDED)
- Avoid larger crits (flee from THREAT)
- Drift toward smaller crits (PREY attraction)
- Rest when alone (no gradients)

**Scope limitation:** Phase B delivers navigation, not predation. Predators drift toward prey-rich areas but L0 avoidance prevents actual contact. Catching/eating/death are post-ABC features that require suppressing avoidance for a selected hunt target.

**Future DNA-driven variations** (deferred, see `docs/biology/todo/`):
- `crowding_affinity` → solitary (-1) to swarm (+1)
- `aggression` → flee vs approach THREAT
- Schooling → match neighbor velocity

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
- [ ] Threat variety: prey flees faster when predator charges
- [ ] Threat variety: prey grazes cautiously near resting predator
- [ ] Hunt drive: predators drift toward PREY-classified cells (L0 avoidance still active)

---

## Files to Modify

| Area | Change |
|------|--------|
| `creatures/components/` | Add DriveState, remove BehaviorMode |
| `creatures/behaviors/` | Add L1 drive system, remove wander |
| `creatures/steering/` | Integrate drive into acceleration |
| `core/simulation.rs` | Update system registration |
