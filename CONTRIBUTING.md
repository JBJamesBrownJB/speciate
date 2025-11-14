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

### Test-Driven Development (TDD) - MANDATORY

**CRITICAL: You MUST follow Test-Driven Development principles at all times.**

#### TDD Workflow

1. **Before ANY code change:**
   - Run `npm test` to verify current state
   - Ensure all tests pass before proceeding
   - If tests fail, FIX THEM FIRST before making any other changes

2. **When making changes:**
   - Write tests FIRST if adding new functionality
   - Make the minimal change needed
   - Run tests IMMEDIATELY after the change
   - If tests fail, revert or fix immediately

3. **NEVER:**
   - Make code changes without running tests
   - Assume code works without test verification
   - Skip tests because "it's a small change"
   - Batch multiple changes before testing

#### Test-First Bug Fixing

**CRITICAL: When debugging, write a failing test BEFORE investigating the bug.**

1. **Reproduce the bug in a test:**
   - Write the simplest test that fails due to the bug
   - Verify the test fails with the current code
   - This proves you understand the bug

2. **Fix the bug:**
   - Make minimal changes to fix the issue
   - Run the test to verify it now passes
   - Run ALL tests to ensure no regressions

3. **NEVER:**
   - Jump straight into "fixing" without a failing test
   - Add console.logs instead of writing tests
   - Assume a fix works without test verification

#### Running Tests

**Frontend tests:**
```bash
cd apps/portal
npm test            # Run full test suite
npm run test:watch  # Run tests in watch mode
```

**Backend tests:**
```bash
cd apps/simulation
cargo test                    # Run all Rust tests
cargo test -- --nocapture     # Run with output
cargo test test_name          # Run specific test
```

**Current test coverage:** 196 tests passing (Portal + Simulation)

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
- **backend-simulation-sam** - Rust simulation, A-Life systems, ECS implementation
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
