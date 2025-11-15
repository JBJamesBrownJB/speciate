# Sprint 8 Backlog

## Sprint: sprint-8-refactor-foundation

---

## Phase 1: Type Safety & Cleanup

- [ ] Fix TypeScript `any` type in `main.ts:108` (updateInspectionPanel)
- [ ] Fix TypeScript `any` type in `main.ts:508` (creatures.map)
- [ ] Fix TypeScript `any` type in `main.ts:561` (sprite click event)
- [ ] Fix TypeScript `any` type in `ElectronIPCClient.ts:8` (window.electron interface)
- [ ] Fix TypeScript `any` type in `ElectronIPCClient.ts:31` (onStateUpdate callback)
- [ ] Fix Rust unused variable warning in `seek.rs:48` (mut position)
- [ ] Fix Rust unused variable warning in `perception/systems.rs:60` (range_sq)
- [ ] Fix Rust unused variable warning in `perception/systems.rs:78` (combined_radii_sq)
- [ ] Remove dead code: `list_snapshots()` function
- [ ] Run `cargo clippy -- -D warnings` and address all suggestions
- [ ] Verify all tests pass: `npm test`
- [ ] Verify all tests pass: `cargo test`

---

## Phase 2: Constant Extraction

- [ ] Create `apps/simulation/src/simulation/creatures/constants.rs` module
- [ ] Extract constants from `wander.rs`: COMFORT_RADIUS, BLEND_CENTER, MAX_WANDER_DISTANCE, WANDER_FORCE_MAGNITUDE, SEEK_FORCE_MAGNITUDE
- [ ] Extract constants from `seek.rs`: MAX_SEEK_FORCE, BRAKE_FORCE, POUNCE_DISTANCE, POUNCE_SPEED_THRESHOLD
- [ ] Extract constants from `avoidance.rs` if any magic numbers exist
- [ ] Update `wander.rs` to use named constants
- [ ] Update `seek.rs` to use named constants
- [ ] Update `avoidance.rs` to use named constants if applicable
- [ ] Add module documentation with DNA migration notes
- [ ] Verify tests still pass after refactoring

---

## Phase 3: Architecture Documentation

- [ ] Create `docs/architecture/behavior-engine.md`
- [ ] Document force accumulation pattern
- [ ] Document capability marker pattern (zero-sized types)
- [ ] Explain hybrid ECS architecture
- [ ] Create behavior flow diagram (ASCII art or mermaid)
- [ ] Create ECS component/system interaction map
- [ ] Document creature state machine (BehaviorMode transitions)
- [ ] Document force priority hierarchy (panic > avoidance > wander)
- [ ] Explain sigmoid blending curves in wander system

---

## Phase 4: Performance Baseline Metrics

- [ ] Add "Performance Baseline" section to HUD in `index.html`
- [ ] Implement target FPS tracking (60 Hz baseline)
- [ ] Implement frame budget calculation (16.67ms target)
- [ ] Add performance status indicators (Optimal/Degraded/Critical)
- [ ] Create rolling average calculation (last 60 frames)
- [ ] Add sparkline graphs for FPS trends
- [ ] Add sparkline graphs for frame budget trends
- [ ] Extract stats calculation logic to separate module
- [ ] Add performance threshold constants
- [ ] Test with live simulation to verify metrics accuracy

---

## Phase 5: Documentation Cleanup

- [ ] Audit all 59 TODO comments in codebase
- [ ] Categorize TODOs: DNA migration, performance, features, cleanup
- [ ] Create `docs/technical-debt.md` tracking document
- [ ] Link high-priority TODOs to future sprint items
- [ ] Update `docs/architecture/electron-architecture.md`
- [ ] Document MessagePack performance characteristics
- [ ] Add bidirectional IPC section to architecture docs
- [ ] Update `CLAUDE.md` sprint status to Sprint 8
- [ ] Add "Performance Baseline Metrics" section to `CLAUDE.md`
- [ ] Document constant extraction pattern in `CLAUDE.md`

---

## Final Verification

- [ ] Run full test suite: `npm test`
- [ ] Run full test suite: `cargo test`
- [ ] Verify zero TypeScript `any` types: `grep -r "any" apps/portal/src/`
- [ ] Verify zero Rust warnings: `cargo clippy -- -D warnings`
- [ ] Verify clean compile: `cargo build`
- [ ] Manual testing: Run Electron app and verify stats pane
- [ ] Code review: Check all changes align with sprint goals
- [ ] Prepare for merge: Update commit message with sprint summary
