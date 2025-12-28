# Phase B: Simple Drive Simplex - Implementation Plan

## Goal

Replace discrete `BehaviorMode` enum with continuous drive-based behavior system.

**Architectural Vision:** The drive simplex is the **macro navigation layer** - "where should I go?" - fed by multiple sensory channels. Phase B implements the foundation; future sprints add sound, scent, seismic, and habitat influences.

---

## Architecture: Two-Tier Drive Integration

Based on zoologist consultation, real animals use layered integration:

```
┌─────────────────────────────────────────────────────────────────┐
│  TIER 0: EMERGENCY (priority override - no blending)           │
│  - Immediate predator charging                                  │
│  - Critical energy depletion                                    │
│  - Amygdala-driven, reflexive                                   │
└─────────────────────────────────────────────────────────────────┘
                              ↓ (if no emergency)
┌─────────────────────────────────────────────────────────────────┐
│  TIER 1: MOTIVATED (weighted sum with state-modulated weights) │
│  - Food seeking (weight × hunger modifier)                      │
│  - Social cohesion (weight × DNA social trait)                  │
│  - Dispersion (weight × crowding aversion)                      │
└─────────────────────────────────────────────────────────────────┘
```

**Phase B Scope:** Implements Tier 1 (weighted sum). Emergency tier deferred to future sprint.

---

## Core Data Structures

### DriveSource Enum (Extensible)

```rust
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum DriveSource {
    Vision = 0,    // Phase B - L1 perception
    Sound = 1,     // Future - mating calls, predator sounds
    Scent = 2,     // Future - pheromones, chemical trails
    Seismic = 3,   // Future - footstep detection
    Habitat = 4,   // Future - biome preferences
}
```

### DriveTier Enum

```rust
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DriveTier {
    Emergency,   // Priority override (Phase B: unused, future)
    Motivated,   // Weighted sum (Phase B: all drives use this)
}
```

### DriveContribution Struct

```rust
#[derive(Clone, Copy, Debug)]
pub struct DriveContribution {
    pub source: DriveSource,
    pub tier: DriveTier,
    pub vector: Vec2,
    pub magnitude: f32,  // 0.0-1.0 normalized
}
```

### DriveState Component

```rust
use smallvec::SmallVec;

#[derive(Component, Clone, Debug, Default)]
pub struct DriveState {
    // Category sums (O(1) access for visualization triangle)
    pub flee: Vec2,
    pub approach: Vec2,
    pub disperse: Vec2,

    // Contribution arrays (cleared each tick after combine)
    // SmallVec<[T; 4]> = stack-allocated for typical case (Vision + Sound + Scent + Seismic)
    pub flee_contributions: SmallVec<[DriveContribution; 4]>,
    pub approach_contributions: SmallVec<[DriveContribution; 4]>,
    pub disperse_contributions: SmallVec<[DriveContribution; 4]>,

    // Final output for steering
    pub combined: Vec2,
}
```

**Why three arrays instead of one?**
- Avoids O(n) filtering per frame to separate categories
- At 10K creatures × 5 contributions = 50K filter operations saved per tick
- Each array maps directly to a simplex triangle vertex

---

## System Architecture: Gather-Then-Process

```
┌─────────────────────────────────────────────────────────────────┐
│                     Per-Tick Pipeline                           │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐          │
│  │ VisionDrive  │  │ SoundDrive   │  │ ScentDrive   │  ...     │
│  │   System     │  │   System     │  │   System     │          │
│  │  (Phase B)   │  │  (Future)    │  │  (Future)    │          │
│  └──────┬───────┘  └──────┬───────┘  └──────┬───────┘          │
│         │                 │                 │                   │
│         ▼                 ▼                 ▼                   │
│  ┌──────────────────────────────────────────────────────────┐  │
│  │                    DriveState Component                   │  │
│  │  flee_contributions: [Vision→Vec2, Sound→Vec2, ...]       │  │
│  │  approach_contributions: [Vision→Vec2, Scent→Vec2, ...]   │  │
│  │  disperse_contributions: [Vision→Vec2, ...]               │  │
│  └──────────────────────────────────────────────────────────┘  │
│                              │                                  │
│                              ▼                                  │
│  ┌──────────────────────────────────────────────────────────┐  │
│  │              DriveCombineSystem (Rayon parallel)          │  │
│  │  1. Sum flee_contributions → flee (with DNA weights)      │  │
│  │  2. Sum approach_contributions → approach                 │  │
│  │  3. Sum disperse_contributions → disperse                 │  │
│  │  4. Blend categories → combined                           │  │
│  │  5. Clear all contribution arrays                         │  │
│  └──────────────────────────────────────────────────────────┘  │
│                              │                                  │
│                              ▼                                  │
│  ┌──────────────────────────────────────────────────────────┐  │
│  │                   SteeringSystem                          │  │
│  │  Reads: DriveState.combined                               │  │
│  └──────────────────────────────────────────────────────────┘  │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

**Key Design Decisions:**

1. **Sensory systems run in parallel** - No write conflicts (each pushes to different source)
2. **Single combine system** - Applies DNA weights, clears arrays, computes output
3. **Clear at end of combine** - Not at start of sensory systems (prevents ordering bugs)

---

## Golden Zone Opportunities

### 1. Freeze Response (Cornered Prey)

When flee directions cancel (surrounded by threats), combined output ≈ zero.

**Result:** Creature freezes - biologically accurate tonic immobility AND computationally efficient (no movement to process).

### 2. Sensory Gating by DNA

Creatures with low sensitivity for a sense skip that system entirely:

```rust
pub fn sound_drive_system(query: Query<(&mut DriveState, &Dna, ...)>) {
    for (mut drive_state, dna, ...) in query.iter_mut() {
        if dna.express_sound_sensitivity() < HEARING_THRESHOLD {
            continue;  // Deaf creature - skip processing
        }
        // ... process sound contributions
    }
}
```

**Result:** Optimization IS the biology - deaf creatures don't process sound.

### 3. Satiation Blindness

Low hunger → reduced approach weight → predator "ignores" nearby prey.

**Result:** Post-kill rest behavior emerges from weight modulation.

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

### Step 1: Add Drive Types and DriveState Component
**New file:** `simulation/drives/mod.rs`

Create the core types:
- `DriveSource` enum
- `DriveTier` enum
- `DriveContribution` struct
- `DriveState` component with SmallVec contribution arrays

**Changes:**
- Add `smallvec` to Cargo.toml dependencies
- Add to `CritBundle` (alongside BehaviorMode initially)
- Add `is_active()` method (magnitude > threshold)

**Test:** Unit tests for contribution pushing and array management

---

### Step 2: Create VisionDriveSystem
**Compute drives from L1Perceptions - the first sensory contributor**

**New file:** `simulation/drives/vision.rs`

```rust
pub fn vision_drive_system(
    mut query: Query<(&Position, &Velocity, &L1Perceptions, &NeighborCache, &mut DriveState)>,
)
```

**Algorithm:**
1. For each L1 perception:
   - THREAT → push to `flee_contributions` (with velocity urgency from L0 neighbors)
   - PREY → push to `approach_contributions`
   - CROWDED → push to `disperse_contributions` (repel direction)
   - EMPTY → push to `disperse_contributions` (attract direction)

**Threat velocity urgency:**
- Check L0 neighbors in threat direction
- Approaching (>2 m/s toward) → magnitude = 1.0
- Retreating → magnitude = 0.2
- Stationary → magnitude = 0.5

**Test:** Integration tests for contribution generation

---

### Step 3: Create DriveCombineSystem
**Separate system that processes all contributions**

**New file:** `simulation/drives/combine.rs`

```rust
pub fn drive_combine_system(
    mut query: Query<(&mut DriveState, &Dna)>,
) {
    // Rayon parallel - collect into Vec first (Sprint 15 pattern)
    let mut entities: Vec<_> = query.iter_mut().collect();

    entities.par_iter_mut().for_each(|(drive_state, dna)| {
        // 1. Weighted sum per category (Phase B: uniform weights)
        drive_state.flee = weighted_sum(&drive_state.flee_contributions);
        drive_state.approach = weighted_sum(&drive_state.approach_contributions);
        drive_state.disperse = weighted_sum(&drive_state.disperse_contributions);

        // 2. Blend into final combined vector
        drive_state.combined = blend_categories(
            drive_state.flee,
            drive_state.approach,
            drive_state.disperse,
        );

        // 3. Clear for next tick
        drive_state.flee_contributions.clear();
        drive_state.approach_contributions.clear();
        drive_state.disperse_contributions.clear();
    });
}
```

**Phase B simplification:** All weights = 1.0. DNA-modulated weights come in future sprint.

**Test:** Unit tests for weighted sum math, blend logic

---

### Step 4: Modify Steering for Drives
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
if let Some(target) = target {
    // Target OVERRIDES drive (for testing/forced encounters)
    accel += seek_toward(target)
} else if drive_state.combined.length_squared() > DRIVE_THRESHOLD_SQ {
    // Normal drive-based movement
    let drive_force = drive_state.combined.normalize() * max_accel * DRIVE_MULT;
    accel += drive_force;
}
// No drives = creature rests (emergent behavior)
// Avoidance still additive after primary drive
```

**Test scenario support:**
- `CritBuilder.as_seeker(x, y)` still works for visual trials
- Production crits spawn without Target → pure drive behavior

---

### Step 5: Remove Wandering System
**Disperse drive replaces random wandering**

**Changes:**
- Remove wander case from steering switch
- Remove `WanderState` from queries
- Keep `Target` component as **override** for testing

**Emergent replacement:**
- Wandering → Disperse drive toward EMPTY cells

---

### Step 6: Remove BehaviorMode
**Final cleanup**

**Files:**
- `creatures/components/state.rs` - Remove enum
- `creatures/builder.rs` - Remove behavior field
- `core/simulation.rs` - Remove type registration
- `behaviors/transitions/systems.rs` - Simplify to age/energy only
- All test files referencing BehaviorMode

---

### Step 7: Visualization (Portal)
**Debug overlays for selected creature**

**7a. L1 Perception Lines**
- Draw lines from creature center → L1 cell centers
- Color by classification: THREAT=Red, PREY=Orange, CROWDED=Yellow, EMPTY=Green
- Only for selected creature (performance)
- Toggle with existing perception overlay (extend, don't replace L0 lines)

**IPC:** Add L1Perceptions to creature debug snapshot
- `l1_perceptions: Vec<{cell_idx, classification, direction_x, direction_y}>`

**7b. Drive Simplex Triangle**
- HUD element showing flee/approach/disperse balance
- Triangle with vertices: FLEE(top), APPROACH(bottom-left), DISPERSE(bottom-right)
- Floating dot position = weighted average of active drives
- Center = resting, near vertex = strong single drive

**IPC:** Add DriveState summary to creature debug snapshot
- `drive_state: {flee_x, flee_y, approach_x, approach_y, disperse_x, disperse_y, combined_x, combined_y}`

---

## System Ordering

```
rebuild_spatial_grid_system
    ↓
aggregate_l1_system
    ↓
update_perception_system     ← Populates L1Perceptions
    ↓
vision_drive_system          ← NEW: Pushes contributions
    ↓
drive_combine_system         ← NEW: Weighted sum, clears arrays
    ↓
behavior_transition_system   ← Simplify (age/energy only)
    ↓
update_steering_system       ← Reads DriveState.combined
    ↓
integrate_motion_system
```

---

## File Structure

```
apps/simulation/src/simulation/
├── drives/
│   ├── mod.rs              # DriveState, DriveSource, DriveTier, DriveContribution
│   ├── combine.rs          # DriveCombineSystem
│   └── contributions/
│       ├── mod.rs
│       └── vision.rs       # VisionDriveSystem (Phase B)
│                           # Future: sound.rs, scent.rs, seismic.rs, habitat.rs
```

---

## Key Files to Modify

| File | Change |
|------|--------|
| `Cargo.toml` | Add `smallvec` dependency |
| `perception/systems.rs` | Add L1Perceptions population |
| `perception/components.rs` | Verify L1Perceptions API |
| `simulation/drives/mod.rs` | NEW - Core types and DriveState |
| `simulation/drives/combine.rs` | NEW - DriveCombineSystem |
| `simulation/drives/contributions/vision.rs` | NEW - VisionDriveSystem |
| `creatures/components/state.rs` | Remove BehaviorMode |
| `creatures/steering/system.rs` | Use DriveState.combined |
| `creatures/builder.rs` | Add DriveState, remove BehaviorMode |
| `core/simulation.rs` | Register new systems |

---

## Emergent Behaviors (No Explicit Code)

| Situation | Behavior | Why |
|-----------|----------|-----|
| Empty area, no neighbors | Rests | No contributions → combined ≈ zero |
| Crowded area | Disperses to emptier cells | CROWDED → disperse contributions |
| Large crit nearby | Flees | THREAT → flee contributions |
| Large crit charging | Explosive flee | velocity urgency = 1.0 |
| Large crit resting | Cautious grazing | velocity urgency = 0.2 |
| Prey-rich area | Drifts toward | PREY → approach contributions |
| Path blocked | Weaves around | L0 avoidance still active |
| Surrounded by threats | Freezes | Flee vectors cancel → zero output |

---

## Phase B vs Future Work

| Phase B Delivers | Future Sprints |
|------------------|----------------|
| DriveState with contribution arrays | Emergency tier (priority override) |
| VisionDriveSystem | SoundDriveSystem, ScentDriveSystem |
| DriveCombineSystem (uniform weights) | DNA-modulated weights |
| Basic flee/approach/disperse | State modulation (hunger affects weights) |
| L1 perception overlay | Habitat influence maps |
| Drive simplex triangle | Personality types from DNA |

---

## Validation Criteria

- [ ] Crits naturally disperse across world
- [ ] Small crits flee from large crits
- [ ] Crits rest when alone (no jittering)
- [ ] Freeze response when surrounded (flee vectors cancel)
- [ ] Avoidance still prevents collisions
- [ ] No BehaviorMode in codebase
- [ ] Threat velocity urgency visible (charge vs rest response)
- [ ] L1 perception overlay shows correct colors
- [ ] Drive simplex triangle displays for selected creature
- [ ] Adding new DriveSource requires no changes to DriveState struct
