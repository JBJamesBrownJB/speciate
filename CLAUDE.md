# Project Instructions for Claude Code

## Quick Reference

**Key Documentation:**
- `docs/biology/done/` - **Implemented biological features** (wandering, perception, seeking, etc.)
- `docs/biology/ideas/dna-driven-design.md` - DNA-driven design principles (detailed)
- `docs/archive/dual-tick/` - ⚠️ ABANDONED architecture (Sprint 11, archived for learning)
- `docs/architecture/electron-architecture.md` - IPC protocol and Electron patterns
- `SPRINTS/` - Current and past sprint plans

**Current Sprint:** Sprint 15 - ECS Optimizations (COMPLETE)
- Branch: `feat/sprint-15-ecs-optimizations`
- Focus: Movement parallelization, perception split queries, brain serialization fix
- Achievements: 6.3x movement speedup (Rayon), 2x vision capacity, 20K creature validation
- See: `SPRINTS/SPRINT_BACKLOG.md`

**Next Sprint:** Sprint 16 - TBD (Stochastic Vision or Spatial Grid)

**Recent Completions:**
- Sprint 15: Rayon parallelization, vision refactor, query type aliases, comprehensive test coverage
- Sprint 13: Zero-copy double-buffer architecture, replacing stdio MessagePack IPC

**Tick Rate:** See `apps/simulation/src/napi_addon/simulation_engine.rs:37` (TARGET_SIMULATION_HZ)
- Replaced configurable tick rate from old stdio system
- Current rate provides capacity improvement vs 60Hz
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

### Golden Zone - ALWAYS SEEK THIS

**The Golden Zone is where a performance optimization IS the biological feature.**

When designing systems, actively look for optimizations that also create emergent biological behavior for free:

| Optimization | Biological Behavior | Golden Zone? |
|--------------|---------------------|--------------|
| Skip perception of small entities | Size domination (giants ignore mice) | ✅ YES |
| Skip stationary targets | Prey freeze = camouflage | ✅ YES |
| Satiated creatures skip prey detection | Post-meal predators rest | ✅ YES |
| FOV culling (only perceive forward) | Realistic vision cone | ✅ YES |
| Arbitrary frame skipping | Nothing biological | ❌ NO |

**Why Golden Zone matters:**
- Performance win + gameplay win = double value
- Biologically accurate behavior emerges "for free"
- Creates interesting player-observable dynamics
- Reduces complexity (one system serves two purposes)

**When designing any perception/behavior system, ask:** "Can we skip work in a way that matches real biology?"

See `docs/biology/todo/` for documented Golden Zone opportunities:
- `motion-detection.md` - Skip stationary entities (prey freeze)
- `hunger-gating.md` - Satiated predators ignore prey

### New Trait Workflow

1. Consult `zoologist-tom` agent FIRST
2. Add gene to DNA system with biological bounds
3. Document in appropriate `docs/biology/done/` or `docs/biology/ideas/` file
4. Implement trait expression (DNA → phenotype → behavior)

**Full details:** `docs/biology/ideas/dna-driven-design.md`

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

## IPC: Binary Buffers, Not JSON - MANDATORY

**CRITICAL: All high-frequency IPC between Rust simulation and TypeScript frontend MUST use binary buffers (Float32Array), NOT JSON serialization.**

### Why This Matters

| IPC Method | Serialization Cost | Example |
|------------|-------------------|---------|
| Binary (Float32Array) | ~0ms (zero-copy) | Creature positions, L1 heatmap |
| JSON (serde + JSON.parse) | 5-20ms | Kills FPS even at low bandwidth |

JSON serialization (`serde_json::to_string()` + `JSON.parse()`) is CPU-bound and causes frame drops even for small payloads. Binary buffers use direct memory access with zero serialization overhead.

### Pattern: Binary Buffer IPC

**Rust (simulation_engine.rs):**
```rust
#[napi]
pub fn fill_buffer(&self, mut buffer: Float32Array) -> i32 {
    let dest = buffer.as_mut();
    // Write directly to buffer
    dest[0] = value1;
    dest[1] = value2;
    count as i32
}
```

**Electron (napi-main.cjs):**
```javascript
const buffer = new Float32Array(MAX_SIZE);
const count = simulationEngine.fillBuffer(buffer);
mainWindow.webContents.send('buffer-update', { buffer: buffer.slice(0, count), count });
```

**TypeScript (main.ts):**
```typescript
window.electron?.onBufferUpdate?.((data) => {
    const x = data.buffer[0];
    const y = data.buffer[1];
});
```

### When JSON is Acceptable

- **Low-frequency data** (< 1 Hz): Config changes, save/load
- **Complex nested structures**: Where binary layout would be impractical
- **Debugging/dev-tools only**: Not on the hot path

### When Binary is REQUIRED

- **Per-tick data**: Creature positions, physics state, L1 cells
- **High-frequency updates**: Anything sent more than once per second
- **Large arrays**: Entity lists, spatial grid data

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

### Parallelization (Sprint 15)

Movement systems use Rayon for multi-core execution:
- **Pattern:** Collect entities → `par_iter_mut()` → write-back
- **Performance:** 6.3x speedup at 10K creatures (25.9ms → 4.1ms)
- **Scaling:** All 16 cores engaged, IPC: 4.25
- **Architecture:** Manual Vec collection required (Bevy's par_iter_mut doesn't engage Rayon in NAPI context)

**Implementation:**
```rust
// Collect entities into Vec for Rayon
let mut entities: Vec<_> = query.iter_mut().collect();

// Parallel physics integration (uses all CPU cores)
entities.par_iter_mut().for_each(|(entity, size, position, velocity, ...)| {
    // Physics logic runs in parallel
});

// Parallel boundary enforcement (reuse Vec)
entities.par_iter_mut().for_each(|(position, velocity, ...)| {
    // Boundary clamping in parallel
});
```

**Key Insights:**
- Two parallel loops reuse same Vec (efficient)
- Automatic write-back through mutable references (no explicit sync)
- Validated at 20K creatures with determinism tests

**See:** `apps/simulation/src/simulation/movement/systems.rs:35-113`, `docs/biology/done/movement-physics.md`

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
- The human in charge must approve any alterations to specification tests in apps/simulation/specs/