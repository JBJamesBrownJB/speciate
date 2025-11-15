# Project Instructions for Claude Code

## Test-Driven Development (TDD) - MANDATORY

**CRITICAL: You MUST follow Test-Driven Development principles at all times.**

### TDD Workflow - ALWAYS Follow These Steps:

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

### Why This Matters

Tests exist to catch breaking changes. In this session, you violated TDD by:
- Writing comprehensive tests but not running them before making changes
- Removing null checks that broke the code
- Having to fix breakage that tests would have caught immediately

**Tests are worthless if you don't use them.**

### Test-First Bug Fixing

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

**Example:**
```rust
// Bug: MessagePack deserialization returns array instead of object
// Step 1: Write failing test
#[test]
fn test_msgpack_uses_struct_map() {
    let state = GameState { tick: 42, creatures: vec![] };
    let bytes = rmp_serde::to_vec(&state).unwrap();
    // Inspect actual bytes to see what format we're getting
    println!("Bytes: {:?}", bytes);

    let decoded: GameState = rmp_serde::from_slice(&bytes).unwrap();
    assert_eq!(decoded.tick, 42); // Passes, but doesn't test the real issue

    // The REAL test: Is it using map format (with field names)?
    // Array format starts with 0x92 (fixarray), map format starts with 0x82 (fixmap)
    assert_eq!(bytes[0], 0x82, "Should use map format, not array");
}

// Step 2: Run test → it fails (bytes[0] = 0x92) → investigate serialization
// Step 3: Fix → add .with_struct_map() → test passes → commit
```

**Why This Matters:**
- **Prevents guessing:** A failing test proves you understand the problem
- **Ensures fix works:** Green test = bug is actually fixed
- **Prevents regressions:** Test stays in suite forever
- **Documents the bug:** Future developers know what broke and why

**Exception:** Environment issues (GPU drivers, Docker config, network) don't need tests.

## DNA-Driven Design - MANDATORY

**CRITICAL: All creature physiology and behavior MUST be encoded in DNA.**

### Core Principle

DNA is not just a feature - it's the **architectural foundation** of our A-Life simulation. DNA encodes **primitive traits** (simple parameters like size, perception range, aggression threshold). Complex behaviors like "social" or "territorial" **emerge** from combinations of these primitives.

### Why This Matters

- **Genetic Crossover:** Sexual reproduction combines parent DNA to create unique offspring
- **Species Identification:** Similar DNA = same species (clustering happens naturally)
- **Emergent Behavior:** Rich variety of strategies, niches, and evolutionary dynamics
- **Systemic Trade-offs:** Large + fast = high energy cost (prevents "god-tier" creatures)
- **Player Engagement:** Creatures feel alive, breeding matters, conservation has meaning

### Emergence, Not Direct Encoding

**DON'T encode complex behaviors:**
- "Sociality" gene → Should emerge from: personal_space + flocking + aggression
- "Intelligence" gene → Should emerge from: perception_range + reaction_speed
- "Dominance" gene → Should emerge from: aggression + size + energy_level

**DO encode primitive traits:**
- Physical parameters: size, speed, perception distance
- Simple thresholds: hunger level, flee threshold, personal space
- Binary flags: flocking yes/no, diurnal/nocturnal

### The Rule

**DON'T:** Hardcode creature traits
- Using magic numbers or global constants
- Setting all creatures to perceive the same distance, avoid obstacles at fixed thresholds
- Makes all creatures identical, eliminates evolution

**DO:** Derive from DNA
- Read trait values from each creature's individual DNA
- Every creature has unique perception range, obstacle avoidance distance, aggression level
- Enables genetic diversity, evolution, and player breeding programs

### Systemic Trade-offs

**Every advantage must have a cost.** Trade-offs are built into physics/biology, not arbitrary balance numbers:

**Examples:**
- Large size = higher speed BUT massive energy consumption (starves faster)
- High speed = escape predators BUT energy burns rapidly during movement
- Long perception = detect threats early BUT cognitive overload in cluttered terrain
- High aggression = secure resources BUT fight injuries and energy waste

**Goal:** Create viable ecological niches, not perfect balance. Every strategy succeeds somewhere, fails elsewhere.

### Workflow for New Traits

1. **Consult zoologist-tom FIRST**
   - Use Task tool with `subagent_type: zoologist-tom`
   - Ask: "What's a realistic range for [trait]?"
   - Ask: "How should [trait] scale with other attributes?"
   - Get biological formulas and rationale

2. **Add gene to DNA system**
   - Set min/max bounds based on zoologist input
   - Document trade-offs (e.g., larger vision costs more energy)

3. **Log decision in docs/biology/biology-notes.md**
   - Format: `Date | Feature | Zoologist Input | Implementation`
   - Creates permanent record for future reference

4. **Implement trait expression**
   - DNA gene → phenotype → behavior
   - Avoid hardcoded constants (use DNA directly)

### Hook Enforcement

The `dna-consultation-check.sh` hook provides guidance when you edit creature code:
- **Triggers on:** Creature components, spawning logic, behavior systems, DNA docs
- **Mode:** Warning + guidance (non-blocking)
- **Reminds:** DNA principle, zoologist consultation, biology-notes.md logging
- **Flags:** Existing hardcoded traits for future migration

### Documentation

- **Full design doc:** `/workspace/docs/biology/dna-driven-design.md`
- **Biology notes:** `/workspace/docs/biology/biology-notes.md` (zoologist consultations log)
- **Zoologist agent:** `.claude/agents/zoologist-tom.md`

### Current Status

**DNA system:** Planned for future sprint (size genes first)

**Technical debt:** Existing traits (`max_speed`, `energy`, `age`) flagged for migration to DNA in future sprints

**Vision:** Fully DNA-driven ecosystem where evolution is visible and emergent gameplay arises from genetic diversity

### Remember

**The DNA is the creature. Everything else is just expression.**

## Code Documentation Standards - MANDATORY

**CRITICAL: Code comments are a code smell. If you need a comment to explain what code does, refactor the code instead.**

### Philosophy: Code First, Comments Never

Comments lie. They go out of sync with code, creating confusion and technical debt. Our source of truth is:
1. **The code itself** (self-documenting through clear names and structure)
2. **Type signatures** (TypeScript/Rust types document contracts)
3. **Tests** (executable documentation of behavior)
4. **docs/** (high-level architecture and rationale)

### Strict Policy - What is BANNED

**NEVER write:**
- ❌ Doc comments (JSDoc `/***/`, Rustdoc `///` or `//!`)
- ❌ Explanatory comments (why code does something)
- ❌ Algorithm descriptions (belongs in docs/)
- ❌ Parameter documentation (`@param` - types already document this)
- ❌ Examples in comments (write tests instead)
- ❌ Historical notes ("old value was X" - check git history)
- ❌ Formula derivations (belongs in `/docs/biology/constants-rationale.md`)

### What is ALLOWED

**ONLY these comment types are permitted:**
- ✅ **Concise constant descriptions:** Inline comments explaining high-level concept
  ```rust
  pub const COMFORT_ZONE: f32 = 20.0; // Distance a critter will wander from home
  pub const PERSONAL_SPACE: f32 = 2.0; // Distance critters prefer from friendlies
  ```
- ✅ **TODO markers:** Track technical debt with sprint references
  ```rust
  // TODO(DNA): Migrate to gene expression
  // TODO(sprint-12): Implement sexual reproduction
  ```
- ✅ **Shell script headers:** Concise functional description ONLY
  ```bash
  #!/bin/bash
  # Launch simulation with health checks and retry logic
  ```

**Even for allowed comments:**
- Keep it to ONE line maximum
- No multi-paragraph explanations
- No examples, formulas, or historical context
- If you need more than one line, the info belongs in docs/

### Enforcement

**Pre-commit hook** (`.claude/hooks/comment-policy-check.sh`) will:
- Block commits with multi-line comments (except TODOs)
- Flag Rustdoc (`///`, `//!`) and JSDoc (`/***/`)
- Warn on inline comments in implementation code (non-constants)

**If the hook blocks you:** The comment is too verbose. Either:
1. Refactor code to be self-documenting
2. Move rationale to `/docs/`
3. Shorten to one concise line (constants only)

### Migration of Existing Knowledge

Scientific rationale extracted from code comments is preserved in:
- **`/docs/biology/constants-rationale.md`** - Kleiber's Law, Reynolds steering, allometric formulas
- **`/docs/architecture/behavior-engine.md`** - Force accumulation, state machines
- **`/docs/biology/biology-notes.md`** - Zoologist consultation log

**Rule:** Before deleting a comment with scientific value, ensure it's documented in `/docs/`.

### Why This Matters

**Real example from our codebase:**
```rust
// BEFORE (62% of file was comments!)
/// Maximum speed for creatures (meters/second)
/// **Value:** 5.0 m/s (18 km/h - wolf trot)
/// **Formula (Kleiber's Law):** `top_speed = 5.0 × body_length^0.25`
/// [10 more lines of examples, history, validation...]
pub const MAX_SPEED: f32 = 50.0;

// AFTER
pub const MAX_SPEED: f32 = 50.0; // Maximum creature speed in m/s
```

**All rationale preserved in `/docs/biology/constants-rationale.md`** - where it belongs.

**Benefits:**
- Code is readable (not buried in comment noise)
- Documentation doesn't lie (updated with code in same commit)
- Scientific knowledge centralized in docs/ (not scattered across 86 files)
- Faster code reviews (read code, not essays)

### Remember

**If you're writing a comment, you're doing it wrong. Refactor instead.**

## Project-Specific Commands

### Testing
```bash
# Frontend (Portal) tests
cd apps/portal
npm test           # Run full test suite
npm run test:watch # Run tests in watch mode

# Backend (Simulation) tests
cd apps/simulation
cargo test         # Run Rust tests
cargo test -- --nocapture  # Run with output
```

### Development
```bash
# Electron desktop app (Phase 1)
cd apps/portal
npm run dev        # Start Electron with simulation subprocess

# Build for distribution
npm run build      # Build frontend
npm run package    # Package with electron-builder (.exe, .dmg, .AppImage)
```

## Code Quality Standards

### Console Logging
- **NEVER** use `console.log()` for debug/verbose output
- **ONLY** use `console.error()` for actual errors
- Remove ALL console.logs during cleanup (except errors)

### TypeScript
- Avoid `any` types - use proper interfaces/types
- Update tests when changing implementation (MIN_ZOOM, MAX_ZOOM, etc.)
- Keep tests synchronized with actual code behavior

### Architecture
- Domain layer: Pure TypeScript (Camera, Viewport)
- Rendering layer: PixiJS integration (GridRenderer, SpriteProvider)
- Infrastructure: External services (WebSocketClient, SpritePool)

## Current Sprint: Sprint 9 - Trials Regression Testing (IN PROGRESS 🚀)

### Sprint Focus
Implement continuous regression testing system ("Trials") for recording and replaying scenarios to test future changes.

### Goals
- **Trial Infrastructure:** Core system for defining/running scenarios with deterministic RNG
- **Spawning Pattern Trial:** Capture current default spawn as baseline
- **Crowd Navigation Trial:** Creature weaving through obstacle grid
- **Documentation:** Trial authoring guide and integration instructions

### Success Criteria
- Both trials triggerable via command
- Deterministic, reproducible results
- All tests passing (100% pass rate)

**See:** `SPRINT_DOCS/SPRINT_PLAN_sprint-9-trials-regression-testing.md`

---

## Previous Sprints

### Sprint 8 - Code Quality & Architecture Foundation (COMPLETE ✅)

**Focus:** Refactor, understand architecture, establish documentation baseline

**Key Outcomes:**
- Type safety cleanup, constant extraction
- behavior-engine.md architecture documentation
- Technical debt inventory (52 items catalogued)
- Performance baseline in stats pane

### Sprint 7 - Electron Standalone Desktop (COMPLETE ✅)

**Focus:** Phase 1 (standalone desktop game) prioritized over Phase 2 (MMO). Established Electron architecture and stdio IPC protocol.

**Completed Goals:**
- ✅ Electron desktop app with stdio IPC
- ✅ MessagePack frame protocol (60 Hz streaming)
- ✅ Rust simulation subprocess
- ✅ Desktop packaging with electron-builder

**Key Technologies:**
- **Electron:** Desktop application framework
- **Rust/Bevy:** Backend simulation subprocess
- **TypeScript + PixiJS:** Frontend rendering
- **IPC:** stdio MessagePack frames

### Phase 1 vs Phase 2

**Phase 1 (Current):** Standalone desktop game
- Electron desktop application
- Local simulation subprocess
- Single-player experience
- Steam distribution

**Phase 2 (Future):** MMO multiplayer
- Microservices architecture
- WebSocket streaming
- Persistent cloud world
- Player economy & trading

## Electron IPC Architecture

**See:** `docs/architecture/electron-architecture.md` for complete IPC patterns, MessagePack protocol, and implementation details.

## Remember

**Run tests. Always. Every time. Before and after changes.**

The hook system will enforce this, but you should internalize it.
