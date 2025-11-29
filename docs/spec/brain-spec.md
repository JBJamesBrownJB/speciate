# Brain System Specification

**Status:** Implemented
**Location:** `apps/simulation/src/simulation/creatures/components/brain.rs`

## Overview

The Brain component controls creature decision-making. It determines when and how creatures evaluate their situation and potentially change behavior.

## Components

### BrainMode

```rust
pub enum BrainMode {
    Normal,   // Standard decision logic based on perception + internal state
    Cycling,  // Forces behavior cycling for testing (Catatonic → Wandering → Seeking)
    Dormant,  // No decisions - static behavior
}
```

### Brain

```rust
pub struct Brain {
    pub mode: BrainMode,
    pub last_decision_time: f64,
}
```

## Decision Timing

Brain uses a **dynamic cooldown** that scales with creature state:

### Scaling Factors

| Factor | Formula | Effect |
|--------|---------|--------|
| Age | `1.0 + (age/100)^2.5 * 2.0` | Older creatures think slower |
| Energy | `1.0 + (1.0 - energy/100)^2.0 * 1.5` | Low energy slows thinking |

### Constants

| Constant | Value | Description |
|----------|-------|-------------|
| `BASE_COOLDOWN_MS` | 150.0 | Base decision interval |
| `AGE_SENSITIVITY` | 2.0 | How much age affects cooldown |
| `MAX_AGE` | 100.0 | Age normalization factor |
| `MAX_ENERGY` | 100.0 | Energy normalization factor |

### Example Cooldowns

| State | Cooldown |
|-------|----------|
| Young (0), full energy (100) | 150ms |
| Old (80), full energy | ~285ms |
| Young, half energy (50) | ~225ms |
| Old (80), half energy | ~427ms |

## Panic Override

Immediate threats bypass the decision cooldown entirely.

```rust
pub fn should_panic(nearest_threat_dist: f32, body_size: f32, energy: f32) -> bool
```

| Constant | Value | Description |
|----------|-------|-------------|
| `PANIC_THRESHOLD` | 2.0 | Body size multiplier for panic distance |

**Panic disabled when:**
- Energy < 5.0 ("giving up" behavior - too weak to react)

## Decision Inputs

Brain considers (when decision logic is implemented):
- **Perception** - nearby entities (current, possibly stale)
- **Energy** - internal energy level
- **Age** - creature age
- **Behavior** - current behavior mode

## Integration

### System Order

1. `update_perception_system` - Updates what creatures can see
2. `behavior_transition_system` - Brain evaluates and decides

### Key Design Decisions

1. **Brain is independent of perception timing** - Brain runs on its own schedule, not triggered by perception updates
2. **Dynamic cooldown** - Older/tired creatures think slower (biologically realistic)
3. **No archetype churn** - All state is enum/field mutations, no component add/remove

## Future: DNA Integration

Parameters to be DNA-encoded in future sprint:
- `base_cooldown_ms` (50-500ms range)
- `age_sensitivity` (0.5-3.0 range)
- `panic_threshold` (1.0-4.0 range)
