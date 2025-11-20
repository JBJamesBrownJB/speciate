# Contributing to Speciate

Thank you for your interest in contributing to Speciate! This document provides guidelines for contributing to the project.

## Table of Contents

- [Getting Started](#getting-started)
- [Development Workflow](#development-workflow)
- [Code Quality Standards](#code-quality-standards)
- [Testing Requirements](#testing-requirements)
- [Submitting Changes](#submitting-changes)
- [AI Development Team](#ai-development-team)

---

## Getting Started

### Prerequisites

**Development Requirements:**
- Rust 1.75+ (`rustc --version`)
- Node.js 18+ (`node --version`)
- npm 10+ (`npm --version`)

### First-Time Setup

1. Clone the repository:
   ```bash
   git clone <repo-url>
   cd speciate
   ```

2. Install dependencies and run:
   ```bash
   cd apps/portal
   npm install
   npm run dev  # Builds Rust + frontend, launches Electron
   ```

See [README.md](README.md#getting-started) for detailed setup instructions.

---

## Development Workflow

### Running the Application

```bash
cd apps/portal
npm run dev  # Development mode with hot reload
```

This command:
- Builds the Rust simulation backend (`apps/simulation/`)
- Compiles the TypeScript frontend with Vite
- Launches the Electron desktop app
- Enables hot-reload for frontend changes

### Project Structure

```
/workspace
├── apps/
│   ├── simulation/         # Rust/Bevy ECS simulation engine
│   │   ├── src/
│   │   └── tests/
│   └── portal/             # PixiJS frontend + Electron wrapper
│       ├── electron/       # Electron main process + preload
│       ├── src/            # TypeScript frontend (PixiJS)
│       └── dist/           # Vite build output
├── docs/
│   ├── strategy/           # Business model, game goal
│   ├── architecture/       # Electron patterns, performance
│   ├── biology/            # DNA design, species, zoologist notes
│   └── gameplay/           # Taming, combat, progression
└── .claude/
    ├── agents/             # AI development team definitions
    ├── commands/           # Custom slash commands
    └── hooks/              # Pre-commit validation scripts
```

### Development Commands

**Frontend (apps/portal):**
```bash
npm run dev          # Development mode (Electron + hot reload)
npm run build        # Build frontend for production
npm run package      # Package desktop app (.exe, .dmg, .AppImage)
npm run type-check   # Run TypeScript type checking
npm test             # Run frontend tests
```

**Backend (apps/simulation):**
```bash
cargo build --release   # Build Rust simulation binary
cargo test              # Run Rust unit tests
cargo test -- --nocapture  # Run tests with output
```

---

## Code Quality Standards

### TypeScript

- **No `any` types** - Use proper interfaces and type annotations
- **Strict null checks** - Handle `null` and `undefined` explicitly
- **Prefer const** - Use `const` over `let` when possible
- **Naming conventions:**
  - Classes: `PascalCase`
  - Functions/variables: `camelCase`
  - Constants: `UPPER_SNAKE_CASE`
  - Interfaces: `PascalCase` (no `I` prefix)

### Rust

- **Follow Clippy lints** - Run `cargo clippy` before committing
- **Document public APIs** - Add doc comments to public functions/structs
- **Avoid `unwrap()`** - Use `?` operator or explicit error handling
- **Naming conventions:**
  - Structs/Enums: `PascalCase`
  - Functions/variables: `snake_case`
  - Constants: `UPPER_SNAKE_CASE`

### Console Logging

- **NEVER** use `console.log()` for debug/verbose output
- **ONLY** use `console.error()` for actual errors
- **ONLY** use `console.warn()` for warnings
- Remove ALL debug console.logs before committing

### Architecture

- **Domain layer:** Pure TypeScript (Camera, Viewport)
- **Rendering layer:** PixiJS integration (GridRenderer, SpriteProvider)
- **Infrastructure:** External services (ElectronIPCClient, SpritePool)

---

## Testing Requirements

**CRITICAL: Test-Driven Development (TDD) is MANDATORY for all contributions.**

### The Red-Green-Refactor Cycle

All code changes must follow the complete TDD workflow:

#### 🔴 RED - Write a Failing Test
1. Write a test that describes the desired behavior
2. Run the test and watch it fail (proves it's testing something new)
3. For bugs: Write a test that reproduces the bug first

#### 🟢 GREEN - Make it Pass
1. Write the minimum code to make the test pass
2. Don't worry about code quality yet
3. Run tests and verify they pass

#### 🔵 REFACTOR - Make it Right
1. Improve code quality WITHOUT changing behavior:
   - Remove duplication (DRY principle)
   - Apply SOLID principles
   - Improve naming and structure
   - Extract methods for clarity
   - Simplify complex logic
2. Run tests after EACH refactoring step to ensure they still pass
3. **Never skip this step** - passing tests don't mean good code

#### 🔁 REPEAT
- Each cycle should be small (2-10 minutes)
- Commit after completing a full cycle with all tests passing

### Running Tests

```bash
# Frontend
cd apps/portal && npm test

# Backend
cd apps/simulation && cargo test
```

### Before Committing
1. Ensure you've completed the REFACTOR phase
2. Run ALL tests (frontend AND backend)
3. Verify all tests pass
4. Only then commit your changes

See [CLAUDE.md](CLAUDE.md#test-driven-development-tdd---mandatory) for complete TDD workflow details.

---

## Submitting Changes

### Before Committing

1. **Run tests:**
   ```bash
   cd apps/portal && npm test
   cd apps/simulation && cargo test
   ```

2. **Run type checker:**
   ```bash
   cd apps/portal && npm run type-check
   ```

3. **Run linter:**
   ```bash
   cargo clippy  # Rust
   ```

4. **Build verification:**
   ```bash
   cd apps/portal
   npm run build  # Verify frontend builds
   ```

### Git Workflow

1. Create a feature branch:
   ```bash
   git checkout -b feat/your-feature-name
   ```

2. Make your changes following TDD workflow

3. Commit with descriptive messages:
   ```bash
   git add .
   git commit -m "feat: Add creature perception system

   - Implement spatial hash grid for O(1) neighbor queries
   - Add perception range based on DNA trait
   - Write 12 unit tests for edge cases"
   ```

4. Push to your branch:
   ```bash
   git push -u origin feat/your-feature-name
   ```

5. Create a pull request with:
   - Clear description of changes
   - Test results
   - Screenshots/videos if UI changes
   - Reference to related issues

### Commit Message Format

Use conventional commits:

```
<type>: <description>

[optional body]

[optional footer]
```

**Types:**
- `feat:` - New feature
- `fix:` - Bug fix
- `refactor:` - Code refactoring
- `test:` - Adding/updating tests
- `docs:` - Documentation changes
- `chore:` - Build/tooling changes

---

## AI Development Team

Speciate uses specialized AI agents (via Claude Code) for development assistance:

### Core Engineering
- **architect-andy** - Technical architecture, system design, performance analysis
- **rusty-ron** - Rust simulation, A-Life systems, ECS implementation
- **ecs-eddy** - ECS optimization, performance profiling, Data-Oriented Design
- **instrumentation-ian** - Linux performance analysis, telemetry pipelines, empirical validation
- **frontend-fanny** - PixiJS rendering, UI/UX, client optimization

### Domain Experts
- **zoologist-tom** - Ecosystem design, biology validation, DNA traits
- **botanist-betsy** - Plant biology, growth systems
- **environment-eddy** - Procedural generation, biomes, terrain
- **gamification-garry** - Game design, balance, player motivation
- **narrative-nancy** - Story design, quests, campaign structure (Phase 1.5+)

### Distribution & QA
- **steam-steve** - Steam integration, achievements, cloud saves
- **playtest-petra** - E2E testing, gameplay validation, UX evaluation
- **qa-karen** - Pre-merge reviews, security, standards

### Project Management
- **pm-pam** - Sprint management, task coordination, agile workflow

When working with the AI team, use the appropriate agent for your task:

```bash
# Example: Consult zoologist for DNA trait design
/dna-consult "What's a realistic perception range for a small predator?"

# Example: Get architectural guidance
# Use Task tool with subagent_type: architect-andy
```

---

## DNA-Driven Design - Core Principle

**CRITICAL: All creature physiology and behavior MUST be encoded in DNA.**

### The Rule

**DON'T:** Hardcode creature traits
- Using magic numbers or global constants
- Setting all creatures to perceive the same distance
- Makes all creatures identical, eliminates evolution

**DO:** Derive from DNA
- Read trait values from each creature's individual DNA
- Every creature has unique perception range, speed, aggression
- Enables genetic diversity, evolution, and player breeding

### Workflow for New Traits

1. **Consult zoologist-tom FIRST** (mandatory)
2. Add gene to DNA system with min/max bounds
3. Log decision in `docs/biology/biology-notes.md`
4. Implement trait expression (DNA → phenotype → behavior)

**See:** [docs/biology/dna-driven-design.md](docs/biology/dna-driven-design.md) for complete specification

---

## Questions?

- Check [docs/](docs/) for technical documentation
- Check [CLAUDE.md](CLAUDE.md) for AI team workflows
- Check [README.md](README.md) for setup and quick start
- Review existing tests for examples

---

**The DNA is the creature. Everything else is just expression.**
