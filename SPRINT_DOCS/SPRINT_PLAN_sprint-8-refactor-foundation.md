# Sprint 8: Code Quality & Architecture Foundation

## Sprint Information

**Sprint Name:** sprint-8-refactor-foundation
**Branch:** feat/sprint-8-refactor-foundation
**Started:** 2025-11-15
**Status:** In Progress

---

## Sprint Goal

Establish a clean, well-documented codebase with strong type safety, clear architecture documentation, and performance baseline metrics to prepare for future DNA implementation.

---

## Key Outcomes

1. **Type Safety & Code Quality:** All TypeScript `any` types removed, Rust warnings fixed, dead code eliminated, clean compile
2. **Architecture Documentation:** Behavior engine architecture documented with diagrams, force accumulation patterns explained, ECS approach clarified
3. **Performance Baseline Metrics:** Stats pane enhanced with FPS targets, frame budget monitoring, and performance deviation tracking

---

## Key Constraints

- **No new features** (refactoring and documentation only)
- **No DNA implementation** (preparation work only - extract magic numbers to constants)
- **Must maintain all existing functionality and test coverage**

---

## Detailed Work Breakdown

### Phase 1: Type Safety & Cleanup (2-3 hours)

**TypeScript Fixes:**
- Fix 5 `any` types in `apps/portal/src/main.ts` and `ElectronIPCClient.ts`
- Create proper type interfaces for PixiJS events
- Update `global.d.ts` with window.electron types

**Rust Cleanup:**
- Fix 3 unused variable warnings in perception/seek systems
- Remove dead `list_snapshots()` function
- Run `cargo clippy` and address suggestions

**Verification:**
- Run `npm test` (verify all 136 tests pass)
- Run `cargo test` (verify clean compile)
- Run `cargo clippy -- -D warnings` (zero warnings)

### Phase 2: Constant Extraction (2-3 hours)

**Move Magic Numbers to Named Constants:**
- Extract hardcoded values from behavior systems (wander, seek, avoidance)
- Create `apps/simulation/src/simulation/creatures/constants.rs` module
- Document each constant with units, rationale, and "TODO: DNA migration" markers
- Update all references to use named constants

**Examples:**
- `COMFORT_RADIUS`, `BLEND_CENTER`, `MAX_WANDER_DISTANCE` (from wander.rs)
- `MAX_SEEK_FORCE`, `BRAKE_FORCE`, `POUNCE_DISTANCE` (from seek.rs)
- Group related constants by behavior system

### Phase 3: Architecture Documentation (3-4 hours)

**Create Architecture Diagrams:**
- Behavior engine force accumulation flow diagram
- ECS component/system interaction map
- Creature state machine (BehaviorMode transitions)

**Document Behavior Engine:**
- Create `docs/architecture/behavior-engine.md`
- Explain force accumulation pattern (why it prevents conflicts)
- Document capability marker pattern (zero-sized types)
- Explain hybrid ECS architecture

**Document Force System:**
- How forces combine (avoidance + wander + seek)
- Priority hierarchy (panic > avoidance > wander)
- Sigmoid blending curves in wander system

### Phase 4: Performance Baseline Metrics (2-3 hours)

**Stats Pane Enhancement:**
- Add "Performance Baseline" section to HUD
- Track target FPS (60 Hz) vs actual FPS
- Show frame budget utilization (16.67ms target)
- Display deviation from baseline (green/yellow/red indicators)
- Add sparkline graphs for performance trends

**Metrics to Add:**
- Target vs Actual FPS
- Frame Budget % (what % of 16.67ms is used)
- Performance Status: "Optimal" / "Degraded" / "Critical"
- Rolling average over last 60 frames

### Phase 5: Documentation Cleanup (1-2 hours)

**TODO Audit:**
- Review all 59 TODO comments in codebase
- Categorize by type: DNA migration, performance, features
- Create tracking document: `docs/technical-debt.md`
- Link TODOs to specific sprint backlog items

**Architecture Docs Update:**
- Update `docs/architecture/electron-architecture.md` with current state
- Document MessagePack frame protocol performance characteristics
- Add section on future bidirectional IPC plans

**CLAUDE.md Updates:**
- Update sprint status to Sprint 8
- Add "Performance Baseline Metrics" section
- Document constant extraction pattern

---

## Success Criteria

✅ **Code Quality:**
- Zero TypeScript `any` types
- Zero Rust compiler warnings
- Zero clippy warnings
- All tests passing (136 TypeScript + all Rust tests)

✅ **Architecture:**
- Behavior engine documented with diagrams
- Force accumulation pattern explained
- ECS architecture clarified
- New developers can understand system in <30 minutes

✅ **Performance Metrics:**
- Stats pane shows performance baseline targets
- Frame budget monitoring active
- Performance deviation visible to player
- Can identify performance degradation quickly

✅ **Preparation for DNA:**
- All magic numbers extracted to named constants
- Constants documented with units and rationale
- Clear migration path to DNA-driven parameters

---

## Estimated Duration

**Total:** 10-15 hours of focused work

**Breakdown:**
- Phase 1 (Type Safety): 2-3 hours
- Phase 2 (Constants): 2-3 hours
- Phase 3 (Architecture): 3-4 hours
- Phase 4 (Performance): 2-3 hours
- Phase 5 (Docs): 1-2 hours
