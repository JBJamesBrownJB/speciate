# Sprint: System Update Frequency (Phase C)

## Outcome

Runtime-adjustable Hz for cognitive systems using entity-ID bucketing.

**Depends on:** Phase A (Dual Grid) - for early-exit optimization only

**Reference:** `docs/performance/todo/system-update-frequency.md`

---

## Core Concept

Frequency control uses **entity ID bucketing** (not spatial position):

```rust
let current_bucket = (tick.get() as usize) % divisor;

if (entity.index() as usize) % divisor != current_bucket {
    // Skip this creature this tick
    return;
}
```

**Why entity-ID instead of L1 cell position:**
- **No visual artifacts**: Creatures in same cell don't update simultaneously (avoids "ripple" effects)
- **Decoupled from grid**: Frequency control independent of spatial architecture
- **Uniform distribution**: Entity IDs naturally spread across buckets

**Zero overhead at full rate:** When `divisor = 1`, all creatures process every tick (no modulo check needed with early return).

---

## Systems to Control

| System | Can Throttle? | Rationale |
|--------|---------------|-----------|
| Perception | Yes | Stale data acceptable (reaction time) |
| Behavior/Simplex | Yes | Decision-making, not physics |
| Steering | Yes | Stale neighbors OK |
| Movement | **NO** | Physics integration requires every-tick |
| L0/L1 Grid Rebuild | **NO** | Perception accuracy depends on current positions |

---

## Implementation Steps

### Step 1: FreqConfig Resource
**File:** `core/resources.rs`

```rust
#[derive(Resource, Clone, Debug)]
pub struct FreqConfig {
    pub perception_divisor: u8,   // 1 = every tick, 10 = every 10th
    pub behavior_divisor: u8,
    pub steering_divisor: u8,
}

impl Default for FreqConfig {
    fn default() -> Self {
        Self {
            perception_divisor: 1,  // Full rate by default
            behavior_divisor: 1,
            steering_divisor: 1,
        }
    }
}
```

### Step 2: Add Bucketing to Perception
**File:** `perception/systems.rs`

```rust
pub fn update_perception_system(
    tick: Res<PhysicsTick>,
    freq: Res<FreqConfig>,
    grid: Res<HierarchicalGrid>,
    mut query: Query<(Entity, &Position, ...)>,
) {
    let divisor = freq.perception_divisor as usize;

    // HOT PATH: Full rate (zero overhead)
    if divisor == 1 {
        // Existing code unchanged
        let mut entities: Vec<_> = query.iter_mut().collect();
        entities.par_iter_mut().for_each(|...| { /* work */ });
        return;
    }

    // THROTTLED PATH: Entity-ID bucketing
    let current_bucket = (tick.get() as usize) % divisor;

    let mut entities: Vec<_> = query.iter_mut().collect();
    entities.par_iter_mut().for_each(|(entity, pos, ...)| {
        if (entity.index() as usize) % divisor != current_bucket {
            return;  // Skip this tick
        }
        // Normal perception logic
    });
}
```

### Step 3: Add Bucketing to Behavior/Steering
Same pattern for other cognitive systems.

### Step 4: IPC Command
Add `SetSystemFrequency { system, divisor }` command.

### Step 5: Dev-UI Inline Sliders
**Location:** Below each system's sparkline in SystemTimingsPanel

**Design:**
- Thin slider bar directly under the sparkline
- Only for controllable systems (perception, behavior, steering)
- No popup/modal
- Immediate feedback via sparkline

**Layout:**
```
┌─────────────────────────────┐
│ perception_system    1.2ms  │
│ ▁▂▃▂▁▂▃▄▃▂▁▂▃▂▁  (sparkline)│
│ ────●─────────────  (slider)│  ← divisor control
├─────────────────────────────┤
│ movement_system      0.8ms  │
│ ▁▁▂▁▁▂▁▁▂▁▁  (sparkline)    │
│ (no slider - can't throttle)│
└─────────────────────────────┘
```

---

## Performance Analysis

| Scenario | Work |
|----------|------|
| Full rate (divisor=1) | Zero overhead, existing code path |
| Throttled (divisor=10) | Only 1/10 of creatures process per tick |

**Why entity-ID bucketing works:**
- Entity IDs are already available (no computation needed)
- Uniform distribution across buckets (no spatial clustering)
- No visual artifacts (creatures update independently of neighbors)
- Single integer modulo per creature (trivial cost)

---

## Validation

- [ ] Full rate (divisor=1) matches baseline performance
- [ ] Throttled mode reduces per-tick CPU usage proportionally
- [ ] Dev-UI can adjust divisors at runtime
- [ ] Simulation remains deterministic at all divisor values
- [ ] No visual artifacts from throttling (smooth creature movement)

---

## Files to Modify

| File | Change |
|------|--------|
| `core/resources.rs` | Add FreqConfig resource |
| `core/simulation.rs` | Insert FreqConfig resource |
| `perception/systems.rs` | Add throttled path with entity-ID bucketing |
| `creatures/behaviors/transitions/systems.rs` | Add throttled path |
| `creatures/steering/system.rs` | Add throttled path |
| `ipc/sim_command.rs` | Add SetSystemFrequency command |
| Dev-UI (optional) | Slider controls |
