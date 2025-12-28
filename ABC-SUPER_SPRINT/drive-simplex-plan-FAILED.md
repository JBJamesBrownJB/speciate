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

### Drive Components (ECS-Optimized Hot/Warm Split)

**ecs-emma recommendation:** Split into hot/warm paths for cache efficiency at 500K scale.

```rust
// HOT PATH: 8 bytes - read every tick by steering
// Steering reads 4MB (split) vs 114MB (monolithic) at 500K creatures
#[derive(Component, Clone, Copy, Default)]
pub struct DriveOutput {
    pub combined: (f32, f32),
}

// WARM PATH: ~100 bytes - written by vision, read by combine
#[derive(Component, Clone, Copy, Default)]
pub struct DriveContributions {
    // Fixed arrays prevent heap allocation at scale (not SmallVec)
    pub flee: [DriveContribution; 4],
    pub flee_count: u8,
    pub approach: [DriveContribution; 4],
    pub approach_count: u8,
    pub disperse: [DriveContribution; 4],
    pub disperse_count: u8,
}

impl DriveContributions {
    /// Push contribution with bounds check - silently ignores if array full.
    /// Phase B: ignore overflow (minor threats dropped). Future: replace weakest.
    pub fn push_flee(&mut self, dir: (f32, f32), mag: f32) {
        if (self.flee_count as usize) < self.flee.len() {
            self.flee[self.flee_count as usize] = DriveContribution { direction: dir, magnitude: mag };
            self.flee_count += 1;
        }
        // else: array full, silently ignore (acceptable for Phase B)
    }
    // Similar for push_approach, push_disperse...
}

// DEV-TOOLS ONLY: Simplex triangle visualization (cold path)
#[cfg(feature = "dev-tools")]
#[derive(Component, Clone, Copy, Default)]
pub struct DriveSimplex {
    pub flee: (f32, f32),
    pub approach: (f32, f32),
    pub disperse: (f32, f32),
}

// FREEZE TIMEOUT: Tracks freeze duration for desperate escape
#[derive(Component, Clone, Copy, Default)]
pub struct FreezeState {
    pub ticks_frozen: u16,           // Incremented when drives ≈ 0
    pub escape_direction: (f32, f32), // Random direction for desperate escape
}

impl FreezeState {
    const DESPERATE_THRESHOLD: u16 = 100;  // ~4.5 seconds at 22Hz

    pub fn is_desperate(&self) -> bool {
        self.ticks_frozen >= Self::DESPERATE_THRESHOLD
    }

    pub fn tick(&mut self) {
        self.ticks_frozen = self.ticks_frozen.saturating_add(1);
        if self.ticks_frozen == Self::DESPERATE_THRESHOLD {
            use rand::Rng;
            let angle = rand::thread_rng().gen_range(0.0..std::f32::consts::TAU);
            self.escape_direction = (angle.cos(), angle.sin());
        }
    }

    pub fn reset(&mut self) {
        self.ticks_frozen = 0;
        self.escape_direction = (0.0, 0.0);
    }
}
```

**Why this split?** (ecs-emma review)
- Hot/warm split: Steering reads 4MB vs 114MB at 500K creatures
- Fixed arrays [T; 4]: No heap allocation (SmallVec spills at scale)
- Three arrays: O(1) access for simplex triangle, avoids O(n) filtering
- FreezeState: Prevents permanent freeze = certain death scenario

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
    mut query: Query<(&Position, &BodySize, &L1Perceptions, &NeighborCache, &mut DriveContributions)>,
)
```

**Algorithm:**
```rust
// Rayon parallel - HIGH VARIANCE workload, small chunks
entities.par_iter_mut().with_min_len(64).for_each(|(l1_perceptions, contributions, size, ...)| {
    let self_biomass = size.mass();

    for perception in l1_perceptions.iter() {
        match perception.classification {
            THREAT => {
                let urgency = calculate_threat_urgency(perception, neighbor_cache);
                contributions.push_flee(perception.direction, urgency);
            }
            PREY => contributions.push_approach(perception.direction, 1.0),
            CROWDED => {
                // COMFORT ZONE: Skip disperse if crowding_ratio in 0.3-1.5 range
                let crowding_ratio = perception.biomass / self_biomass;
                if crowding_ratio < 0.3 || crowding_ratio > 1.5 {
                    contributions.push_disperse(-perception.direction, 0.5);  // AWAY
                }
                // else: Comfortable density, no disperse needed
            }
            EMPTY => contributions.push_disperse(perception.direction, 0.3),  // TOWARD
        }
    }
});
```

**Comfort Zone (zoologist-tom):**
- `crowding_ratio` = neighbor biomass / self biomass
- Range 0.3-1.5 = "comfortable" - no disperse drive generated
- Below 0.3 = too sparse, seek company (mild disperse toward)
- Above 1.5 = too crowded, disperse away

**Threat velocity urgency:**
- Check L0 neighbors in threat direction
- Charging (>2 m/s toward) → magnitude = 1.0
- Stationary → magnitude = 0.5
- Retreating → magnitude = 0.2

**Chunk Size:** 64-128 (not 256) due to high per-entity variance in L1 perception count.

**Test:** Integration tests for contribution generation, comfort zone skipping

---

### Step 3: Create DriveCombineSystem
**Separate system that processes all contributions**

**New file:** `simulation/drives/combine.rs`

**Drive Priority Weights (zoologist-tom):**
- FLEE: 1.0 (survival dominates)
- APPROACH: 0.6-0.8 (hunting is secondary)
- DISPERSE: 0.2-0.4 (comfort is tertiary)

```rust
pub fn drive_combine_system(
    mut query: Query<(&mut DriveContributions, &mut DriveOutput, Option<&mut DriveSimplex>)>,
) {
    // Rayon parallel - LOW VARIANCE workload, larger chunks
    let mut entities: Vec<_> = query.iter_mut().collect();

    entities.par_iter_mut().with_min_len(256).for_each(|(contributions, output, simplex)| {
        // 1. Weighted sum per category
        let mut flee_vec = weighted_sum(&contributions.flee[..contributions.flee_count as usize]);
        let mut approach_vec = weighted_sum(&contributions.approach[..contributions.approach_count as usize]);
        let mut disperse_vec = weighted_sum(&contributions.disperse[..contributions.disperse_count as usize]);

        // 2. CRITICAL: Clamp magnitude to 1.0 BEFORE weighting
        // Prevents "Summation Overpower" - 10 prey items shouldn't override 1 predator
        flee_vec = clamp_magnitude(flee_vec, 1.0);
        approach_vec = clamp_magnitude(approach_vec, 1.0);
        disperse_vec = clamp_magnitude(disperse_vec, 1.0);

        // 3. Apply priority weights (now Flee always wins: 1.0 > 0.7 > 0.3)
        const FLEE_WEIGHT: f32 = 1.0;
        const APPROACH_WEIGHT: f32 = 0.7;
        const DISPERSE_WEIGHT: f32 = 0.3;

        let combined = (
            flee_vec.0 * FLEE_WEIGHT + approach_vec.0 * APPROACH_WEIGHT + disperse_vec.0 * DISPERSE_WEIGHT,
            flee_vec.1 * FLEE_WEIGHT + approach_vec.1 * APPROACH_WEIGHT + disperse_vec.1 * DISPERSE_WEIGHT,
        );

        // 4. Write to hot-path component (steering reads this)
        output.combined = combined;

        // 5. Dev-tools: capture simplex for visualization (clamped values)
        #[cfg(feature = "dev-tools")]
        if let Some(simplex) = simplex {
            simplex.flee = flee_vec;
            simplex.approach = approach_vec;
            simplex.disperse = disperse_vec;
        }

        // 6. Clear contributions for next tick
        contributions.flee_count = 0;
        contributions.approach_count = 0;
        contributions.disperse_count = 0;
    });
}

// Helper: Clamp vector magnitude to max without changing direction
fn clamp_magnitude(v: (f32, f32), max: f32) -> (f32, f32) {
    let mag_sq = v.0 * v.0 + v.1 * v.1;
    if mag_sq > max * max && mag_sq > 0.0001 {
        let scale = max / mag_sq.sqrt();
        (v.0 * scale, v.1 * scale)
    } else {
        v
    }
}
```

**Chunk Size:** 256 (low variance - all entities do same simple math).

**Phase B:** Fixed priority weights. DNA-modulated weights come in future sprint.

**Test:** Unit tests for weighted sum math, priority hierarchy

---

### Step 4: Modify Steering for Drives (with Freeze Timeout)
**Replace BehaviorMode switch with drive-based steering + desperate escape**

**File:** `steering/system.rs`

**Before:**
```rust
match creature_state.behavior {
    Wandering => accel += wander()
    Seeking => accel += seek()
}
```

**After (with freeze timeout):**
```rust
// Priority 1: Explicit target (for tests/trials)
if target.has_explicit_target() {
    let result = apply_seek(position, velocity, target, size);
    if result.arrived {
        target.clear();  // Clear target instead of setting Catatonic
    } else {
        acceleration.ax += result.acceleration.0;
        acceleration.ay += result.acceleration.1;
    }
}
// Priority 2: Desperate escape (freeze timeout exceeded)
else if freeze_state.is_desperate() {
    // Pick random escape direction, burst acceleration
    let escape_dir = freeze_state.escape_direction;
    let burst_force = max_accel * DESPERATE_ESCAPE_MULT;  // 1.5-2.0x normal
    acceleration.ax += escape_dir.0 * burst_force;
    acceleration.ay += escape_dir.1 * burst_force;
    freeze_state.reset();  // Clear freeze counter
}
// Priority 3: Drive-based steering (normal operation)
else if magnitude_sq(drive_output.combined) > DRIVE_THRESHOLD_SQ {
    let drive_dir = normalize(drive_output.combined);
    let drive_force = (drive_dir.0 * max_accel * DRIVE_MULT,
                       drive_dir.1 * max_accel * DRIVE_MULT);
    acceleration.ax += drive_force.0;
    acceleration.ay += drive_force.1;
    freeze_state.reset();  // Clear freeze counter (not frozen)
}
// Priority 4: Frozen (drives cancel out)
else {
    // Track freeze duration
    freeze_state.tick();  // Increment freeze counter
    // No acceleration = creature rests/freezes
}
// Avoidance still additive after primary drive
```

**Freeze Timeout (zoologist-tom recommendation):**
- Real prey freeze when escape routes blocked (tonic immobility)
- But prolonged freezing = certain death (predator will eventually reach)
- Desperate escape burst = "last ditch" gamble to break through
- Random direction because no good option exists (any direction equally risky)
- ~4.5 seconds (100 ticks at 22Hz) before desperate escape triggers

**Test scenario support:**
- `CritBuilder.as_seeker(x, y)` still works for visual trials
- Production crits spawn without Target → pure drive behavior
- Freeze timeout ensures no creature freezes forever

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
| `perception/systems.rs` | Add L1Perceptions population (Step 0) |
| `perception/components.rs` | Verify L1Perceptions API |
| `simulation/drives/mod.rs` | NEW - DriveSource, DriveContribution types |
| `simulation/drives/types.rs` | NEW - DriveOutput, DriveContributions, DriveSimplex, FreezeState |
| `simulation/drives/vision.rs` | NEW - VisionDriveSystem (with comfort zone) |
| `simulation/drives/combine.rs` | NEW - DriveCombineSystem (with priority weights) |
| `creatures/components/state.rs` | REMOVE: BehaviorMode enum, behavior field |
| `creatures/steering/system.rs` | Replace BehaviorMode match with drive+freeze steering |
| `creatures/steering/wander.rs` | DELETE: Replaced by disperse drive |
| `creatures/builder.rs` | Add DriveOutput, DriveContributions, FreezeState; remove `in_behavior()` |
| `behaviors/transitions/systems.rs` | Remove behavior logic (keep age/energy) |
| `core/simulation.rs` | Register new systems, update ordering |
| `instrumentation/mod.rs` | Add `vision_drive_us`, `drive_combine_us` timing fields |

---

## Emergent Behaviors (No Explicit Code)

| Situation | Behavior | Why |
|-----------|----------|-----|
| Empty area, no neighbors | Rests | No contributions → combined ≈ zero |
| Crowded area | Disperses to emptier cells | CROWDED → disperse contributions |
| Comfortable density (0.3-1.5x) | Natural grouping | Comfort zone skips disperse |
| Large crit nearby | Flees | THREAT → flee contributions |
| Large crit charging | Explosive flee | velocity urgency = 1.0 |
| Large crit resting | Cautious grazing | velocity urgency = 0.2 |
| Prey-rich area | Drifts toward | PREY → approach contributions |
| Path blocked | Weaves around | L0 avoidance still active |
| Surrounded by threats | Freezes | Flee vectors cancel → zero output |
| Frozen too long (~4.5s) | Desperate escape burst | FreezeState timeout → random bolt |

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

---

## Migration Strategy: CLEAN SWAP

**Decision:** Remove `BehaviorMode` completely when drives are working (no coexistence period).

**Rationale:** Cleaner codebase, no dual-path complexity.

**Implementation:** See Step 4 for full steering integration with:
- Priority 1: Explicit target (tests/trials)
- Priority 2: Desperate escape (freeze timeout)
- Priority 3: Drive-based steering
- Priority 4: Frozen state (tracks duration)

**Files to Delete:**
- `creatures/steering/wander.rs` - Disperse drive replaces wandering

---

## Existing Spec Migration

**Key Insight:** 39 seeker specs work unchanged (Target override preserved).

| Spec Type | Count | Status |
|-----------|-------|--------|
| Seeker-based | 39 | **Works unchanged** - Target override |
| Catatonic-based | 5 | **Works unchanged** - Dormant brain |
| Wanderer-based | 7 | **Fix required** - Convert to drive-based |

**Wanderer specs to fix:**
- `specs/behavior/size-variation-stability.toml` (1)
- `specs/performance/100k_medium_sparse.toml`
- `specs/performance/10k-wanderers-world-spread.toml`
- `specs/performance/200k-wanderers-world-spread.toml`
- `specs/performance/many-wanderers-dense.toml`
- `specs/performance/many-wanderers-medium-density.toml`

**Fix approach:** Replace `creature_type = "wanderer"` with drive-based creature (no Target → uses disperse drive).

---

## BDD Specifications (TOML Specs)

Write these FIRST (Red phase). Location: `apps/simulation/specs/behavior/drives/`

### 1. `drive-flee-from-threat.toml`
Small creature (1m) near large creature (5m) should flee without explicit BehaviorMode.
- **Assertion:** `distance_increased` from predator by min 20m
- **Watch:** Small creature accelerates away from large creature

### 2. `drive-flee-urgency-charging.toml`
Small creature should flee MORE urgently when predator charges vs stationary.
- **Assertion:** `flee_distance_greater` ratio > 1.5
- **Watch:** Charging predator causes 2x+ flee distance

### 3. `drive-disperse-from-crowded.toml`
Creature in crowded area should drift toward empty cells.
- **Assertion:** `distance_from_center_increased` by min 10m
- **Watch:** Creature drifts away from dense cluster

### 4. `drive-rest-when-isolated.toml`
Isolated creature should remain stationary (no jitter).
- **Assertion:** `position_stable` with max_drift 2m
- **Critical:** Validates equilibrium state

### 5. `drive-freeze-when-surrounded.toml`
Creature surrounded by threats should freeze (tonic immobility).
- **Assertion:** `position_stable` with max_drift 3m
- **Golden Zone:** Flee vectors cancel → zero output

### 5b. `drive-desperate-escape.toml`
Creature frozen too long (~4.5s) should trigger desperate escape burst.
- **Assertion:** `position_changed` after 110 ticks, min_distance 10m
- **Watch:** After prolonged freeze, creature suddenly bolts in random direction
- **Biological:** Prevents permanent freeze = certain death scenario

### 5c. `drive-priority-flee-over-food.toml` (CRITICAL)
Creature between 1 predator and 10 prey should flee, not approach.
- **Assertion:** `distance_increased` from predator
- **Failure Mode:** Without clamping, 10 prey vectors sum to override 1 predator
- **Watch:** Creature flees predator despite abundant food
- **Validates:** Magnitude clamping before priority weights

### 6. `drive-approach-prey.toml`
Large creature near small creatures should drift toward them.
- **Assertion:** `distance_decreased` toward cluster by min 15m
- **Watch:** Predator moves toward cluster but weaves around individuals

### 7. `drive-target-override.toml`
Explicit Target should override drive steering.
- **Assertion:** `creature_reached_target`
- **Regression:** Validates `.as_seeker()` still works

### 8. `drive-no-behavior-mode.toml`
After clean swap, no creature should have BehaviorMode component.
- **Assertion:** `no_behavior_mode_component`
- **Meta-test:** Validates architectural cleanup

---

## TDD Checklist

**Red Phase (Write Failing Tests First):**
- [ ] Create `specs/behavior/drives/` folder
- [ ] Write 10 new drive BDD specs (8 core + desperate-escape + priority-flee-over-food)
- [ ] All new specs FAIL initially (drives not implemented)
- [ ] Existing 44 seeker/catatonic specs still PASS (baseline)

**Green Phase (Make Tests Pass):**
- [ ] Step 0: L1Perceptions populated → new specs still fail
- [ ] Step 1: Drive components added → new specs still fail
- [ ] Step 2: VisionDriveSystem → some drive specs start passing
- [ ] Step 3: DriveCombineSystem (with magnitude clamping) → drive specs pass
- [ ] Step 4: Steering integration + freeze timeout → flee/disperse/rest/escape specs pass
- [ ] Step 5: System ordering → all new drive specs pass
- [ ] Step 6: BehaviorMode removal → `no-behavior-mode` spec passes
- [ ] Fix 7 wanderer specs → convert to drive-based creatures

**Refactor Phase:**
- [ ] Performance: < 2ms drive computation at 360K
- [ ] Code cleanup: Remove dead code from old behavior system
- [ ] All 54+ specs pass (44 existing + 10 new)
