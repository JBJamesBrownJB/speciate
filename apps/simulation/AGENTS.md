# apps/simulation — Rust + Bevy ECS Area Guide

The artificial-life engine: a Bevy ECS (`bevy_ecs` 0.14) data-oriented simulation compiled into an in-process Node native addon via NAPI-RS. This is the "Prove Scale" pillar — trading-grade latency engineering applied to creature simulation.

**See `/AGENTS.md` for global rules** (TDD, DNA-driven design, doc standards, Portal-vs-DevUI, binary IPC). This file holds simulation-specific rules only.

---

## Build / Test / Dev

All commands run from `apps/simulation/`. The NAPI addon is built through the napi-rs CLI (`napi build`), not bare `cargo build`. The portal's dev/build scripts call these for you.

### NAPI addon (the shippable artifact)

| Command | What it does |
|---|---|
| `npm run build` | Release addon: `napi build --platform --release --features dev-tools,napi` (runs `check-freshness` first) |
| `npm run build:debug` | Debug addon: `napi build --platform --features dev-tools,napi` |

Note: both scripts build **with `dev-tools` enabled**. There is no production-clean NAPI build script — a `default = []` production addon requires a manual `napi build` without the feature flag.

### Cargo (Rust unit / integration tests)

| Command | Notes |
|---|---|
| `cargo test` | Default features (no dev-tools) |
| `cargo test --features test-helpers` | Required for `territory_behavior_test`, `max_wander_distance_test`, `debug_wander_test`, `branch_profile` |
| `cargo test --features dev-tools` | Required for `command_system_integration`, `trial_integration`; also enables instrumentation |
| `cargo test --features dhat-heap` | Required for `memory_leak_profile` (disables MiMalloc) |
| `cargo bench` | Criterion bench `simulation_bench` (harness = false) |

### Feature-flag matrix (must all compile)

- `cargo build --no-default-features` — verifies the code compiles without instrumentation.
- `cargo build --features dev-tools` — dev build (perf-event / git2 / sysinfo; perf-event is Linux-only).

Features (`Cargo.toml`): `default = []`, `dev-tools = [perf-event, git2, sysinfo]`, `test-helpers`, `napi`, `dhat-heap`.

**Testing anti-pattern:** never assert hardcoded values derived from tunable constants — assert relationships / behavior-over-time instead. Constants change during tuning, and value-pinned tests break for non-bugs.

---

## Specification tests — `apps/simulation/specs/` (human approval required)

- `specs/` holds TOML specification tests (`behavior/`, `physics/`, `performance/`, incl. 100k/200k perf specs).
- **Changing any file in `specs/` requires explicit human approval.** Do not edit, relax, or delete a spec to make a build pass.
- Run specs (release build is mandatory for realistic timing):
  `cargo test --release --features dev-tools --test spec_runner -- --nocapture`
- Convenience wrapper (run from the **repo root**, not `apps/simulation/`): `scripts/run-specs.sh` (see `specs/README.md`).
- Perf budget: tick < 50 ms at 20 Hz; assertion `max_avg_tick_latency` ≤ 50000 µs.

---

## Architecture Patterns (verify against code; `file:line` references are current)

### Tick rate — 20.0 Hz (single source of truth)
- `src/napi_addon/simulation_engine.rs:39` → `const TARGET_SIMULATION_HZ: f32 = 20.0;` → delta_time = 0.05 s.
- Single-tick schedule. (There is no dual-tick / multi-schedule architecture — that approach was explored and abandoned; do not reintroduce it.)

### Zero-copy NAPI Float32Array IPC
- `src/napi_addon/simulation_engine.rs:422` `get_buffer() -> Float32Array`, `:457` `fill_buffer(buffer: Float32Array) -> i32` (JS-owned buffer filled in place to avoid V8 GC churn).
- `DoubleBuffer` + atomic swap (Bevy thread writes back, JS reads front); `ThreadsafeFunction` for telemetry events.
- **No JSON on the hot path.** Per-tick creature/perception data is binary only. (`rmp-serde` exists in deps but is for binary snapshot *persistence*, not IPC.)

### Force accumulation (additive steering)
- Steering systems ADD to `Acceleration` (`accel.ax += force`); the movement system integrates. Acceleration is zeroed during integration.
- Steering is fused: wander / seek / avoidance / flee collapse into a single query + single `Vec::collect` + single Rayon barrier in `src/simulation/creatures/steering/system.rs`.
- Integration: `src/simulation/movement/systems.rs:108-109` (`velocity.vx += acceleration.ax * dt`).

### Two-level spatial grid — L0 20 m / L1 60 m
- `src/simulation/spatial/constants.rs:1` `CELL_SIZE: f32 = 20.0`; `:4` `L1_CELL_SIZE = CELL_SIZE * 3.0` (= 60 m, 3×3 L0 cells).
- L0 is double-buffered (perception reads front while rebuild writes back); L1 is single-buffered, rebuilt from L0 each tick. See `src/simulation/spatial/hierarchical.rs`.
- **Gotcha:** perception's fixed 3×3 L0 scan uses a manual `L0_CELLS_RADIUS` because a Rust `const` cannot `ceil()`. If `L0_SCAN_RADIUS` / cell sizes change, this radius must be updated **by hand** — `src/simulation/perception/systems.rs:30-32`.

### Capability-marker ECS — added at spawn, never removed
- ZST markers `CanSeek` / `CanFlee` / `CanWander` / `CanAvoidObstacles` in `src/simulation/creatures/components/capabilities.rs` (each `#[derive(Component, …, Reflect)] #[reflect(Component)]`).
- Enables zero-cost archetype filtering (`Query<…, With<CanSeek>>`) with no archetype thrashing. There are **zero** `.remove::<Can*>` calls in the codebase — this rule is genuinely enforced, not aspirational.
- There is **no mortality/death system yet**; do not add despawning to hot-path systems. The planned deferred-`Dead`-marker approach is a design note in `../../docs/biology/ideas/mortality.md`, not a current rule.

### Frequency throttling — power-of-2
- `src/simulation/core/frequency_throttle.rs:5-28`: `bucket_mask = divisor - 1`, `should_process = (entity_index & mask) == current_bucket` (1 CPU cycle vs ~30 for modulo).
- **Divisor MUST be a power of 2** (2 / 4 / 8). Used by perception: `src/simulation/perception/systems.rs:110,130` (with a `bypass_throttle` escape hatch).

### Rayon movement parallelization
- `src/simulation/movement/systems.rs:15` `use rayon::prelude::*`; `:54` `let mut entities: Vec<_> = query.iter_mut().collect();`; `:60` `entities.par_iter_mut().with_min_len(256).for_each(…)`.
- Manual `Vec::collect` is required: Bevy's `par_iter_mut` does not engage Rayon in the NAPI context. Physics integration + boundary enforcement + rotation are merged into one parallel loop.

---

## Instrumentation (dev-tools)

- Every ECS system carries `#[cfg(feature = "dev-tools")] timings: Res<SystemTimings>` and calls `crate::time_system!(timings, "name")` (e.g. `src/simulation/movement/systems.rs:31-34`). Registry: `src/instrumentation/mod.rs`.
- **dev-tools ADDS, never forks behavior.** Write the core logic once (no `#[cfg]` inside it); dev-tools code only observes/augments results. Forked code paths hide bugs and create testing gaps.
- A *new* instrumented system must update **all** of: `SystemTimings` (registry), the Rust `SystemTimingsSnapshot` (serde → camelCase), **both** TS `SystemTimingsSnapshot` interfaces (`apps/dev-ui/src/types.ts` **and** `apps/portal/src/types/GameState.ts` — they are duplicated and must stay in sync), and the dev-ui `SystemTimingsPanel.tsx` sparkline. The metrics surface lives in **dev-ui**, never portal.
- **Hardware counters are Linux-only**: `perf-event` is gated to `cfg(target_os = "linux")` (`Cargo.toml:49-50`); there is no Windows/macOS equivalent API.

---

## Naming & save-state conventions

- **"Crit" vs "Creature" is intentional, not tech debt.** "Crit" = lightweight identifiers/builders (`CritId`, `CritBuilder`); "Creature" = stateful/full-API (`CreatureState`, `CreatureSnapshot`). Spawn via `CritBuilder` / `spawn_crit` (the old `spawn_creature(x, y, …)` is deprecated).
- **New `CritBundle` components must handle save-state**, via one of:
  1. Type-registry registration in `src/simulation/core/simulation.rs` (serializable scalars/small structs), or
  2. Runtime reconstruction in `from_save_state()` in `src/persistence/snapshot.rs` (fixed-size arrays, entity refs, caches).
  Either path needs a `test_save_state_*` test. Uses `bevy_reflect` / `bevy_scene`.

---

## Stale claims removed from the old guide

Stale claims — do **not** reintroduce: stdio / MessagePack IPC (replaced by zero-copy NAPI Float32Array); the 22.2 Hz / dual-tick framing (actual is 20.0 Hz single-tick); "150K–200K" as the headline target (use the honest ladder — 1M stretch / 500K Linux validated / 20K Windows experimental); Sprint N framing (replaced by `docs/ROADMAP.md` pillars; history in `sprint_summaries/`); the dead link `docs/architecture/napi-architecture.md` (real file: `docs/architecture/electron-architecture.md`).

---

## Key docs

- `docs/architecture/data-oriented-design.md` — why ECS = latency engineering
- `docs/architecture/rust-js-thesis.md` — the Rust × JS seam thesis
- `docs/architecture/electron-architecture.md` — IPC / desktop build
- `apps/simulation/specs/README.md` — spec-test runner details
- Bevy ECS book: https://bevyengine.org/learn/
