# Speciate - AI Life Simulation

A server-authoritative AI life simulation game with a player-driven economy, built with Rust, TypeScript, and Node.js.

> **Reminder:** Use `claude --dangerously-skip-permissions` to bypass permission prompts during development.

## Overview

Speciate is a simulation where:
- **Non-player organisms** (plants and creatures) exhibit emergent DNA-driven behaviors
- **Players** participate as avatars in the ecosystem, gathering resources and crafting
- **Economy** is managed by a separate secure ledger service with ACID guarantees
- **Simulation** runs at 10 TPS on the server with client-side interpolation for smooth 60 FPS rendering

## Project Architecture

```
┌─────────────────────────────────────────────────────┐
│                   Frontend (UI)                     │
│     TypeScript, Pixi.js 8.x, Vite 7, WebSocket      │
│         Real-time rendering at 60 FPS               │
│              (apps/ui/)                             │
└──────────────────┬──────────────────────────────────┘
                   │ WebSocket
                   │ (10 TPS updates)
      ┌────────────┴────────────┐
      │                         │
┌─────▼─────────────┐   ┌──────▼──────────────┐
│  Simulation       │   │  Ledger             │
│  Server           │   │  Microservice       │
│  (Rust)           │   │  (Node.js)          │
│  Custom ECS,      │   │  Express,           │
│  Tokio,           │   │  PostgreSQL         │
│  WebSocket        │   │  (Planned)          │
│ (apps/simulation/)│   │ (apps/ledger/)      │
└───────────────────┘   └─────────────────────┘
```

## Getting Started

### Prerequisites

- **Rust** 1.70+ (with Cargo)
- **Node.js** 22.12+ (for Vite 7 ESM support)
- **npm** 10+

### Quick Start - Run the Complete Stack

```bash
# 1. Start the Rust simulation server (Terminal 1)
cd apps/simulation
cargo run
# Server will start on ws://localhost:8080/ws

# 2. Start the frontend dev server (Terminal 2)
cd apps/ui
npm install
npm run dev
# Frontend will start on http://localhost:3000

# 3. Open your browser
# Navigate to http://localhost:3000
# You should see a cyan circle moving across a black canvas
# The HUD displays FPS, Tick, Ping, and connection status
```

## Application Components

The project is organized as a monorepo with three independent applications:

### 1. Simulation Server
**Rust | Custom ECS | Tokio | WebSocket | 10 TPS**

Location: `apps/simulation/`

The server-authoritative simulation engine that manages:
- Custom HashMap-based Entity Component System
- Deterministic physics and state updates at 10 TPS
- WebSocket broadcasting to connected clients
- Position, velocity, and health tracking for entities

**Tech Stack:**
- `tokio` - Async runtime
- `tokio-tungstenite` - WebSocket server
- `serde_json` - Message serialization

**Run:**
```bash
cd apps/simulation
cargo run
# Listens on ws://localhost:8080/ws
# Health check: http://localhost:8080/health
```

### 2. Frontend Application
**TypeScript | Pixi.js 8.x | Vite 7 | WebSocket**

Location: `apps/ui/`

The client-side web interface providing:
- Real-time rendering at 60 FPS with Pixi.js
- WebSocket client with auto-reconnection
- Linear interpolation (10 TPS → 60 FPS smooth motion)
- HUD showing FPS, tick count, ping, and connection status
- Entity visualization with position tracking

**Tech Stack:**
- `pixi.js@8.14.0` - WebGL/WebGPU rendering
- `vite@7.0.0` - Build tool and dev server
- `typescript@5.9.3` - Type safety

**Run:**
```bash
cd apps/ui
npm install
npm run dev
# Development: http://localhost:3000
# Production build: npm run build
```

### 3. Ledger Microservice
**Node.js | TypeScript | PostgreSQL | Express**

Location: `apps/ledger/` (Planned - Not yet implemented)

The secure, immutable economy service that will track:
- Player resources and currency
- Transaction history with ACID guarantees
- Trade and exchange validation
- Inventory management

**Status:** Planned for future sprint

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

We use a **feature branch workflow** with sprint-based development cycles.

### Starting a New Sprint

```bash
/start-sprint
```

This will:
1. Create a feature branch: `feat/sprint-<name>`
2. Initialize SPRINT_DOCS with plan and backlog
3. Set up the sprint structure

### Ending a Sprint

```bash
/end-sprint
```

This will:
1. Generate sprint summary and archive documentation
2. Clean up sprint files
3. Provide instructions for merging to main

### Current Sprint Status

Check the current sprint progress in:
- `SPRINT_DOCS/PLAN.md` - Sprint goals and implementation plan
- `SPRINT_DOCS/BACKLOG.md` - Task tracking
- `SPRINT_DOCS/PROGRESS.md` - Session logs and notes

## Contributing

Please see [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines on:
- Branch naming conventions
- Commit message format
- Code review process
- Deployment procedures

## AI Development Team

The Speciate project uses specialized AI agents (via Claude Code) to assist with development:

### Core Engineering
- **architect-andy** - Technical design and system architecture
- **backend-simulation-sam** - Rust simulation engine implementation
- **backend-ledger-larry** - Economy ledger microservice (Node.js/TypeScript)
- **frontend-fanny** - Client-side rendering with Pixi.js and UI/UX

### Domain Experts
- **botanist-betsy** - Flora biology, genetics, and resource production
- **zoologist-tom** - Fauna behaviors and ecosystem dynamics
- **environment-eddy** - Procedural world generation and biomes
- **gamification-garry** - Game balance and player motivation

### Operations
- **play-test-petra** - End-to-end testing and quality assurance
- **devops-daria** - CI/CD, infrastructure, and deployment
- **qa-karen** - Pre-merge code review and validation
- **pm-pam** - Sprint management and task coordination
- **mr-motivator** - Vision alignment and team focus

Invoke agents with `/pam <request>` or use the Task tool for specialized work.

## Resources

### Project Documentation
- `.claude/spec/` - Project specification and architecture
- `SPRINT_DOCS/` - Current sprint plan and progress

### Technology Documentation
- **[Pixi.js 8.x Documentation](https://pixijs.com/8.x/guides)** - Rendering library
- **[Tokio Documentation](https://tokio.rs/)** - Async runtime for Rust
- **[Vite Documentation](https://vite.dev/)** - Frontend build tool
- **[Rust Book](https://doc.rust-lang.org/book/)** - Learning Rust
- **[TypeScript Handbook](https://www.typescriptlang.org/docs/)** - TypeScript guide