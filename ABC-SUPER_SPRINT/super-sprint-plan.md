# ABC Super Sprint: Performance + Drive Architecture

## Goal

Transform the simulation from discrete behavior states to continuous drive-based behavior, with performance infrastructure enabling 500K+ creatures.

---

## Phases

| Phase | Name | Complexity | Focus | Status |
|-------|------|------------|-------|--------|
| **A** | Dual Spatial Grid | Medium | Infrastructure + Size Domination | ✓ COMPLETE |
| **C** | System Update Frequency | Small | Runtime Hz Control | Pending |
| **B** | Simple Drive Simplex | Large | Continuous Drives (Loner Behavior) | Pending |

**Order Rationale:** A and C are performance/infrastructure. B is a major behavior overhaul. Do infrastructure first for immediate wins.

---

## Phase A: Dual Spatial Grid ✓ COMPLETE

**What:** Add L1 coarse grid (30m) on top of existing L0 fine grid (10m).

**Why:**
- Early-exit optimization: creatures in empty areas skip detailed perception
- Size domination: large crits don't see small ones (emergent behavior)
- Foundation for L1-based navigation in Phase B

**Delivered:**
- L1 BioSignature (total_mass, max_size, creature_count) ✓
- Perception threshold (5% of body mass) ✓
- Early-exit optimization in perception system ✓
- Portal visualization (G key cycles: Off → L0 → L1) ✓
- L1 cell size sent via telemetry IPC ✓
- L1 hover query with info panel (replaces heatmap streaming) ✓
- Unit tests for L1CellInfo calculations (5 tests) ✓

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

---

## Phase B: Simple Drive Simplex

**What:** Replace BehaviorMode enum with continuous drives.

**Why:**
- Emergent behavior from simple rules
- Extensible for future complex drives (fight/flight/freeze, gregariousness)
- Cleaner architecture (no state machine)

**Delivers:**
- DriveState component
- L1 repulsion (away from large crits) + attraction (toward empty cells)
- Two-layer steering: drives (forward/back) + avoidance (lateral)
- All crits as "loners" by default

---

## Architectural Principles

### Force Separation

| Layer | Controls | Direction |
|-------|----------|-----------|
| **Layer 1: Drives** | Strategic intent (where to go) | Forward/backward |
| **Layer 2: Avoidance** | Tactical evasion (obstacles) | Lateral |

**Result:** Avoidance always appears to dominate without explicit priority logic.

### Emergent Behaviors

| Situation | What Happens |
|-----------|--------------|
| Empty L1 cell | Crit rests |
| Crowded L1 cell | Drifts toward emptier cells |
| Large crit nearby | Small crit moves away |
| All cells equal | Rests in equilibrium |

---

## Success Criteria

- [ ] Phase A: L1 aggregation < 0.5ms, early-exit reduces sparse perception 50%+
- [ ] Phase C: Zero overhead at divisor=1, linear scaling with throttling
- [ ] Phase B: Drive computation < 2ms, visible loner behavior
- [ ] Overall: 18% tick time reduction, 500K creatures @ 10Hz viable

---

## Future Extensions (Not in ABC)

After ABC completes, drive architecture supports:
- Fight/flight/freeze
- Aggression / boldness genes
- Gregariousness (prefer crowds)
- Schooling (match neighbor velocity)
- Tau-based threat (velocity consideration)

These build on the Layer 1/Layer 2 separation established in Phase B.
