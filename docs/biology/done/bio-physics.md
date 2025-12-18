The source of truth for the physical forces a creature can apply to itself.

## Self-Inflicted Forces

**Single Source of Truth:** `apps/simulation/src/simulation/creatures/steering/`

All acceleration added to a creature from its own muscular effort originates in the fused steering system (`steering/system.rs`). Specifically:

| Lines | Behavior | Description |
|-------|----------|-------------|
| 190-191 | Wander | Random exploration force |
| 198-199 | Seek | Target-directed force |
| 213-214 | Avoidance | Obstacle repulsion force |

The movement system (`movement/systems.rs`) only RESETS acceleration to zero after integration - it never adds forces.

**Implication:** Any test validating that creatures cannot exceed physical limits need only test the steering system's output.

---

## Physics Constants Audit

Source files:
- `creatures/constants/physics.rs` - Core physics values
- `creatures/constants/behavior.rs` - Behavior-specific parameters

### USED Constants

#### Core Physics (in `physics.rs`)

| Constant | Value | Used By |
|----------|-------|---------|
| `MAX_SPEED` | 20.0 m/s | `movement/systems.rs`, `steering/system.rs`, `builder.rs` |
| `MAX_ACCELERATION` | 10.0 m/s² | `core/components.rs:86` (max_force calculation) |
| `MAX_TURN_RATE_RAD` | 3.14 rad/s | `movement/systems.rs:53` |
| `DRAG_COEFFICIENT` | 2.0 | `movement/systems.rs:37` |
| `STOPPED_THRESHOLD` | 0.05 m/s | `movement/systems.rs:54` |
| `NOISE_SPEED_THRESHOLD_SQ` | 0.01 | `movement/systems.rs:114` |
| `DEFAULT_MASS` | 35.0 kg | `core/components.rs:82` (mass = DEFAULT_MASS × length³) |

#### Force Budget (in `behavior.rs`)

| Constant | Value | Used By |
|----------|-------|---------|
| `WANDER_FORCE_MULT` | 0.25 (25%) | `steering/system.rs:48` |
| `SEEK_FORCE_MULT` | 0.7 (70%) | `steering/system.rs:78` |

#### Avoidance (in `behavior.rs`)

| Constant | Value | Used By |
|----------|-------|---------|
| `PERSONAL_SPACE_MULTIPLIER` | 2.0 | `perception/components.rs:167` |
| `SEEKING_SPACE_REDUCTION` | 0.5 | `steering/system.rs:208` |
| `EMERGENCY_BRAKE_DISTANCE` | 0.5 m | `steering/system.rs:101` |
| `ENERGY_MODIFIER` | 0.1-1.0 | `perception/components.rs:152-153` |

#### Wander (in `behavior.rs`)

| Constant | Value | Used By |
|----------|-------|---------|
| `WANDER_RADIUS` | 10.0 m | `builder.rs:215` |
| `WANDER_DISTANCE` | 20.0 m | `builder.rs:216` |
| `ANGLE_CHANGE` | 4.5 deg | `builder.rs:217` |

---

### Removed Constants (Cleanup Completed)

The following constants were removed from the codebase as they were defined but never used:

**From `behavior.rs` (12 constants removed):**
- Force budget: `EMERGENCY_FORCE_MULT`, `PURSUIT_FORCE_MULT`, `CRUISE_FORCE_MULT`, `BRAKE_FORCE_MULT`
- TTC system: `TTC_SLOW_THRESHOLD`, `TTC_STOP_THRESHOLD`, `TTC_RANGE`, `TTC_RANGE_INV`, `MIN_SLOW_ZONE_BODY_LENGTHS`
- Seek behavior: `POUNCE_THRESHOLD`, `POUNCE_SPEED`, `ARRIVAL_THRESHOLD`

**From `physics.rs` (1 constant removed):**
- `DT` (tick_controller uses its own `FIXED_DT`)

**Note:** `SEEK_FORCE_MULT` was changed from aliasing `PURSUIT_FORCE_MULT` to an inline value `UnitInterval::new(0.7)`.

