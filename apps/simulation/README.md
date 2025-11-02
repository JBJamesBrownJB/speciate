# Simulation Server

The core server-authoritative AI life simulation engine built with Rust, using the Bevy ECS framework.

## Overview

This simulation server manages:
- **Non-player organisms** (plants and creatures) with emergent DNA-driven behaviors
- **Player avatar** it is the authority on what the player can and can't do communicating with the frontend which send commands to control the player.
- **Entity Component System (ECS)** for high-performance entity management
- **Server tick loop** running at 20 Hz with deterministic state updates
- **API endpoints** for clients to query simulation state and send player actions

## Technology Stack

- **Bevy ECS 0.14** - Entity Component System for high-performance simulation
- **Tokio** - Async runtime for networking
- **Serde/Serde JSON** - Serialization for data exchange
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

```bash
# Run the hello world simulation demo
cargo run

# Run with logging output
RUST_LOG=info cargo run

# Run with verbose logging
RUST_LOG=debug cargo run
```

### Expected Output

```
=== Speciate: Hello World Simulation ===
Starting simulation engine...

Spawning 5 demo entities...
  Entity #1 spawned at (0.0, 0.0) with velocity (1.0, -0.5)
  Entity #2 spawned at (10.0, 0.0) with velocity (1.5, -0.7)
  ...

Running simulation for 100 ticks (5 seconds at 20Hz)...

Tick: 20 | Time: 1.00s | Entities: 5
  Entity #1: pos=(10.00, -5.00) vel=(1.00, -0.50)
  Entity #2: pos=(20.75, -10.25) vel=(1.50, -0.70)
  ...

=== Simulation Complete ===
Final State: 100 ticks executed in X.XXX seconds
Active entities: 5
Simulation tick rate: 20 Hz (0.05s per tick)
Average wall time per tick: X.XXXX ms
```

## Testing

```bash
# Run all unit and integration tests
cargo test

# Run tests with output
cargo test -- --nocapture

# Run only simulation module tests
cargo test simulation::

# Run with specific logging
RUST_LOG=debug cargo test -- --nocapture
```

## Building for Release

```bash
cargo build --release
./target/release/speciate-simulation
```

## Project Structure

```
simulation/
├── Cargo.toml                    # Project manifest and dependencies
├── Cargo.lock                    # Dependency lock file
├── src/
│   ├── main.rs                   # Application entry point
│   ├── lib.rs                    # Library root
│   ├── simulation.rs             # Core ECS simulation engine
│   ├── components.rs             # ECS component definitions
│   ├── systems/                  # ECS systems (TBD)
│   └── ...                       # Additional modules
├── tests/                        # Integration tests
├── benches/                      # Performance benchmarks
└── README.md                     # This file
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

### Simulation Loop

```
for each tick (20 Hz = 0.05 seconds per tick):
  1. Update entity positions based on velocities
  2. Apply physics and interactions
  3. Handle aging and death
  4. Process reproduction
  5. Send state updates to clients/ledger
```

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

## API Contracts

The simulation server exposes REST/WebSocket APIs for:
- Querying entity state and positions
- Handling player actions
- Broadcasting simulation updates
- Reporting economic transactions to the ledger service

See `/docs/architecture/API_CONTRACTS.md` for detailed specifications (in root project).

## Future Enhancements

- [ ] Persistent storage integration
- [ ] Distributed simulation across multiple servers
- [ ] Advanced genetic algorithms
- [ ] Dynamic biome generation
- [ ] Creature AI behaviors
- [ ] Resource economics integration

## Contributing

See the root project's [CONTRIBUTING.md](../../CONTRIBUTING.md) for guidelines on:
- Branch naming conventions
- Commit message format
- Code review process
- Testing requirements

## License

MIT - See LICENSE file in root project for details
