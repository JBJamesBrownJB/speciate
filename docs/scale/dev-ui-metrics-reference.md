# Dev-UI Metrics Reference 📖

> **Status: 📖 Reference.** What every metric in the dev-tools window means, what a
> healthy value looks like, and how to move it. Audience: anyone reading the harness
> while profiling. Source of truth for the fields is `apps/dev-ui/src/types.ts`
> (`TelemetryFrame`) and the Rust `TelemetrySnapshot` (`apps/simulation/src/ipc/bridge/telemetry.rs`).

The simulation runs a fixed **20 Hz** tick → a **50 ms budget per tick**. Almost every
"is this healthy?" judgement reduces to: *is `totalTickUs` comfortably under 50 000 µs?*

Availability legend: **All** = every OS · **Linux** = needs `perf_event` (badge shown elsewhere) · **Windows** = Windows-only panel.

---

## System Timings (All) — the workhorse

Per-system wall-clock time, measured with `std::time::Instant`. This is where you find
*what* is eating the tick. All values are microseconds (µs); 1 ms = 1000 µs.

| Metric | What it is | What it indicates | How to improve |
|--------|-----------|-------------------|----------------|
| `totalTickUs` | Whole-tick duration | The headline. < 50 000 µs = keeping 20 Hz. The sum the budget is measured against. | Attack the largest sub-system below first. |
| `perceptionUs` | Neighbour detection (spatial-grid queries) | Usually the biggest cost at scale. High = lots of creatures perceiving lots of neighbours. | Frequency-throttle perception (power-of-2 bucketing), tighter FOV/range genes, skip perceiving tiny entities (Golden Zone). |
| `movementUs` | Velocity + physics integration (rotation fused in) | Scales ~linearly with population. | Rayon parallelization (`par_iter_mut`); ensure SoA columns stay cache-resident. |
| `steeringUs` | Fused wander/seek/avoidance forces | Force accumulation cost. | Frequency-throttle steering; reduce per-creature force sources. |
| `behaviorTransitionUs` | Behavior state switching | Normally small. Spikes = churn in behavior changes. | Throttle behavior system; hysteresis on transitions. |
| `spatialGridRebuildUs` | Rebuilding the L0/L1 grids each tick | O(occupied cells), not total cells (so world size barely matters — see windows-parity §3.1). | Already efficient; only grows with population, not map size. |
| `l1AggregationUs` / `l2AggregationUs` | Hierarchical grid aggregation (mass/size rollups) | Cost of the two-level grid bookkeeping. | Throttle aggregation frequency if not needed every tick. |
| `exportPositionsUs` | Writing creature positions into the IPC buffer | Scales with visible creatures. | Viewport culling (export only on-screen creatures). |
| `captureDebugAccelUs` | Debug-only acceleration capture | Should be ~0 unless perception-debug is active. | Disable perception debug when not inspecting. |
| `cellsQueriedTotal` | Spatial cells visited this tick (reset-on-read) | Proxy for perception work. Rising fast = perception getting expensive. | Same levers as `perceptionUs`. |
| `archetypeCount` | Distinct ECS archetypes | Many archetypes = fragmented memory layout (DOD smell). Currently hardcoded 0 — not yet wired. | Fewer component combinations; capability-marker ZSTs. |
| `entityCount` | Live entities the metric pass saw | Sanity check vs creature count. Currently 0 — not yet wired. | n/a (instrumentation TODO). |

---

## Parallelism (All)

How well the engine spreads work across cores.

| Metric | What it is | What it indicates | How to improve |
|--------|-----------|-------------------|----------------|
| `cpuCoresTotal` | Logical cores available | Hardware ceiling. | n/a. |
| `cpuCoresActive` | Cores with > 10 % usage | How many cores the work actually lights up. Far below total during a heavy tick = poor parallelism. | Coarser Rayon chunks (`with_min_len`), fewer/fatter parallel regions; on Windows watch oversubscription vs libuv/V8 threads. |
| `cpuUtilizationPct` | Mean utilisation across cores | Sustained ~100 % on one core only = serial bottleneck (e.g. the un-paused master loop). | Parallelize the hot serial stage; pace the master loop. |
| `estimatedParallelismFactor` | `activeCores / totalCores` | 1.0 = using everything; 0.1 = mostly single-threaded. | As above. |
| `concurrentSystemsEstimate` | Rough count of systems running in parallel | Heuristic only. | n/a. |

---

## Process Memory (All)

| Metric | What it is | What it indicates | How to improve |
|--------|-----------|-------------------|----------------|
| `processMemoryBytes` / working set | Resident memory (RSS) of the process | Should grow with population then plateau. A steady climb at constant population = a leak. | Pre-allocate to capacity, reuse scratch buffers (clear + refill, retain capacity), avoid per-tick `Vec` growth. |

> Cross-platform via `sysinfo` (Windows reads the OS working set; Linux reads `/proc`).

---

## Hardware Performance Cockpit (Linux only)

True CPU performance counters via `perf_event_open`. **No Windows user-space equivalent**
— on Windows this panel is replaced by a "Linux only" badge (see
[windows-parity-strategy.md](./windows-parity-strategy.md) §4). These answer *why* the
CPU is slow at the silicon level.

| Metric | What it is | What it indicates | How to improve |
|--------|-----------|-------------------|----------------|
| `ipc` (instructions/cycle) | Work done per CPU clock | Higher is better. > 2.0 = healthy; < 1.0 = the CPU is stalling (waiting on memory/branches) rather than computing. | Improve cache locality (SoA, hot/cold split), reduce branch misprediction. |
| `l1dMissRate` | L1 data-cache miss rate | High = the working set doesn't fit in L1; the hot loop is chasing pointers / striding badly. | Keep the per-tick working set small and contiguous; iterate columns, not scattered entities. |
| `llcMissRate` | Last-level (L3) cache miss rate | High = going to main memory often (the expensive case). | Shrink working set; block/tile the data; move cold fields out of the hot struct. |
| `branchMissRate` | Branch-mispredict rate | High = unpredictable branches stalling the pipeline. | Branchless code on hot paths; sort/group to make branches predictable; avoid per-entity `match` in the inner loop. |
| `cyclesDelta` / `instructionsDelta` | Raw counts since last sample | Feed IPC; mostly diagnostic. | n/a. |
| `frontendStallRatio` / `backendStallRatio` | Fraction of cycles stalled at pipeline front/back end | Front = instruction-fetch/decode bound; back = execution/memory bound (usually the one to chase). | Backend stalls → cache work (above). Frontend → code size / I-cache. |

---

## Windows Process (Windows only)

Cheap Win32 process telemetry that partially fills the gap where the Linux PMU panel
can't run. Source: `apps/simulation/src/instrumentation/windows_metrics.rs`.

| Metric | What it is | What it indicates | How to improve |
|--------|-----------|-------------------|----------------|
| `processCyclesPerSec` | Process **reference** cycles/sec (`QueryProcessCycleTime`, RDTSC-based — **not** true core-clock cycles) | Coarse "how much CPU is this process burning". Rising with no extra creatures = wasted work (e.g. busy-spin). | Pace the master loop; reduce per-tick work. Treat as a trend, not an absolute. |
| `pageFaultsPerSec` | Rate of page faults | See the note below — a few hundred/sec is normal. | Reduce per-tick allocation churn if it climbs into the thousands and correlates with disk I/O. |
| `pageFaultCount` | Cumulative page faults since start | Ever-increasing by nature; only the *rate* is interesting. | n/a. |
| `workingSetBytes` | Resident memory (same idea as Process Memory) | Plateau good; steady climb = leak. | Same as Process Memory. |

### What are "page faults"? (and why ~200–300/sec is fine)

A **page fault** happens when the process touches a virtual-memory page that isn't
currently mapped into its working set. There are two kinds:

- **Soft (minor) fault** — the page is already in RAM (e.g. freshly allocated memory, or
  shared with another process); the OS just maps it in. **Cheap** (microseconds). This is
  the overwhelming majority of what you're seeing.
- **Hard (major) fault** — the page must be read from disk (the page file). **Expensive**
  (milliseconds) and a sign of real memory pressure / swapping.

Windows' `PageFaultCount` (which this metric is built on) counts **both** kinds and does
not distinguish them. **200–300 faults/sec is normal, benign background noise** — it's
just the process touching newly-allocated or first-use pages as it runs. It is *not* a
problem and needs no action.

When to actually care: a **sustained, climbing** fault rate (thousands/sec) that tracks
rising disk activity — that's hard faults / swapping, meaning the process wants more RAM
than is available. The fix there is to use less memory (pre-allocate, reuse buffers,
smaller per-creature footprint), not to chase the fault counter itself.

---

## NAPI Buffer (All)

The zero-copy `Float32Array` seam between Rust and the renderer.

| Metric | What it is | What it indicates | How to improve |
|--------|-----------|-------------------|----------------|
| `napiBufferUsed` / `napiBufferCapacity` | Creatures written vs buffer capacity | How full the position buffer is. | Grow capacity if regularly full; viewport-cull to write fewer. |
| `napiBufferCapacityPct` | Used / capacity | Near 100 % = the buffer is the limit, not the engine. | Increase `DEFAULT_MAX_CREATURES` or cull. |

---

## Quick triage recipe

1. Is `totalTickUs` < 50 000? If yes, you have headroom — smoothness problems are
   **render-side**, not the engine (see `docs/testing/bugs/jitter-high-populations.md`).
2. If no, find the biggest `*Us` sub-system and apply its row above.
3. On Linux, confirm *why* with the cockpit: low `ipc` + high `llcMissRate` = memory-bound
   → shrink the working set. High `branchMissRate` → branchless hot paths.
4. On Windows, you won't have IPC/cache counters — use System Timings + Windows Process
   cycles/working-set as the proxy, and profile with WPA / PIX / Tracy for the silicon view.
