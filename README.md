# Speciate - AI Life Simulation

A server-authoritative AI life simulation game with a player-driven economy, built with Rust, TypeScript, and Node.js.

# Remind user of this cmd, he forgets all the time
claude --dangerously-skip-permissions

## Overview

Speciate is a simulation where:
- **Non-player organisms** (plants and creatures) exhibit emergent DNA-driven behaviors
- **Players** participate as avatars in the ecosystem, gathering resources and crafting
- **Economy** is managed by a separate secure ledger service with ACID guarantees
- **Simulation** runs at 20 Hz on the server with client-side prediction on the frontend

## Project Architecture

```
┌─────────────────────────────────────────────────────┐
│                   Frontend (UI)                     │
│     TypeScript, React/Vue, Pixi.js, Vite            │
│              (apps/ui/README.md)                    │
└──────────────────┬──────────────────────────────────┘
                   │
      ┌────────────┴────────────┐
      │                         │
┌─────▼─────────────┐   ┌──────▼──────────────┐
│  Simulation       │   │  Ledger             │
│  Server           │   │  Microservice       │
│  (Rust)           │   │  (Node.js)          │
│  Bevy ECS,        │   │  Express,           │
│  Tokio            │   │  PostgreSQL         │
|  DB TBD!          |   |                     |
│ (apps/simulation/ │   │ (apps/ledger/       │
│  README.md)       │   │  README.md)         │
└───────────────────┘   └─────────────────────┘
```

## Getting Started

### Quick Start

```bash
# Clone the repository
git clone https://github.com/anthropics/speciate.git
cd speciate

# Each app has its own setup
# See apps/simulation/README.md, apps/ledger/README.md, apps/ui/README.md
```

## Application Components

The project is organized as a monorepo with three independent applications:

### 1. [Simulation Server](apps/simulation/README.md)
**Rust | Bevy ECS | Tokio | 20 Hz tick rate**

The server-authoritative simulation engine that manages:
- Entity Component System for all organisms
- Deterministic physics and interactions
- Player action processing
- Real-time state updates to clients

**Getting Started:**
```bash
cd apps/simulation
cargo run
```

### 2. [Ledger Microservice](apps/ledger/README.md)
**Node.js | TypeScript | PostgreSQL | ACID**

The secure, immutable economy service that tracks:
- Player resources and currency
- Transaction history
- Trade and exchange validation
- Inventory management

**Getting Started:**
```bash
cd apps/ledger
npm install
npm run dev
```

### 3. [Frontend Application](apps/ui/README.md)
**TypeScript | Pixi.js | Vite | React/Vue**

The client-side web interface providing:
- Real-time simulation rendering
- Player interaction and controls
- Economy UI and inventory
- Client-side prediction for low latency

**Getting Started:**
```bash
cd apps/ui
npm install
npm run dev
```

## Project Structure

```
speciate/                                   # Monorepo root
├── apps/
│   ├── simulation/                         # Rust simulation server
│   │   ├── Cargo.toml
│   │   ├── src/
│   │   └── README.md                       # Simulation docs
│   ├── ledger/                             # Node.js ledger microservice
│   │   ├── package.json
│   │   ├── src/
│   │   └── README.md                       # Ledger docs
│   └── ui/                                 # Frontend application
│       ├── package.json
│       ├── src/
│       └── README.md                       # UI docs
├── docs/
│   ├── architecture/
│   │   ├── ARCHITECTURE.md                 # System design
│   │   └── API_CONTRACTS.md                # Service APIs
│   ├── development/
│   │   └── SETUP.md                        # Dev environment
│   └── deployment/
│       └── DEPLOYMENT.md                   # Deploy guides
├── scripts/
│   ├── build-all.sh                        # Build all services
│   └── test-all.sh                         # Test all services
├── .claude/
│   ├── spec/                               # Project specification
│   └── commands/                           # CLI commands
├── SPRINT_BACKLOG.md                       # Active sprint tasks
├── SPRINT_DOCS/                            # Sprint documentation
├── CONTRIBUTING.md                         # Contribution guidelines
└── README.md                                # This file
```

## Development Workflow

### Creating a Feature Branch

```bash
/sprint-start <sprint-name>
```

This will:
1. Create a feature branch: `feat/<sprint-name>`
2. Initialize SPRINT_DOCS with plan and backlog
3. Guide you through sprint setup

### Closing a Sprint

```bash
/sprint-end
```

This will:
1. Archive sprint summary
2. Clean up sprint documentation
3. Prepare for merge to main

## Contributing

Please see [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines on:
- Branch naming conventions
- Commit message format
- Code review process
- Deployment procedures

## AI Team

The Speciate project uses specialized AI agents to assist with development:

- **architect-andy** - Technical design and architecture standards
- **backend-simulation-sam** - Rust ECS simulation engine
- **backend-ledger-larry** - Economy ledger microservice
- **frontend-fanny** - Client rendering and UI
- **botanist-betsy** - Flora simulation logic
- **zoologist-tom** - Fauna and creature behaviors
- **environment-eddy** - World generation and biomes
- **gamification-garry** - Game mechanics and progression
- **play-test-petra** - Testing and QA
- **devops-daria** - Infrastructure and deployment
- **pm-pam** - Project management

## Resources

- **[Project Specification](docs/architecture/ARCHITECTURE.md)** - Detailed technical design
- **[API Contracts](docs/architecture/API_CONTRACTS.md)** - Inter-service communication
- **[Bevy ECS Documentation](https://docs.rs/bevy_ecs/)** - ECS framework docs
- **[Pixi.js Documentation](https://pixijs.download/release/docs/index.html)** - Rendering library
- **[Rust Book](https://doc.rust-lang.org/book/)** - Learning Rust