# Project Instructions for Claude Code

## Quick Reference

**Key Documentation:**
- `docs/architecture/dual-tick-simulation.md` - Current performance architecture (30Hz physics, 20Hz AI, 90Hz render)
- `docs/biology/dna-driven-design.md` - DNA-driven design principles (detailed)
- `docs/architecture/electron-architecture.md` - IPC protocol and Electron patterns
- `docs/biology/biology-notes.md` - Zoologist consultation log
- `SPRINT_DOCS/` - Current and past sprint plans

**Current Sprint:** Sprint 11 - Dual-Tick Architecture
- Branch: `feat/sprint-11-dual-tick-architecture`
- Focus: Physics/AI tick separation for 150K-200K creature scale
- See: `SPRINT_DOCS/SPRINT_PLAN_sprint-11-dual-tick-architecture.md`

---

## Test-Driven Development (TDD) - MANDATORY

**CRITICAL: Run tests before AND after every change.**

### TDD Workflow

1. **Before ANY change:** Run tests, ensure they pass
2. **When adding features:** Write tests FIRST
3. **When fixing bugs:** Write failing test FIRST, then fix
4. **After changes:** Run tests IMMEDIATELY

**NEVER:**
- Make changes without running tests
- Assume code works without verification
- Skip tests for "small changes"
- Jump into fixing without a failing test

**Exception:** Environment issues (GPU drivers, Docker config) don't need tests.

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
