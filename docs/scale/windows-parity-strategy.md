# Windows Parity Strategy 🚧

> **Status: 🚧 In-progress (NOW).** Serves **Pillar 1 — Prove Scale** (`docs/ROADMAP.md`). This is a living investigation, not a finished result. Several decisive facts remain **unverified** — they are flagged explicitly. Honesty mandate applies: measured numbers are marked MEASURED; everything else is HYPOTHESIS or PREDICTED.

Owner: architect-andy (cross-OS parity strategy). Implementation lanes: rusty-ron (Rust), ecs-emma (ECS hot loop), instrumentation-ian (telemetry).

---

## 1. Problem & current state

Two distinct, **independent** problems wear the same "Windows" label. Do not conflate them.

### 1a. THE PROBLEM — the runtime ceiling
| Platform | Population ceiling | Status |
|----------|-------------------|--------|
| Linux | 500,000 | Validated / benchmarked |
| Windows | ~10,000 | Experimental; massive slowdown beyond |

A ~50x gap. The Linux number is a measured release/CI figure; the Windows number is the observed dev-runtime ceiling. **The magnitude alone (50x) is too large for any single OS-level scheduling tax** — it points to either a build-path confounder or an algorithmic hot path the Windows scheduler then amplifies.

### 1b. The metrics gap — observability
The dev-ui shows **zero per-system timings on Windows**. Root cause is structural, not a missing crate: hardware perf counters use the `perf-event` crate which is Linux-only, and the entire `dev-tools` feature is transitively Linux-gated through it. `SystemTimings` is only registered under `dev-tools` (`apps/simulation/src/simulation/core/dev_tools.rs:17`), so without that feature the timing macros are no-ops (`apps/simulation/src/lib.rs:50-54`) and `get_system_timings` returns all-zeros (`apps/simulation/src/simulation/core/simulation.rs:378-380`). On Windows the dev-ui receives a `systemTimings` object with every field 0 — we are **flying blind on the exact platform where the problem lives**.

These two problems interact: we cannot efficiently rank the runtime causes (1a) without first restoring observability (1b).

---

## 2. Ranked root causes (with evidence)

Ranked by prior probability × evidence strength. Confidence is the team's, not certainty.

### R1 — Debug/under-optimized NAPI addon at runtime (HIGH confidence) — **BUILD-WIRING CONFIRMED; magnitude pending A/B; DECISIVE**
The default dev-on-Windows loop loads a **debug** (`opt-level=0`) addon while the Linux 500k figure is a measured **release** build. A debug build of a Rayon-parallelised ECS hot loop runs 20–50x slower than release. **500k / 50 ≈ 10k — this single cause fits the observed ceiling exactly.**

- **CONFIRMED (build wiring, MEASURED 2026-06-20):** `npm run setup` → `setup:rust` → `npm run build:debug` → `napi build --platform --features dev-tools,napi` — **no `--release`** (`apps/simulation/package.json:22-23`; `[profile.dev] opt-level = 0` in `apps/simulation/Cargo.toml`). `npm run dev` (`apps/portal/package.json:12`) does **not** rebuild Rust — it runs whatever `setup` built, i.e. the **debug** addon. Only `npm run dev:release` (`:13`, via `dev:rust-release` → `build` → `napi build --release`) loads the optimized addon. So the everyday path is debug-by-default. The release profile (`lto="fat"`, `codegen-units=1`, `panic="abort"`, `opt-level=3`) only activates with `--release`.
- **Still UNVERIFIED:** the *magnitude*. The criterion bench is **always optimized**, so it CANNOT reproduce a debug-runtime ceiling. Quantifying it needs a **runtime A/B** (`npm run dev` at 10k vs `npm run dev:release` at 50k+), not the bench. See Open Question Q1.
- This MUST be quantified before attributing anything to the OS. If the A/B confirms the magnitude, most OS-level work below is secondary.

### R2 — Per-tick Rayon fork-join overhead amplified by Windows scheduling (MEDIUM confidence, Windows-specific)
The schedule is a single, fully `.after()`-chained `Schedule` with ~10+ Rayon fork-join barriers per tick (perception, movement, 4× grid rebuild, export `par_sort`). At 10k entities per-system useful work is small relative to fork/join + OS thread-wake cost.

- Evidence: `apps/simulation/src/simulation/core/simulation.rs:75-93` builds one serialized schedule; perception (`.../perception/systems.rs:116`), movement (`.../movement/systems.rs:60`), four grid-rebuild fork-joins (`.../spatial/grid.rs:321,332,368,383`), export sort (`.../ipc/bridge/bevy_app.rs:366`).
- Windows mechanism: std Mutex/Condvar/RwLock (used by Rayon latches/parking) sit on `WaitOnAddress`, itself layered on `NtWaitForAlertByThreadId` — documented by Rust std maintainers to over-spin (rust-lang/rust#121956). Rayon Windows slowdown is documented (rayon-rs#642/#730/#795). Each barrier's tail latency is far higher than Linux CFS.
- Explains how Windows **amplifies** a bottleneck; unlikely to be the whole 50x alone.

### R3 — Windows timer resolution + un-paused busy-spin master thread (MEDIUM confidence, Windows-specific)
The un-paused tick loop never sleeps between ticks (`apps/simulation/src/napi_addon/simulation_engine.rs:234-346` + `apps/simulation/src/simulation/tick_controller.rs:65-112`); it busy-spins, pegging a core at 100% and oversubscribing against Rayon workers under coarse Windows quanta (default 15.625 ms timer). Sub-ms sleeps on the pause path (`simulation_engine.rs:247`, `Duration::from_millis(50)`) and double-buffer (10 µs sleeps in `apps/simulation/src/ipc/bridge/double_buffer.rs`) quantize up. Since Win10 v2004, `timeBeginPeriod` is per-process; a process that never calls it inherits coarse granularity.

- Worth a low-tens-of-% jitter/throughput tax + frees a full core. **Not the 50x.** Depends on Open Question Q5 (does Electron already call `timeBeginPeriod(1)`?).

### R4 — Sparse-grid bookkeeping overhead (HIGH prior) — **REFUTED by measurement**
Hypothesis: the production grid is fixed at 10km × 10km (~252k L0 + ~28k L1 cells) hosting only ~10k creatures, so perception cell-walk and rebuild pay full grid-size cost. **Tested and rejected — see §3.1.** The grid was already engineered to be insensitive to cell count (rebuild clears only previously-non-empty cells, `apps/simulation/src/simulation/spatial/grid.rs:308-327`). Tighter world-bounds sizing would recover essentially nothing. **Do not pursue.**

### R5 — Missing `target-cpu=native` (LOW confidence) — **RESOLVED: symmetric, not a Windows regression**
**No `.cargo/config.toml` exists anywhere in the repo (MEASURED 2026-06-20)**, so `target-cpu=native` is unset on *both* platforms — both get the same SSE2/x86-64 baseline codegen. This cannot explain a Windows-vs-Linux delta (Q2 resolved). It remains a potential *absolute* speedup for both OSes (memory-bound sparse-grid walks vectorize poorly, so expect modest gains), not a parity lever. Demoted accordingly.

### R6 — Microsoft Defender real-time scanning (LOW confidence, Windows-specific)
Defender's FS minifilter scans the native addon load + persistence file IO. Real and measurable per-process (`New-MpPerformanceRecording` / `Get-MpPerformanceReport`), but bounded — up to ~30% on IO-heavy paths, single-to-low-double-digit on a steady compute loop. Affects load/IO, not the per-tick compute ceiling. A contributor, not the 50x.

### R7 — NAPI seam is not truly zero-copy under Electron (MEDIUM confidence)
Under Electron's V8 Memory Cage (21+), external-memory ArrayBuffers are forbidden; napi-rs copies the Vec into a sandbox buffer. `get_buffer` (`apps/simulation/src/napi_addon/simulation_engine.rs:436`) does two copies per poll (`.to_vec` + `Float32Array::new`); the existing `fill_buffer` (`:457`) already avoids both via `copy_from_slice` into a JS-owned buffer. Windows first-touch page-fault cost makes this worse. A fixable per-poll tax and a **thesis-honesty correction** (it is a single-memcpy seam, not zero-copy), not the 50x.

**Rejected hypotheses** (do not re-investigate): mimalloc is already the `#[global_allocator]` (`apps/simulation/src/lib.rs:23`); the release LTO profile is already correct; Bevy archetype layout is platform-agnostic; SQLite is off the hot path; HardwareMetrics-corrupts-throttling is implausible because the documented throttle is power-of-2 **tick** bucketing, metrics-independent (pending Q3).

---

## 3. Validated experiments

Two PoCs ran on the real criterion harness (release/bench profile, this Windows machine, warm cache). Numbers below the bench-machine line are **MEASURED**; everything labelled PREDICTED is not.

### 3.1 World-bounds sparsity sweep — MEASURED — hypothesis REFUTED
Fixed population at 10k, swept world size (and therefore L0 cell count) to isolate sparse-grid overhead from population cost.

| World side | ~L0 cells | Per-tick (MEASURED) |
|-----------|-----------|---------------------|
| 1 km | ~2.7k | 911.6 µs |
| 5 km | ~63k | 913.3 µs |
| 10 km | ~250k | 931.6 µs |

**A ~90x increase in allocated cells produced only ~2.2% slower ticks — flat within noise.** Per-tick time does NOT scale with cell count. R4 is **refuted**. The grid rebuild is O(occupied cells), not O(total cells) (`apps/simulation/src/simulation/spatial/grid.rs:308-327`). Population-adaptive bounds sizing would recover nothing. The bench was reverted cleanly; protected WIP files untouched.

**Redirect:** the bench-vs-production blind spot is NOT world size. Next diagnostics needed — a clean **population-only sweep** at fixed small world (the existing 100k-random-DNA bench conflates population, random DNA, and a 5km×4km world simultaneously), plus Linux-side `perf_event` counters to localize divergence.

### 3.2 Always-on per-tick wall-clock telemetry — MEASURED — verdict PROMISING (ship it)
Adds two always-compiled fields to `TelemetrySnapshot` — `time_dropped_ms` and `ticks_missed_budget` — populated from `std::time::Instant` in the NAPI run loop, plus an unconditional total-tick baseline. Surfaces the existing-but-discarded `TickController` drop signal (currently only `eprintln`'d, `apps/simulation/src/napi_addon/simulation_engine.rs:371-376`) into the dev-ui every ~0.55 s, on **both** OSes.

| Bench | Before (MEASURED) | After (MEASURED) | Delta |
|-------|------|-------|-------|
| `tick_scaling/creatures/10000` | 900.14 µs | 886.71 µs | within noise |
| `tick_scaling/creatures/100000` | 7.1372 ms | 6.7810 ms | within noise |

**Overhead is negligible** (~40 ns/tick = 0.08% of the 50 ms budget — 2× `Instant::elapsed` + 3× atomic store at 20 Hz, all outside the Bevy hot loop). The criterion "improvement" is pure Windows QPC timing variance — the PoC writes zero Bevy hot-path code. Compiles clean, all tests pass, reverted cleanly. **This is the first machine-readable signal of WHEN the Windows ceiling is hit and HOW severe the overrun is** — it answers Open Question Q6 directly once shipped.

### What is NOT yet measured (honesty)
- **The single decisive A/B** (debug-runtime vs release-runtime at scale, R1/Q1) has NOT been run. Until it is, the 50x attribution is open.
- R2/R3 (Rayon fork-join + scheduler) are well-grounded **mechanisms** but their Speciate-specific **magnitude on Windows is unmeasured** — requires a Windows profile (WPA "CPU Usage (Precise)", PIX, or Tracy).
- All §4 metrics-parity tiers are **designed, not yet built**.

---

## 4. Windows metrics-parity plan

**Principle: honesty-first. Do NOT fabricate the Linux `HardwareSnapshot` fields on Windows.** The Linux `perf-event` model (in-process, per-thread PMU **counting** between two lines via `perf_event_open`) has **no user-space Windows equivalent**. That asymmetry is itself part of the engineering narrative — label it "reduced (no PMU)", never fake zeros. Keep the `cfg(target_os="linux")` gate on `hardware_metrics.rs` (protected WIP); build a **separate** `WindowsHardwareMetrics` returning a partial, flagged snapshot over the **unchanged** Float32Array seam.

Key reframe: **measurement precision is NOT the gap — scheduling is.** `std::time::Instant` is already QPC-backed and sub-µs on Windows. Do not swap it for manual QPC expecting a perf win; QPC fixes waiting/scheduling nothing.

### Tier 1 — ship now, zero privilege/driver, HIGH value
- `std::time::Instant` (already QPC-backed) for per-system wall-clock timing — the workhorse, delivered by §3.2 for total-tick and by the per-system ungate below.
- `QueryThreadCycleTime` / `QueryProcessCycleTime` (`realtimeapiset.h` via `windows-rs`, behind `cfg(target_os="windows")`) — a per-thread/process CPU-cycle proxy. **Label it "reference cycles", NOT true core-clock cycles** (it is RDTSC-based). Localizes which Bevy system regresses on Windows without admin.

### Tier 2 — cheap process context, no admin
- PDH (`windows::Win32::System::Performance`) for % Processor Time, context-switches/sec, page-faults/sec, working-set RSS.
- Replace the Linux-only `/proc/self/statm` read in `apps/simulation/src/instrumentation/parallelization.rs:56-70` with a `cfg`-dispatched `GetProcessMemoryInfo` on Windows so process memory is non-zero.
- **PDH has NO cache/IPC/stall data — do not claim it does.**

### Tier 3 — opt-in "deep profile (admin)", MEDIUM–HIGH effort
- ETW + PMC (`TraceSetInformation` / `TraceProfileSourceConfigInfo` via `windows-rs`, or external `wpr`/`xperf -pmc`). The **only** path to true IPC + CacheMisses + BranchMispredictions parity. Requires `SeSystemProfilePrivilege` (admin), is a **sampling/tracing** model (not in-process counting). Gate behind a UI toggle + admin check; never default-on.

### Rejected / maximum-fidelity-only
- **`rdpmc`** faults (#GP) in ring 3 because Windows leaves `CR4.PCE=0` with no supported toggle — needs a kernel driver. **Do not pursue.**
- **Intel PCM** gives genuine per-core IPC/L2/L3 but ships its own signed kernel driver + C++ FFI — disproportionate for a portfolio sim. Document as "maximum-fidelity, not baseline" only.

### What CANNOT be matched on Windows in user space (state plainly as "Linux-only")
- Frontend/backend stall ratios and L1I miss rate — model-specific extended PMU events only, brittle.
- L1D miss rate — no architectural Windows event (PCM exposes L2/L3, not L1).
- True IPC and cache/branch counters — admin-only (Tier 3).

### Feature-flag prerequisite
Split `dev-tools` into two sub-features so software instrumentation builds on Windows: `dev-tools-hw` (Linux-only: `perf-event`) and `dev-tools-sw` (cross-platform: `SystemTimings`, `ParallelizationMetrics`, `git2`, `sysinfo`). Today `dev-tools` is transitively Linux-gated (`apps/simulation/Cargo.toml:49-51,61`), so you cannot get per-system timings on Windows at all. This split is the gating prerequisite for Tier 1's per-system view and touches no protected source logic.

---

## 5. Prioritized roadmap to 150k–200k Windows

Effort: S/M/L. Impact and Confidence are the team's honest estimates.

| # | Action | Effort | Impact | Confidence | Notes |
|---|--------|--------|--------|-----------|-------|
| **1** | **Quantify R1 (build wiring already confirmed debug-by-default).** Run the runtime A/B: `npm run dev` (debug) at 10k vs `npm run dev:release` (release) at 50k+, measure the per-tick / ceiling delta. | **S** | **Potentially decisive (up to ~50x)** | HIGH it's worth doing; build path already confirmed debug — only magnitude unknown | Do this FIRST. Cheapest, highest-prior. The bench cannot surface it. |
| **2** | **Ship §3.2 telemetry + ungate per-system timings on Windows** (`dev-tools` split). | **S–M** | Unblocks all ranking | HIGH | Restores observability — every later action is blind without it. |
| **3** | **De-spin the un-paused loop + `timeBeginPeriod(1)`** (`cfg(target_os="windows")`). | **S** | Low-tens-of-% + frees a core (PREDICTED) | MEDIUM magnitude, HIGH it helps | Pace to next ~50 ms boundary; touches no protected file. Gate on Q5. |
| **4** | **Switch portal hot path `get_buffer`→`fill_buffer`** + reframe thesis as single-memcpy. | **S** | Removes per-poll double-copy (PREDICTED) | MEDIUM | Also a portfolio-honesty fix (R7). |
| **5** | **Profile Windows at 10k–20k** (WPA/PIX/Tracy) to measure R2/R3 magnitude; rank perception vs rebuild vs export. | **M** | Localizes the real hot path | HIGH it's necessary | Required before any Rayon surgery. |
| **6** | **Fuse/coalesce Rayon fork-joins; persistent pool sized to physical-cores-minus-headroom; fatter `par_iter` chunks (`with_min_len`).** | **M–L** | Multiple-x on the per-tick overhead (PREDICTED) | MEDIUM | Driven by #5's findings. Account for libuv/V8 oversubscription. |
| **7** | **Population-only bench sweep** (fix small world, sweep pop) to find super-linear vs linear scaling. | **S** | Diagnostic clarity | HIGH | Disentangles the 100k-random-DNA bench's three conflated variables. |
| **8** | **A/B `target-cpu=native`** via `.cargo/config.toml`. | **S** | Lower (memory-bound) (PREDICTED) | LOW | Only after R1 ruled out (Q2). |
| **9** | **Defender exclusions / ReFS Dev Drive performance mode**, measured via `Get-MpPerformanceReport`. | **S** | Bounded (≤~30% IO, less on compute) | MEDIUM | Contributor, not the gap. |

**Path to target:** if R1 is confirmed (action 1), the realistic 150k–200k cross-platform goal may be largely a build-path fix plus scheduler hygiene (3,4) and Rayon tuning (6). If R1 is refuted, the gap is an algorithmic/contention hot path that actions 5–7 must localize before it can be closed. **We do not yet know which world we are in — that is what action 1 decides.**

---

## 6. Risks & open questions

### Risks
- **Over-attributing to the OS.** The 50x is suspiciously close to a debug-vs-release factor. Building OS-level fixes before ruling out R1 risks wasted effort on a secondary cause.
- **Bench blindness.** The criterion bench is always optimized and uses 1km bounds — it reproduces neither a debug-runtime ceiling (R1) nor production density. Treat green benches as necessary-but-insufficient; the runtime A/B and a Windows profile are the real evidence.
- **Protected WIP.** `apps/simulation/Cargo.toml` and `apps/simulation/src/instrumentation/hardware_metrics.rs` are user WIP — never revert, stash, or modify. All Windows metrics work goes in a **separate** `WindowsHardwareMetrics`.
- **Thesis honesty.** The "zero-copy" claim is inaccurate under Electron's V8 cage. `docs/architecture/rust-js-thesis.md` and `docs/architecture/electron-architecture.md` should be corrected to "single-memcpy SoA seam" to preserve the portfolio's honesty mandate.

### Open questions (decisive ones first)
- **Q1 (PARTIALLY RESOLVED 2026-06-20):** The build wiring is now confirmed — `dev:rust`/`setup:rust` use `build:debug` (no `--release`) and `npm run dev` loads that debug addon (see R1). The *remaining* open part is purely the **magnitude**: run the runtime A/B (`npm run dev` 10k vs `npm run dev:release` 50k+) to measure how much of the ~50x the debug build accounts for. Proof = runtime A/B, not the bench.
- **Q2 (RESOLVED 2026-06-20):** No `.cargo/config.toml` exists, so `target-cpu=native` is unset on both platforms → R5 is symmetric, not a parity cause. Closed.
- **Q3:** Do any perception/throttle systems READ `HardwareMetrics` fields to size buckets/LOD? Documented throttle is metrics-independent tick bucketing; if confirmed, the Windows-stub-corrupts-throttling hypothesis is dead.
- **Q4:** Is the DoubleBuffer genuinely lock-free or an `Arc<Mutex<DoubleBuffer>>`? Diagnosis found a Mutex despite a "lock-free, zero contention" comment; if JS polls at ~2× tick rate, contended hand-off is costlier on Windows. Tier-1 lock-wait telemetry would measure it.
- **Q5:** Does Electron/Node call `timeBeginPeriod(1)` before the addon initializes? If yes, deprioritize action 3's timer work; if no, it is worth a low-tens-of-% Windows win.
- **Q6:** What is the actual tick-budget-miss creature count on Windows, and is the degradation linear or super-linear? §3.2's `time_dropped_ms` answers this directly once shipped — it tells us whether we chase an algorithmic cliff (super-linear) or a constant-factor build/scheduler tax (linear).

---

*Cross-references:* `docs/ROADMAP.md` (Pillar 1), `docs/architecture/core-architectures.md` (spatial grid, Rayon parallelization, frequency throttling), `docs/architecture/rust-js-thesis.md` and `docs/architecture/electron-architecture.md` (seam honesty correction).
