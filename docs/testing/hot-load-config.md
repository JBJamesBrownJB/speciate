# Hot-Reload Configuration System

**Status:** Deferred (originally Sprint 19, moved to future sprint)
**Dependencies:** Spec-Driven Framework (Sprint 19) should complete first

---

## Goal

Decouple hardcoded simulation constants from the binary to allow runtime tuning without recompilation.

---

## Original Scope (Deferred)

### Configuration Extraction

**Location:** `assets/config/`
**Format:** TOML

**Proposed Files:**
- `physics.toml` - Drag, gravity, impulse multipliers
- `biology.toml` - Metabolism rates, vision ranges, size/mass ratios
- `world.toml` - Map bounds, grid cell sizes

### Implementation Details

**Crate:** Use `bevy_common_assets` (or `toml` + `serde`) to deserialize TOML directly into Bevy Resources (`Res<PhysicsConfig>`, `Res<BiologyConfig>`).

**Hot Reloading:** Enable asset watching so tweaking a value in `physics.toml` updates the running simulation immediately.

---

## Dev-UI Integration (Deferred)

The original plan included:
- Basic config editor in Dev-UI
- Runtime tweaking of config values through Dev-UI panels
- IPC command: `DevSetConfig` to push changes to simulation

---

## Why Deferred

Sprint 19 focuses on the **core spec-driven testing framework**:
1. Spec trial schema and parsing
2. Trial director (state machine)
3. Headless test runner
4. Dual-mode trials (automated + visual)

Hot-reload configuration adds complexity that is orthogonal to the spec framework. Constants can still be adjusted via code changes during development.

---

## Implementation Notes (For Future Sprint)

When implementing:
1. Start with `physics.toml` as proof of concept
2. Ensure determinism: hot-reload disabled in headless test mode
3. Add config versioning to prevent drift
4. Consider TOML validation schema

**References:**
- Bevy asset system: https://bevyengine.org/learn/book/assets/
- TOML format: https://toml.io/
- `bevy_common_assets` crate: https://crates.io/crates/bevy_common_assets
