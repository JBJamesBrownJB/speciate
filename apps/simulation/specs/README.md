# Spec-Driven Testing Framework

Declarative specification testing with dual-mode execution:
- **Headless** - `cargo test` for CI/automated testing (max CPU speed)
- **Visual** - Dev-UI dropdown for interactive verification

## Folder Structure

```
specs/
├── behavior/     # AI behaviors (seeking, avoidance, wandering)
├── physics/      # Collision and physics scenarios
└── performance/  # Load testing and stress tests
```

## Creating a New Spec

### Minimal Example

```toml
[meta]
name = "My Test"
timeout_seconds = 10

[[spawns]]
type = "single"
x = 0.0
y = 0.0
creature_type = "wanderer"
```

### Full Example with Assertions

```toml
[meta]
name = "Seeker Reaches Target"
description = "Verify seeker navigates to target"
timeout_seconds = 30
seed = 12345

[[assertions]]
type = "ticks_completed"
count = 500

[[assertions]]
type = "creature_reached_target"
tag = "seeker"

[[spawns]]
type = "single"
tag = "seeker"
x = -20.0
y = 0.0
creature_type = "seeker"
target_x = 20.0
target_y = 0.0
```

## Available Assertions

| Type | Parameters | Description |
|------|------------|-------------|
| `ticks_completed` | `count` | Simulation ran N ticks without crash |
| `creature_count` | `min`, `max` | Population count within range |
| `no_overlaps` | (none) | Zero creature collisions across all ticks |
| `max_overlaps` | `count` | At most N overlapping pairs per tick |
| `max_overlap_depth` | `depth` | Max penetration depth <= N units |
| `max_ticks_with_overlaps` | `count` | At most N ticks had any overlap |
| `creature_reached_target` | `tag` | Tagged creature arrived at target |
| `max_avg_tick_latency` | `microseconds` | Avg tick time under threshold |

### Overlap Assertion Examples

```toml
# Strict - no overlaps allowed at any point
[[assertions]]
type = "no_overlaps"

# Lenient - allow up to 5 overlapping pairs per tick
[[assertions]]
type = "max_overlaps"
count = 5

# Depth limit - overlaps must be shallow (< 0.5 units penetration)
[[assertions]]
type = "max_overlap_depth"
depth = 0.5

# Duration limit - at most 200 ticks can have ANY overlap
[[assertions]]
type = "max_ticks_with_overlaps"
count = 200
```

### Count and Duration Examples

```toml
# Ensure population stays within bounds
[[assertions]]
type = "creature_count"
min = 100
max = 200

# Ensure simulation runs for at least 500 ticks
[[assertions]]
type = "ticks_completed"
count = 500
```

### Performance Assertion Examples

```toml
# Avg tick must complete in under 5ms (5000 microseconds)
[[assertions]]
type = "max_avg_tick_latency"
microseconds = 5000

# For performance specs, combine with creature count
[[assertions]]
type = "creature_count"
min = 2500
max = 2500

[[assertions]]
type = "max_avg_tick_latency"
microseconds = 10000  # 10ms with 2500 creatures
```

## Spawn Patterns

### Single

Spawn one creature at a specific position.

```toml
[[spawns]]
type = "single"
tag = "optional-tag"      # For assertion tracking
x = 10.0
y = 20.0
creature_type = "seeker"  # catatonic | seeker | wanderer
target_x = 50.0           # Optional: seeker target X
target_y = 0.0            # Optional: seeker target Y
```

### Grid

Spawn creatures in a rectangular grid.

```toml
[[spawns]]
type = "grid"
tag = "crowd"             # Optional
start_x = -50.0           # Top-left corner X
start_y = -50.0           # Top-left corner Y
spacing = 2.0             # Distance between creatures
rows = 10
cols = 10
creature_type = "catatonic"
grid_offset_y = 0.5       # Optional: stagger every other row
```

### Circle

Spawn creatures in a circular arrangement.

```toml
[[spawns]]
type = "circle"
tag = "seekers"           # Optional
center_x = 0.0
center_y = 0.0
radius = 50.0             # Spawn radius
count = 8                 # Number of creatures
creature_type = "seeker"
target_x = 0.0            # Optional: shared target X
target_y = 0.0            # Optional: shared target Y
```

## Creature Types

| Type | Behavior |
|------|----------|
| `catatonic` | Stationary, no movement |
| `seeker` | Moves toward target position |
| `wanderer` | Random walk behavior |

## Running Specs

### Headless (CI/automated)

```bash
# From project root - runs all specs with output
./scripts/run-specs.sh

# Or manually (MUST use --release for accurate performance measurements)
cd apps/simulation
cargo test --release --features dev-tools --test spec_runner -- --nocapture
```

**Important:** Always use `--release` for performance tests. Debug builds are ~10x slower and will fail latency assertions.

### Visual (Dev-UI)

```bash
npm run dev
# Specs appear in dropdown organized by category
```

## Variants (Parameterized Testing)

Run specs multiple times with varying parameters:

```toml
[variants]
crit_size = { min = 0.5, max = 2.0, steps = 10 }
speed = { min = 1.0, max = 5.0, steps = 5 }
```

This runs the spec `10 * 5 = 50` times with different combinations.

## Meta Configuration

```toml
[meta]
name = "Required Name"
description = "Optional description"
timeout_seconds = 10      # WALL-CLOCK seconds to run test
seed = 12345              # Optional: deterministic randomness
```

## Understanding delta_time

**`delta_time`** = "How much game-time passes per tick" (always 0.05s, not configurable)

### In the Real Game

The game loop uses a FIXED delta_time and SLEEPS to maintain real-time:
```
Frame 1: delta_time = 0.05s (fixed), compute = 32ms, sleep 18ms
Frame 2: delta_time = 0.05s (fixed), compute = 28ms, sleep 22ms
```
- `delta_time` is a FIXED constant (0.05s = 1/20Hz)
- Game SLEEPS after each tick to maintain real-time 20Hz
- A creature at 10 units/sec moves `10 × 0.05 = 0.5` units per tick
- Game time and real time stay synchronized via sleep

### In Headless Tests (Behavior & Physics)

Tests run as fast as CPU allows - no waiting between ticks:
```
Tick 1: delta_time = 0.05, compute time = 0.1ms, no wait
Tick 2: delta_time = 0.05, compute time = 0.1ms, no wait
... 1000 ticks complete in ~100ms total ...
```
- Same `delta_time` as production (0.05)
- Same physics accuracy as production
- But no 50ms sleep between ticks → runs 500× faster
- **Goal:** Test correctness quickly

### In Performance Tests

Tests measure if we can keep up with real-time:
```
Tick 1: delta_time = 0.05, took 32ms ✓ (under 50ms budget)
Tick 2: delta_time = 0.05, took 55ms ✗ (over 50ms budget!)
```
- Same `delta_time` as production (0.05 for 20Hz)
- Measure actual wall-clock time per tick
- If tick takes longer than 50ms, game can't run at 20Hz
- **Goal:** Verify production viability

## Timeout Configuration

`timeout_seconds` specifies how long the test runs in **wall-clock time** (real seconds on your clock).

- Test runs until that many real seconds pass
- Fast simulations complete more ticks in that time
- `ticks_completed` assertion verifies enough ticks finished

### Guidelines

| Test Type | timeout_seconds | Notes |
|-----------|-----------------|-------|
| Behavior | `10` | Runs fast (no wait between ticks) |
| Physics | `10` | Runs fast (no wait between ticks) |
| Performance | `30` | Measures real tick compute time |

**Note:** All tests use production `delta_time` (0.05 for 20Hz). This is hardcoded and cannot be overridden.

### Performance Test Example

```toml
[meta]
name = "200K Performance Test"
timeout_seconds = 30      # Run for 30 real seconds

[[assertions]]
type = "ticks_completed"
count = 500               # Must complete 500+ ticks in 30s

[[assertions]]
type = "max_avg_tick_latency"
microseconds = 50000      # Each tick must finish in <50ms (20Hz budget)
```

**Key insight:** Production runs at 20Hz (50ms/tick). Performance tests should:
1. Assert `max_avg_tick_latency` ≤ 50000μs (50ms budget)
2. This proves the simulation can keep up with real-time

## Best Practices

1. **Name specs clearly** - Use descriptive names that indicate what's being tested
2. **Set reasonable timeouts** - `timeout_seconds` should be long enough for the behavior to complete
3. **Use tags for tracking** - Tag specific creatures you want to verify with assertions
4. **Start with lenient assertions** - Use `max_overlaps` before `no_overlaps` during development
5. **Organize by category** - Put behavior tests in `behavior/`, stress tests in `performance/`
