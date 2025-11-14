# Sprint 7: Tauri Standalone

**Branch:** `feat/sprint-7-tauri-standalone`
**Created:** 2025-11-10
**Status:** Active

---

## 🎯 Sprint Goal

**Migrate to Tauri for standalone single-player desktop application**

Transform Speciate from a multi-service MMO architecture (NATS, Broadcaster, Ledger) to a unified Tauri desktop app bundling Rust simulation + PixiJS frontend.

---

## 📦 Key Outcomes

1. **NATS, Broadcaster, and Ledger removed** - Clean architecture with only sim + PixiJS + admin-portal
2. **Tauri wrapper functional** - Single executable runs simulation + frontend locally
3. **Admin portal integrated** - Dev UI accessible within Tauri app for testing/debugging

---

## ⚙️ Key Constraints

1. **Get Tauri working** - Focus on functional integration, not premature optimization
2. **TDD maintained** - All tests must continue passing (196 tests baseline)
3. **Clean code** - Proper architecture, no hacks, maintainable for Phase 1 development

---

## 📋 Planned Tasks

### Phase 1: Tauri Setup & Skeleton
- [ ] Install Tauri CLI and dependencies
- [ ] Create basic Tauri project structure
- [ ] Configure `tauri.conf.json` (window size, permissions, build settings)
- [ ] Verify Tauri dev mode launches successfully

### Phase 2: Frontend Integration
- [ ] Move PixiJS portal code into Tauri frontend
- [ ] Remove WebSocket connection logic
- [ ] Implement Tauri `invoke()` for IPC (placeholder initially)
- [ ] Verify frontend renders in Tauri window

### Phase 3: Backend Integration
- [ ] Integrate Bevy simulation into Tauri backend
- [ ] Implement lock-free snapshot system (crossbeam ArrayQueue)
- [ ] Create `#[tauri::command] get_game_state()` function
- [ ] Wire snapshot queue: Bevy writes, Tauri reads

### Phase 4: Admin Portal Integration
- [ ] Bundle admin-dev-ui into Tauri (as separate route or window)
- [ ] Implement dev commands via Tauri (spawn, clear, speed)
- [ ] Remove NATS dependency from admin commands

### Phase 5: Code Removal & Cleanup
- [ ] Delete `apps/broadcaster/` directory
- [ ] Delete `simulation/crates/nats_client/`
- [ ] Delete `apps/ledger/` (planned, never implemented)
- [ ] Remove NATS Docker Compose infrastructure
- [ ] Remove interpolation logic from frontend
- [ ] Update README.md with Tauri quick start

### Phase 6: Testing & Validation
- [ ] Run existing test suite (must pass 196+ tests)
- [ ] Manually test: spawn creatures, observe movement
- [ ] Verify 1000 creatures @ 60+ FPS (performance baseline)
- [ ] Test save/load functionality (if time permits)

---

## 🚫 Out of Scope (Defer to Future Sprints)

- Dual-tick refactor (20 Hz AI, 90 Hz physics) - Sprint 8
- DNA system implementation - Sprint 6 Phase 3 continuation
- Player interaction UI - Sprint 8+
- Steam integration - Sprint 10+
- Narrative campaign - Phase 1.5 (post-Early Access)

---

## 🎯 Success Criteria

- [ ] Single `npm run tauri dev` command launches working app
- [ ] Admin portal accessible and functional (spawn, clear, speed)
- [ ] Creatures spawn, move, and display correctly in PixiJS
- [ ] No NATS, Broadcaster, or network code remains
- [ ] All existing tests pass (196+ baseline)
- [ ] Code is clean, documented, and ready for Sprint 8

---

## 📊 Sprint Metrics

**Target Duration:** 5-7 days
**Baseline Tests:** 196 passing (Portal + Simulation)
**Performance Target:** 1000 creatures @ 60 FPS minimum

---

## 🔗 References

- [Tauri Architecture Documentation](../docs/architecture/tauri-architecture.md)
- [Business Strategy (Phase 1)](../docs/strategy/biz-strategy.md)
- [ALL_CHANGE.md Processing Summary](../docs/ALL_CHANGE.md)
- [MMO Architecture Archive](../docs/architecture/archived/MMO_STREAMING.md)

---

**Next Steps:** Begin with Phase 1 (Tauri Setup & Skeleton)
