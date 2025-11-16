# ECS System Instrumentation Guide

This guide explains how to add performance timing to ECS systems using the zero-cost instrumentation framework.

## Quick Start: Add Timing to a System

**3 lines of code** to instrument any ECS system:

```rust
// 1. Add imports at top of file
#[cfg(feature = "dev-tools")]
use crate::instrumentation::SystemTimings;
#[cfg(feature = "dev-tools")]
use bevy_ecs::system::Res;

// 2. Add conditional parameter to function signature
pub fn my_new_system(
    // ... existing parameters ...
    #[cfg(feature = "dev-tools")] timings: Res<SystemTimings>,
) {
    // 3. Add macro call at start of function body
    #[cfg(feature = "dev-tools")]
    crate::time_system!(timings, "my_new_system");

    // ... rest of system logic unchanged ...
}
```

The `time_system!` macro creates an RAII guard that:
- Records start time when created
- Writes elapsed microseconds to `SystemTimings` when dropped
- Compiles to nothing in production builds (zero overhead)

## Adding a New Timing Field

If you need to track a new system that doesn't fit existing categories:

### 1. Update Rust SystemTimings (`apps/simulation/src/instrumentation/mod.rs`)

```rust
#[derive(Resource)]
pub struct SystemTimings {
    pub movement_us: AtomicU64,
    pub perception_us: AtomicU64,
    pub behavior_us: AtomicU64,
    pub my_new_system_us: AtomicU64,  // ADD NEW FIELD
}

impl SystemTimings {
    pub fn new() -> Self {
        Self {
            movement_us: AtomicU64::new(0),
            perception_us: AtomicU64::new(0),
            behavior_us: AtomicU64::new(0),
            my_new_system_us: AtomicU64::new(0),  // INITIALIZE
        }
    }

    pub fn time(&self, name: &str) -> TimingGuard<'_> {
        let target = match name {
            "movement" => &self.movement_us,
            "perception" => &self.perception_us,
            "behavior" => &self.behavior_us,
            "my_new_system" => &self.my_new_system_us,  // ADD MATCH ARM
            _ => panic!("Unknown system: {}", name),
        };
        TimingGuard::new(target)
    }

    pub fn snapshot(&self) -> SystemTimingsSnapshot {
        SystemTimingsSnapshot {
            movement_us: self.movement_us.load(Ordering::Relaxed),
            perception_us: self.perception_us.load(Ordering::Relaxed),
            behavior_us: self.behavior_us.load(Ordering::Relaxed),
            my_new_system_us: self.my_new_system_us.load(Ordering::Relaxed),  // ADD
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]  // IMPORTANT: Converts to myNewSystemUs in JSON
pub struct SystemTimingsSnapshot {
    pub movement_us: u64,
    pub perception_us: u64,
    pub behavior_us: u64,
    pub my_new_system_us: u64,  // ADD FIELD
}
```

### 2. Update Portal TypeScript (`apps/portal/src/types/GameState.ts`)

```typescript
export interface SystemTimingsSnapshot {
  movementUs: number;
  perceptionUs: number;
  behaviorUs: number;
  myNewSystemUs: number;  // ADD (camelCase)
}
```

### 3. Update Dev-UI TypeScript (`apps/dev-ui/src/types.ts`)

```typescript
export interface SystemTimingsSnapshot {
  movementUs: number;
  perceptionUs: number;
  behaviorUs: number;
  myNewSystemUs: number;  // ADD (camelCase)
}
```

### 4. Update Dev-UI Panel (`apps/dev-ui/src/components/SystemTimingsPanel.tsx`)

Add a new canvas ref and sparkline rendering for the new timing.

## Understanding the Sparklines

The Dev-UI panel (port 5174) displays real-time sparklines for each instrumented system:

### Color Coding
- **Green** (`#4ec9b0`): Normal performance (< 5ms)
- **Amber** (`#f0a830`): Warning threshold (5-10ms)
- **Red** (`#f48771`): Danger threshold (> 10ms)

### Thresholds
- `WARNING_THRESHOLD_US = 5000` (5ms)
- `DANGER_THRESHOLD_US = 10000` (10ms)

### History
- 120-frame buffer (4 seconds at 30 Hz physics tick)
- Dashed red line shows danger threshold
- Graph auto-scales to max(danger_threshold, max_value)

## Verification

### 1. Build with dev-tools (should compile)
```bash
cargo build --features dev-tools
```

### 2. Build without dev-tools (should compile, zero overhead)
```bash
cargo build --no-default-features
```

### 3. Run instrumentation tests
```bash
cargo test --features dev-tools --test instrumentation_test
```

### 4. Verify binary size difference
```bash
cargo build --release --no-default-features && ls -lh target/release/speciate
cargo build --release --features dev-tools && ls -lh target/release/speciate
```

Expected: ~300KB difference (instrumentation code + atomics)

## Currently Instrumented Systems

| System | Timing Field | Location |
|--------|-------------|----------|
| `update_perception_system` | `perception_us` | `src/simulation/perception/systems.rs` |
| `integrate_motion_system` | `movement_us` | `src/simulation/movement/systems.rs` |
| `seek_system` | `behavior_us` | `src/simulation/creatures/behaviors/seek.rs` |

## Architecture Notes

- **Thread Safety**: Uses `AtomicU64` fields for parallel system execution
- **Zero Cost**: `#[cfg(feature = "dev-tools")]` compiles to nothing in production
- **RAII Pattern**: `TimingGuard` auto-records on drop (even on panic)
- **No Ordering Constraints**: Uses `Res<SystemTimings>` (read-only), not `ResMut`

## Files Overview

```
apps/simulation/
├── src/instrumentation/mod.rs     # Core timing infrastructure
├── src/lib.rs                     # time_system! macro (feature-gated)
└── tests/instrumentation_test.rs  # Unit tests

apps/portal/
└── src/types/GameState.ts         # TypeScript interface

apps/dev-ui/
├── src/types.ts                   # TypeScript interface
├── src/components/SystemTimingsPanel.tsx  # React sparkline component
└── src/index.css                  # Styling
```
