# Per-System Update Frequency Control

## Overview

Add runtime-adjustable update frequency for cognitive ECS systems via dev-ui controls. Each adjustable system gets its own configurable frequency, displayed in Hz. Constants file sets startup defaults; runtime adjustment via sparkline popover in dev-ui.

**Important:** Only cognitive/decision-making systems can be frequency-controlled. Physics systems (movement, spatial grid) must run every tick.

## Review Findings (Specialist Agents)

**Reviewed by:** rusty-ron (Rust), frontend-fanny (Dev-UI), ecs-emma (ECS Architecture)

| System | Verdict | Rationale |
|--------|---------|-----------|
| Perception | ✅ Adjustable | Already sliced, validated pattern |
| Behavior Transition | ✅ Adjustable | Already sliced, validated pattern |
| Steering | ✅ Adjustable | Decision-making, not physics. Stale neighbor data acceptable. |
| Movement | ❌ NEVER | Physics integration requires every-tick for stability |
| Spatial Grid | ❌ NEVER | Perception accuracy depends on current positions |

**Key Validations:**
- Double modulus pattern: CORRECT
- Rayon par_iter_mut with filter: WORKS (filter is cheap O(N), parallel work is expensive)
- Runtime filter vs archetype: CORRECT (archetype migration would be expensive)
- Steering + stale acceleration: SAFE (acceleration resets to zero after integration)

---

## Core Concept: Double Modulus Pattern

```rust
creature.update_bucket % active_buckets == tick % active_buckets
```

- **MAX_BUCKETS = 100** (fixed, for bucket assignment at spawn)
- **Per-system active_buckets** (1..=100, runtime adjustable)
- All creatures still get processed; just grouped differently based on active count
- **active_buckets = 1** means every creature every tick (100% frequency)
- **active_buckets = 10** means 10% of creatures per tick (1/10th frequency)

## Terminology

| Old Term | New Term | Meaning |
|----------|----------|---------|
| slice_id | update_bucket | Fixed bucket assignment (0..99) |
| UPDATE_SLICE_COUNT | active_buckets | How many buckets are active (1..100) |
| Slice | Update frequency | Hz-based display (sim_hz / active_buckets) |

## Systems Classification

### ✅ Adjustable (Cognitive/Decision-Making)

| System | File | Default | Max Safe Reduction |
|--------|------|---------|-------------------|
| Perception | `perception/systems.rs` | 10 Hz (buckets=2) | 2 Hz (buckets=10) |
| Behavior Transition | `behaviors/transitions/systems.rs` | 10 Hz (buckets=2) | 2 Hz (buckets=10) |
| Steering | `steering/system.rs` | 20 Hz (buckets=1) | 5 Hz (buckets=4) |

### ❌ Fixed (Physics/Accuracy-Critical)

| System | File | Reason |
|--------|------|--------|
| Movement | `movement/systems.rs` | Physics stability (Euler integration) |
| Spatial Grid | `spatial/systems.rs` | Perception accuracy, collision detection |

---

## Implementation Steps

### Phase 1: Rust - Constants & Resources

#### 1.1 System Frequency Defaults
**File:** `apps/simulation/src/simulation/creatures/constants/performance.rs`

```rust
pub const MAX_BUCKETS: u8 = 100;

/// Per-system default bucket counts (higher = lower frequency)
/// Formula: Hz = sim_tick_rate / bucket_count
pub mod defaults {
    pub const PERCEPTION: u8 = 2;           // 10Hz @ 20Hz sim
    pub const BEHAVIOR_TRANSITION: u8 = 2;  // 10Hz
    pub const STEERING: u8 = 1;             // 20Hz (every tick)
    // Movement and spatial_grid are NOT configurable (always every tick)
}
```

#### 1.2 Runtime Resource
**File:** `apps/simulation/src/simulation/core/resources.rs`

```rust
#[derive(Resource, Clone, Serialize)]
pub struct SystemFrequencyConfig {
    pub perception: u8,
    pub behavior_transition: u8,
    pub steering: u8,
    // Movement and spatial_grid intentionally excluded - always run every tick
}

impl Default for SystemFrequencyConfig {
    fn default() -> Self {
        use crate::simulation::creatures::constants::defaults;
        Self {
            perception: defaults::PERCEPTION,
            behavior_transition: defaults::BEHAVIOR_TRANSITION,
            steering: defaults::STEERING,
        }
    }
}
```

#### 1.3 Rename Component
**File:** `apps/simulation/src/simulation/creatures/components/update_slice.rs`

Rename `UpdateSlice` → `UpdateBucket`:
```rust
#[derive(Component, Clone, Copy, Debug, Serialize, Deserialize, Reflect)]
pub struct UpdateBucket {
    pub id: u8,  // 0..MAX_BUCKETS
}
```

#### 1.4 Update CritBuilder
**File:** `apps/simulation/src/simulation/creatures/builder.rs`

```rust
update_bucket: UpdateBucket::new((id % MAX_BUCKETS as u32) as u8),
```

### Phase 2: Rust - Update Adjustable Systems

#### 2.1 Common Pattern
```rust
pub fn some_system(
    physics_tick: Res<PhysicsTick>,
    freq_config: Res<SystemFrequencyConfig>,
    mut query: Query<(/* ... */, &UpdateBucket)>,
) {
    let active = freq_config.this_system as u64;
    let current_bucket = (physics_tick.get() % active) as u8;

    let mut entities: Vec<_> = query
        .iter_mut()
        .filter(|(.., bucket)| {
            (bucket.id as u64 % active) as u8 == current_bucket
        })
        .collect();

    entities.par_iter_mut().for_each(|(/* ... */)| {
        // System logic
    });
}
```

#### 2.2 Systems to Update
- `perception/systems.rs:update_perception_system` → use `freq_config.perception`
- `behaviors/transitions/systems.rs:behavior_transition_system` → use `freq_config.behavior_transition`
- `steering/system.rs:update_steering_system` → use `freq_config.steering` (NEW)

**Note:** Movement and spatial_grid systems remain unchanged (no frequency control).

### Phase 3: Rust - IPC Commands

#### 3.1 Command Variant
**File:** `apps/simulation/src/ipc/commands.rs`

```rust
SetSystemFrequency { system: String, bucket_count: u8 },
```

#### 3.2 Command Handler
**File:** `apps/simulation/src/ipc/command_executor.rs`

```rust
SimCommand::SetSystemFrequency { system, bucket_count } => {
    let mut config = world.resource_mut::<SystemFrequencyConfig>();
    let count = bucket_count.clamp(1, 100);
    match system.as_str() {
        "perception" => config.perception = count,
        "behavior_transition" => config.behavior_transition = count,
        "steering" => config.steering = count,
        _ => eprintln!("Unknown or non-adjustable system: {}", system),
    }
}
```

#### 3.3 NAPI Method
**File:** `apps/simulation/src/napi_addon/simulation_engine.rs`

```rust
#[napi]
pub fn set_system_frequency(&self, system: String, bucket_count: u8) -> Result<()>
```

### Phase 4: Electron Bridge

#### 4.1 Preload
**File:** `apps/portal/electron/preload.cjs`

```javascript
setSystemFrequency: (system, bucketCount) => {
  ipcRenderer.send('set-system-frequency', { system, bucketCount });
},
```

#### 4.2 Main Process
**File:** `apps/portal/electron/napi-main.cjs`

```javascript
ipcMain.on('set-system-frequency', (event, { system, bucketCount }) => {
  simulationEngine.setSystemFrequency(system, bucketCount);
});
```

### Phase 5: Dev-UI Controls

#### 5.1 System Configuration
**File:** `apps/dev-ui/src/constants/systemFrequency.ts`

```typescript
export const ADJUSTABLE_SYSTEMS = ['perception', 'behavior_transition', 'steering'];
export const FIXED_SYSTEMS = ['movement', 'spatial_grid'];  // Never adjustable

export const SYSTEM_LIMITS: Record<string, { maxBuckets: number; warningBuckets: number }> = {
  'perception': { maxBuckets: 10, warningBuckets: 5 },      // Min 2Hz, warn at 4Hz
  'behavior_transition': { maxBuckets: 10, warningBuckets: 5 },
  'steering': { maxBuckets: 4, warningBuckets: 3 },         // Min 5Hz, warn at 6.7Hz
};
```

#### 5.2 Frequency Control Popover
**File:** `apps/dev-ui/src/components/SystemFrequencyControl.tsx`

- Shows system name and current Hz (calculated from bucket count)
- Slider for bucket count (1 to system's maxBuckets)
- Warning banner when below warningBuckets threshold
- "Reset" button restores default
- Popover opens on sparkline row click
- Fixed systems show "Always runs every tick" (no slider)

#### 5.3 SystemTimingsPanel Integration
- Make sparkline rows clickable
- Click → open popover for that system
- Adjustable systems: show slider
- Fixed systems: show info message explaining why not adjustable

### Phase 6: Telemetry Extension

Include current frequency config in telemetry:
```rust
pub system_frequencies: SystemFrequencyConfig,
```

Dev-UI uses this to show current values when opening popover.

### Phase 7: Tests

#### Rust Tests
- Double modulus logic: all creatures processed over N ticks
- Boundary: bucket_count = 1 (every tick), bucket_count = MAX_BUCKETS
- IPC command validation (rejects invalid system names)
- Steering slicing: verify stale acceleration behavior is correct

#### Dev-UI Tests
- Hz calculation from bucket count
- Slider respects system-specific limits
- Fixed systems show info, not slider
- Popover behavior

---

## File Summary

| File | Change |
|------|--------|
| `constants/performance.rs` | MAX_BUCKETS + per-system defaults |
| `core/resources.rs` | SystemFrequencyConfig resource (3 adjustable systems) |
| `components/update_slice.rs` | Rename to UpdateBucket |
| `creatures/builder.rs` | Use MAX_BUCKETS |
| `perception/systems.rs` | Use freq_config.perception |
| `behaviors/transitions/systems.rs` | Use freq_config.behavior_transition |
| `steering/system.rs` | Add frequency control (NEW) |
| `ipc/commands.rs` | SetSystemFrequency variant |
| `napi_addon/simulation_engine.rs` | set_system_frequency method |
| `portal/electron/preload.cjs` | setSystemFrequency bridge |
| `portal/electron/napi-main.cjs` | IPC handler |
| `dev-ui/constants/systemFrequency.ts` | System limits config (NEW) |
| `dev-ui/components/SystemFrequencyControl.tsx` | Popover component (NEW) |
| `dev-ui/components/SystemTimingsPanel.tsx` | Clickable rows |

---

## Design Decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| MAX_BUCKETS | 100 | Future-proofs for very slow systems |
| Adjustable systems | 3 only | Physics must run every tick |
| Default steering | 1 (every tick) | Smooth by default, user can reduce |
| Default perception/behavior | 2 | Balanced perf/responsiveness |
| UI display | Hz | More intuitive than bucket counts |
| Rename slice → bucket | Yes | "Bucket" is clearer grouping metaphor |
| Per-system limits | Yes | Prevent unsafe frequency reduction |

---

## Future Use Cases

See `docs/performance/ideas/dynamic-system-frequency.md` for:
1. Automatic performance scaling (FPS-based)
2. Fast-forward mode
3. Quality presets
4. Distance-based LOD
