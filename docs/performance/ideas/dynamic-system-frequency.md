# Dynamic System Frequency Adjustment

## Concept

Automatically adjust per-system update frequencies at runtime based on performance metrics and gameplay state. The game dynamically trades off simulation fidelity for frame rate stability.

## Prerequisites

Requires: `docs/performance/done/system-update-frequency.md` (per-system configurable frequencies) - ✅ Implemented in Phase C

## Important Constraints

**Only cognitive systems can be frequency-adjusted:**
- ✅ Perception (decision-making, can lag)
- ✅ Behavior Transition (state changes, can lag)
- ✅ Steering (force calculation, can use stale data)

**Physics systems must run every tick:**
- ❌ Movement (Euler integration stability)
- ❌ Spatial Grid (perception accuracy)

## Use Cases

### 1. Automatic Performance Scaling

When FPS drops below target, automatically reduce frequencies of non-critical systems:

```
if fps < 55:
    perception.frequency -= 1Hz
    behavior.frequency -= 1Hz
if fps > 58:
    perception.frequency += 0.5Hz  # Slower recovery
```

**Priority order for reduction:**
1. Perception (least visible impact)
2. Behavior transitions
3. Steering (noticeable but acceptable)
4. Movement (last resort - causes choppy visuals)
5. Spatial grid (never reduce below movement, causes perception bugs)

### 2. Fast-Forward Mode

For time-lapse gameplay or "skip to interesting moment":

```
fast_forward_mode:
    perception.frequency = 2Hz   # Minimal AI updates
    behavior.frequency = 2Hz
    steering.frequency = 5Hz
    movement.frequency = 10Hz    # Still smooth-ish
    simulation_speed = 4x        # Combined with time scale
```

Player can skip hours of ecological time in minutes of real time.

### 3. Quality Presets

User-selectable performance profiles:

| Preset | Perception | Behavior | Steering | Movement |
|--------|------------|----------|----------|----------|
| Ultra  | 20Hz       | 20Hz     | 20Hz     | 20Hz     |
| High   | 10Hz       | 10Hz     | 20Hz     | 20Hz     |
| Medium | 5Hz        | 5Hz      | 15Hz     | 20Hz     |
| Low    | 2Hz        | 2Hz      | 10Hz     | 15Hz     |
| Potato | 1Hz        | 1Hz      | 5Hz      | 10Hz     |

### 4. Distance-Based LOD

Creatures far from camera get lower frequency updates:

```
distance_to_camera = creature.position - camera.center
if distance > 500:
    creature.frequency_multiplier = 0.25  # 1/4 updates
elif distance > 200:
    creature.frequency_multiplier = 0.5   # 1/2 updates
else:
    creature.frequency_multiplier = 1.0   # Full updates
```

Combines with bucket assignment for hierarchical LOD.

### 5. Population-Based Scaling

At high creature counts, automatically reduce frequency:

```
if creature_count > 50000:
    global_frequency_scale = 0.5
elif creature_count > 20000:
    global_frequency_scale = 0.75
else:
    global_frequency_scale = 1.0
```

## Implementation Architecture

### Frequency Controller Resource

```rust
#[derive(Resource)]
pub struct FrequencyController {
    // Targets from user/preset
    pub target_fps: f32,
    pub quality_preset: QualityPreset,

    // Current adjustments
    pub current_frequencies: SystemFrequencyConfig,

    // Metrics for feedback loop
    pub fps_history: RingBuffer<f32, 60>,
    pub adjustment_cooldown: f32,
}
```

### Control Loop System

```rust
pub fn frequency_control_system(
    mut controller: ResMut<FrequencyController>,
    mut freq_config: ResMut<SystemFrequencyConfig>,
    diagnostics: Res<FrameTimeDiagnostics>,
) {
    let current_fps = diagnostics.fps();
    controller.fps_history.push(current_fps);

    // Only adjust every 0.5 seconds
    if controller.adjustment_cooldown > 0.0 {
        controller.adjustment_cooldown -= delta_time;
        return;
    }

    let avg_fps = controller.fps_history.average();

    if avg_fps < controller.target_fps - 5.0 {
        // Reduce quality
        reduce_frequency(&mut freq_config, FrequencyPriority::NonCritical);
        controller.adjustment_cooldown = 0.5;
    } else if avg_fps > controller.target_fps - 2.0 {
        // Can afford to increase quality
        increase_frequency(&mut freq_config, &controller.quality_preset);
        controller.adjustment_cooldown = 1.0;  // Slower recovery
    }
}
```

## Visual Feedback

When frequencies are reduced, dev-ui could show:
- Warning indicators on affected system sparklines
- "Performance mode active" banner
- Estimated fidelity percentage

## Considerations

### Hysteresis

Avoid oscillation with:
- Cooldown between adjustments (0.5-1s)
- Different thresholds for up/down (e.g., reduce at 55fps, increase at 58fps)
- Smooth transitions (lerp frequencies over 0.5s)

### Critical Systems

Some systems have hard floors:
- Movement: Never below 10Hz (unplayable choppiness)
- Spatial grid: Must match or exceed movement (perception correctness)

### User Override

Always allow manual control in dev-ui to override automatic adjustments.

## Future Extensions

### Per-Creature Priority

VIP creatures (player-controlled, plot-relevant) get full frequency:
```rust
#[derive(Component)]
pub struct FrequencyPriority(pub f32);  // 1.0 = full, 0.5 = half, etc.
```

### Predictive Scaling

Use derivative of creature count to anticipate population explosions:
```
if d_creature_count/dt > 1000/second:
    preemptively_reduce_frequency()
```

### Telemetry-Driven Tuning

Collect anonymized performance data to optimize default presets for common hardware configurations.
