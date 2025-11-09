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

**DNA system:** Planned for Sprint 6 Phase 3 (size genes first)

**Technical debt:** Existing traits (`max_speed`, `energy`, `age`) flagged for migration to DNA in future sprints

**Vision:** Fully DNA-driven ecosystem where evolution is visible and emergent gameplay arises from genetic diversity

### Remember

**The DNA is the creature. Everything else is just expression.**

## Project-Specific Commands

### Testing
```bash
npm test           # Run full test suite (196 tests)
npm run test:watch # Run tests in watch mode
```

### Development
```bash
npm run dev        # Start Vite dev server (Portal)
npm run build      # Production build
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

## Current Sprint: Sprint 6 - Learning to Walk

### Recent Changes
- Fixed 1m×1m grid with viewport culling
- Grid visible only at zoom >= 20 px/m
- Removed 60 FPS console spam
- World size: 2000km × 2000km
- Camera zoom range: 0.0005 - 200 px/m

### Active Files
- `/workspace/apps/portal/` - Frontend portal application
- Tests must pass: 196/196 ✓

## Remember

**Run tests. Always. Every time. Before and after changes.**

The hook system will enforce this, but you should internalize it.
