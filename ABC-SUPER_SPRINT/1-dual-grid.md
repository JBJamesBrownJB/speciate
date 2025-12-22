# Phase A: Dual Spatial Grid

## Status: IN PROGRESS

Infrastructure complete. Now implementing two-stage perception with size domination.

---

## Prior Work (Complete)

| Component | Status | Notes |
|-----------|--------|-------|
| L1 Coarse Grid (30m cells) | Done | `spatial/coarse_grid.rs` |
| BioSignature struct | Done | `total_mass`, `max_size`, `creature_count` |
| L1 Aggregation System | Done | L0 → L1 every tick |
| HierarchicalGrid wrapper | Done | `spatial/hierarchical.rs` |
| Portal visualization | Done | G key cycles: Off → L0 → L1 |
| L1 hover query | Done | Cell info panel on mouse hover |
| Perception threshold field | Done | `perception.threshold = mass * 0.05` |

**Bug identified:** Current L1 early-exit only checks perceiver's own cell, not target cells. Size domination not working.

---

## Architecture (Team Consultation)

### Design Decision: Fixed-Size Components + Pure Functions

After consulting architect-andy and ecs-emma, the recommended architecture is:

**Option B+C Hybrid**: Fixed-size components with isolated pure functions for testability.

### Why NOT Vec-based Components

```rust
// BAD - heap allocation = cache miss at 500K scale
pub struct L1Perceptions {
    cells: Vec<L1CellPerception>,
}

// GOOD - inline array, sequential memory, prefetcher happy
pub struct L1Perceptions {
    count: u8,
    cells: [L1CellPerception; 48],
}
```

- 500K creatures with `Vec` = 500K random heap accesses
- Fixed array = sequential memory access
- Archetype stability guaranteed (size never changes)

### Memory Budget (500K creatures)

| Component | Bytes/Entity | 500K Total |
|-----------|--------------|------------|
| NeighborCache | ~120 | 60 MB |
| L1Perceptions (48 cells) | ~770 | 385 MB |
| **Total** | ~890 | **445 MB** |

**Verdict:** Acceptable for target hardware.

### Architecture Diagram

```
┌─────────────────────────────────────────────────────────────────┐
│                      COMPONENTS (stable)                         │
├─────────────────────────────────────────────────────────────────┤
│  NeighborCache     │ Fixed array [NeighborData; 7]              │
│  L1Perceptions     │ Fixed array [L1CellPerception; 48]         │
│  PerceptionConfig  │ DNA-derived ranges, thresholds             │
└─────────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────────┐
│              PURE FUNCTIONS (testable, isolated)                 │
├─────────────────────────────────────────────────────────────────┤
│  classify_l1_cell(biosig, my_mass, my_size, is_own_cell)       │
│      -> L1Classification { Empty, Threat, Prey, Crowded }      │
│                                                                 │
│  should_perceive_entity(threshold, target_mass, dist, fov)     │
│      -> bool                                                    │
│                                                                 │
│  compute_l1_drive(l1_perceptions)                              │
│      -> Vec2 (direction)  [Phase B]                            │
└─────────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────────┐
│                   TYPE ALIASES (queries.rs)                      │
├─────────────────────────────────────────────────────────────────┤
│  PerceptionQuery   │ Full query with all perception components  │
│  L1PerceptionQuery │ Read L1Perceptions for drive system        │
│  SteeringQuery     │ Read NeighborCache for steering            │
└─────────────────────────────────────────────────────────────────┘
```

### Data Structures

```rust
pub const MAX_L1_PERCEPTIONS: usize = 48;

#[repr(u8)]
#[derive(Clone, Copy, Default, PartialEq, Eq)]
pub enum L1Classification {
    #[default]
    Empty   = 0,  // Nothing worth perceiving
    Threat  = 1,  // Contains creature larger than me
    Prey    = 2,  // Contains creatures smaller than me (huntable)
    Crowded = 3,  // Contains visible mass but no threat/prey
}

#[derive(Clone, Copy)]
#[repr(C)]
pub struct L1CellPerception {
    pub cell_idx: u32,              // 4 bytes
    pub classification: L1Classification,  // 1 byte
    pub _pad: [u8; 3],              // 3 bytes alignment
    pub direction: Vec2,            // 8 bytes
}
// Total: 16 bytes per cell (cache-line friendly)

#[derive(Component)]
pub struct L1Perceptions {
    count: u8,
    cells: [L1CellPerception; MAX_L1_PERCEPTIONS],
}
// Total: ~770 bytes per creature
```

### Extensibility Path

| Future Feature | Extension Point |
|----------------|-----------------|
| Phase B Drives | Add `compute_l1_drive()` pure function |
| Gregariousness | New `L1Classification::Kin` variant |
| Species recognition | Extend BioSignature with species hash |
| Complex AI | Upgrade to trait-based classifiers |

---

## Biology Consultation (zoologist-tom)

### Size Domination Threshold: Approved

**5% threshold is biologically plausible** based on predator energy economics:
- Lion (500kg) ignoring mouse (0.5kg) = correct behavior
- Pursuit energy > caloric return for tiny prey

**Future enhancement:** Dual-threshold for "peripheral awareness"
```
ignore_threshold = my_mass * 0.02   // Completely invisible
notice_threshold = my_mass * 0.10   // Worth tracking
```
Creatures between 2-10% exist in peripheral zone - noticed but not tracked.

### GOLDEN ZONE Opportunities

Performance optimizations that give FREE emergent biological behavior:

| Optimization | Performance Win | Free Biology | Entertainment | Status |
|--------------|-----------------|--------------|---------------|--------|
| **Size domination** | Skip tiny entities | Energy economics | High | Phase A |
| **FOV culling** | Skip rear cells | Blind spot | Medium | Phase A |
| **Motion detection** | Skip stationary entities | Prey freeze = camouflage | Very High | Deferred (see `docs/biology/todo/`) |
| **Hunger gating** | Skip prey when full | Satiated predators rest | High | Deferred (see `docs/biology/todo/`) |
| **Distance attenuation** | Sample fewer far cells | Visual acuity degrades | High | Future |

### Classification Set (Phase A)

| Classification | Condition | Response |
|----------------|-----------|----------|
| EMPTY | count=0 OR mass < threshold | Safe passage, wander target |
| THREAT | max_size > my_size | Flee, avoid |
| PREY | max_size < my_size * 0.3 | Hunt, approach |
| CROWDED | Has mass, no threat/prey | Avoid (default behavior) |

**Note:** DNA-driven crowding response (solitary vs social) deferred to future. See `docs/biology/todo/crowding-affinity.md`.

### Research References

- [Madrona Framework](https://madrona-engine.github.io/shacklett_siggraph23.pdf) - ECS for high-performance batched AI
- [Hytale ECS](https://hytale.com/news/2024/6/summer-2024-technical-explainer-hytale-s-entity-component-system-oPwpCAMdI) - Decoupling data/logic for parallelization
- [big-brain](https://github.com/zkat/big-brain) - Utility AI pattern: Scorers + Actions
- [bevy_observed_utility](https://github.com/ItsDoot/bevy_observed_utility) - Scoring → Picking → Acting lifecycle

---

## Algorithm

```
PERCEPTION SYSTEM (per creature)
================================

1. CALCULATE L1 CELLS TO SCAN
   - Use perception.range to determine L1 cell radius
   - Apply FOV culling (reuse collect_cells_sorted_fov pattern)
   - Result: variable number of L1 cells based on creature size/FOV

2. L1 SCAN (classify each L1 cell)
   For each L1 cell in range + FOV:
     biosig = l1_grid.get(cell)
     if my_cell:
       biosig.total_mass -= my_mass
       biosig.count -= 1

     CLASSIFY (pure function):
       EMPTY:   count == 0 OR total_mass < threshold
       THREAT:  max_size > my_size
       PREY:    max_size < my_size * 0.3
       CROWDED: has mass, no threat/prey

     Store: L1Perceptions.push({ cell_idx, direction, classification })

3. L0 SCAN (always 9 cells max, for steering)
   For each of 9 adjacent L0 cells:
     Check parent L1 cell classification
     If EMPTY for me -> skip this L0 cell (size domination optimization)
     Otherwise: scan entities with per-entity filtering

     For each entity in L0 cell (pure function filter):
       SKIP if: entity == self
       SKIP if: distance > perception_range
       SKIP if: entity.mass < my_threshold  <- SIZE DOMINATION
       SKIP if: outside FOV
       ADD to NeighborCache

4. OUTPUTS
   NeighborCache: for steering systems (fixed array)
   L1Perceptions: for Phase B drive system (fixed array)
```

### L1 Scan Scope (Decision: Variable + FOV)

L1 scan uses perception range and FOV culling (same as L0):
- 5m creature: range ~100m -> ~4 L1 cells radius -> ~49 cells, ~25 after FOV cull
- L1 biosignatures are cheap (12 bytes each), checking 40 is trivial
- Consistent with existing perception architecture
- Strategic awareness scales with perception range

### Self-Pollution Solution

When creature checks its own L1 cell:
- Subtract own mass from `total_mass`
- Decrement `count` by 1
- Cannot adjust `max_size` (don't know second-largest)
- Solution: Own cell always gets L0 scan (small cost, guarantees correctness)

---

## Testing Strategy

### Pure Functions (Unit Testable Without ECS)

#### classification.rs

```rust
pub fn classify_l1_cell(
    biosig: &BioSignature,
    my_mass: f32,
    my_size: f32,
    is_my_cell: bool,
) -> L1Classification
```

**Unit tests:**
- Empty cell -> Empty
- Cell with only tiny crits (below threshold) -> Empty
- Cell with giant (max_size > my_size) -> Threat
- Cell with small crits (max_size < my_size * 0.3) -> Prey
- Cell with many medium crits (total_mass > threshold, no threat/prey) -> Crowded
- Own cell subtraction works correctly

#### entity_filter.rs

```rust
pub fn should_perceive_entity(
    my_threshold: f32,
    target_mass: f32,
    distance_sq: f32,
    perception_range_sq: f32,
    in_fov: bool,
) -> bool
```

**Unit tests:**
- Target below threshold -> false (size domination)
- Target above threshold, in range, in FOV -> true
- Target above threshold, out of range -> false
- Target above threshold, out of FOV -> false

### Integration Tests

Test that perception system correctly:
- Calls L1 classifier with correct biosignatures
- Skips L0 cells when L1 says "empty"
- Applies per-entity size filtering
- Outputs correct NeighborCache and L1Perceptions

**Test trials:** `apps/simulation/specs/behavior/l1-*.toml` (6 files)

---

## Validation Checklist

- [x] L1 aggregation produces correct totals
- [x] Portal shows both grids (G key cycling)
- [x] L1 hover query shows correct cell info
- [x] L1 aggregation < 1ms at 20K creatures
- [ ] L1 classifier unit tests pass
- [ ] Entity filter unit tests pass
- [ ] Size domination: Giant ignores mouse (doesn't perceive)
- [ ] Size domination: Mouse perceives giant
- [ ] L0 scan skips cells marked EMPTY by L1
- [ ] L1Perceptions populated for Phase B
- [ ] PREY classification working (predator sees prey-rich cells)

---

## Files to Create/Modify

| File | Change |
|------|--------|
| `perception/classification.rs` | NEW: L1Classification enum, classify_l1_cell() |
| `perception/entity_filter.rs` | NEW: should_perceive_entity() |
| `perception/components.rs` | ADD: L1Perceptions, L1CellPerception structs |
| `perception/queries.rs` | NEW: Type aliases for perception queries |
| `perception/systems.rs` | MODIFY: Two-stage perception algorithm |
| `perception/mod.rs` | Export new modules |
