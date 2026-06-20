> 📖 **Reference** — canonical narrative for Speciate's data-oriented / ECS competitive advantage. Verified against source; every technical claim cites `path:line`. Summarized in [`README.md`](../../README.md) and woven into [`rust-js-thesis.md`](./rust-js-thesis.md).

# Data-Oriented Design: Trading-Grade Latency Engineering for Artificial Life

## The hook: where this discipline comes from

The author spent years chasing latency in trading platforms — microsecond-sensitive systems where a cache miss in the hot path is the difference between a fill and a miss. Speciate applies that same discipline to a different problem: simulating up to 500K creatures (Linux, validated) per tick, in real time.

The thesis of this document is narrow and verifiable: **the CPU-efficiency edge does not come from "Rust is fast" hand-waving. It comes from data-oriented design (DOD), and the Entity Component System (ECS) is what makes DOD the default rather than the exception.** And — this is the part that separates the claim from marketing — the engine is instrumented with real Linux hardware performance counters, so the wins are measured, not asserted.

---

## DOD vs OOP: why the layout is the algorithm

Object-oriented code models a creature as an object: a `Creature` that owns its position, velocity, perception, and behavior, scattered across the heap behind pointers. Iterating 500K of them means chasing 500K pointers, and every pointer-chase is a roll of the dice against the cache. The CPU spends most of its time waiting on memory it can't predict.

Data-oriented design inverts this. Instead of "an array of creatures," you store "an array of positions, an array of velocities, an array of …" — Struct-of-Arrays (SoA). A hot loop that integrates motion touches only the columns it needs, walks them linearly, and the hardware prefetcher sees the access pattern coming.

In Speciate this is not bespoke plumbing — it is a property of the framework. The engine runs on **Bevy ECS 0.14** ([`Cargo.toml:12`](../../apps/simulation/Cargo.toml)). Bevy stores components in **archetype tables**, where each component type is a contiguous column. Speciate's components are deliberately small, copyable POD structs — `Position{x,y}`, `Velocity{vx,vy}`, `Acceleration{ax,ay}`, `BodySize`, `Rotation` ([`core/components.rs`](../../apps/simulation/src/simulation/core/components.rs)) — so those columns pack tightly and iterate fast.

> **Honesty note.** The hand-rolled SoA in this repo is the *spatial grid* (see [Win 3](#win-3-cache-residency--l3-is-my-ram)), not the component store. Component contiguity comes from Bevy's archetype tables. The grid adds a second, purpose-built contiguous SoA buffer on top.

---

## The four concrete wins

### Win 1 — Branch prediction: uniform data → predictable branches

A modern CPU pipeline is dozens of instructions deep. A mispredicted branch flushes that pipeline. The defense is to feed hot loops *homogeneous* data so the branches they contain resolve the same way over and over.

Speciate's design keeps creatures uniform. Every creature is spawned from one bundle, `CritBundle` ([`creatures/builder.rs`](../../apps/simulation/src/simulation/creatures/builder.rs)), so they all share one archetype and one set of tables. Branchy work is hoisted out of the loop where possible — for example, perception compares **squared** distances instead of taking square roots and branching on the result (`is_in_fov`, [`perception/systems.rs:53-68`](../../apps/simulation/src/simulation/perception/systems.rs)), and the movement loop tracks `speed_sq`/`speed_computed` flags to avoid redundant `sqrt` ([`movement/systems.rs:117-156`](../../apps/simulation/src/simulation/movement/systems.rs)).

**Measured:** branch miss rate of **0.77%** at 150K population ([`150k_mixed_density_2025-12-05T18-05-00.json:314`](../performance/snapshots/150k_mixed_density_2025-12-05T18-05-00.json)) — a real Linux PMU figure, not an estimate.

### Win 2 — IPC: contiguous storage keeps the execution units fed

Instructions-per-cycle (IPC) is the bluntest measure of whether the CPU is doing work or stalling. High IPC means the SoA columns are arriving from cache fast enough to keep the execution ports busy.

**Measured, with the nuance stated plainly:**

- **Whole-tick IPC at 150K population: ~1.74** ([`150k_mixed_density_2025-12-05T18-05-00.json:310`](../performance/snapshots/150k_mixed_density_2025-12-05T18-05-00.json)) — the honest, sustained, population-scale number, aggregated across the entire tick.
- **Isolated movement-loop IPC: ~4.25** — recorded in the Sprint 15 benchmark ([`sprint_summaries/sprint-15-ecs-optimizations_summary.md`](../../sprint_summaries/sprint-15-ecs-optimizations_summary.md)). This is the *tight* number for the most cache-friendly loop, not the whole tick.

These are different measurements and this document refuses to conflate them. The 4.25 is the best-case tight loop; the ~1.7 is what the full simulation actually sustains at population. The honest figure is the impressive one — a real workload holding ~1.7 IPC at 150K entities.

IPC is computed in-engine as `instructions_delta / cycles_delta` ([`hardware_metrics.rs:518-519`](../../apps/simulation/src/instrumentation/hardware_metrics.rs)).

### Win 3 — Cache residency: "L3 is my RAM"

The whole game is the cache hierarchy. A creature's hot working set is engineered to live in CPU cache and not spill to DRAM. The author's framing: **L3 cache is my RAM** — the working set lives in cache across every stripe of the program, and main memory is the slow path you visit as little as possible.

The mechanisms that earn this:

- **Spatial grid as a true DOD structure.** Neighbor lookup uses a hand-rolled two-level grid: L0 = 20m cells, L1 = 60m cells (`CELL_SIZE = 20.0`, `L1_CELL_SIZE = CELL_SIZE * 3.0 = 60.0`, [`spatial/constants.rs`](../../apps/simulation/src/simulation/spatial/constants.rs)). L0 packs **all** entities into a *single contiguous* `Vec<PerceptionProxy>` plus a `cells: Vec<(start, count)>` slice table — the source comment is blunt: *"Zero pointer chasing during queries — all data is contiguous in memory"* ([`spatial/grid.rs:63-103`](../../apps/simulation/src/simulation/spatial/grid.rs)). Each `PerceptionProxy` is `#[repr(C)]` and exactly 32 bytes ([`spatial/grid.rs:28-49`](../../apps/simulation/src/simulation/spatial/grid.rs) for the struct; size asserted by test at `grid.rs:1047`) — so two proxies fit in one 64-byte cache line.
- **Zero-allocation rebuild.** The grid rebuilds every tick via a **parallel counting sort** with fixed bounds: parallel atomic count → prefix sum over only the non-empty cells → parallel scatter ([`spatial/grid.rs:280-414`](../../apps/simulation/src/simulation/spatial/grid.rs)). The scratch buffers (`atomic_counters`, `entity_scratch`, `prev_non_empty`) are pre-allocated and reused, and only previously-populated cells are cleared — no per-tick heap traffic, no O(total-cells) scan.
- **Precompute and cache trig / reciprocals out of the hot loop.** `Rotation` caches `cos_radians`/`sin_radians` so perception reads the facing vector directly instead of calling `atan2`/`sin`/`cos` per entity (comment: *"avoids 400K trig calls per tick"*, [`core/components.rs:212-259`](../../apps/simulation/src/simulation/core/components.rs)). `BodySize.inv_sqrt_length` is precomputed and refreshed only on `Changed<BodySize>` ([`movement/systems.rs:222-226`](../../apps/simulation/src/simulation/movement/systems.rs)), hoisting reciprocal-sqrt out of the loop.
- **Release profile tuned for predictable codegen:** `opt-level = 3`, `lto = "fat"`, `codegen-units = 1`, `panic = "abort"` ([`Cargo.toml:80-84`](../../apps/simulation/Cargo.toml)), plus `#[inline(always)]` on grid hot-path accessors.

**Measured:** at 150K population, **L1D miss rate 4.35%**, **L1I miss rate 0.27%**, LLC miss rate 24.17% ([`150k_mixed_density_2025-12-05T18-05-00.json:311-313`](../performance/snapshots/150k_mixed_density_2025-12-05T18-05-00.json)). A 4.35% L1-data miss rate at this population is the cache residency claim made concrete.

### Win 4 — Archetype-stable layout: no churn, no thrash

Cache residency only holds if the layout itself holds still. The enemy is **archetype churn**: adding or removing a component migrates an entity to a different archetype table, copying its data and invalidating the cache lines that were warm. Do that in a hot loop and the SoA advantage evaporates.

Speciate's defense is the **capability-marker pattern**. Capabilities are zero-sized types — `CanSeek`, `CanFlee`, `CanWander`, `CanAvoidObstacles` ([`creatures/components/capabilities.rs:5-19`](../../apps/simulation/src/simulation/creatures/components/capabilities.rs)). ZSTs cost zero per-entity memory; they exist only to tag the archetype so systems can filter with `With<CanSeek>`. All four are inserted **once, at spawn**, via `CritBundle`.

The mechanism that guarantees stability: the old per-capability builder toggles (`with_seeking`, `with_fleeing`, `with_wandering`, `with_avoidance`) are now `#[deprecated]` **no-ops** — every creature gets every capability ([`creatures/builder.rs:109-136`](../../apps/simulation/src/simulation/creatures/builder.rs), comment: *"All creatures now have all capabilities by default"*). And capabilities are **never removed at runtime**: a project-wide search for `remove::<Can*>` returns **zero hits in source** — it is documented as the anti-pattern to *avoid* in [`apps/simulation/AGENTS.md`](../../apps/simulation/AGENTS.md). One stable archetype, no migration, sustained cache residency.

> **Honesty note.** The "add a `Dead` marker, never remove" death pattern is documented in [`docs/biology/ideas/mortality.md`](../biology/ideas/mortality.md), but there is currently **no `Dead` component or death system in source** — `despawn` exists only for bulk teardown ([`spatial/systems.rs:118`](../../apps/simulation/src/simulation/spatial/systems.rs)). Treat archetype-stable death handling as *design intent, not yet shipped*.

### Bonus — Fearless parallelism over already-contiguous data

Because the data is already laid out as contiguous columns, parallelizing the hot loops is almost free. Movement collects its query into a `Vec` and drives `par_iter_mut().with_min_len(256)`, fusing integration, drag, noise, turn-rate limiting, boundary clamp, and rotation write-back into one parallel pass ([`movement/systems.rs:54-219`](../../apps/simulation/src/simulation/movement/systems.rs)). Perception parallelizes the same way with a smaller `min_len(64)` to load-balance its heavier, variable workload ([`perception/systems.rs:99-116`](../../apps/simulation/src/simulation/perception/systems.rs)). The grid rebuild is parallel too. Rust's compile-time data-race-freedom is what makes this *fearless* — no defensive locking in the hot path.

> The documented headline — **~6.3x movement speedup (25.9ms → 4.1ms @ 10K), all 16 cores** — comes from the Sprint 15 benchmark run ([`sprint_summaries/sprint-15-ecs-optimizations_summary.md`](../../sprint_summaries/sprint-15-ecs-optimizations_summary.md)), a point-in-time result. The *parallelism* is directly visible in source; the *magnitude* belongs to that benchmark.

### Shedding work cheaply — power-of-2 frequency throttling

Not every creature needs to think every tick. Cognitive load is shed with **bitwise bucketing**: `(entity_index & (divisor - 1)) == current_bucket` ([`core/frequency_throttle.rs:25-27`](../../apps/simulation/src/simulation/core/frequency_throttle.rs)). Divisors are clamped to powers of 2 ({2,4,8}) precisely so the modulo becomes a single-cycle AND — the in-code rationale is *"1 CPU cycle vs 30 cycles for modulo"* ([`frequency_throttle.rs:1-4`](../../apps/simulation/src/simulation/core/frequency_throttle.rs)). A test proves the bucketing is correctness-preserving: every entity is processed exactly once per divisor-length cycle.

---

## The proof: real hardware counters, not assertions

This is what makes the document credible. The engine profiles itself the way you'd profile a trading hot path — with the Linux Performance Monitoring Unit, reading the same counters you'd reach for when hunting microseconds.

All measurement lives in [`instrumentation/hardware_metrics.rs`](../../apps/simulation/src/instrumentation/hardware_metrics.rs), built on the `perf_event` crate, gated `#[cfg(target_os = "linux")]`, and behind the `dev-tools` feature flag — production builds carry zero cost. Counters are opened in PMU **groups** for atomic, coherent reads:

| Counter | `perf_event` kind | Source |
|---|---|---|
| CPU cycles | `Hardware::CPU_CYCLES` | [`hardware_metrics.rs:146`](../../apps/simulation/src/instrumentation/hardware_metrics.rs) |
| Instructions retired | `Hardware::INSTRUCTIONS` | [`hardware_metrics.rs:155`](../../apps/simulation/src/instrumentation/hardware_metrics.rs) |
| Cache references (LLC) | `Hardware::CACHE_REFERENCES` | [`hardware_metrics.rs:174`](../../apps/simulation/src/instrumentation/hardware_metrics.rs) |
| Cache misses (LLC) | `Hardware::CACHE_MISSES` | [`hardware_metrics.rs:183`](../../apps/simulation/src/instrumentation/hardware_metrics.rs) |
| Branch instructions | `Hardware::BRANCH_INSTRUCTIONS` | [`hardware_metrics.rs:201`](../../apps/simulation/src/instrumentation/hardware_metrics.rs) |
| Branch misses | `Hardware::BRANCH_MISSES` | [`hardware_metrics.rs:210`](../../apps/simulation/src/instrumentation/hardware_metrics.rs) |
| Frontend stalled cycles | `Hardware::STALLED_CYCLES_FRONTEND` (optional) | [`hardware_metrics.rs:227`](../../apps/simulation/src/instrumentation/hardware_metrics.rs) |
| Backend stalled cycles | `Hardware::STALLED_CYCLES_BACKEND` (optional) | [`hardware_metrics.rs:236`](../../apps/simulation/src/instrumentation/hardware_metrics.rs) |
| L1D read miss / access | `Cache{L1D, READ, MISS/ACCESS}` (optional) | [`hardware_metrics.rs:253-277`](../../apps/simulation/src/instrumentation/hardware_metrics.rs) |
| L1I read miss / access | `Cache{L1I, READ, MISS/ACCESS}` (optional) | [`hardware_metrics.rs:284-308`](../../apps/simulation/src/instrumentation/hardware_metrics.rs) |
| LLC read miss | `Cache{LL, READ, MISS}` (optional) | [`hardware_metrics.rs:310-321`](../../apps/simulation/src/instrumentation/hardware_metrics.rs) |

From these, `read()` derives IPC, the L1D/L1I/LLC miss rates, branch miss rate, and frontend/backend stall ratios ([`hardware_metrics.rs:518-558`](../../apps/simulation/src/instrumentation/hardware_metrics.rs)). It is wired into the live loop: counters are initialized in `register_dev_resources`, read each iteration via `read_hardware_counters()`, and shipped into the `TelemetrySnapshot` ([`ipc/bridge/bevy_app.rs`](../../apps/simulation/src/ipc/bridge/bevy_app.rs)).

Two safeguards keep the numbers honest:

- **Multiplexing detection.** When the CPU can't keep all counters resident, it time-slices them. The engine compares `time_enabled` vs `time_running` and warns when coverage drops below 95% ([`hardware_metrics.rs:384-396`](../../apps/simulation/src/instrumentation/hardware_metrics.rs)) — a genuine accuracy guard.
- **Honest read semantics.** Counters stay enabled continuously; `read()` computes deltas since the previous read. So a reading reflects the whole process between two reads, **not** an isolated single-system measurement. This document states that explicitly rather than dressing up whole-tick numbers as per-system ones.

### Honest limits

- **Windows has no hardware counters.** The `perf_event` path is Linux-only by construction; on every other platform it compiles to a no-op stub returning zeros ([`hardware_metrics.rs:647-667`](../../apps/simulation/src/instrumentation/hardware_metrics.rs)). The Windows track is experimental (≈20K), not officially supported.
- **The richer `EcsMetrics` cache-efficiency system is a spec, not code.** `ecs-metrics-specification.md` describes a `cache_efficiency_score`, archetype-fragmentation analytics, and ~19 derived fields — but a search for `collect_ecs_metrics` / `cache_efficiency_score` / `top_3_archetype` across the source returns **zero matches**. The shipped `EcsMetrics` struct ([`instrumentation/snapshot.rs:20-26`](../../apps/simulation/src/instrumentation/snapshot.rs)) has three fields: archetype count, entity count, tick ms. Do not claim cache-efficiency scoring exists.
- **`active_cores * 0.7` parallelism factor is a heuristic estimate**, explicitly labeled as such ([`instrumentation/parallelization.rs:114-124`](../../apps/simulation/src/instrumentation/parallelization.rs)), not a measured count.

---

## The framing line

**Data-oriented design via ECS is trading-grade latency engineering applied to artificial life** — and it is the deeper reason the Rust backend wins, beyond "fearless parallelism." The layout *is* the algorithm: contiguous SoA columns keep branches predictable, execution units fed, and the hot working set resident in cache. The capability-marker archetype keeps that layout from thrashing. And the whole thing is measured the way a hot path should be — with real PMU counters, multiplexing guards, and the discipline to report the honest ~1.7 whole-tick IPC alongside the 4.25 best case.

---

## See also

- [`rust-js-thesis.md`](./rust-js-thesis.md) — why Rust for the backend, JS/Pixi for the front
- [`core-architectures.md`](./core-architectures.md) — index of all core architectural principles
- [`../scale/README.md`](../scale/README.md) — scale ladder and the honesty caveats on status badges
- [`../performance/done/rayon-parallelization.md`](../performance/done/rayon-parallelization.md) — the Rayon movement work
- [`apps/simulation/AGENTS.md`](../../apps/simulation/AGENTS.md) — ECS patterns (capability markers, archetype stability)
