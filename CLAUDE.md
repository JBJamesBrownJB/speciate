# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

**Speciate** is a persistent, server-authoritative artificial life simulation with a player-driven economy. Players influence a shared world populated by autonomous, evolving agents through indirect control, survival mechanics, and economic progression.

## Architecture

This is a **Headless Server / Thin Client / Microservice** architecture with three core components:

### 1. Simulation Server (Rust - Authoritative)
- **Technology**: Rust with `bevy_ecs`, `bevy_app`, `tokio`, `axum`
- **Role**: Single source of truth for all real-time simulation logic
- **Communication**: WebSocket for client state sync, REST API calls to Economy Ledger
- **Critical**: NEVER directly accesses the database; all asset changes go through the Economy Ledger Microservice
- **Tick Rate**: Fixed ~20Hz simulation tick
- **Testing**: Follows Chicago School TDD (Outside-In); tests MUST be written first
- **ECS Philosophy**:
  - Systems are small, stateless, single-responsibility
  - Components are simple data structs (e.g., `Velocity`, `Dna`, `Energy`)
  - Logic is decoupled using events or marker components
- **Database**: For persistence ONLY via `sqlx`/`SQLite`; gameplay uses in-memory ECS components

### 2. Economy Ledger Microservice (Node.js/TypeScript)
- **Technology**: Node.js + TypeScript, PostgreSQL (ACID-compliant)
- **Role**: Final authority on all player assets
- **Interface**: REST API for external communication
- **Critical**: Ensures economic security and prevents cheating

### 3. Client (TypeScript - Thin Client)
- **Technology**: TypeScript, Pixi.js (WebGL/WebGPU), Vite, HTML/DOM for UI
- **Role**: Rendering, input, client-side prediction only
- **Performance Requirements**:
  - 60-90 FPS rendering from server's 10-20 Hz updates
  - Interpolation for smooth position/rotation transitions
  - Client-side prediction for player avatar with server reconciliation
  - View-culling and LOD for distant entities
  - Sprite batching and texture atlases for draw call minimization

## Development Workflow

### Git Strategy (Trunk-Based Development)
- **Main Branch**: `main` - production-ready code
- **Feature Branches**: `feat/sprint-[N]/[task-name]` - merge via squash/rebase
- **Fix Branches**: `fix/[brief-description]` - urgent bug fixes
- **PoC Branches**: `poc/[name]` - experimental work; NEVER merge until fully verified

### Sprint Management
- Sprints are 2-5 day time-boxed units
- All work tracked in `SPRINT_BACKLOG.md` (to be created at project root)
- Every code change must link to a task in the backlog and a Git commit
- Session continuity: Always read backlog at start to resume from last checkpoint

### Quality Assurance
- **Pre-merge**: Code review agent (`qa-karen`) must verify technical compliance and run tests
- **E2E Testing**: Playtest agent (`play-test-petra`) used SPARINGLY for:
  - Multi-service communication features
  - Critical UX/fluidity changes
  - New visual system integrations
- Results logged in `PLAYTEST_REPORT.md`

## Specialized Agent System

This project uses specialized agents defined in `.claude/agents/`. Key agents:

- **architect-andy**: High-level architecture, API contracts, integration standards
- **pm-pam**: Sprint planning, task breakdown, workflow coordination
- **backend-simulation-sam**: Rust ECS simulation logic, A-Life systems
- **backend-ledger-larry**: Node.js Economy Ledger API and PostgreSQL
- **frontend-fanny**: Pixi.js rendering, procedural visuals, UI/UX
- **environment-eddy**: Procedural terrain/biome generation
- **devops-daria**: Docker, CI/CD (GitHub Actions), Terraform for GCP
- **zoologist-tom**: Biological accuracy consultant for A-Life systems
- **botanist-betsy**: Plant biology and genetics consultant
- **gamification-garry**: Game balance and player motivation consultant
- **qa-karen**: Pre-merge code review and testing
- **play-test-petra**: End-to-end gameplay testing

Refer to individual agent files for detailed responsibilities.

## Core Technical Principles

### Security & Server Authority
- Client is NEVER trusted
- All player actions use "Predict & Reconcile" model
- Server validates requests against in-memory ECS components
- Assets are "Server-Granted" only; clients never report what they have

### DNA-Driven Design
- All organisms (flora and fauna) have DNA defining behavior and appearance
- Roles (herbivore, predator, etc.) emerge from DNA parameters, not hardcoded labels
- Procedural generation for infinite variation
- Player avatars are NOT exempt from simulation (can be prey)

### Performance Targets
- Hundreds of thousands of concurrent agents
- Fixed 20Hz server tick rate
- 60-90 FPS client rendering
- Low-latency asset delivery via Cloud CDN

## Infrastructure (Google Cloud Platform)

All cloud resources provisioned via **Terraform**:
- **Compute**: GKE or Cloud Run for Rust server and Node.js microservice
- **Database**: Cloud SQL for PostgreSQL
- **Storage**: Cloud Storage + Cloud CDN for static assets
- **Networking**: VPC, load balancers, Cloud DNS

Local development uses `.devcontainer` with Docker Compose orchestrating all services.

## Key Documentation (To Be Created)

The architect is responsible for these foundational documents:
- `API_CONTRACT.md`: REST endpoints, JSON schemas, error codes between services
- `ECS_STANDARDS.md`: Component design rules, units of measure standards
- `ASSET_STRATEGY.md`: Asset storage, texture atlas, CDN deployment policies
- `SPRINT_BACKLOG.md`: Current sprint tasks and status
- `SESSION_LOG.md`: Session-by-session progress log
- `PLAYTEST_REPORT.md`: E2E test results

## Project Specification

The authoritative project specification is located at `.claude/spec/ProjectSpec.md`. All architectural decisions and implementations must align with this specification.

## Philosophy

- **Emergent Behavior**: Simple low-level rules create complex high-level behavior (inspired by "The Nature of Code" by Daniel Shiffman)
- **Scientific Grounding**: A-Life systems informed by real-world biology and zoology
- **Agent Agency**: All creatures have genetics, life cycles, and autonomous decision-making
- **Decoupling**: Strict service boundaries maintained for scalability and security
- **Environment Parity**: Local, dev, and prod environments must be functionally identical
