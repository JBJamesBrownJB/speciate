# The Rust × JS Thesis

> **Speciate** — a million-creature artificial-life engine, where Rust's fearless
> parallelism meets the web's visual playground.

This document is the showcase argument behind Speciate. It is written for a sharp
technical reader — a hiring engineer skimming for signal, or a systems programmer
deciding whether the architecture is principled or accidental. Every claim here is
traceable to code in `apps/`; file paths are cited inline so you can check the work.

The thesis is one sentence:

> **A hybrid Rust-core / web-frontend architecture is the right shape for a
> high-throughput simulation — and the seam between the two halves can be made
> nearly free.**

Most hybrid designs collapse at that seam: they pay a serialization tax on every
frame that erases the throughput the native core bought them. Speciate's argument
is that the seam is a solved problem (zero-copy NAPI `Float32Array`), which frees
each half to do what it is uniquely good at.

Three ideas carry the argument:

1. **Fearless parallelism** — data-race-freedom *by construction* lets us
   parallelize aggressively without defensive locking.
2. **The type system as a guardrail** — ownership and the borrow checker make
   AI-agent-authored systems code safe to ship, because the compiler is the
   reviewer that never gets tired.
3. **The zero-copy seam** — a single shared `Float32Array` gives Rust's throughput
   *and* the web's reach, without the serialization tax that sinks most hybrid
   designs.

---

## Why two languages at all?

A simulation engine and a visual sandbox have opposite centers of gravity.

The **simulation** is a tight numerical loop run hundreds of thousands of times per
tick. It wants: predictable memory layout, no garbage-collector pauses, true
multi-core parallelism, and determinism. That is Rust's home turf.

The **frontend** is a rendering, shader, and UI problem. It wants: the richest
graphics ecosystem on the planet, hot-reload iteration speed, and frictionless
distribution (a browser tab, or an Electron app that ships everywhere). That is the
web's home turf — PixiJS over WebGL, the npm ecosystem, Electron packaging.

Picking one language means losing one of these. A pure-JS engine cannot hit the
throughput. A pure-Rust frontend throws away the world's deepest visual ecosystem
and the web's distribution story. The interesting engineering question is not
"which language" but **"can the boundary between them be made cheap enough that you
get both sets of strengths and pay for neither?"**

Speciate's answer is yes, and the rest of this document is the evidence.

---

## Idea 1 — Fearless parallelism: data-race-freedom by construction

The phrase "fearless concurrency" is Rust marketing, but it describes a concrete
engineering fact: the borrow checker makes a data race a *compile error*. You cannot
hand the same mutable data to two threads. This changes how aggressively you are
willing to parallelize, because the usual penalty for getting it wrong — a
heisenbug that surfaces once per 10⁷ ticks in production — is simply unavailable.

### Where it shows up: parallel physics integration

The movement system is the hot path. Every creature integrates acceleration into
velocity and velocity into position every physics tick. In
[`apps/simulation/src/simulation/movement/systems.rs`](../../apps/simulation/src/simulation/movement/systems.rs)
the integration loop collects the ECS query into a `Vec` and drives it with Rayon:

```rust
// Collect entities into Vec for Rayon parallel processing
let mut entities: Vec<_> = query.iter_mut().collect();

// Parallel physics integration + boundary enforcement + rotation
entities.par_iter_mut().with_min_len(256).for_each(
    |(entity, size, position, velocity, acceleration, creature_state, rotation)| {
        // ... Euler integration, drag, turn-rate limiting, boundary clamp ...
    },
);
```

Each closure invocation owns a disjoint set of `&mut` component references. The
compiler has *proven* there is no aliasing before this code runs — there is no lock,
no atomic, no defensive copy in the inner loop. The `with_min_len(256)` is a
work-granularity tuning knob, not a safety mechanism; safety is already guaranteed
by the types.

The measured result (Sprint 15, validated in
[`sprint_summaries/sprint-15-ecs-optimizations_summary.md`](../../sprint_summaries/sprint-15-ecs-optimizations_summary.md)):

| Metric | Value |
|--------|-------|
| Movement time @ 10K creatures (serial) | 25.9 ms |
| Movement time @ 10K creatures (Rayon) | 4.1 ms |
| Speedup | **6.3×** |
| Cores engaged | all 16 |

A 6.3× wall-clock win on a 16-core machine is a real, honest number — not linear
(memory bandwidth and the collect step are the ceiling), but it is the kind of win
you only chase casually when the language has removed the fear.

### A practical wrinkle, documented honestly

Bevy's own `par_iter_mut()` does **not** engage Rayon when the ECS is driven from
inside a NAPI addon (the runtime context differs from a standard Bevy app). The
working pattern is the manual `Vec` collect shown above. This is recorded in
`apps/simulation/AGENTS.md` and in the Sprint 15 notes so the next engineer does not
rediscover it the hard way. Calling out the sharp edge is part of the honesty
mandate — the architecture is principled, but it is not frictionless, and pretending
otherwise would insult the reader.

### Determinism is part of the bargain

Parallelism normally trades away reproducibility. Speciate keeps determinism by
keeping the parallel work *associative and order-independent*: each creature's
update reads a consistent snapshot and writes only its own components. Determinism
is validated at 20K creatures in the Sprint 15 test suite. Deterministic + parallel
is the combination that makes a simulation both fast *and* debuggable — you can
replay a divergence instead of guessing at it.

---

## Data-oriented design: trading-grade latency engineering

Fearless parallelism is the headline, but it is not the deepest reason the Rust
backend wins. The deeper reason is **data-oriented design** — and it is the same
discipline used to chase microseconds in trading platforms, pointed at a
500K-creature simulation instead of an order book. The author spent years on
latency-sensitive trading systems; Speciate is that craft applied to artificial
life. An ECS is the vehicle: it forces you to write *data-oriented* code instead of
object-oriented code, and data-oriented code is where the CPU-efficiency edge lives.

The argument, concretely:

- **The cache hierarchy is the whole game.** Bevy 0.14 stores components in archetype
  tables — each component type is a contiguous column (Structure-of-Arrays). Iterating
  a system walks those columns linearly, which is exactly what the hardware prefetcher
  is built for. The framing the author works to: *"L3 cache is my RAM"* — the hot
  working set is engineered to stay **resident in CPU cache** rather than spilling to
  DRAM. The neighbor-lookup grid goes further with a hand-rolled SoA: one contiguous
  buffer of 32-byte proxies (two per cache line), "zero pointer chasing during queries"
  ([`spatial/grid.rs`](../../apps/simulation/src/simulation/spatial/grid.rs)).

- **Stable archetypes sustain that residency.** The capability markers from Idea 1 are
  added once at spawn and **never removed** (verified: zero `remove::<Can*>` in source —
  `apps/simulation/AGENTS.md` calls it an anti-pattern). No archetype churn means
  no layout thrash, so the cache-friendly column layout stays put across the whole run.

- **Predictable branches keep the pipeline full.** Hot loops iterate uniform, homogeneous
  data, so branches are predictable and mispredictions are rare. The loops are written
  to stay branch- and cache-friendly: cached `cos`/`sin` on `Rotation` (no per-tick
  `atan2`), a precomputed `inv_sqrt_length` refreshed only on `Changed<BodySize>`,
  squared-distance comparisons throughout, and a release profile tuned for predictable
  codegen (`lto = "fat"`, `codegen-units = 1`, `panic = "abort"` in
  [`Cargo.toml`](../../apps/simulation/Cargo.toml)).

- **Rayon rides the contiguity.** Because the data is already contiguous and Rust
  guarantees data-race-freedom, parallel iteration scales without defensive locking —
  fearless parallelism (Idea 1) is the *consequence* of the data-oriented layout, not a
  separate trick.

### The proof: real hardware performance counters

The credible part is that none of this is asserted from a diagram — the engine is
**instrumented with real CPU performance counters**, profiled the same way you would
profile a trading hot path.
[`apps/simulation/src/instrumentation/hardware_metrics.rs`](../../apps/simulation/src/instrumentation/hardware_metrics.rs)
opens Linux `perf_event` PMU groups and reads **CPU cycles, instructions retired,
cache references and cache misses, branch instructions and branch misses, and
frontend/backend stalled cycles**, with optional L1D / L1I / LLC cache events. It
derives IPC and the various miss rates, and warns when PMU multiplexing coverage drops
below 95% so the numbers stay honest. These counters are wired live into the dev-tools
telemetry loop ([`ipc/bridge/bevy_app.rs`](../../apps/simulation/src/ipc/bridge/bevy_app.rs))
and behind a feature flag, so production builds pay nothing.

Two honest qualifications:

- **IPC depends on what you measure.** The isolated tight movement loop hit **IPC ≈ 4.25**
  in the Sprint 15 benchmark; whole-tick IPC at population, captured in the aggregated
  Linux perf snapshots ([`docs/performance/snapshots/`](../performance/snapshots/)), is
  **≈ 1.7** with an L1D miss rate around 4.3% and a branch miss rate under 1%. Both are
  real measurements of different things — don't conflate them.
- **Linux only.** The `perf_event` path is Linux-only by construction; on Windows it
  compiles to a no-op stub, so there is no hardware-counter data on Windows yet.

The richer per-archetype cache-efficiency scoring described in the metrics specs is
**design intent, not shipped code** — flagged here so the showcase doesn't overclaim.

**The line to land:** data-oriented design via ECS is trading-grade latency
engineering applied to artificial life — measured with the same hardware counters,
chasing the same cache and pipeline wins. The full treatment, with every counter and
storage detail cited, lives in
[`docs/architecture/data-oriented-design.md`](./data-oriented-design.md).

---

## Idea 2 — The type system as a guardrail for AI-authored systems code

This is the part of the thesis most relevant to how Speciate is actually built, and
the part most likely to be undersold elsewhere.

Much of this codebase is authored with AI agents. That is a liability in most
languages: an agent confidently writes plausible code, and plausible-but-wrong
systems code is exactly the kind that compiles, passes a smoke test, and corrupts
state under load three weeks later. The usual mitigation is human review, which is
slow and imperfect.

Rust changes the economics. **Deliberately constraining the backend to a language
with ownership, lifetimes, and `Send`/`Sync` bounds turns the compiler into a
tireless reviewer of an entire class of bugs.** An agent that writes a data race, a
use-after-free, an aliased mutable borrow, or a thread-unsafe shared structure does
not produce a subtle runtime fault — it produces a *compile error*, immediately, at
authoring time. The feedback loop closes in seconds, before the code is ever run.

Concretely, the type system is doing review work in several places:

- **Capability markers as zero-sized types.** In
  [`apps/simulation/src/simulation/creatures/components/capabilities.rs`](../../apps/simulation/src/simulation/creatures/components/capabilities.rs),
  `CanSeek`, `CanFlee`, `CanWander`, and `CanAvoidObstacles` are empty structs. They
  encode *what an entity is allowed to do* directly in the type-driven archetype.
  A system that queries `With<CanSeek>` is structurally incapable of touching a
  creature that lacks the capability. The agent cannot "forget a guard" because the
  guard is the query type, enforced by the ECS.

- **`Send`/`Sync` bounds on the threading seam.** The simulation runs on a dedicated
  Bevy thread while JavaScript calls in from the main thread
  ([`apps/simulation/src/napi_addon/simulation_engine.rs`](../../apps/simulation/src/napi_addon/simulation_engine.rs)).
  Everything crossing that boundary — `Arc<Mutex<DoubleBuffer>>`, the
  `crossbeam_channel` command queue, the atomic flags — is auto-checked for thread
  safety by the compiler. An agent that tries to share something non-`Sync` across
  the threads gets a hard error, not a race.

- **Newtypes and bounded values.** Domain invariants (unit intervals, tick counts,
  power-of-2 throttle divisors) are encoded in types like the `unit_interval`
  helper and the `FrequencyThrottle` constructor, so out-of-range values are caught
  at construction rather than producing silent garbage downstream.

The payoff is a development model where the human (or the agent's human supervisor)
reviews *architecture and behavior*, and delegates *memory- and thread-safety
review* to the compiler. The tests cover behavior (`cargo test`, the spec suite in
`apps/simulation/specs/`); the type system covers safety. That division of labor is
what makes AI-assisted systems programming credible here, rather than reckless.

This is a deliberate architectural choice, not a happy accident: the backend
language was constrained *because* a stronger compiler is a better guardrail for
fast, agent-driven iteration.

---

## Idea 3 — The zero-copy seam: throughput *and* reach, no serialization tax

This is where most hybrid architectures die, so it gets the most scrutiny.

### The tax that kills naive designs

The intuitive way to connect a native simulation to a JS frontend is to serialize
state to JSON (or MessagePack) and send it over a pipe. Speciate *used* to do
exactly this — length-prefixed MessagePack over stdio. It does not anymore, and any
documentation still showing stdio/MessagePack as the current IPC is **stale**.

The reason it was abandoned is the serialization tax. Re-encoding every creature's
position into a wire format and re-parsing it on the JS side cost on the order of
**~30 ms per frame** at scale — enough to erase the entire throughput advantage of
the Rust core. A native engine that spends 30 ms/frame serializing is not a fast
engine; it is a slow engine with extra steps.

### The fix: one shared `Float32Array`

The current IPC is a lock-free double buffer exposed to JavaScript as a zero-copy
`Float32Array` via NAPI-RS. The mechanism, in two files:

The buffer itself
([`apps/simulation/src/ipc/bridge/double_buffer.rs`](../../apps/simulation/src/ipc/bridge/double_buffer.rs))
is two `Vec<f32>` in a Structure-of-Arrays layout — all IDs, then all X, then all Y,
then all rotations — with an atomic pointer swap between the writer (Bevy) and the
reader (JS):

```rust
pub struct DoubleBuffer {
    read: Vec<f32>,
    write: Vec<f32>,
    size: usize,
}

pub fn swap(&mut self) {
    std::mem::swap(&mut self.read, &mut self.write);  // pointer swap, no copy
}
```

The NAPI surface
([`apps/simulation/src/napi_addon/simulation_engine.rs`](../../apps/simulation/src/napi_addon/simulation_engine.rs))
hands JavaScript a view straight into that memory, or fills a JS-owned buffer in
place:

```rust
#[napi]
pub fn fill_buffer(&self, mut buffer: Float32Array) -> i32 {
    let dest = buffer.as_mut();          // direct write into JS-owned memory
    // ... copy active region of read slice ...
}
```

There is no JSON encode, no parse, no intermediate allocation on the hot path. The
SoA layout is also cache-friendly on both ends and maps cleanly onto what the
renderer wants — the GPU consumes flat position arrays, so the wire format *is* the
render format.

The honest detail: the `fill_buffer` path exists specifically because the
per-call `Float32Array::new(...)` allocation was not being GC'd cleanly by V8.
Passing a long-lived JS-owned buffer and writing into it via `as_mut()` sidesteps
that. The safety argument is documented at the call site: JS polling is
single-threaded, so the `as_mut()` aliasing is sound. That is the kind of detail
that separates a real zero-copy implementation from a diagram of one.

### Why this is the whole ballgame

Once the seam is ~free, the two-language split stops being a compromise and becomes
pure upside:

- Rust keeps its throughput, determinism, and fearless parallelism on the engine
  side.
- The web keeps its shader ecosystem, its UI tooling, and its distribution story on
  the frontend side (PixiJS/WebGL today, with a browser-distribution path alongside
  the Electron desktop build).
- Neither side pays the tax that normally makes "native core + web UI" a bad trade.

The zero-copy seam is the load-bearing claim of the entire thesis. The other two
ideas are about what each half does well; this one is about why you can have both.

---

## The supporting cast (why scale is plausible)

Fearless parallelism and a free seam are necessary but not sufficient for a
million creatures. The engine earns its scale headroom with a handful of
ECS-and-spatial techniques, all in `apps/simulation/src/simulation/`:

- **Two-level spatial grid.** L0 cells are 20 m
  ([`spatial/constants.rs`](../../apps/simulation/src/simulation/spatial/constants.rs):
  `CELL_SIZE = 20.0`); L1 cells are 3×3 L0 cells = 60 m and aggregate
  *BioSignatures* rather than individual entities
  ([`spatial/hierarchical.rs`](../../apps/simulation/src/simulation/spatial/hierarchical.rs)).
  Near-field perception reads L0; far-field reads the cheap L1 summary. The L0 grid
  is double-buffered so perception reads the front while the rebuild writes the back.

- **Frequency throttling via power-of-2 bucketing.**
  [`core/frequency_throttle.rs`](../../apps/simulation/src/simulation/core/frequency_throttle.rs)
  buckets entities by `entity_index & (divisor - 1)` — a single-cycle bitwise AND
  instead of a 30-cycle modulo — so expensive systems process a fixed fraction of
  the population each tick and every entity is still serviced exactly once per
  cycle. (Unit tests in that file prove the once-per-cycle invariant.)

- **Capability-marker ECS for archetype stability.** Capabilities are added once at
  spawn and never removed (verified: zero `remove::<Can*>` in source), so behavioral
  changes flip an enum rather than thrashing archetypes. Stable archetypes keep Bevy's
  query iteration cache-friendly at population scale — see the data-oriented design
  section above. (Archetype-stable *death* handling is documented design intent, not
  yet shipped.)

Each of these is, ideally, a **Golden Zone** optimization — a performance win that
*is* a biological feature (a giant ignoring a mouse because the L1 summary filtered
it out is both a culling optimization and emergent size-dominance behavior). That
design principle is documented project-wide; here it is enough to note that the
scale headroom is built from many such small, principled wins rather than one heroic
trick.

---

## Honest status: validated → target → stretch

Engineers are reading this, so here is the ladder without the marketing gloss.

| Tier | Population | Status |
|------|-----------:|--------|
| **Validated** | 500,000 | Actually tested on Linux — the supported, benchmarked baseline. |
| **Peak run** | ~900,000 | Windows — sustained 20 Hz, single session, **not yet CI-benchmarked.** |
| **Stretch / north star** | 1,000,000 | The "art of the possible" target. Not yet reached — ~10% of tick budget away. |

The headline of one million creatures is a *target*, deliberately framed as the art
of the possible — not a benchmark we are quietly claiming. The honest validated state
is **500K on Linux**.

What was, for a long time, a known unresolved limitation — Windows topping out around
~20K with the root cause "under investigation" — turned out **not** to be an engine
limit at all. It was a render-delivery defect (a free-running poll fighting the
producer's clock); fixing it (push-on-swap + snapshot interpolation) let Windows run a
single session at **~900K creatures at a sustained 20 Hz**, the tick at ~49 ms of its
50 ms budget. That is a *peak run*, not a validated number — single machine, not yet
CI-benchmarked — and it is stated that way on purpose. The honesty cuts both ways:
we name the open problems, and we don't inflate the wins past what the receipts show.

Making these numbers continuously trustworthy (rather than point-in-time claims) is
the job of Pillar 1: the deterministic test framework, the live metrics dashboard,
and cross-OS CI. Until that CI is live, the status badges in the README are static
placeholders, and they say so.

---

## In one breath

Speciate is a bet that the best shape for a high-throughput simulation is a Rust
core and a web frontend, joined by a zero-copy seam. Rust's ownership model lets us
parallelize the hot loop without fear (6.3× on movement, all cores, deterministic at
20K) *and* turns the compiler into a safety reviewer that makes AI-assisted systems
work credible. The web brings the visuals and the distribution. And the NAPI
`Float32Array` double buffer makes the boundary between them nearly free — which is
the only reason you get to keep both sets of strengths at once.

The numbers are honest: 500K validated on Linux, a ~900K peak run on Windows (single
session, not yet CI-benchmarked), one million as the declared stretch target — now
within ~10% of the tick budget. The architecture is principled and the limits are
stated. That combination — ambition with receipts — is the showcase.

---

### Source map (verify the claims)

| Claim | File |
|-------|------|
| Rayon parallel movement, `with_min_len(256)` | `apps/simulation/src/simulation/movement/systems.rs` |
| 6.3× speedup, all 16 cores, 20K determinism | `sprint_summaries/sprint-15-ecs-optimizations_summary.md` |
| Zero-copy double buffer (SoA, pointer swap) | `apps/simulation/src/ipc/bridge/double_buffer.rs` |
| NAPI `Float32Array` surface, `fill_buffer`/`as_mut` | `apps/simulation/src/napi_addon/simulation_engine.rs` |
| Capability markers (ZST) | `apps/simulation/src/simulation/creatures/components/capabilities.rs` |
| Two-level grid (L0 20m / L1 60m) | `apps/simulation/src/simulation/spatial/{constants,hierarchical}.rs` |
| Contiguous SoA proxy grid, 32-byte proxies | `apps/simulation/src/simulation/spatial/grid.rs` |
| Power-of-2 frequency throttle | `apps/simulation/src/simulation/core/frequency_throttle.rs` |
| Hardware PMU counters (cycles, IPC, cache/branch misses, stalls) | `apps/simulation/src/instrumentation/hardware_metrics.rs` |
| Counter wiring into telemetry loop | `apps/simulation/src/ipc/bridge/bevy_app.rs` |
| Aggregated Linux perf snapshots (whole-tick IPC ≈ 1.7) | `docs/performance/snapshots/*.json` |
| IPC ≈ 4.25 (isolated movement loop) | `sprint_summaries/sprint-15-ecs-optimizations_summary.md` |
