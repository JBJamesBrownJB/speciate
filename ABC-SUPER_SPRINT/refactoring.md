# Refactoring Backlog

Issues identified during Phase A cleanup that need addressing later.

---

## Naming Confusion: `checked_cells` vs `skipped_cells`

**Location:**
- `perception/debug.rs:55` - field named `checked_cells`
- `perception/systems.rs` - passes `skipped_cells` as `checked_cells` parameter
- Frontend uses both names inconsistently

**Problem:** The field is named `checked_cells` but contains `skipped_cells`. Confusing for anyone reading the code.

**Fix:** Rename to `skipped_cells` everywhere:
- `perception/debug.rs` - rename field
- `perception/systems.rs` - update parameter name
- `ipc/bridge/perception_debug_buffer.rs` - update buffer protocol
- `portal/src/types/GameState.ts` - rename TypeScript interface field
- `portal/src/rendering/overlays/SpatialGridOverlay.ts` - rename local variable
- `portal/src/infrastructure/ipc/ElectronIPCClient.ts` - update buffer parsing

**Effort:** Medium (touches 6+ files, binary protocol)

---
