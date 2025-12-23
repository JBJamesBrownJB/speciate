# ABC Super Sprint: Performance + Drive Architecture

## Goal

Transform the simulation from discrete behavior states to continuous drive-based behavior, with performance infrastructure enabling 500K+ creatures.

---

## What You'll See After Each Phase

| Phase | What You'll Observe | Key Mechanics |
|-------|---------------------|---------------|
| **A: Dual Grid** | Giants walk through mice (ignore them). Mice flee from giants. Predators notice prey-rich areas. | Size domination, L1 classification (EMPTY/THREAT/PREY/CROWDED), FOV culling |
| **B: Drive Simplex** | Crits spread out naturally. Predators drift toward prey-rich areas (but avoid individuals). Prey grazes near resting predators but flees when charged. No more random wandering. | L1 drives (repulsion/attraction), threat velocity urgency, remove BehaviorMode |
| **C: Frequency Control** | Same behaviors, but running at 500K+ creatures. Dev-UI sliders to tune perception/behavior Hz. | Entity-ID bucketing, runtime Hz adjustment |

---

## Scope: What ABC Is NOT

ABC delivers **navigation gesture** without **predation execution**.

| What ABC Delivers | What ABC Does NOT Deliver |
|-------------------|---------------------------|
| Predators navigate toward prey-rich L1 cells | Predators catching prey |
| Prey flees from threat-classified areas | Prey actually being eaten |
| Size-based asymmetric perception | Death, reproduction, energy transfer |
| Continuous drives replacing state machine | Actual hunting behavior |

**Critical limitation:** The two-layer steering architecture means predators will **avoid** prey when close:

1. **Layer 1 (L1 drives):** Predator attracted to PREY-classified cells → moves toward herd
2. **Layer 2 (L0 avoidance):** Once near actual creatures → lateral dodge kicks in

**Result:** Predators drift toward prey-rich areas, then weave around individuals without ever making contact. It's the opposite of hunting - more like "magnetically attracted to the herd but repelled by individuals."

**Why this is OK:** ABC establishes the navigation infrastructure. Actual predation requires:
- Hunt state that suppresses L0 avoidance for selected target
- Catch mechanics (collision → capture)
- Energy transfer (eating)
- Death system

These are **post-ABC features** that build on the drive architecture.

---

## Phases

| Phase | Name | Complexity | Focus | Status |
|-------|------|------------|-------|--------|
| **A** | Dual Spatial Grid | Medium | Infrastructure + Size Domination | IN PROGRESS |
| **4** | Better Avoidance | Small | TTC-Based Anti-Collision | **NEW** |
| **C** | System Update Frequency | Small | Runtime Hz Control | Pending |
| **B** | Simple Drive Simplex | Large | Continuous Drives (Loner Behavior) | Pending |

**Order Rationale:** A establishes grid infrastructure. **Phase 4** fixes collision avoidance using TTC (creatures steer around each other properly). C is performance tuning. B is the major behavior overhaul (flee/chase/drives).

---

## Phase A: Dual Spatial Grid (IN PROGRESS)

**What:** Add L1 coarse grid (30m) on top of existing L0 fine grid (10m). Implement two-stage perception with size domination.

**Why:**
- Early-exit optimization: creatures in empty areas skip detailed perception
- Size domination: large crits don't see small ones (emergent behavior)
- Foundation for L1-based navigation in Phase B

**Infrastructure (Complete):**
- L1 BioSignature (total_mass, max_size, creature_count) ✓
- Perception threshold field (5% of body mass) ✓
- Portal visualization (G key cycles: Off → L0 → L1) ✓
- L1 hover query with info panel ✓
- Unit tests for L1CellInfo calculations ✓

**Current Work:**
- Two-stage perception (L1 classify → L0 scan)
- L1 classification: EMPTY, THREAT, PREY, CROWDED
- Size domination filtering (per-entity and per-cell)
- L1Perceptions component (fixed array, 48 cells max)

**You'll See:**
- 5m giant walks straight through crowd of 0.5m mice (doesn't perceive them)
- Mice scatter away from giant (they perceive the threat)
- Perception lines show asymmetry (mice → giant, not giant → mice)
- L1 grid overlay shows cell classifications

**Details:** See `1-dual-grid.md`

---

## Phase 4: Better Avoidance (NEW)

**What:** Fix collision avoidance using Time-to-Collision (TTC).

**Scope:** Anti-collision ONLY. Flee/chase/predator dynamics come in Phase B.

**Why:**
- Current bug: Perception detects at ~8m, avoidance kicks in at ~0.25m edge
- Not enough time to steer around large obstacles
- Root cause: `max_interaction_distance` gate filters out perceived neighbors

**Delivers:**
- TTC-based urgency (closing speed determines reaction)
- Golden Zone: skip calculation for diverging paths
- Simpler code (one formula replaces multiple checks)

**Core Formula:**
```
closing_speed = dot(relative_velocity, direction_to_them)
if closing_speed <= 0: skip  // Moving apart
ttc = edge_distance / closing_speed
urgency = (critical_time / ttc).clamp(0, 1)
force = urgency² * max_accel
```

**You'll See:**
- Fast approaches trigger strong avoidance
- Parallel/diverging paths don't waste force
- Creatures smoothly steer around each other

**Details:** See `4-better-avoid.md`

---

## Phase B: Simple Drive Simplex

**What:** Replace BehaviorMode enum with continuous drives.

**Why:**
- Emergent behavior from simple rules
- Extensible for future complex drives
- Cleaner architecture (no state machine)

**Delivers:**
- DriveState component
- L1 repulsion (away from THREAT cells)
- L1 attraction (toward EMPTY cells)
- Hunt drive (toward PREY cells)
- Threat velocity urgency (flee faster when charged)
- Two-layer steering: drives (forward/back) + avoidance (lateral)

**You'll See (new):**
- Crits naturally disperse (no hardcoded wandering)
- Predators drift toward prey-rich areas (but L0 avoidance prevents actual contact)
- Prey relaxes near stationary predator, panics when it moves toward them
- No more random direction changes - all movement is gradient-following
- "Resting" emerges when no gradients exist

**Removed:**
- BehaviorMode enum (Catatonic, Wandering, Seeking, Fleeing)
- Wandering system

**Details:** See `2-simple-drive-simplex.md`

---

## Phase C: System Update Frequency

**What:** Runtime-adjustable Hz for cognitive systems (perception, behavior, steering).

**Why:**
- Reduce CPU usage proportionally with throttling
- Zero overhead at full rate (divisor=1)
- Dev-UI control for performance tuning

**Delivers:**
- FreqConfig resource with per-system divisors
- Entity-ID bucketing (no visual artifacts)
- Inline sliders in dev-ui below sparklines

**You'll See:**
- Same behaviors at massive scale (500K+ creatures)
- Dev-UI Hz sliders for performance tuning
- Smooth performance even at high creature counts

**Details:** See `3-frequency-control.md`

---

## Architectural Principles

### Force Separation

| Layer | Controls | Direction |
|-------|----------|-----------|
| **Layer 1: Drives** | Strategic intent (where to go) | Forward/backward |
| **Layer 2: Avoidance** | Tactical evasion (obstacles) | Lateral |

**Result:** Avoidance always appears to dominate without explicit priority logic.

### L1 Classification

| Classification | Condition | Response |
|----------------|-----------|----------|
| EMPTY | count=0 OR mass < threshold | Safe passage, wander target |
| THREAT | max_size > my_size | Flee, avoid |
| PREY | max_size < my_size * 0.3 | Hunt, approach |
| CROWDED | Has mass, no threat/prey | Avoid (default) |

### Emergent Behaviors

| Situation | What Happens | Why |
|-----------|--------------|-----|
| Empty L1 cell | Crit rests | No gradients |
| Crowded L1 cell | Drifts toward emptier cells | Avoidance drive |
| Large crit nearby | Small crit moves away | THREAT repulsion |
| Large crit charging | Small crit flees explosively | flee_urgency = 1.0 |
| Large crit resting | Small crit grazes nearby | flee_urgency = 0.2 |
| Prey-rich cell | Predator drifts toward area (avoids individuals) | PREY attraction + L0 avoidance |
| All cells equal | Rests in equilibrium | No gradient |

---

## Success Criteria

- [ ] Phase A: L1 classification working, size domination visible
- [ ] Phase B: Drive computation < 2ms, visible loner behavior, no BehaviorMode
- [ ] Phase C: Zero overhead at divisor=1, linear scaling with throttling
- [ ] Overall: 500K creatures @ 10Hz viable

---

## Future Extensions (Not in ABC)

After ABC completes, drive architecture supports (see `docs/biology/todo/`):
- Motion detection (prey freeze = camouflage)
- Hunger gating (satiated predators rest, starving giants chase mice)
- Crowding affinity DNA gene (solitary vs social)
- Fight/flight/freeze
- Aggression / boldness genes
- Schooling (match neighbor velocity)

These build on the Layer 1/Layer 2 separation established in Phase B.
