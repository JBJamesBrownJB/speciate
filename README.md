# Speciate - AI Life Simulation

A server-authoritative artificial life simulation with emergent DNA-driven behaviors and a player-driven economy.

> See [Project_Spec.md](Project_Spec.md) for the complete technical specification.

## Architecture

The system uses a microservices architecture with clean separation of concerns:

![Architecture Diagram](Architecture_High.png).

This means that we only need context, tools, strategies that match the specific concern of each part of the system.

### Components

#### Portal (React + PixiJS)
[TODO: Add details]

Client-side rendering at 60 FPS with interpolation, WebSocket connection management, and player interaction.

**Location:** `apps/ui/` | [Details →](apps/ui/README.md)

#### Player Commander (Node.js)

Authentication, input validation, command routing to simulation. Stateless gateway for player actions.
This microservice's job is to recieve and orchestrate commands and actions from players and negotiate valid commands to the simulation server which is the ultimate source of truth for the players avatar.

**Status:** Planned

#### Broadcaster

Real-time WebSocket distribution of simulation state updates to connected clients. Subscribes to NATS message broker and relays simulation frames to browser clients at 20 Hz.

This microservice's job is to receive pushed state updates from the simulation (via NATS) and broadcast them efficiently to portals, of which there may be thousands.

**Tech:** Node.js, TypeScript, NATS.js, WebSocket (ws library)
**Location:** `apps/broadcaster/` | [Details →](apps/broadcaster/README.md)
**Status:** ✅ **Implemented** (Sprint 6)

#### Simulation (Rust)

Server-authoritative ECS simulation engine running at 20 Hz. Single source of truth for all game state. Publishes agent positions, velocities, and rotations to NATS message broker in real-time.

Maintains a 'Ports & Adapters / Clean Architecture' where I/O (NATS publishing, player commands) runs on separate threads with minimal impact on the core simulation loop.

Uses Rust for extreme performance and Bevy ECS (Entity Component System) for hardware optimization.

**Tech:** Rust, Bevy ECS, async-nats, tokio
**Location:** `apps/simulation/` | [Details →](apps/simulation/README.md)
**Status:** ✅ **NATS Publishing Implemented** (Sprint 6)

#### Sprite Generator
[TODO: Add details]

Procedural sprite generation service for dynamically discovered species. Caches assets to CDN.
This microservice's job is to do the important work of dynamically building, caching and serving sprites for the infinite variabilty of creatures and plants from the simulation.

It is not decided how it will do this just yet...

**Status:** Planned

#### Ledger (Node.js)
[TODO: Add details]

Secure economy microservice with PostgreSQL backend. ACID-compliant transaction handling for all player assets.
This microservice's job is to maintain a secure and consistant transaction ledger to ensure that player owned resources such as wood, biomass, DNA (which represents that the player has ownership of a species).

**Location:** `apps/ledger/` | **Status:** Planned

---

## Getting Started

### Prerequisites

- **Rust** 1.70+ (with Cargo)
- **Node.js** 22.12+ (for Vite 7 ESM support)
- **npm** 10+
- **Docker** and **Docker Compose** (for NATS infrastructure)
- **VS Code** with Dev Containers extension (recommended for development)

### Quick Start (Streaming Pipeline - Sprint 6)

The current implementation demonstrates the **real-time streaming architecture**:

**1. Start NATS Message Broker:**
```bash
cd infrastructure/local
docker compose up -d
```

**2. Start Broadcaster (WebSocket Service):**
```bash
# From devcontainer
cd apps/broadcaster
npm install
npm run dev
```

**3. Start Simulation (Data Publisher):**
```bash
# From devcontainer
cd apps/simulation
cargo run
```

**4. Test the WebSocket Stream:**
```bash
# Install wscat globally
npm install -g wscat

# Connect to the stream
wscat -c ws://localhost:8080/stream
```

You should see JSON frames streaming at ~20 Hz with agent positions and velocities.

**See [infrastructure/local/README.md](infrastructure/local/README.md) for detailed setup instructions.**

---

**Note:** The UI (`apps/ui`) is not yet connected to the streaming pipeline. This integration is planned for Sprint 7.

---

## Application Components

### Simulation Server (Rust)
Headless ECS simulation engine using Bevy 0.14. Manages physics, agent behaviors, and deterministic state updates at 20 Hz. Currently console-only with no network layer.

**Tech:** `bevy_ecs`, `bevy_app`, `rand` | [Details →](apps/simulation/README.md)

### Frontend Application (TypeScript)
Web-based client with Pixi.js rendering, interpolation for smooth 60 FPS motion, and real-time state synchronization. (Currently planned - WebSocket integration pending)

**Tech:** `pixi.js@8.14.0`, `vite@7.0.0`, `typescript@5.9.3` | [Details →](apps/ui/README.md)

### Ledger Microservice (Node.js)
Secure, ACID-compliant economy service with PostgreSQL persistence. Tracks player resources, transactions, and inventory.

**Tech:** Node.js, TypeScript, PostgreSQL, Express | **Status:** Planned

---

## AI Development Team

Speciate uses specialized AI agents (via Claude Code) for development:

### Core Engineering
- **architect-andy** - Technical blueprints, communication contracts, architectural standards
- **backend-simulation-sam** - Rust simulation engine, A-Life systems, ECS implementation
- **backend-ledger-larry** - Node.js economy ledger, PostgreSQL, ACID transactions
- **frontend-fabian** - Client rendering, Pixi.js optimization, UI/UX design

### Domain Experts
- **botanist-betsy** - Plant biology, genetics, growth cycles, biomass production
- **zoologist-tom** - Ecosystem design, creature behaviors, emergent dynamics
- **environment-eddy** - Procedural world generation, biomes, terrain systems
- **gamification-garry** - Game balance, player motivation, economic design

### Operations
- **playtest-petra** - End-to-end testing, gameplay validation, UX evaluation
- **devops-daria** - CI/CD pipelines, Google Cloud infrastructure, Terraform
- **qa-karen** - Pre-merge code review, test validation, security checks
- **pm-pam** - Sprint management, task coordination, documentation
- **mr-motivator** - Vision alignment, team focus, philosophical guidance

---

## Resources

### Project Documentation
- **[Project_Spec.md](Project_Spec.md)** - Complete technical specification
- **`.claude/spec/`** - Detailed architecture and standards
- **`.claude/agents/`** - AI agent definitions

### Technology Documentation
- **[Pixi.js 8.x Documentation](https://pixijs.com/8.x/guides)** - Rendering library
- **[Bevy ECS](https://docs.rs/bevy_ecs/)** - Entity Component System
- **[Tokio Documentation](https://tokio.rs/)** - Async runtime for Rust
- **[Vite Documentation](https://vite.dev/)** - Frontend build tool
- **[Rust Book](https://doc.rust-lang.org/book/)** - Learning Rust
- **[TypeScript Handbook](https://www.typescriptlang.org/docs/)** - TypeScript guide
