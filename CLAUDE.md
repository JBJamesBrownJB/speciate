# Project Instructions for Claude Code

## Quick Reference

**Key Documentation:**
- `docs/spec/` - **Live specification of implemented features** (brain-spec.md, etc.)
- `docs/archive/dual-tick/` - ⚠️ ABANDONED architecture (Sprint 11, archived for learning)
- `docs/architecture/napi-architecture.md` - Current NAPI-RS integration (zero-copy buffers)
- `docs/biology/dna-driven-design.md` - DNA-driven design principles (detailed)
- `docs/architecture/electron-architecture.md` - IPC protocol and Electron patterns
- `docs/biology/biology-notes.md` - Zoologist consultation log
- `SPRINT_DOCS/` - Current and past sprint plans

**Current Sprint:** Sprint 14 - Interpolation, Vision Refactor & Data-Oriented Design (IN PROGRESS)
- Branch: `feat/sprint-14-interpolation-perception`
- Focus: Scale to 150K-200K creatures via interpolation, vision optimization, and ECS refactoring
- Status: Phase 1 complete (tick rate validated at 22.2Hz)
- See: `SPRINT_DOCS/SPRINT_PLAN_sprint-14-interpolation-perception.md`

**Recent Completion:** Sprint 13 delivered zero-copy double-buffer architecture, replacing stdio MessagePack IPC

**Tick Rate:** 22.2Hz (hardcoded in `simulation_engine.rs`, ~45ms per tick)
- Replaced configurable tick rate from old stdio system
- Provides 2.7x capacity improvement vs 60Hz
- Sufficient for 150K-200K creature target

---

## Test-Driven Development (TDD) - MANDATORY

**CRITICAL: Follow the Red-Green-Refactor cycle for ALL changes.**

### TDD Workflow: Red-Green-Refactor

The complete TDD cycle has three mandatory stages:

#### 1. 🔴 RED - Write a Failing Test
- **Before ANY change:** Write a test that describes the desired behavior
- The test MUST fail initially (proving it tests something new)
- Write the test for the interface you wish existed
- **For bugs:** Write a test that reproduces the bug

#### 2. 🟢 GREEN - Make it Pass
- Write the **minimum** code to make the test pass
- Don't worry about perfection yet
- Focus on getting to green as quickly as possible
- Verify the test passes

#### 3. 🔵 REFACTOR - Make it Right
- **Improve code quality WITHOUT changing behavior**
- Apply SOLID principles
- Remove duplication (DRY)
- Improve naming and structure
- Extract methods/functions for clarity
- Simplify complex logic
- **Verify tests still pass after each refactoring step**

#### 4. 🔁 REPEAT
- Start the cycle again for the next small increment
- Each cycle should be 2-10 minutes, not hours

### Before Starting Any Work
- Run ALL tests, ensure they pass (clean baseline)

### After Completing Red-Green-Refactor
- Run ALL tests IMMEDIATELY
- Commit only when all tests pass

**NEVER:**
- Skip the RED phase (no test = no code)
- Skip the REFACTOR phase (passing tests ≠ good code)
- Make changes without running tests
- Assume code works without verification
- Skip tests for "small changes"
- Jump into fixing without a failing test first

**Exception:** Environment issues (GPU drivers, Docker config) don't need tests.

---

## Specification Documentation - MANDATORY

**CRITICAL: Update `docs/spec/` when implementing features.**

### What Goes in Specs

The `docs/spec/` folder contains **live documentation of IMPLEMENTED features**:
- Current behavior (not planned/future)
- Constants and their values
- Component structures
- System interactions
- Design decisions with rationale

### When to Update

**After implementing a feature:**
1. Create or update the relevant spec file (e.g., `brain-spec.md`, `movement-spec.md`)
2. Document what IS, not what WILL BE
3. Include actual constant values from code
4. Describe system interactions

**Spec files:**
- `brain-spec.md` - Brain component, decision timing, panic override
- `movement-spec.md` - Movement systems, steering behaviors
- `perception-spec.md` - Vision system, neighbor detection
- (Add more as features are implemented)

### Format

Each spec should include:
- **Status:** Implemented/Partial/Planned
- **Location:** Source file paths
- **Overview:** What the system does
- **Components:** Structs and enums
- **Constants:** Hardcoded values with descriptions
- **Integration:** How it connects to other systems

---

## DNA-Driven Design - MANDATORY

**CRITICAL: All creature traits MUST be encoded in DNA.**

### Core Principle

DNA encodes **primitive traits** (size, perception range, aggression threshold). Complex behaviors **emerge** from combinations of these primitives.

**DO:** Derive traits from individual creature DNA
- Physical parameters: size, speed, perception distance
- Simple thresholds: hunger level, flee threshold
- Binary flags: flocking yes/no, diurnal/nocturnal

**DON'T:** Hardcode traits with magic numbers or global constants

### Trade-offs

Every advantage must have a cost (built into physics/biology):
- Large size = higher speed BUT massive energy consumption
- High speed = escape predators BUT burns energy rapidly
- Long perception = detect threats BUT cognitive overload

**Goal:** Create viable ecological niches, not perfect balance.

### New Trait Workflow

1. Consult `zoologist-tom` agent FIRST
2. Add gene to DNA system with biological bounds
3. Log decision in `docs/biology/biology-notes.md`
4. Implement trait expression (DNA → phenotype → behavior)

**Full details:** `docs/biology/dna-driven-design.md`

---

## Application Architecture: Portal vs Dev-UI - MANDATORY

**CRITICAL: The project has TWO separate frontend applications. Do NOT confuse them!**

### Portal (`apps/portal/`)
- **Purpose:** End-user game client (will be distributed to PLAYERS)
- **Technology:** PixiJS renderer + TypeScript domain logic
- **UI:** Minimal HUD (FPS, creature count, zoom, scale bar)
- **Displays:** Game world, creatures, player controls, gameplay UI ONLY
- **Rule:** NEVER add developer metrics, profiling, charts, or debugging UI to portal

### Dev-UI (`apps/dev-ui/`)
- **Purpose:** Developer tools window (ONLY for development, NEVER shipped)
- **Technology:** React + TypeScript
- **UI:** Performance metrics, hardware counters, system timings, spawn controls
- **Displays:** ALL developer-facing metrics, profiling, charts, debugging tools
- **Rule:** ALL performance metrics and debugging displays belong in dev-ui, NOT portal

### Critical Distinction

**Portal = Game (for players)**
**Dev-UI = Metrics (for developers)**

**If adding hardware counters, profiling, performance graphs:**
→ **dev-ui**, NOT portal!

**If adding gameplay UI, creature rendering, player controls:**
→ **portal**, NOT dev-ui!

**Think:** "Would a PLAYER see this?"
- YES → portal
- NO (it's for developers) → dev-ui

---

## Code Documentation Standards - MANDATORY

**CRITICAL: Code comments are a code smell. Refactor instead.**

### Source of Truth (in order)

1. The code itself (self-documenting names/structure)
2. Type signatures (contracts)
3. Tests (executable documentation)
4. `docs/` (architecture and rationale)

### What is BANNED

- Doc comments (JSDoc `/***/`, Rustdoc `///`)
- Explanatory comments
- Algorithm descriptions in code
- Parameter documentation
- Examples in comments (write tests instead)

### What is ALLOWED

- **Concise constant descriptions:** One-line inline comments
  ```rust
  pub const COMFORT_ZONE: f32 = 20.0; // Distance critter wanders from home
  ```
- **TODO markers:** With context
  ```rust
  // TODO(DNA): Migrate to gene expression
  ```
- **Shell script headers:** One-line description only

**Rule:** If you need more than one line, it belongs in `docs/`.

---

## Commands

### Testing
```bash
# Frontend
cd apps/portal && npm test

# Backend
cd apps/simulation && cargo test
```

### Development
```bash
# Run Electron desktop app
cd apps/portal && npm run dev

# Build/package
npm run build && npm run package
```

---

## Code Quality

### Console Logging
- **NEVER** use `console.log()` for debug output
- **ONLY** use `console.error()` for actual errors

### TypeScript
- Avoid `any` types
- Keep tests synchronized with implementation

### Architecture
- Domain layer: Pure TypeScript
- Rendering layer: PixiJS integration
- Infrastructure: External services

---

## Project Context

**Phase 1 (Current):** Standalone desktop game
- Electron + Rust/Bevy subprocess
- TypeScript + PixiJS frontend
- MessagePack stdio IPC
- Steam distribution target

**Phase 2 (Future):** MMO multiplayer
- Microservices architecture
- WebSocket streaming
- Persistent cloud world

---

## Remember

**Run tests. Always. Every time. Before and after changes.**
