# Getting Started with Speciate

Welcome to **Speciate**, an artificial life simulation where evolution emerges from DNA-driven creature behavior. This guide will get you oriented and productive quickly.

---

## Project Overview

**Speciate** is a standalone desktop A-Life simulation being developed for Steam.

**Goal:** Single-player evolutionary sandbox where players breed creatures, watch ecosystems emerge, and discover new species.

**Tech Stack:**
- **Tauri:** Desktop application framework (Rust + TypeScript)
- **Rust (Bevy ECS):** Simulation backend (physics, AI, genetics)
- **TypeScript + PixiJS:** Frontend rendering (60 FPS visuals)
- **IPC:** Lock-free snapshot queue bridges Rust ↔ TypeScript

**Features:**
- Local simulation (no network, no server)
- DNA-driven evolution (every trait encoded genetically)
- Emergent behavior from simple rules
- Steam achievements, cloud saves, Workshop support

**Current Branch:** `feat/sprint-7-tauri-standalone`

---

## Quick Start

### Prerequisites

**Required:**
- **Rust** 1.70+ (`rustc --version`)
- **Node.js** 18+ (`node --version`)
- **Tauri CLI** (`cargo install tauri-cli`)
- **Git**

**Optional:**
- **VS Code** with Rust Analyzer extension

### Clone and Setup

```bash
# Clone repository
git clone https://github.com/your-org/speciate.git
cd speciate

# Install frontend dependencies
cd apps/portal
npm install
cd ../..

# Verify Tauri environment
cargo tauri --version
```

### Run the Game

```bash
# Launch Tauri dev mode (simulation + frontend)
cargo tauri dev

# Alternative: Run components separately
# Terminal 1: Rust simulation
cd apps/simulation
cargo run

# Terminal 2: Frontend dev server
cd apps/portal
npm run dev
```

**First launch:** Takes 2-5 minutes (Rust compilation). Subsequent launches are faster.

### Run Tests

```bash
# Frontend tests (TypeScript)
cd apps/portal
npm test

# Backend tests (Rust)
cd apps/simulation
cargo test

# Or use slash command: /test-all
```

---

## Project Structure

```
speciate/
├── apps/
│   ├── portal/              # Frontend (TypeScript + PixiJS)
│   │   ├── src/
│   │   │   ├── domain/     # Core logic (Camera, Viewport)
│   │   │   ├── rendering/  # PixiJS rendering (GridRenderer)
│   │   │   └── core/       # Utilities, constants
│   │   └── ARCHITECTURE.md # Frontend architecture guide
│   │
│   └── simulation/          # Backend (Rust + Bevy ECS)
│       ├── src/
│       │   ├── simulation/ # ECS systems (movement, AI, genetics)
│       │   ├── spawner.rs  # Creature spawning
│       │   └── components.rs # ECS components
│       └── CLAUDE.md        # ECS patterns, Tauri integration
│
├── src-tauri/               # Tauri IPC bridge
│   ├── src/
│   │   ├── main.rs         # Tauri entry point
│   │   └── commands/       # IPC command handlers
│   └── tauri.conf.json     # Tauri configuration
│
├── docs/
│   ├── architecture/       # System design docs
│   ├── biology/            # DNA, genetics, zoology
│   └── GETTING_STARTED.md  # This file
│
├── .claude/
│   ├── agents/             # Specialist AI agents
│   ├── hooks/              # Pre/post tool use hooks
│   └── commands/           # Slash commands (/test-all, /tauri-check)
│
└── CLAUDE.md               # Project-wide instructions (TDD, DNA-driven)
```

---

## Core Concepts

### 1. DNA-Driven Design (Mandatory)

**Rule:** All creature traits MUST be encoded in DNA, not hardcoded.

**Why:**
- Enables genetic variation (every creature is unique)
- Allows sexual reproduction (parent DNA → offspring)
- Creates emergent behavior (fast predators vs. slow herbivores)
- Makes player breeding meaningful

**Pattern:**
```rust
// ❌ BAD: Hardcoded trait
if distance < 5.0 { avoid(); }

// ✅ GOOD: DNA-driven trait
if distance < creature.dna.personal_space { avoid(); }
```

**Workflow:**
1. Consult **zoologist-tom** agent for trait bounds
2. Add gene to DNA system with min/max range
3. Log decision in `/docs/biology/biology-notes.md`

**See:** `/workspace/CLAUDE.md` (DNA-Driven Design section)

---

### 2. Tauri IPC Bridge

**Architecture:**

```
Frontend (60 FPS) <--IPC--> Tauri <---> Simulation (20/90 Hz)
  TypeScript                Rust        Bevy ECS
```

**Frontend → Backend (Commands):**
```typescript
import { invoke } from '@tauri-apps/api/tauri';

// Query world state
const snapshot = await invoke<WorldSnapshot>('get_world_snapshot');

// Spawn creature
await invoke('spawn_creature', { x: 100, y: 200 });
```

**Backend → Frontend (Events):**
```rust
app_handle.emit_all("creature_died", CreatureDeathEvent {
    id: entity_id,
    cause: "starvation"
}).unwrap();
```

**See:** `/workspace/CLAUDE.md` (Tauri IPC Patterns section)

---

### 3. Dual-Tick Architecture

The simulation runs on **two tick rates**:

| System | Rate | Purpose |
|--------|------|---------|
| **FixedUpdate** | 20 Hz | AI decisions, reproduction, energy |
| **Update** | 90 Hz | Physics, movement, collision |

**Why:** AI is expensive. Running it at 20 Hz instead of 60 Hz saves 66% CPU.

**See:** `/workspace/apps/simulation/CLAUDE.md` (Dual-Tick Architecture section)

---

### 4. Test-Driven Development (Mandatory)

**Workflow:**
1. Run tests BEFORE making changes (`npm test` or `cargo test`)
2. Write test for new feature FIRST
3. Implement feature
4. Run tests IMMEDIATELY after change
5. NEVER batch changes without testing

**Enforcement:** Pre-tool use hook blocks edits if tests fail (bypass with `SKIP_TEST_HOOK=1` during refactoring).

**See:** `/workspace/CLAUDE.md` (TDD section)

---

## Key Documents

### For New Developers

**Start here:**
1. `/workspace/CLAUDE.md` - Project principles (TDD, DNA-driven, Tauri patterns)
2. `/workspace/docs/GETTING_STARTED.md` - This file
3. `/workspace/apps/simulation/CLAUDE.md` - ECS patterns, Tauri integration
4. `/workspace/apps/portal/ARCHITECTURE.md` - Frontend architecture

### For Specific Tasks

**Creature/DNA work:**
- `/workspace/docs/biology/dna-driven-design.md` - DNA architecture
- `/workspace/docs/biology/biology-notes.md` - Zoologist consultations log
- `.claude/agents/zoologist-tom.md` - Biology expert agent

**Tauri/IPC work:**
- `/workspace/CLAUDE.md` (Tauri IPC Patterns section)
- `/workspace/apps/simulation/CLAUDE.md` (Tauri Integration section)
- `.claude/agents/tauri-tina.md` - Tauri specialist agent

**Game design:**
- `.claude/agents/gamification-garry.md` - Game balance consultant
- `.claude/agents/narrative-nancy.md` - Story/quest designer

**Steam integration:**
- `.claude/agents/steam-steve.md` - Steam distribution specialist

---

## Slash Commands

Claude Code provides custom slash commands for common workflows:

| Command | Purpose |
|---------|---------|
| `/test-all` | Run all tests (Rust + TypeScript) |
| `/tauri-check` | Verify Tauri environment |
| `/run-simulation` | Launch Tauri dev mode |
| `/sprint-status` | Show current sprint progress |
| `/dna-consult` | Consult zoologist-tom and auto-log |
| `/start-sprint` | Initialize new sprint workflow |
| `/end-sprint` | Close sprint with QA verification |

**Example:**
```
User: /tauri-check
Claude: [Verifies Rust, Node, Tauri CLI, runs test build]
```

---

## Specialist Agents

Claude Code uses specialized AI agents for domain expertise:

| Agent | Use For |
|-------|---------|
| **backend-simulation-sam** | Rust ECS systems, physics, AI logic |
| **frontend-fanny** | TypeScript, PixiJS rendering, UI/UX |
| **tauri-tina** | Tauri IPC, desktop builds, Rust ↔ TS bridge |
| **zoologist-tom** | Biology, genetics, trait bounds |
| **botanist-betsy** | Plant systems (future) |
| **environment-eddy** | World generation, biomes |
| **architect-andy** | Technical blueprints, system design |
| **gamification-garry** | Game balance, player motivation |
| **narrative-nancy** | Story, quests, campaign design |
| **steam-steve** | Steam integration, achievements |
| **qa-karen** | Code review, pre-merge checks |
| **pm-pam** | Sprint planning, task management |
| **playtest-petra** | E2E gameplay testing

---

## Common Workflows

### Add a New Creature Trait

1. Consult **zoologist-tom**:
   - "What's a realistic range for [trait]?"
   - "How should [trait] scale with size?"
2. Add gene to DNA system (min/max bounds)
3. Update trait expression in ECS components
4. Log decision in `/docs/biology/biology-notes.md`
5. Write tests for new behavior
6. Run `/test-all` to verify

**Helper:** Use `/dna-consult <question>` to auto-consult and log.

---

### Add a New Tauri IPC Command

1. Define Rust command in `src-tauri/src/commands/`:
   ```rust
   #[tauri::command]
   async fn my_command(state: State<'_, GameState>) -> Result<Data, String> {
       // Query ECS world, return data
   }
   ```
2. Register in `src-tauri/src/main.rs`:
   ```rust
   .invoke_handler(tauri::generate_handler![my_command])
   ```
3. Call from frontend:
   ```typescript
   const result = await invoke<Data>('my_command');
   ```
4. Document in `/workspace/CLAUDE.md` (Tauri IPC Patterns)

---

### Debug Simulation Performance

1. Run `/run-simulation` to launch desktop app
2. Check FPS in console (should be 60 FPS)
3. If low FPS:
   - Check creature count (100+ is expensive)
   - Profile ECS systems (`cargo flamegraph`)
   - Verify dual-tick (FixedUpdate = 20 Hz, Update = 90 Hz)
4. Consult **tauri-tina** for optimization guidance

---

## Troubleshooting

### "Tauri CLI not found"
```bash
cargo install tauri-cli
cargo tauri --version  # Verify installation
```

### "Tests failing"
```bash
# Run tests to see failures
/test-all

# Fix tests, then verify
npm test           # Frontend
cargo test         # Backend
```

### "Port already in use"
- Check if another dev server is running
- Kill process: `lsof -ti:3000 | xargs kill` (Mac/Linux)

### "Build fails"
```bash
# Verify environment
/tauri-check

# Clean and rebuild
cargo clean
cargo tauri build --debug
```

---

## Contributing

### Before Making Changes

1. Run `/sprint-status` to see current goals
2. Ensure tests pass: `/test-all`
3. Read relevant docs (see Key Documents above)

### During Development

1. **Follow TDD:** Write tests first, run tests often
2. **Consult agents:** Use specialist agents for domain expertise
3. **Log DNA decisions:** Use `/dna-consult` for trait changes
4. **Check docs:** Hooks will remind you to update documentation

### Before Committing

1. Run `/test-all` - all tests must pass
2. Remove `console.log()` (hook will warn)
3. Follow commit convention: `feat:`, `fix:`, `docs:`, etc.
4. Pre-commit hook will check for secrets

### Sprint Workflow

1. **Start:** `/start-sprint` - Creates branch, initializes docs
2. **Work:** Implement features, run tests, commit often
3. **End:** `/end-sprint` - QA verification, generates summary

---

## Next Steps

**New developers:**
1. Read `/workspace/CLAUDE.md` (project principles)
2. Run `/tauri-check` to verify environment
3. Run `/run-simulation` to see the game
4. Read `/workspace/apps/simulation/CLAUDE.md` (ECS patterns)
5. Browse `.claude/agents/` to see available experts

**Ready to code:**
- Check `/sprint-status` for current tasks
- Consult relevant agents for guidance
- Follow TDD workflow (test → code → test)
- Use `/dna-consult` for creature trait changes

**Need help:**
- Ask **pm-pam** for task clarification
- Ask **architect-andy** for design questions
- Ask domain agents (zoologist, gamification, etc.) for expertise

---

## Resources

**Internal:**
- Project instructions: `/workspace/CLAUDE.md`
- ECS guide: `/workspace/apps/simulation/CLAUDE.md`
- Frontend architecture: `/workspace/apps/portal/ARCHITECTURE.md`
- Biology design: `/workspace/docs/biology/dna-driven-design.md`

**External:**
- Bevy ECS: https://bevyengine.org/learn/book/
- Tauri: https://tauri.app/v1/guides/
- PixiJS: https://pixijs.com/guides
- Nature of Code: https://natureofcode.com/ (A-Life inspiration)

---

**Welcome to Speciate! Let's create life from code. 🧬**
