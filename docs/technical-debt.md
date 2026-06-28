# Technical Debt Inventory

**Last Updated:** 2025-11-15
**Total Items:** 52

This document tracks all TODO comments, architectural decisions, and technical debt across the codebase. Items are categorized by priority and sprint target.

---

## Priority Legend

- **P0 (Critical):** Blocks core gameplay or causes bugs
- **P1 (High):** Important for game balance, performance, or player experience
- **P2 (Medium):** Quality-of-life improvements, nice-to-have features
- **P3 (Low):** Future considerations, speculative improvements

---

## Category 1: DNA-Driven Design (P1) [46 items]

**Priority:** High (Phase 1)
**Effort:** 3-4 weeks
**Risk:** Medium (large refactor, but well-documented)

### Overview

The DNA system is the architectural foundation of our A-Life simulation. Currently, all creature traits (speed, perception, aggression, etc.) are hardcoded constants. These must be migrated to DNA gene expression to enable genetic diversity, evolution, and player breeding programs.

**Current State:**
- Hardcoded constants in `constants.rs`, `components.rs`
- All creatures identical (no genetic variation)
- No evolution, no breeding, no species differentiation

**Target State:**
- DNA component with gene expression (`dna.express_gene("agility")`)
- Genetic crossover during reproduction (parent DNA → offspring DNA)
- Emergent species from genetic clustering
- Player breeding programs with visible trait inheritance

### Migration Plan (Phased Approach)

**Phase 1: DNA Infrastructure**
- [ ] Create `DNA` component with gene storage (`HashMap<String, f32>`)
- [ ] Implement gene expression API (`dna.express_gene(name) -> f32`)
- [ ] Add DNA to creature spawning (random initialization)
- [ ] Write unit tests for gene bounds and validation
- **Effort:** 1 week
- **Risk:** Low (isolated system)

**Phase 2: Movement & Physics Migration**
- [ ] Migrate `MAX_SPEED` → `dna.express_gene("agility")` (range: 20-80 m/s)
- [ ] Migrate `MAX_ACCELERATION` → derived from agility × body_mass
- [ ] Migrate `MAX_TURN_RATE` → `dna.express_gene("agility") / body_length^1.33`
- [ ] Update all movement systems to read from DNA
- **Effort:** 1 week
- **Risk:** Medium (affects all creatures, needs careful testing)

**Phase 3: Behavior Constants Migration**
- [ ] Migrate SEEKING constants → DNA genes (strength, precision)
- [ ] Migrate TERRITORY constants → DNA genes (comfort_radius, attachment)
- [ ] Migrate STEERING constants → DNA genes (avoidance, caution)
- [ ] Migrate PERCEPTION constants → DNA genes (perception, spacing)
- **Effort:** 1-2 weeks
- **Risk:** Medium (affects AI behavior, needs extensive testing)

**Phase 4: Genetic Crossover & Evolution**
- [ ] Implement sexual reproduction (parent DNA → offspring DNA)
- [ ] Add mutation system (small random gene variations)
- [ ] Implement species identification (genetic similarity clustering)
- [ ] Add player breeding UI (select parents, view offspring genes)
- **Effort:** 2 weeks
- **Risk:** High (new gameplay system, balance implications)

### Affected Files & Line Counts

| File | TODO Count | Category |
|------|------------|----------|
| `movement/constants.rs` | 26 | Physics, steering, perception constants |
| `creatures/components/*.rs` | 8 | Creature state, wander params, flee params |
| `creatures/behaviors/*.rs` | 5 | Behavior system constants |
| `creatures/builder.rs` | 5 | Wander state initialization |
| `perception/components.rs` | 5 | Perception range, personal space |

### Biological Consultation Required

**Before implementing**, consult `zoologist-tom` agent for:
- Realistic gene ranges (e.g., perception: 3-20× body_length)
- Allometric scaling formulas (e.g., speed ∝ body_length^0.25)
- Trade-off systems (large + fast = high energy cost)
- Niche viability (every strategy succeeds somewhere)

**See:** `docs/biology/dna-driven-design.md` for full specification

---

## Category 2: Behavior System Completion (P1) [5 items]

**Priority:** High (Phase 2)
**Effort:** 1-2 weeks
**Risk:** Low (well-defined systems)

### Items

#### 2.1 Fleeing Behavior (Planned)
**File:** `creatures/behaviors/flee.rs`, `behaviors/transitions.rs`
**Status:** Stub implementation exists, not active
**TODO:**
- [ ] Implement flee force calculation (steer away from threat)
- [ ] Add threat detection in perception system
- [ ] Add fleeing → wandering transition (when threat > perception range)
- [ ] Test flee + avoidance interaction (should combine forces)
- **Effort:** 3-4 days
- **Rationale:** Required for predator/prey dynamics

#### 2.2 Resting Behavior (Planned)
**File:** `behaviors/transitions.rs`, `components/state.rs`
**Status:** Commented out in BehaviorMode enum
**TODO:**
- [ ] Uncomment `BehaviorMode::Resting` variant
- [ ] Implement energy recovery (0.02/tick restoration)
- [ ] Add wandering → resting transition (when energy < 50%)
- [ ] Add resting → wandering transition (when energy > 80%)
- **Effort:** 2-3 days
- **Rationale:** Required for energy management, creature survival

#### 2.3 Feeding Behavior (Planned)
**File:** `behaviors/transitions.rs`
**Status:** Planned, not implemented
**TODO:**
- [ ] Add `BehaviorMode::Feeding` variant
- [ ] Implement food detection in perception
- [ ] Add feeding → seeking transition (pursue food when detected)
- [ ] Implement biomass consumption (food → energy conversion)
- **Effort:** 1 week
- **Rationale:** Required for ecosystem dynamics, food chain

#### 2.4 Full State Machine (Planned)
**File:** `behaviors/transitions.rs:50-51`
**Status:** Simplified (Catatonic/Seeking/Wandering only)
**TODO:**
- [ ] Implement priority-based state selection (threat > hunger > rest > wander)
- [ ] Add energy consumption per behavior type
- [ ] Add random transitions for behavioral diversity
- [ ] Test state machine with all behaviors active
- **Effort:** 1 week
- **Rationale:** Required for lifelike A-Life behavior

#### 2.5 Energy Cost System (Planned)
**File:** `behaviors/transitions.rs:13-24`
**Status:** Constants defined, not actively used
**TODO:**
- [ ] Enable energy consumption in `behavior_transition_system`
- [ ] Test creature death from starvation (energy <= 0)
- [ ] Balance energy costs for gameplay (not too harsh, not too lenient)
- **Effort:** 2-3 days
- **Rationale:** Required for creature survival mechanics

---

## Category 3: Performance Optimization (P2) [0 items]

**Priority:** Medium (Phase 3)
**Effort:** Variable
**Risk:** Low (isolated optimization)

### Items

*No items currently in this category.*

**Note:** Spatial partitioning optimization has been promoted to Sprint 16. See `SPRINTS/spatial-grid/SPRINT_PLAN.md` for the implementation plan.

---

## Category 4: Architecture & Organization (P3) [1 item]

**Priority:** Low (Future cleanup)
**Effort:** 2-3 days
**Risk:** Very Low (refactor only, no logic changes)

### Items

#### 4.1 Move BodySize to Rendering Module
**File:** `components.rs:32`
**Status:** BodySize lives in creatures module, but only used for rendering
**TODO:**
- [ ] Move `BodySize` component to `rendering/` module
- [ ] Update imports across codebase
- [ ] Verify all tests still pass
- **Effort:** 2-3 days
- **Rationale:** Better separation of concerns (ECS vs rendering)

---

## Category 5: Future Enhancements (P3) [2 items]

**Sprint Target:** TBD (post-Phase 1 release)
**Effort:** Variable
**Risk:** Speculative

### Items

#### 5.1 Dynamic Home Position
**File:** `creatures/components/state.rs:86-90`
**Status:** Home position is currently fixed at spawn point
**TODO:**
- [ ] Research dynamic home position strategies:
  - Nest/burrow building mechanics
  - Territory migration based on resource availability
  - Seasonal movement patterns
- [ ] Consult zoologist-tom for biological realism
- [ ] Prototype and playtest
- **Effort:** 2-3 weeks (if pursued)
- **Rationale:** Adds depth to territory behavior, but not MVP

#### 5.2 Capability Dynamic Management
**File:** `creatures/builder.rs:19-22`
**Status:** Capabilities are permanent (added at spawn, never removed)
**TODO:**
- [ ] Consider making capabilities mutable (e.g., injury disables CanSeek)
- [ ] Design system for capability gain/loss (leveling, evolution)
- [ ] Benchmark archetype change overhead (is it worth the complexity?)
- **Effort:** 1-2 weeks (if pursued)
- **Rationale:** Interesting for progression systems, but adds complexity

---

## Category 6: Production / "Ship It" Build Is Absent / Inconsistent (P2)

**Priority:** Medium — not blocking now (no release is scheduled), but **must be sorted before any
real ship/package**. Logged 2026-06-28 (JB), deferred to ship time.

### The problem

There is **no coherent production build configuration**. The script that `package` /
`package:win` / `package:mac` / `package:linux` actually invoke does **not** match what a shipped
artifact should be. Two concrete defects in the production path
(`apps/portal` `package` → `build` → `build:rust` → `apps/simulation` `build`):

1. **Ships the slower (un-fused) corridor.** `apps/simulation` `build` is
   `--release --features dev-tools,napi` — **no `fuse-act`**. So the packaged game runs the
   *separate* act corridor even though fusion is proven faster (~3–5 ms/tick at 1M, replicated —
   see `docs/scale/FUSING/plan.md` Phase 2). The ship build leaves the headline perf win on the
   floor.
2. **Ships *with* `dev-tools`.** That contradicts the project's own rule (AGENTS.md / this doc's
   header on `constants.rs`… and `apps/simulation/Cargo.toml`: *"PRODUCTION-SAFE: dev-tools is NOT
   in default features … Production builds: cargo build --release (no dev-tools)"*). dev-tools pulls
   in `perf-event` / `git2` / `sysinfo` and the per-system timing instrumentation — overhead and
   deps that have no business in a player build.

Net: the intended ship build is **`--release --features napi,fuse-act`** (fused, no dev-tools), and
nothing currently produces it.

### Current build matrix (for reference)

| Script (consumer) | Features today | Should be |
|---|---|---|
| `apps/simulation` `build` ← portal `build:rust` ← **`package`** (prod) | `dev-tools,napi` | **`napi,fuse-act`** (fused, no dev-tools) |
| `apps/simulation` `build` ← `dev:release:unfused` (dev) | `dev-tools,napi` | unchanged — dev wants separate + instrumentation |
| `apps/simulation` `build:fused` ← `dev:release` (dev) | `dev-tools,napi,fuse-act` | unchanged — dev wants fused + instrumentation |
| `apps/simulation` `build:debug` ← `setup`/`dev:rust` | `dev-tools,napi` (debug) | unchanged |

The conflict: `apps/simulation` `build` is **shared** by the production path *and* the dev
`dev:release:unfused` path, which want opposite feature sets — so it can't simply be edited in
place.

### Intended fix (do at ship time)

- [ ] Add `apps/simulation` `build:ship` = `napi build --platform --release --features napi,fuse-act`
  (fused, **no** dev-tools).
- [ ] Repoint `apps/portal` `build:rust` → `npm run build:ship` so `package*` ship the fused,
  dev-tools-free addon. Leave `build` as-is for `dev:release:unfused`.
- [ ] **Verify before shipping:** with dev-tools off, the per-system timing macro (`time_system!`,
  `#[cfg(feature="dev-tools")]`) doesn't run, so `system_timings` telemetry is **zeros**. Confirm the
  *player-facing* portal HUD doesn't depend on those (the dev-ui, which does, is **never shipped** —
  so this is expected to be fine, but check). Also confirm `getTelemetry`'s stubbed
  hardware/parallelization fields don't break any portal runtime path.
- [ ] Confirm `cargo build --release --no-default-features --features napi,fuse-act` compiles clean
  (the production combo) and the napi `.node` loads.
- [ ] Once verified, update `docs/scale/FUSING/plan.md` (Phase 2 "Result") and remove this item.

**Effort:** ~half a day incl. verification. **Risk:** Low-Medium (changes the shipped artifact;
the dev-tools-off path is a supported compile gate but hasn't been exercised as a *packaged* run).

---

## Tracking & Metadata

### Recently Completed
**Achievements:**
- ✅ ECS optimization with Rayon parallelization (6.3x movement speedup)
- ✅ Vision system refactor (split queries, 2x capacity)
- ✅ Energy-modulated personal space (biological hunger mechanics)
- ✅ Type safety cleanup (TypeScript `any` removal, Rust warnings fixed)
- ✅ Constant extraction (TERRITORY, SEEKING structs, comprehensive tests)
- ✅ Architecture documentation (behavior-engine.md, 17 pages)
- ✅ Technical debt inventory (this document)

### Next Priority: DNA Foundation

**Priorities:**
1. **DNA Infrastructure** (P1, 1 week) - Enables all future genetic systems
2. **Movement DNA Migration** (P1, 1 week) - First concrete DNA traits (speed, agility)
3. **Behavior System Completion** (P1, 1 week) - Fleeing, resting for ecosystem dynamics

**Rationale:**
- DNA system is foundational (blocks genetic evolution, breeding, species)
- Movement migration is low-risk proof-of-concept
- Behavior completion enables richer A-Life simulation

---

## Notes

**General Principles:**
- All DNA migrations require zoologist consultation (`zoologist-tom` agent)
- Log all biological decisions in `docs/biology/biology-notes.md`
- Follow TDD: write tests FIRST, then migrate
- Run `cargo test` before and after EVERY migration

**Workflow for DNA Migration:**
1. Consult `zoologist-tom` for gene ranges and formulas
2. Add gene to DNA component with bounds
3. Update behavior system to read from DNA
4. Write unit tests for gene expression
5. Integration test with full simulation
6. Log decision in biology-notes.md

**See Also:**
- `/AGENTS.md` - DNA-driven design principles
- `docs/biology/dna-driven-design.md` - DNA architecture spec
- `docs/biology/biology-notes.md` - Zoologist consultation log
- `docs/architecture/behavior-engine.md` - Behavior system architecture

---

**End of Technical Debt Inventory**
