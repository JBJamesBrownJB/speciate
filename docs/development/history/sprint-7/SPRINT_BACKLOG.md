# Sprint Backlog

This file tracks active and completed sprints for the Speciate project.

---

## Sprint 7: Tauri Standalone (ACTIVE)

**Branch:** `feat/sprint-7-tauri-standalone`
**Started:** 2025-11-10
**Goal:** Migrate to Tauri for standalone single-player desktop application

**Status:** 🟢 In Progress

**Key Tasks:**
- [ ] Tauri setup and skeleton project
- [ ] Frontend integration (PixiJS in Tauri)
- [ ] Backend integration (Bevy simulation with IPC)
- [ ] Admin portal integration
- [ ] Code removal (NATS, Broadcaster, Ledger)
- [ ] Testing and validation

**Success Criteria:**
- Single `npm run tauri dev` launches working app
- Admin portal functional (spawn, clear, speed)
- 1000 creatures @ 60+ FPS
- 196+ tests passing
- Clean code, no network infrastructure

**See:** [SPRINT_PLAN_sprint-7-tauri-standalone.md](./SPRINT_PLAN_sprint-7-tauri-standalone.md)

---

## Previous Sprints

### Sprint 6: Learning to Walk ✅ Complete (Nov 6-9, 2025)

**Achievements:**
- Seeking behavior with Reynolds steering
- Territory-based wandering with elastic tether
- Locomotion noise (Perlin-based organic wobble)
- Body radius volumetric physics
- NATS WebSocket support (port 9224)
- Admin portal with live spawning
- Single-gate spawning architecture
- 133 passing tests

**Branch:** Merged to `main`

---

### Sprint 5: Performance Instrumentation ✅ Complete

**Achievements:**
- Streaming architecture research
- NATS integration
- Performance profiling tools
- Documentation updates

---

## Sprint Template

```markdown
## Sprint X: [Name] ([Status])

**Branch:** `feat/sprint-x-name`
**Started:** YYYY-MM-DD
**Goal:** [Single sentence goal]

**Status:** 🟢 In Progress | 🟡 Blocked | 🔴 Failed | ✅ Complete

**Key Tasks:**
- [ ] Task 1
- [ ] Task 2

**Success Criteria:**
- Criterion 1
- Criterion 2

**See:** [SPRINT_PLAN_sprint-x-name.md](./SPRINT_PLAN_sprint-x-name.md)
```
