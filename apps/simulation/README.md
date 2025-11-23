# Simulation Server

The core AI life simulation engine built with Rust, using the Bevy ECS framework. This is a **console-only, headless simulation** service with no network layer, designed for clean separation of concerns.

## Overview

This simulation service manages:
- **Non-player organisms** (creatures) with emergent DNA-driven behaviors
- **Entity Component System (ECS)** for high-performance entity management
- **Simulation tick loop** running at 20 Hz with deterministic state updates
- **Console logging** for monitoring simulation state and performance

**Architecture Note:** This service is intentionally stripped down to pure simulation logic. Network communication, serialization, and player interaction will be handled by separate services.

## Technology Stack

- **Bevy ECS 0.14** - Entity Component System for high-performance simulation
- **Rust std** - Standard library threading and timing (no async overhead)
- **rand** - Random number generation for creature behaviors
- **log + env_logger** - Lightweight logging
- **Rust 1.70+** - Systems programming language

## Prerequisites

- **Rust 1.70+** - Install from [rustup.rs](https://rustup.rs/)

## Installation

```bash
# Build the simulation server
cargo build

# Build optimized release binary
cargo build --release
```

## Running the Simulation

The simulation supports three starting modes:

### 1. Default Mode

Run with hardcoded default configuration (1 creature, 180x130 world):

```bash
cargo run
```

### 2. Start Fresh from TOML Config

Load world parameters and spawn random creatures from a TOML state file:

```bash
cargo run -- --state sim_state.toml
```

**State File Format (TOML):**

Minimal example:
```toml
[metadata]
version = "1.0"
description = "Simple simulation"
created_at = "2025-11-04T12:00:00Z"

[world]
width = 180.0
height = 130.0

[spawn]
random_creatures = 10000
```

Full example with optional fields:
```toml
[metadata]
version = "1.0"
description = "Custom simulation state"
created_at = "2025-11-04T12:00:00Z"

[world]
width = 180.0
height = 130.0

[spawn]
random_creatures = 10000

# Optional: Creature size range (defaults to 0.5-2.0)
size_min = 0.5
size_max = 2.0
```

**Required fields:** `random_creatures`
**Optional fields:** size range (defaults to 0.5-2.0)
**Note:** Creatures now spawn at world center (0, 0) for tuning purposes

### 3. Resume from Save State

Load a complete simulation state from a binary save file (MessagePack):

```bash
# Load a specific save state:
cargo run -- --load-snapshot save-states/2025-11-23_14-30-00.msgpack

# Or let the app auto-load the most recent save:
# (In NAPI/Electron mode, this happens automatically)
```

Save states preserve exact creature states including:
- Position, velocity, acceleration
- Creature energy, age, behavior mode
- World boundaries
- Entity IDs

## Automatic Save States

The simulation automatically saves state to protect against data loss:

### Periodic Saves
- **Automatic**: Saves every 5 minutes by default (configurable)
- **Non-blocking**: Runs on separate thread, doesn't slow down simulation
- **Auto-cleanup**: Keeps last 20 save states, deletes older ones
- **Performance**: Only 1-2ms impact on main thread every 5 minutes

### Shutdown Saves
- **On Ctrl+C**: Gracefully saves state when you press Ctrl+C
- **On SIGTERM**: Also saves on termination signals
- **Guaranteed**: Waits for all pending saves to complete before exiting

### Save State Files

Save states are stored in the `save-states/` directory:

```
save-states/
  ├── 2025-11-23_12-00-00.msgpack
  ├── 2025-11-23_12-05-00.msgpack
  ├── 2025-11-23_12-10-00.msgpack
  └── 2025-11-23_12-15-00.msgpack  (most recent)
```

- **Format**: `YYYY-MM-DD_HH-MM-SS.msgpack` (timestamp-based naming)
- **Both periodic and shutdown** saves create timestamped files
- **Auto-load**: On startup, Electron app automatically loads the most recent file
- **Retention**: Only last 20 save states kept, older ones automatically deleted

### Configuration

Save behavior is configured in `src/config.rs` via `SaveStateConfig`:

```rust
pub struct SaveStateConfig {
    pub enabled: bool,         // Enable/disable saves (default: true)
    pub interval_secs: u64,    // Seconds between saves (default: 300 = 5 min)
    pub keep_last_n: usize,    // Max saves to keep (default: 20)
}
```

To change defaults, modify `SaveStateConfig::default()` in `src/config.rs`.

### Logging

```bash
# Run with info-level logging (default)
RUST_LOG=info cargo run

# Run with verbose debug logging
RUST_LOG=debug cargo run
```
## Testing

```bash
# Run all unit tests (fast - these can run in parallel)
cargo test --lib
cargo test --bin speciate

# Run integration tests (must be serial to avoid race conditions)
cargo test --test save_state_integration -- --test-threads=1

# Run ALL tests properly
cargo test --lib && cargo test --bin speciate && cargo test --test save_state_integration -- --test-threads=1

# Run tests with output
cargo test -- --nocapture

# Run only simulation module tests
cargo test simulation::

# Run with specific logging
RUST_LOG=debug cargo test -- --nocapture
```

**Note**: Integration tests share the `save-states/` directory and must run with `--test-threads=1` to avoid race conditions.
## Building for Release

```bash
# Standard release build
cargo build --release

# Optimized build with CPU-native instructions (recommended for max performance)
RUSTFLAGS="-C target-cpu=native" cargo build --release

# Run the simulation
./target/release/speciate
```

## Architecture

### Entity Component System (ECS)

The simulation uses an ECS architecture where:

- **Entities** are unique identifiers for objects in the simulation
- **Components** are data containers (Position, Velocity, Health, DNA, etc.)
- **Systems** are functions that operate on entities with specific components

### Core Components

- **Position** - 2D coordinates (x, y)
- **Velocity** - Movement vector (vx, vy)
- **Health** - Current and maximum health points
- **EntityId** - Unique identifier for each entity
- **DNA** - Genetic information (future)

## Development

### Code Style

```bash
# Format code
cargo fmt

# Lint code
cargo clippy -- -D warnings
```

### Performance

- Release builds use LTO (Link Time Optimization) and level 3 optimizations
- Development builds prioritize fast compilation
- See `Cargo.toml` for profile settings
- Current performance: ~13-14ms per tick for 10,000 creatures

## Service Architecture

This is a **headless, console-only simulation service**. It is intentionally decoupled from:
- Network communication (WebSocket, HTTP)
- Serialization/deserialization overhead
- Player interaction handling
- Economy ledger transactions

These concerns will be handled by separate microservices that communicate with the simulation engine.
