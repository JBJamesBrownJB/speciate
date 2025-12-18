# Size-Dependent Speed and Acceleration

## Status: TODO

Consulted with zoologist-tom. Ready for implementation.

---

## Current State

- `MAX_SPEED = 20 m/s` - Global constant for ALL creatures
- `MAX_ACCELERATION = 10 m/s²` - Global constant for ALL creatures
- Turn rate is already size-dependent: `turn_rate ∝ 1/size^1.33`

---

## Biological Rationale

### Speed Scaling

Locomotion speed follows power-law scaling:
- **Stride length** scales with leg length (∝ size)
- **Stride frequency** scales inversely with size
- Net effect: shallow positive exponent

Empirical relationship: `max_speed ∝ size^0.25` (quarter-power scaling)

### Acceleration Scaling

Acceleration constrained by force/mass ratio:
- Muscle force ∝ cross-sectional area ∝ size²
- Mass ∝ volume ∝ size³
- Therefore: acceleration ∝ size² / size³ = size^-1

Softened for gameplay: `max_acceleration ∝ size^-0.67`

---

## Formulas

### Maximum Speed
```
max_speed = BASE_SPEED × size^0.25
          = 8.0 × size^0.25

Clamped to [3.0, 15.0] m/s
```

### Maximum Acceleration
```
max_acceleration = BASE_ACCEL × size^-0.67
                 = 12.0 × size^-0.67

Clamped to [2.0, 25.0] m/s²
```

---

## Reference Values

| Size (m) | Max Speed (m/s) | Max Acceleration (m/s²) | Niche |
|----------|-----------------|-------------------------|-------|
| 0.5      | 6.7             | 19.0                    | Explosive, evasion specialist |
| 1.0      | 8.0             | 12.0                    | Reference (balanced) |
| 2.0      | 9.5             | 7.6                     | Faster cruise, slower accel |
| 5.0      | 12.0            | 4.1                     | High top speed, ponderous |

---

## Constants (Rust)

```rust
// Speed scaling
pub const BASE_SPEED: f32 = 8.0;           // m/s at 1.0m size
pub const SPEED_EXPONENT: f32 = 0.25;
pub const MIN_SPEED: f32 = 3.0;
pub const MAX_SPEED_CAP: f32 = 15.0;

// Acceleration scaling
pub const BASE_ACCEL: f32 = 12.0;          // m/s² at 1.0m size
pub const ACCEL_EXPONENT: f32 = -0.67;
pub const MIN_ACCEL: f32 = 2.0;
pub const MAX_ACCEL_CAP: f32 = 25.0;
```

---

## DNA Genes (Future)

Add multipliers for individual variation (NOT free advantages):

```rust
pub struct Dna {
    // ... existing fields ...

    /// Multiplier for base speed (0.7 - 1.3)
    pub speed_factor: f32,

    /// Multiplier for base acceleration (0.7 - 1.3)
    pub acceleration_factor: f32,
}
```

### Modified Formulas with DNA
```
max_speed = BASE_SPEED × size^0.25 × dna.speed_factor
max_acceleration = BASE_ACCEL × size^-0.67 × dna.acceleration_factor
```

### Trade-offs (Required)

| High Gene Value | Cost |
|-----------------|------|
| speed_factor > 1.0 | +15% base metabolism |
| acceleration_factor > 1.0 | -20% stamina endurance |

---

## Ecological Outcome

Combined with turn rate (`1/size^1.33`):

| Size | Turn Rate | Acceleration | Top Speed | Strategy |
|------|-----------|--------------|-----------|----------|
| 0.5m | Very High | Explosive    | Moderate  | Evasion, tight spaces |
| 1.0m | Medium    | Balanced     | Balanced  | Generalist |
| 5.0m | Low       | Slow         | High      | Open terrain pursuit |

**No god-tier creatures.** Every size has trade-offs:
- Small escapes through maneuverability
- Large catches in open terrain through sustained speed
- Medium is viable everywhere but dominant nowhere

---

## Implementation Notes

### Files to Modify
- `src/simulation/creatures/constants/physics.rs` - Add constants
- `src/simulation/movement/systems.rs` - Use size-dependent values
- `src/simulation/creatures/dna.rs` - Add speed/accel genes (future)

### System Dependencies
- Movement system needs creature size for speed clamping
- Steering systems need size for force magnitude limits
