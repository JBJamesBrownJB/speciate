# Phase B: Simple Drive Simplex - Implementation Plan

## Goal

Replace discrete `BehaviorMode` enum with continuous drive-based behavior system.

---

## Current State (from exploration)

| Component | Status | Location |
|-----------|--------|----------|
| `BehaviorMode` enum | EXISTS (Catatonic, Seeking, Wandering, Waiting) | `creatures/components/state.rs:9-23` |
| Behavior transitions | NONE - creatures stay in spawn state forever | `behaviors/transitions/systems.rs` |
| `L1Perceptions` component | EXISTS but NEVER POPULATED | `perception/components.rs:200-248` |
| `classify_l1_cell()` | WORKS (results discarded after early-exit) | `perception/classification.rs:29-59` |
| TTC avoidance | WORKS (Phase 4 complete) | `steering/avoidance.rs` |
| Steering | Force accumulation: primary (exclusive) + avoidance (additive) | `steering/system.rs` |

---

## Implementation Steps

### Step 0: Populate L1Perceptions
**Foundation for all drive computation**

**File:** `perception/systems.rs`

Add L1 scan after L0 perception:
1. Iterate L1 cells within perception range + FOV
2. Classify each using existing `classify_l1_cell()`
3. Compute normalized direction to cell center
4. Store non-EMPTY cells in `L1Perceptions` component

**Test:** Spawn creature, verify L1Perceptions.count() > 0

---

### Step 1: Add DriveState Component
**New component for continuous drives**

**New file:** `creatures/components/drive.rs`

```rust
#[derive(Component, Clone, Copy, Debug, Default)]
pub struct DriveState {
    pub flee: (f32, f32),      // Repulsion from THREAT
    pub hunt: (f32, f32),      // Attraction to PREY
    pub disperse: (f32, f32),  // Repulsion from CROWDED, attraction to EMPTY
    pub combined_direction: (f32, f32),
    pub combined_magnitude: f32,
}
```

**Changes:**
- Add to `CritBundle` (alongside BehaviorMode initially)
- Add `is_active()` method (replaces `BehaviorMode::is_active()`)

**Test:** Unit tests for drive combination math

---

### Step 2: Create L1 Drive System
**Compute drives from L1Perceptions**

**New file:** `creatures/behaviors/drive.rs`

```rust
pub fn compute_l1_drives_system(
    query: Query<(&Position, &Velocity, &L1Perceptions, &NeighborCache, &mut DriveState)>,
)
```

**Algorithm:**
1. For each L1 perception:
   - THREAT → accumulate flee (with velocity urgency from L0 neighbors)
   - PREY → accumulate hunt
   - CROWDED → accumulate disperse (repel)
   - EMPTY → accumulate disperse (attract)
2. Update combined drive

**Threat velocity urgency:**
- Check L0 neighbors in threat direction
- Approaching (>2 m/s toward) → urgency = 1.0
- Retreating → urgency = 0.2
- Stationary → urgency = 0.5

**Test:** Integration tests for emergent behaviors

---

### Step 3: Modify Steering for Drives
**Replace BehaviorMode switch with drive-based steering**

**File:** `steering/system.rs`

**Before:**
```rust
match creature_state.behavior {
    Wandering => accel += wander()
    Seeking => accel += seek()
}
```

**After:**
```rust
if drive_state.is_active() {
    let (dir_x, dir_y) = drive_state.combined_direction;
    let drive_force = drive_state.combined_magnitude * max_accel * DRIVE_MULT;
    accel += (dir_x * drive_force, dir_y * drive_force);
}
// No drives = creature rests (emergent behavior)
```

**Avoidance unchanged:** Still additive, runs after primary drive

---

### Step 4: Remove Wandering, Keep Target Override
**No more random direction changes, but keep explicit target-seeking for tests**

**Changes:**
- Remove wander case from steering switch
- Remove `WanderState` from queries
- Keep `Target` component as **override** for testing/forced encounters

**Steering priority:**
```rust
if let Some(target) = target {
    // Target OVERRIDES drive (for testing/forced encounters)
    accel += seek_toward(target)
} else if drive_state.is_active() {
    // Normal drive-based behavior
    accel += drive_direction()
}
// Avoidance still additive
```

**Test scenario support:**
- `CritBuilder.as_seeker(x, y)` still works for visual trials
- Crowd-navigation, giant-vs-mice trials use Target to force encounters
- Production crits spawn without Target → pure drive behavior

**Emergent replacement:**
- Wandering → Disperse drive toward EMPTY cells

---

### Step 5: Remove BehaviorMode
**Final cleanup**

**Files:**
- `creatures/components/state.rs` - Remove enum
- `creatures/builder.rs` - Remove behavior field
- `core/simulation.rs` - Remove type registration
- `behaviors/transitions/systems.rs` - Simplify to age/energy only
- All test files referencing BehaviorMode

---

### Step 6: Visualization (Portal) - IN SCOPE
**Debug overlays for selected creature**

**6a. L1 Perception Lines**
- Draw lines from creature center → L1 cell centers
- Color by classification: THREAT=Red, PREY=Orange, CROWDED=Yellow, EMPTY=Green
- Only for selected creature (performance)
- Toggle with existing perception overlay (extend, don't replace L0 lines)

**IPC:** Add L1Perceptions to creature debug snapshot
- `l1_perceptions: Vec<{cell_idx, classification, direction_x, direction_y}>`

**Files:**
- `napi_addon/simulation_engine.rs` - Add L1Perceptions to debug snapshot
- `portal/src/rendering/` - New overlay or extend existing

**6b. Drive Simplex Triangle**
- HUD element showing flee/hunt/disperse balance
- Triangle with vertices: FLEE(top), HUNT(bottom-left), DISPERSE(bottom-right)
- Floating dot position = weighted average of active drives
- Center = resting, near vertex = strong single drive

**IPC:** Add DriveState to creature debug snapshot
- `drive_state: {flee, hunt, disperse, combined_direction, combined_magnitude}`

**Files:**
- `napi_addon/simulation_engine.rs` - Add DriveState to debug snapshot
- `portal/src/components/` - New DriveSimplex component

---

## System Ordering

```
rebuild_spatial_grid_system
    ↓
aggregate_l1_system
    ↓
update_perception_system     ← Add L1Perceptions population
    ↓
compute_l1_drives_system     ← NEW
    ↓
behavior_transition_system   ← Simplify (age/energy only)
    ↓
update_steering_system       ← Use DriveState
    ↓
integrate_motion_system
```

---

## Key Files to Modify

| File | Change |
|------|--------|
| `perception/systems.rs` | Add L1Perceptions population |
| `perception/components.rs` | Verify L1Perceptions API |
| `creatures/components/drive.rs` | NEW - DriveState component |
| `creatures/components/state.rs` | Remove BehaviorMode |
| `creatures/behaviors/drive.rs` | NEW - Drive computation system |
| `creatures/steering/system.rs` | Use drives instead of BehaviorMode |
| `creatures/builder.rs` | Add DriveState, remove BehaviorMode |
| `core/simulation.rs` | Register new systems |

---

## Emergent Behaviors (No Explicit Code)

| Situation | Behavior | Why |
|-----------|----------|-----|
| Empty area, no neighbors | Rests | No drives active |
| Crowded area | Disperses to emptier cells | CROWDED repulsion |
| Large crit nearby | Flees | THREAT repulsion |
| Large crit charging | Explosive flee | urgency = 1.0 |
| Large crit resting | Cautious grazing | urgency = 0.2 |
| Prey-rich area | Drifts toward | PREY attraction |
| Path blocked | Weaves around | L0 avoidance still active |

---

## Validation Criteria

- [ ] Crits naturally disperse across world
- [ ] Small crits flee from large crits
- [ ] Crits rest when alone (no jittering)
- [ ] Avoidance still prevents collisions
- [ ] No BehaviorMode in codebase
- [ ] Threat velocity urgency visible (charge vs rest response)
- [ ] L1 perception overlay shows correct colors
- [ ] Drive simplex displays for selected creature
