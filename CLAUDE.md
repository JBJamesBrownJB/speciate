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

## Current Sprint: Sprint 8 - Code Quality & Architecture Foundation (COMPLETE ✅)

### Sprint Focus
Refactor, understand code and architecture, small bug fixes. Clean understandable code and strategy for behavior engine. Stats pane cleanup and new baseline stats established.

### Completed Goals
- ✅ **Phase 1:** Type safety cleanup (removed 5 TypeScript `any`, fixed 10 Rust warnings)
- ✅ **Phase 2:** Constant extraction (created TERRITORY & SEEKING structs, 6 validation tests)
- ✅ **Phase 3:** behavior-engine.md architecture documentation (17-page comprehensive guide)
- ✅ **Phase 4:** Performance baseline section in stats pane (Target FPS, Frame Budget, Tick Rate)
- ✅ **Phase 5:** Technical debt inventory (catalogued 52 items, categorized by priority)

### Key Outcomes
- **Code Quality:** Removed TypeScript `any` types, fixed clippy warnings, cleaner codebase
- **Constants Refactor:** Extracted 13 magic numbers to named constants (TERRITORY, SEEKING)
- **Documentation:** behavior-engine.md explains force accumulation, state machines, DNA roadmap
- **Technical Debt:** Complete inventory with migration plans (46 DNA items, 5 behavior items)
- **Performance Metrics:** Baseline targets visible in HUD (60 FPS, 16.67ms budget, 20 Hz tick)

### Active Areas
- `/workspace/docs/architecture/` - Architecture documentation
- `/workspace/docs/technical-debt.md` - Technical debt tracking
- `/workspace/apps/simulation/src/simulation/movement/constants.rs` - Centralized constants
- `/workspace/apps/portal/index.html` - Performance baseline UI

---

## Previous Sprints

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

## Electron IPC Patterns

### Core Principle

Electron IPC uses **stdio MessagePack streaming** between the Rust simulation subprocess and the Electron main process. The simulation writes length-prefixed binary frames to stdout at 60 Hz, which the main process reads, deserializes, and forwards to the renderer via `webContents.send()`.

**Communication is currently unidirectional:** Backend → Frontend only. Frontend cannot send commands to the simulation yet (planned for future sprint).

### Architecture Overview

```
┌─────────────────────────────────────────────────────────────┐
│  RUST SUBPROCESS               ELECTRON MAIN    RENDERER    │
│  (apps/simulation)             (electron/main)  (src/)      │
├────────────────────────────────────────────────────────────┤
│                                                              │
│  stdout.write()  ─────────→  child.stdout  ──────────→      │
│  (MessagePack)               .on('data')   webContents      │
│  60 Hz                       deserialize   .send()          │
│                              GameState     'state-update'   │
│                                                │             │
│                                                ▼             │
│                                         window.electron      │
│                                         .onStateUpdate()     │
│                                                              │
└─────────────────────────────────────────────────────────────┘
```

### MessagePack Frame Protocol

**Wire Format:**
```
┌────────────────┬──────────────────────┐
│ Length (4B)    │ Payload (N bytes)    │
│ Big Endian u32 │ MessagePack binary   │
└────────────────┴──────────────────────┘
```

**Rust (Simulation - Writer):**
```rust
use rmp_serde;
use std::io::{self, Write};

/// Write a single MessagePack frame to stdout (60 Hz)
fn write_state_frame(state: &GameState) -> io::Result<()> {
    // Serialize state to MessagePack
    let payload = rmp_serde::to_vec(state)
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;

    // Write 4-byte big-endian length prefix
    let len = payload.len() as u32;
    io::stdout().write_all(&len.to_be_bytes())?;

    // Write MessagePack payload
    io::stdout().write_all(&payload)?;
    io::stdout().flush()?;

    Ok(())
}

// Called from Bevy FixedUpdate system at 60 Hz
fn snapshot_system(world: &World) {
    let state = create_game_state_snapshot(world);
    if let Err(e) = write_state_frame(&state) {
        eprintln!("[Simulation] Failed to write state: {}", e);
    }
}
```

**JavaScript (Electron Main - Reader):**
```javascript
const { spawn } = require('child_process');
const msgpack = require('@msgpack/msgpack');

// Spawn Rust simulation subprocess
const simulation = spawn('./apps/simulation/target/release/speciate', [], {
  stdio: ['ignore', 'pipe', 'pipe'],
});

let buffer = Buffer.alloc(0);

// Read stdout frames
simulation.stdout.on('data', (chunk) => {
  buffer = Buffer.concat([buffer, chunk]);

  // Process all complete frames in buffer
  while (buffer.length >= 4) {
    // Read 4-byte length prefix (big-endian u32)
    const frameLength = buffer.readUInt32BE(0);
    const totalLength = 4 + frameLength;

    // Wait for complete frame
    if (buffer.length < totalLength) break;

    // Extract and decode MessagePack payload
    const payload = buffer.slice(4, totalLength);
    const state = msgpack.decode(payload);

    // Send to renderer
    mainWindow.webContents.send('state-update', state);

    // Remove processed frame from buffer
    buffer = buffer.slice(totalLength);
  }
});
```

**TypeScript (Renderer - Receiver):**
```typescript
// apps/portal/electron/preload.cjs exposes API via contextBridge
const { contextBridge, ipcRenderer } = require('electron');

contextBridge.exposeInMainWorld('electron', {
  onStateUpdate: (callback: (state: GameState) => void) => {
    ipcRenderer.on('state-update', (_event, state) => callback(state));
  },
  removeStateUpdateListener: () => {
    ipcRenderer.removeAllListeners('state-update');
  },
});

// apps/portal/src/infrastructure/ipc/ElectronIPCClient.ts
export class ElectronIPCClient implements IPCClient {
  constructor() {
    window.electron?.onStateUpdate((state: GameState) => {
      // Update sprite positions, camera, etc.
      this.handleStateUpdate(state);
    });
  }

  private handleStateUpdate(state: GameState): void {
    // 60 Hz state updates drive rendering loop
    this.latestState = state;
    this.callbacks.forEach(cb => cb(state));
  }
}
```

### Performance Characteristics

**Frame Rate:** 60 Hz state streaming (16.67ms per frame)
**Frame Size:** ~4-12 KB per frame (200 creatures × 20 bytes/creature)
**Latency:** 16-33ms (1-2 frames) main → renderer propagation
**Throughput:** ~240-720 KB/s (well within stdout buffer capacity)

### Error Handling

**Rust Side:**
- **Serialization failure:** Log error, skip frame (frontend keeps rendering last good state)
- **Broken pipe:** Electron crashed, exit simulation gracefully
- **Partial write:** Flush after every frame to prevent buffering issues

**Electron Side:**
- **Malformed frame:** Log error, discard partial buffer, wait for next frame
- **Subprocess crash:** Detect via `child.on('exit')`, show error dialog, close app
- **Deserialization failure:** Skip frame, log error (likely MessagePack schema mismatch)

### Type Safety

**Keep TypeScript and Rust types synchronized:**

```rust
// apps/simulation/src/ipc/game_state.rs
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GameState {
    pub tick: u64,
    pub creatures: Vec<CreatureSnapshot>,
    pub timestamp_ms: u64,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CreatureSnapshot {
    pub id: u32,
    pub x: f32,
    pub y: f32,
    pub heading: f32,
    pub body_radius: f32,
    pub energy: f32,
}
```

```typescript
// apps/portal/src/types/GameState.ts
export interface GameState {
  tick: number;
  creatures: CreatureSnapshot[];
  timestamp_ms: number;
}

export interface CreatureSnapshot {
  id: number;
  x: number;
  y: number;
  heading: number;
  body_radius: number;
  energy: number;
}
```

**Validation:** Use `zod` or JSON Schema to validate frames in Electron main before forwarding to renderer (defense in depth).

### Best Practices

1. **Never block stdout** - Simulation writes at 60 Hz; blocking causes frame drops
2. **Keep payloads small** - Serialize only what renderer needs (<20 KB/frame target)
3. **Version your schema** - Add `schema_version` field to GameState for future migrations
4. **Use map format** - Configure `rmp_serde` with `.with_struct_map()` for field-name encoding
5. **Test desync** - Verify frontend handles dropped frames gracefully (stale state rendering)

### Future: Bidirectional IPC

**Planned for Sprint 8:** Frontend → Backend commands for player interactions.

**Approach:** stdin commands using same MessagePack framing:
- Frontend sends commands via IPC: `window.electron.sendCommand('spawn_creature', {x, y})`
- Electron main writes to `simulation.stdin`
- Rust reads stdin in separate thread, queues commands for next tick
- Commands execute in Bevy system, results flow back via stdout

**See:** `docs/architecture/electron-architecture.md` for full design

## Remember

**Run tests. Always. Every time. Before and after changes.**

The hook system will enforce this, but you should internalize it.
