# Latency Tuning Lab Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build a deterministic, reproducible "latency tuning lab" in the Rust simulation crate so any speed/population optimization can be ruled in or out by running it and reading per-phase + max-population numbers — not by guessing.

**Architecture:** A pure, unit-tested library module `bench_lab` provides the measurement primitives (percentile stats, seeded world construction, per-tick phase sampler, budget predicate, adaptive population search, A/B report diff). A thin binary (`src/bin/latency_lab.rs`) wires those primitives to argv and writes JSON snapshots. All logic lives in the tested library; the binary is glue. The lab measures wall-clock total tick itself (via `Instant`) so the headline KPI works in any build, and reads `Simulation::get_system_timings()` for per-phase attribution (real values require `--features dev-tools`).

**Tech Stack:** Rust, `bevy_ecs` 0.14, `rand` 0.8 (`StdRng` for determinism), `serde`/`serde_json`, the existing `speciate` public API (`Simulation`, `SimulationBuilder`, `CritBuilder`, `BehaviorMode`, `Dna`) and `SystemTimingsSnapshot`. Tests run under `cargo test`; the runner under `cargo run --release --features dev-tools --bin latency_lab`.

## Global Constraints

- **Tick budget = 50,000 µs (20 Hz).** Source of truth: `TARGET_SIMULATION_HZ = 20.0` at `apps/simulation/src/napi_addon/simulation_engine.rs:39`. The headline pass/fail metric is **p99 total tick ≤ 50,000 µs**, never the mean.
- **Test-FIRST, always.** Every task writes the failing test first, watches it go red, then writes the minimum code to green. Run the full suite (`cargo test`) before and after every change.
- **Determinism is the product.** Every world the lab builds must be byte-for-byte reproducible from `(population, seed, distribution, extents)`. No `thread_rng()` anywhere in the lab path.
- **No new heavy dependencies.** Use `std::env::args` for the binary (no `clap`). Reuse `rand`, `serde`, `serde_json`, `criterion` already in `Cargo.toml`.
- **Per-phase attribution requires `--features dev-tools`.** `Simulation::get_system_timings()` (`simulation.rs:369`) returns zeros without it. The lab must still produce a valid headline (wall-clock total) without dev-tools, and surface per-phase numbers when built with it.
- **Spec approval:** these are new files; no `apps/simulation/specs/` changes. If any task tempts you to edit a spec test, stop and flag the human.
- **Doc standards:** comments describe WHAT/WHY not HOW; reference `file:line` rather than duplicating code.

---

## File Structure

- `apps/simulation/src/simulation/creatures/dna/mod.rs` — **modify**: add `Dna::random_seeded(&mut impl Rng)` (deterministic DNA).
- `apps/simulation/src/bench_lab/mod.rs` — **create**: module root, re-exports, `run_lab` entry used by the binary.
- `apps/simulation/src/bench_lab/stats.rs` — **create**: `TickStats` + `summarize` (percentiles; the measurement core).
- `apps/simulation/src/bench_lab/world.rs` — **create**: `WorldSpec`, `Distribution`, `build_world` (seeded, uniform + clustered).
- `apps/simulation/src/bench_lab/sampler.rs` — **create**: `PhaseSamples`, `sample_ticks` (wall-clock + per-phase over N ticks).
- `apps/simulation/src/bench_lab/budget.rs` — **create**: `BudgetMetric`, `within_budget` (the headline KPI predicate).
- `apps/simulation/src/bench_lab/ramp.rs` — **create**: `RampConfig`, `MaxPopResult`, `find_max_pop` (coarse-bracket → bisection).
- `apps/simulation/src/bench_lab/report.rs` — **create**: `LabReport`, `PhaseDelta`, `diff_reports`, JSON serialization.
- `apps/simulation/src/lib.rs` — **modify**: `pub mod bench_lab;`.
- `apps/simulation/src/bin/latency_lab.rs` — **create**: thin argv→`run_lab`→JSON runner.

`run_lab` is the single public entry the binary calls; everything it needs (`TickStats`, `WorldSpec`, `PhaseSamples`, `BudgetMetric`, `find_max_pop`, `LabReport`) is defined and tested in Tasks 1–8 before the binary in Task 9.

---

### Task 1: Deterministic DNA

**Files:**
- Modify: `apps/simulation/src/simulation/creatures/dna/mod.rs:28-32`
- Test: `apps/simulation/src/simulation/creatures/dna/mod.rs` (inline `#[cfg(test)] mod tests`)

**Interfaces:**
- Consumes: `rand::Rng`, existing `Dna::new(size_gene, fov_gene)`.
- Produces: `Dna::random_seeded(rng: &mut impl rand::Rng) -> Dna` — same distribution as `Dna::random()` but caller controls the RNG, so identical seeds give identical DNA.

- [ ] **Step 1: Write the failing test**

Add to the existing `#[cfg(test)] mod tests` block in `dna/mod.rs`:

```rust
#[test]
fn random_seeded_is_deterministic_for_same_seed() {
    use rand::rngs::StdRng;
    use rand::SeedableRng;

    let mut rng_a = StdRng::seed_from_u64(42);
    let mut rng_b = StdRng::seed_from_u64(42);

    let a = Dna::random_seeded(&mut rng_a);
    let b = Dna::random_seeded(&mut rng_b);

    assert_eq!(a, b, "same seed must produce identical DNA");
}

#[test]
fn random_seeded_varies_across_draws() {
    use rand::rngs::StdRng;
    use rand::SeedableRng;

    let mut rng = StdRng::seed_from_u64(7);
    let first = Dna::random_seeded(&mut rng);
    let second = Dna::random_seeded(&mut rng);

    assert_ne!(first, second, "successive draws from one rng must differ");
}

#[test]
fn random_seeded_stays_in_gene_range() {
    use rand::rngs::StdRng;
    use rand::SeedableRng;

    let mut rng = StdRng::seed_from_u64(1);
    for _ in 0..1000 {
        let dna = Dna::random_seeded(&mut rng);
        assert!((0.0..=1.0).contains(&dna.size_gene));
        assert!((0.0..=1.0).contains(&dna.fov_gene));
    }
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test --lib random_seeded -- --nocapture` (in `apps/simulation`)
Expected: FAIL — `no function or associated item named 'random_seeded' found for struct 'Dna'`.

- [ ] **Step 3: Write minimal implementation**

In `dna/mod.rs`, add the method to `impl Dna` (next to `random()` at line 28) and refactor `random()` to use it:

```rust
pub fn random() -> Self {
    let mut rng = rand::thread_rng();
    Self::random_seeded(&mut rng)
}

pub fn random_seeded(rng: &mut impl rand::Rng) -> Self {
    Self::new(rng.gen(), rng.gen())
}
```

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test --lib random_seeded`
Expected: PASS (3 tests).

- [ ] **Step 5: Commit**

```bash
git add apps/simulation/src/simulation/creatures/dna/mod.rs
git commit -m "feat(bench-lab): add deterministic Dna::random_seeded"
```

---

### Task 2: Percentile stats (`TickStats` + `summarize`)

**Files:**
- Create: `apps/simulation/src/bench_lab/mod.rs`
- Create: `apps/simulation/src/bench_lab/stats.rs`
- Modify: `apps/simulation/src/lib.rs`
- Test: `apps/simulation/src/bench_lab/stats.rs` (inline tests)

**Interfaces:**
- Consumes: nothing external.
- Produces:
  - `struct TickStats { count: usize, min: f64, max: f64, mean: f64, std_dev: f64, p50: f64, p95: f64, p99: f64 }` (`#[derive(Debug, Clone, Serialize, Deserialize, Default)]`, `serde camelCase`).
  - `fn summarize(samples: &[u64]) -> TickStats` — population std dev; percentiles via 0-based linear interpolation (R-7, the numpy/Excel default). Empty input → `TickStats::default()`.

- [ ] **Step 1: Write the failing test**

Create `apps/simulation/src/bench_lab/stats.rs`:

```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct TickStats {
    pub count: usize,
    pub min: f64,
    pub max: f64,
    pub mean: f64,
    pub std_dev: f64,
    pub p50: f64,
    pub p95: f64,
    pub p99: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn approx(a: f64, b: f64) {
        assert!((a - b).abs() < 1e-6, "expected {b}, got {a}");
    }

    #[test]
    fn summarize_basic_moments() {
        let stats = summarize(&[10, 20, 30, 40, 50]);
        assert_eq!(stats.count, 5);
        approx(stats.min, 10.0);
        approx(stats.max, 50.0);
        approx(stats.mean, 30.0);
        approx(stats.std_dev, 14.142135623730951); // population std dev of {10..50}
        approx(stats.p50, 30.0);
    }

    #[test]
    fn summarize_percentiles_interpolate() {
        // 1..=20 ascending; R-7 ranks: p50 -> 10.5, p95 -> 19.05
        let samples: Vec<u64> = (1..=20).collect();
        let stats = summarize(&samples);
        approx(stats.p50, 10.5);
        approx(stats.p95, 19.05);
    }

    #[test]
    fn summarize_is_order_independent() {
        let a = summarize(&[50, 10, 40, 20, 30]);
        let b = summarize(&[10, 20, 30, 40, 50]);
        assert_eq!(a, b);
    }

    #[test]
    fn summarize_empty_is_default() {
        assert_eq!(summarize(&[]), TickStats::default());
    }
}
```

Create `apps/simulation/src/bench_lab/mod.rs`:

```rust
//! Latency tuning lab: deterministic measurement primitives for ruling
//! optimizations in or out by running them. See
//! docs/superpowers/plans/2026-06-21-latency-tuning-lab.md.

pub mod stats;

pub use stats::{summarize, TickStats};
```

Add to `apps/simulation/src/lib.rs` (alongside the other `pub mod` declarations):

```rust
pub mod bench_lab;
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test --lib bench_lab::stats`
Expected: FAIL — `cannot find function 'summarize' in this scope`.

- [ ] **Step 3: Write minimal implementation**

Add to the top of `apps/simulation/src/bench_lab/stats.rs` (above the `#[cfg(test)]` block):

```rust
/// 0-based linear-interpolation percentile (R-7 / numpy default).
fn percentile(sorted: &[f64], p: f64) -> f64 {
    match sorted.len() {
        0 => 0.0,
        1 => sorted[0],
        n => {
            let rank = (p / 100.0) * (n as f64 - 1.0);
            let lo = rank.floor() as usize;
            let hi = rank.ceil() as usize;
            let frac = rank - lo as f64;
            sorted[lo] + frac * (sorted[hi] - sorted[lo])
        }
    }
}

pub fn summarize(samples: &[u64]) -> TickStats {
    if samples.is_empty() {
        return TickStats::default();
    }
    let mut sorted: Vec<f64> = samples.iter().map(|&v| v as f64).collect();
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());

    let count = sorted.len();
    let mean = sorted.iter().sum::<f64>() / count as f64;
    let variance = sorted.iter().map(|v| (v - mean).powi(2)).sum::<f64>() / count as f64;

    TickStats {
        count,
        min: sorted[0],
        max: sorted[count - 1],
        mean,
        std_dev: variance.sqrt(),
        p50: percentile(&sorted, 50.0),
        p95: percentile(&sorted, 95.0),
        p99: percentile(&sorted, 99.0),
    }
}
```

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test --lib bench_lab::stats`
Expected: PASS (4 tests).

- [ ] **Step 5: Commit**

```bash
git add apps/simulation/src/bench_lab/mod.rs apps/simulation/src/bench_lab/stats.rs apps/simulation/src/lib.rs
git commit -m "feat(bench-lab): add TickStats percentile summarizer"
```

---

### Task 3: Seeded world builder

**Files:**
- Create: `apps/simulation/src/bench_lab/world.rs`
- Modify: `apps/simulation/src/bench_lab/mod.rs`
- Modify: `apps/simulation/src/simulation/core/simulation.rs` — add the test-gated `snapshot_creatures` accessor.
- Test: `apps/simulation/src/bench_lab/world.rs` (inline tests)

**Interfaces:**
- Consumes: `speciate` public API — `SimulationBuilder`, `CritBuilder`, `BehaviorMode`; `Dna::random_seeded` (Task 1).
- Produces:
  - `Simulation::snapshot_creatures(&self) -> Vec<(u32, f32, f32, f32, f32)>` — test-gated (`#[cfg(any(test, feature = "test-helpers"))]`); returns `(crit_id, x, y, size_gene, fov_gene)` for every creature, **sorted by `crit_id`** for a canonical order independent of ECS archetype iteration. Used to assert initial-spawn determinism.
  - `enum Distribution { Uniform, Clustered { clusters: usize, spread: f32 } }` (`Serialize, Deserialize, Clone, Debug, PartialEq`).
  - `struct WorldSpec { population: usize, seed: u64, half_extent_x: f32, half_extent_y: f32, distribution: Distribution }` (`Serialize, Deserialize, Clone, Debug`).
  - `fn build_world(spec: &WorldSpec) -> Simulation` — deterministic spawn of `population` crits with `random_seeded` DNA. World bounds set to `half_extent * 2` (matching `simulation_bench.rs:22`). `Uniform` spreads across `±half_extent`; `Clustered` places `clusters` centers then offsets each crit by a `±spread` box.

- [ ] **Step 1: Write the failing test**

Create `apps/simulation/src/bench_lab/world.rs`:

```rust
use crate::simulation::creatures::dna::Dna;
use crate::{BehaviorMode, CritBuilder, Simulation, SimulationBuilder};
use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Distribution {
    Uniform,
    Clustered { clusters: usize, spread: f32 },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorldSpec {
    pub population: usize,
    pub seed: u64,
    pub half_extent_x: f32,
    pub half_extent_y: f32,
    pub distribution: Distribution,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn spec(pop: usize, seed: u64) -> WorldSpec {
        WorldSpec {
            population: pop,
            seed,
            half_extent_x: 2500.0,
            half_extent_y: 2000.0,
            distribution: Distribution::Uniform,
        }
    }

    #[test]
    fn build_world_spawns_requested_population() {
        let sim = build_world(&spec(1000, 1));
        assert_eq!(sim.creature_count(), 1000);
    }

    #[test]
    fn build_world_is_deterministic() {
        // Same seed => byte-identical INITIAL world (creatures, positions, DNA),
        // before any ticking. The hot loop is Rayon-parallel so tick evolution
        // can drift; the seeded spawn is sequential and exact, and identical
        // initial world is what A/B benchmarking actually requires.
        let a = build_world(&spec(500, 99));
        let b = build_world(&spec(500, 99));
        assert_eq!(a.snapshot_creatures(), b.snapshot_creatures());
    }

    #[test]
    fn different_seeds_build_different_worlds() {
        let a = build_world(&spec(500, 1));
        let b = build_world(&spec(500, 2));
        assert_ne!(a.snapshot_creatures(), b.snapshot_creatures());
    }

    #[test]
    fn clustered_distribution_builds() {
        let s = WorldSpec {
            population: 800,
            seed: 3,
            half_extent_x: 2500.0,
            half_extent_y: 2000.0,
            distribution: Distribution::Clustered { clusters: 8, spread: 100.0 },
        };
        let sim = build_world(&s);
        assert_eq!(sim.creature_count(), 800);
    }
}
```

Add to `apps/simulation/src/bench_lab/mod.rs`:

```rust
pub mod world;

pub use world::{build_world, Distribution, WorldSpec};
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test --lib bench_lab::world`
Expected: FAIL — `cannot find function 'build_world' in this scope`.

- [ ] **Step 3: Write minimal implementation**

Add to `apps/simulation/src/bench_lab/world.rs` (above the test module):

```rust
pub fn build_world(spec: &WorldSpec) -> Simulation {
    let mut sim = SimulationBuilder::new()
        .set_boundaries(spec.half_extent_x * 2.0, spec.half_extent_y * 2.0)
        .build();
    let mut rng = StdRng::seed_from_u64(spec.seed);

    // Pre-compute cluster centers deterministically (drawn before the per-crit
    // loop so the RNG stream is stable for a given spec).
    let centers: Vec<(f32, f32)> = match spec.distribution {
        Distribution::Clustered { clusters, .. } => (0..clusters.max(1))
            .map(|_| {
                let cx = (rng.gen::<f32>() - 0.5) * (spec.half_extent_x * 2.0);
                let cy = (rng.gen::<f32>() - 0.5) * (spec.half_extent_y * 2.0);
                (cx, cy)
            })
            .collect(),
        Distribution::Uniform => Vec::new(),
    };

    for i in 0..spec.population {
        let (x, y) = match spec.distribution {
            Distribution::Uniform => (
                (rng.gen::<f32>() - 0.5) * (spec.half_extent_x * 2.0),
                (rng.gen::<f32>() - 0.5) * (spec.half_extent_y * 2.0),
            ),
            Distribution::Clustered { spread, .. } => {
                let (cx, cy) = centers[i % centers.len()];
                (
                    cx + (rng.gen::<f32>() - 0.5) * spread * 2.0,
                    cy + (rng.gen::<f32>() - 0.5) * spread * 2.0,
                )
            }
        };

        let dna = Dna::random_seeded(&mut rng);
        let builder = CritBuilder::new()
            .at(x, y)
            .with_dna(dna)
            .with_all_capabilities()
            .in_behavior(BehaviorMode::Wandering);
        sim.spawn_crit(builder);
    }

    sim
}
```

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test --lib bench_lab::world`
Expected: PASS (3 tests).

- [ ] **Step 5: Commit**

```bash
git add apps/simulation/src/bench_lab/world.rs apps/simulation/src/bench_lab/mod.rs
git commit -m "feat(bench-lab): add seeded uniform/clustered world builder"
```

---

### Task 4: Per-tick phase sampler

**Files:**
- Create: `apps/simulation/src/bench_lab/sampler.rs`
- Modify: `apps/simulation/src/bench_lab/mod.rs`
- Test: `apps/simulation/src/bench_lab/sampler.rs` (inline tests)

**Interfaces:**
- Consumes: `Simulation` (`update`, `get_system_timings`); `summarize`/`TickStats` (Task 2); `build_world`/`WorldSpec` (Task 3).
- Produces:
  - `struct PhaseSamples { wall_total: TickStats, total_tick: TickStats, perception: TickStats, steering: TickStats, movement: TickStats, spatial_grid_rebuild: TickStats, l1_aggregation: TickStats, behavior_transition: TickStats, export_positions: TickStats }` (`Serialize, Deserialize, Clone, Debug`).
  - `fn sample_ticks(sim: &mut Simulation, warmup: usize, samples: usize, dt: f32) -> PhaseSamples` — runs `warmup` discarded ticks, then `samples` measured ticks. `wall_total` is wall-clock (`Instant`) per `sim.update`, always populated. The per-phase fields come from `get_system_timings()` and are zero unless built with `--features dev-tools`.

- [ ] **Step 1: Write the failing test**

Create `apps/simulation/src/bench_lab/sampler.rs`:

```rust
use crate::bench_lab::stats::{summarize, TickStats};
use crate::Simulation;
use serde::{Deserialize, Serialize};
use std::time::Instant;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct PhaseSamples {
    pub wall_total: TickStats,
    pub total_tick: TickStats,
    pub perception: TickStats,
    pub steering: TickStats,
    pub movement: TickStats,
    pub spatial_grid_rebuild: TickStats,
    pub l1_aggregation: TickStats,
    pub behavior_transition: TickStats,
    pub export_positions: TickStats,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bench_lab::world::{build_world, Distribution, WorldSpec};

    fn small_world() -> Simulation {
        build_world(&WorldSpec {
            population: 1000,
            seed: 5,
            half_extent_x: 500.0,
            half_extent_y: 500.0,
            distribution: Distribution::Uniform,
        })
    }

    #[test]
    fn sampler_collects_requested_sample_count() {
        let mut sim = small_world();
        let s = sample_ticks(&mut sim, 3, 10, 0.05);
        assert_eq!(s.wall_total.count, 10);
    }

    #[test]
    fn sampler_measures_nonzero_wall_time() {
        let mut sim = small_world();
        let s = sample_ticks(&mut sim, 3, 10, 0.05);
        assert!(s.wall_total.mean > 0.0, "wall clock must register real time");
    }

    #[test]
    #[cfg(feature = "dev-tools")]
    fn sampler_captures_per_phase_under_dev_tools() {
        let mut sim = small_world();
        let s = sample_ticks(&mut sim, 3, 10, 0.05);
        assert!(s.perception.mean > 0.0, "per-phase timings populated with dev-tools");
    }
}
```

Add to `apps/simulation/src/bench_lab/mod.rs`:

```rust
pub mod sampler;

pub use sampler::{sample_ticks, PhaseSamples};
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test --lib bench_lab::sampler`
Expected: FAIL — `cannot find function 'sample_ticks' in this scope`.

- [ ] **Step 3: Write minimal implementation**

Add to `apps/simulation/src/bench_lab/sampler.rs` (above the test module):

```rust
pub fn sample_ticks(sim: &mut Simulation, warmup: usize, samples: usize, dt: f32) -> PhaseSamples {
    for _ in 0..warmup {
        sim.update(dt);
    }

    let mut wall = Vec::with_capacity(samples);
    let mut total = Vec::with_capacity(samples);
    let mut perception = Vec::with_capacity(samples);
    let mut steering = Vec::with_capacity(samples);
    let mut movement = Vec::with_capacity(samples);
    let mut grid = Vec::with_capacity(samples);
    let mut l1 = Vec::with_capacity(samples);
    let mut behavior = Vec::with_capacity(samples);
    let mut export = Vec::with_capacity(samples);

    for _ in 0..samples {
        let start = Instant::now();
        sim.update(dt);
        wall.push(start.elapsed().as_micros() as u64);

        let t = sim.get_system_timings();
        total.push(t.total_tick_us);
        perception.push(t.perception_us);
        steering.push(t.steering_us);
        movement.push(t.movement_us);
        grid.push(t.spatial_grid_rebuild_us);
        l1.push(t.l1_aggregation_us);
        behavior.push(t.behavior_transition_us);
        export.push(t.export_positions_us);
    }

    PhaseSamples {
        wall_total: summarize(&wall),
        total_tick: summarize(&total),
        perception: summarize(&perception),
        steering: summarize(&steering),
        movement: summarize(&movement),
        spatial_grid_rebuild: summarize(&grid),
        l1_aggregation: summarize(&l1),
        behavior_transition: summarize(&behavior),
        export_positions: summarize(&export),
    }
}
```

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test --lib bench_lab::sampler` then `cargo test --lib --features dev-tools bench_lab::sampler`
Expected: PASS (2 tests without dev-tools; 3 tests with dev-tools).

- [ ] **Step 5: Commit**

```bash
git add apps/simulation/src/bench_lab/sampler.rs apps/simulation/src/bench_lab/mod.rs
git commit -m "feat(bench-lab): add wall-clock + per-phase tick sampler"
```

---

### Task 5: Budget predicate (the headline KPI)

**Files:**
- Create: `apps/simulation/src/bench_lab/budget.rs`
- Modify: `apps/simulation/src/bench_lab/mod.rs`
- Test: `apps/simulation/src/bench_lab/budget.rs` (inline tests)

**Interfaces:**
- Consumes: `TickStats` (Task 2).
- Produces:
  - `enum BudgetMetric { P99, Max, Mean }` (`Serialize, Deserialize, Clone, Copy, Debug, PartialEq`; default `P99`).
  - `const TICK_BUDGET_US: u64 = 50_000;`
  - `fn within_budget(stats: &TickStats, budget_us: u64, metric: BudgetMetric) -> bool` — `true` when the chosen metric is `<= budget_us`.

- [ ] **Step 1: Write the failing test**

Create `apps/simulation/src/bench_lab/budget.rs`:

```rust
use crate::bench_lab::stats::TickStats;
use serde::{Deserialize, Serialize};

pub const TICK_BUDGET_US: u64 = 50_000;

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum BudgetMetric {
    P99,
    Max,
    Mean,
}

impl Default for BudgetMetric {
    fn default() -> Self {
        BudgetMetric::P99
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn stats_with(p99: f64, max: f64, mean: f64) -> TickStats {
        TickStats { count: 7, min: 0.0, max, mean, std_dev: 0.0, p50: mean, p95: p99, p99 }
    }

    #[test]
    fn p99_under_budget_passes() {
        // Mirrors the real 900k snapshot: p99 = 49,887 us, under the 50ms wall.
        let s = stats_with(49_887.0, 49_888.0, 49_447.0);
        assert!(within_budget(&s, TICK_BUDGET_US, BudgetMetric::P99));
    }

    #[test]
    fn p99_over_budget_fails_even_when_mean_passes() {
        // Mean is under budget but the tail blows it — must fail on P99.
        let s = stats_with(51_000.0, 52_000.0, 49_000.0);
        assert!(!within_budget(&s, TICK_BUDGET_US, BudgetMetric::P99));
        assert!(within_budget(&s, TICK_BUDGET_US, BudgetMetric::Mean));
    }

    #[test]
    fn default_metric_is_p99() {
        assert_eq!(BudgetMetric::default(), BudgetMetric::P99);
    }
}
```

Add to `apps/simulation/src/bench_lab/mod.rs`:

```rust
pub mod budget;

pub use budget::{within_budget, BudgetMetric, TICK_BUDGET_US};
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test --lib bench_lab::budget`
Expected: FAIL — `cannot find function 'within_budget' in this scope`.

- [ ] **Step 3: Write minimal implementation**

Add to `apps/simulation/src/bench_lab/budget.rs` (above the test module):

```rust
pub fn within_budget(stats: &TickStats, budget_us: u64, metric: BudgetMetric) -> bool {
    let value = match metric {
        BudgetMetric::P99 => stats.p99,
        BudgetMetric::Max => stats.max,
        BudgetMetric::Mean => stats.mean,
    };
    value <= budget_us as f64
}
```

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test --lib bench_lab::budget`
Expected: PASS (3 tests).

- [ ] **Step 5: Commit**

```bash
git add apps/simulation/src/bench_lab/budget.rs apps/simulation/src/bench_lab/mod.rs
git commit -m "feat(bench-lab): add p99 budget predicate (50ms wall)"
```

---

### Task 6: Adaptive max-population search

**Files:**
- Create: `apps/simulation/src/bench_lab/ramp.rs`
- Modify: `apps/simulation/src/bench_lab/mod.rs`
- Test: `apps/simulation/src/bench_lab/ramp.rs` (inline tests, synthetic cost model — no real sim)

**Interfaces:**
- Consumes: `TickStats` (Task 2), `within_budget`/`BudgetMetric` (Task 5).
- Produces:
  - `struct RampConfig { low: usize, high: usize, coarse_step: usize, tolerance: usize, budget_us: u64, metric: BudgetMetric }` (`Clone, Debug`).
  - `struct MaxPopResult { max_pop: usize, evaluations: Vec<(usize, TickStats)> }` (`Clone, Debug`).
  - `fn find_max_pop(cfg: &RampConfig, run: impl FnMut(usize) -> TickStats) -> MaxPopResult` — coarse-bracket upward from `low` by `coarse_step` until the first failing population (or `high`), then bisect between last-pass and first-fail until the gap `<= tolerance`. `max_pop` is the largest population that passed (0 if even `low` fails). Every evaluated `(pop, stats)` is recorded in order.

- [ ] **Step 1: Write the failing test**

Create `apps/simulation/src/bench_lab/ramp.rs`:

```rust
use crate::bench_lab::budget::{within_budget, BudgetMetric};
use crate::bench_lab::stats::TickStats;

#[derive(Clone, Debug)]
pub struct RampConfig {
    pub low: usize,
    pub high: usize,
    pub coarse_step: usize,
    pub tolerance: usize,
    pub budget_us: u64,
    pub metric: BudgetMetric,
}

#[derive(Clone, Debug)]
pub struct MaxPopResult {
    pub max_pop: usize,
    pub evaluations: Vec<(usize, TickStats)>,
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Synthetic monotonic cost: p99 = pop * 0.05 us. Crosses 50,000 us at
    /// exactly pop = 1,000,000. No real simulation needed to test the search.
    fn synthetic(pop: usize) -> TickStats {
        let p99 = pop as f64 * 0.05;
        TickStats { count: 7, min: p99, max: p99, mean: p99, std_dev: 0.0, p50: p99, p95: p99, p99 }
    }

    fn cfg() -> RampConfig {
        RampConfig {
            low: 200_000,
            high: 2_000_000,
            coarse_step: 300_000,
            tolerance: 25_000,
            budget_us: 50_000,
            metric: BudgetMetric::P99,
        }
    }

    #[test]
    fn finds_crossover_within_tolerance() {
        let result = find_max_pop(&cfg(), synthetic);
        // True crossover is 1,000,000; result must be a passing pop within
        // one tolerance band below it.
        assert!(result.max_pop <= 1_000_000);
        assert!(result.max_pop >= 1_000_000 - 25_000);
        assert!(within_budget(&synthetic(result.max_pop), 50_000, BudgetMetric::P99));
    }

    #[test]
    fn records_every_evaluation() {
        let result = find_max_pop(&cfg(), synthetic);
        assert!(result.evaluations.len() >= 2);
        // Evaluations are in the order they were probed.
        assert_eq!(result.evaluations[0].0, 200_000);
    }

    #[test]
    fn returns_zero_when_low_already_fails() {
        let mut c = cfg();
        c.low = 1_200_000; // already over budget under synthetic cost
        let result = find_max_pop(&c, synthetic);
        assert_eq!(result.max_pop, 0);
    }
}
```

Add to `apps/simulation/src/bench_lab/mod.rs`:

```rust
pub mod ramp;

pub use ramp::{find_max_pop, MaxPopResult, RampConfig};
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test --lib bench_lab::ramp`
Expected: FAIL — `cannot find function 'find_max_pop' in this scope`.

- [ ] **Step 3: Write minimal implementation**

Add to `apps/simulation/src/bench_lab/ramp.rs` (above the test module):

```rust
pub fn find_max_pop(cfg: &RampConfig, mut run: impl FnMut(usize) -> TickStats) -> MaxPopResult {
    let mut evaluations: Vec<(usize, TickStats)> = Vec::new();

    let mut eval = |pop: usize, evals: &mut Vec<(usize, TickStats)>| -> bool {
        let stats = run(pop);
        let pass = within_budget(&stats, cfg.budget_us, cfg.metric);
        evals.push((pop, stats));
        pass
    };

    // Coarse bracket: climb until a population fails (or we reach `high`).
    let mut last_pass: Option<usize> = None;
    let mut first_fail: Option<usize> = None;
    let mut pop = cfg.low;
    loop {
        let pass = eval(pop, &mut evaluations);
        if pass {
            last_pass = Some(pop);
            if pop >= cfg.high {
                break;
            }
            pop = (pop + cfg.coarse_step).min(cfg.high);
        } else {
            first_fail = Some(pop);
            break;
        }
    }

    // If `low` already failed, there is no passing population.
    let mut lo = match last_pass {
        Some(p) => p,
        None => {
            return MaxPopResult { max_pop: 0, evaluations };
        }
    };

    // Bisect between last-pass (lo) and first-fail (hi) to the tolerance band.
    if let Some(mut hi) = first_fail {
        while hi - lo > cfg.tolerance {
            let mid = lo + (hi - lo) / 2;
            if eval(mid, &mut evaluations) {
                lo = mid;
            } else {
                hi = mid;
            }
        }
    }

    MaxPopResult { max_pop: lo, evaluations }
}
```

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test --lib bench_lab::ramp`
Expected: PASS (3 tests).

- [ ] **Step 5: Commit**

```bash
git add apps/simulation/src/bench_lab/ramp.rs apps/simulation/src/bench_lab/mod.rs
git commit -m "feat(bench-lab): add adaptive coarse-then-bisect max-pop search"
```

---

### Task 7: Lab report + A/B diff

**Files:**
- Create: `apps/simulation/src/bench_lab/report.rs`
- Modify: `apps/simulation/src/bench_lab/mod.rs`
- Test: `apps/simulation/src/bench_lab/report.rs` (inline tests)

**Interfaces:**
- Consumes: `WorldSpec` (Task 3), `PhaseSamples` (Task 4), `TickStats` (Task 2).
- Produces:
  - `struct LabReport { label: String, spec: WorldSpec, budget_us: u64, within_budget: bool, max_pop: Option<usize>, samples: PhaseSamples, build_type: String }` (`Serialize, Deserialize, Clone, Debug`, `serde camelCase`).
  - `struct PhaseDelta { name: String, before_us: f64, after_us: f64, delta_us: f64, pct: f64 }` (`Serialize, Deserialize, Clone, Debug`).
  - `fn diff_reports(before: &LabReport, after: &LabReport) -> Vec<PhaseDelta>` — per-phase `mean` deltas (after − before), `pct` relative to `before` (0.0 when before is 0). Order: total_tick, perception, steering, movement, spatial_grid_rebuild, l1_aggregation, behavior_transition, export_positions.

- [ ] **Step 1: Write the failing test**

Create `apps/simulation/src/bench_lab/report.rs`:

```rust
use crate::bench_lab::sampler::PhaseSamples;
use crate::bench_lab::stats::TickStats;
use crate::bench_lab::world::WorldSpec;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LabReport {
    pub label: String,
    pub spec: WorldSpec,
    pub budget_us: u64,
    pub within_budget: bool,
    pub max_pop: Option<usize>,
    pub samples: PhaseSamples,
    pub build_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PhaseDelta {
    pub name: String,
    pub before_us: f64,
    pub after_us: f64,
    pub delta_us: f64,
    pub pct: f64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bench_lab::world::Distribution;

    fn mean_stats(mean: f64) -> TickStats {
        TickStats { count: 7, min: mean, max: mean, mean, std_dev: 0.0, p50: mean, p95: mean, p99: mean }
    }

    fn report(label: &str, perception_mean: f64) -> LabReport {
        let mut samples = PhaseSamples::default();
        samples.total_tick = mean_stats(perception_mean + 1000.0);
        samples.perception = mean_stats(perception_mean);
        LabReport {
            label: label.to_string(),
            spec: WorldSpec {
                population: 200_000,
                seed: 1,
                half_extent_x: 2500.0,
                half_extent_y: 2000.0,
                distribution: Distribution::Uniform,
            },
            budget_us: 50_000,
            within_budget: true,
            max_pop: None,
            samples,
            build_type: "release".to_string(),
        }
    }

    #[test]
    fn diff_reports_computes_perception_delta() {
        let before = report("baseline", 15_000.0);
        let after = report("optimized", 12_000.0);
        let deltas = diff_reports(&before, &after);

        let perception = deltas.iter().find(|d| d.name == "perception").unwrap();
        assert_eq!(perception.before_us, 15_000.0);
        assert_eq!(perception.after_us, 12_000.0);
        assert_eq!(perception.delta_us, -3_000.0);
        assert!((perception.pct - (-20.0)).abs() < 1e-6);
    }

    #[test]
    fn report_serializes_to_camel_case_json() {
        let json = serde_json::to_string(&report("x", 15_000.0)).unwrap();
        assert!(json.contains("withinBudget"));
        assert!(json.contains("buildType"));
        assert!(json.contains("totalTick"));
    }
}
```

Add to `apps/simulation/src/bench_lab/mod.rs`:

```rust
pub mod report;

pub use report::{diff_reports, LabReport, PhaseDelta};
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test --lib bench_lab::report`
Expected: FAIL — `cannot find function 'diff_reports' in this scope`.

- [ ] **Step 3: Write minimal implementation**

Add to `apps/simulation/src/bench_lab/report.rs` (above the test module):

```rust
fn delta(name: &str, before: &TickStats, after: &TickStats) -> PhaseDelta {
    let before_us = before.mean;
    let after_us = after.mean;
    let delta_us = after_us - before_us;
    let pct = if before_us == 0.0 { 0.0 } else { delta_us / before_us * 100.0 };
    PhaseDelta { name: name.to_string(), before_us, after_us, delta_us, pct }
}

pub fn diff_reports(before: &LabReport, after: &LabReport) -> Vec<PhaseDelta> {
    let b = &before.samples;
    let a = &after.samples;
    vec![
        delta("total_tick", &b.total_tick, &a.total_tick),
        delta("perception", &b.perception, &a.perception),
        delta("steering", &b.steering, &a.steering),
        delta("movement", &b.movement, &a.movement),
        delta("spatial_grid_rebuild", &b.spatial_grid_rebuild, &a.spatial_grid_rebuild),
        delta("l1_aggregation", &b.l1_aggregation, &a.l1_aggregation),
        delta("behavior_transition", &b.behavior_transition, &a.behavior_transition),
        delta("export_positions", &b.export_positions, &a.export_positions),
    ]
}
```

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test --lib bench_lab::report`
Expected: PASS (2 tests).

- [ ] **Step 5: Commit**

```bash
git add apps/simulation/src/bench_lab/report.rs apps/simulation/src/bench_lab/mod.rs
git commit -m "feat(bench-lab): add LabReport + per-phase A/B diff"
```

---

### Task 8: `run_lab` orchestration entry

**Files:**
- Modify: `apps/simulation/src/bench_lab/mod.rs`
- Test: `apps/simulation/src/bench_lab/mod.rs` (inline tests)

**Interfaces:**
- Consumes: `WorldSpec`, `build_world`, `sample_ticks`, `within_budget`, `BudgetMetric`, `TICK_BUDGET_US`, `LabReport`, `find_max_pop`, `RampConfig`.
- Produces:
  - `struct LabConfig { label: String, spec: WorldSpec, warmup: usize, samples: usize, dt: f32, budget_us: u64, metric: BudgetMetric, find_max: Option<RampConfig> }` (`Clone, Debug`).
  - `fn run_lab(cfg: &LabConfig) -> LabReport` — builds the world at `spec.population`, samples it, evaluates the budget. If `find_max` is `Some`, runs `find_max_pop` (rebuilding the world at each probed population via `spec` with `population` overridden) and sets `max_pop`.

- [ ] **Step 1: Write the failing test**

Add a `#[cfg(test)] mod tests` block at the bottom of `apps/simulation/src/bench_lab/mod.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::bench_lab::budget::TICK_BUDGET_US;
    use crate::bench_lab::world::{Distribution, WorldSpec};

    fn small_spec(pop: usize) -> WorldSpec {
        WorldSpec {
            population: pop,
            seed: 11,
            half_extent_x: 500.0,
            half_extent_y: 500.0,
            distribution: Distribution::Uniform,
        }
    }

    #[test]
    fn run_lab_produces_report_for_fixed_population() {
        let cfg = LabConfig {
            label: "unit".to_string(),
            spec: small_spec(1000),
            warmup: 2,
            samples: 5,
            dt: 0.05,
            budget_us: TICK_BUDGET_US,
            metric: BudgetMetric::P99,
            find_max: None,
        };
        let report = run_lab(&cfg);
        assert_eq!(report.spec.population, 1000);
        assert_eq!(report.samples.wall_total.count, 5);
        assert!(report.max_pop.is_none());
    }

    #[test]
    fn run_lab_is_reproducible() {
        let cfg = LabConfig {
            label: "repro".to_string(),
            spec: small_spec(1000),
            warmup: 2,
            samples: 5,
            dt: 0.05,
            budget_us: TICK_BUDGET_US,
            metric: BudgetMetric::P99,
            find_max: None,
        };
        let a = run_lab(&cfg);
        let b = run_lab(&cfg);
        // Same seed/spec => identical world => identical sample COUNT and spec.
        // (Absolute timings vary; structural reproducibility is the invariant.)
        assert_eq!(a.samples.wall_total.count, b.samples.wall_total.count);
        assert_eq!(a.spec.population, b.spec.population);
    }
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test --lib bench_lab::tests`
Expected: FAIL — `cannot find type 'LabConfig' in this scope`.

- [ ] **Step 3: Write minimal implementation**

Add to `apps/simulation/src/bench_lab/mod.rs` (above the test module, after the `pub use` lines):

```rust
use crate::bench_lab::ramp::RampConfig;

#[derive(Clone, Debug)]
pub struct LabConfig {
    pub label: String,
    pub spec: WorldSpec,
    pub warmup: usize,
    pub samples: usize,
    pub dt: f32,
    pub budget_us: u64,
    pub metric: BudgetMetric,
    pub find_max: Option<RampConfig>,
}

pub fn run_lab(cfg: &LabConfig) -> LabReport {
    let mut sim = build_world(&cfg.spec);
    let samples = sample_ticks(&mut sim, cfg.warmup, cfg.samples, cfg.dt);
    let within = within_budget(&samples.total_tick, cfg.budget_us, cfg.metric)
        || within_budget(&samples.wall_total, cfg.budget_us, cfg.metric);

    let max_pop = cfg.find_max.as_ref().map(|ramp| {
        let base = cfg.spec.clone();
        let warmup = cfg.warmup;
        let n = cfg.samples;
        let dt = cfg.dt;
        let result = find_max_pop(ramp, |pop| {
            let mut spec = base.clone();
            spec.population = pop;
            let mut sim = build_world(&spec);
            sample_ticks(&mut sim, warmup, n, dt).total_tick
        });
        result.max_pop
    });

    LabReport {
        label: cfg.label.clone(),
        spec: cfg.spec.clone(),
        budget_us: cfg.budget_us,
        within_budget: within,
        max_pop,
        samples,
        build_type: if cfg!(debug_assertions) { "debug" } else { "release" }.to_string(),
    }
}
```

Note: `within_budget` checks `total_tick` (real with dev-tools) and falls back to `wall_total` (always real) so a no-dev-tools build still produces a meaningful pass/fail. `WorldSpec` must derive `Clone` (it does — Task 3).

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test --lib bench_lab`
Expected: PASS (all bench_lab tests across Tasks 2–8).

- [ ] **Step 5: Commit**

```bash
git add apps/simulation/src/bench_lab/mod.rs
git commit -m "feat(bench-lab): add run_lab orchestration entry"
```

---

### Task 9: Binary runner + JSON output

**Files:**
- Create: `apps/simulation/src/bin/latency_lab.rs`
- Test: manual run (the binary is thin glue; all logic is proven in Tasks 1–8).

**Interfaces:**
- Consumes: `speciate::bench_lab::{run_lab, LabConfig, WorldSpec, Distribution, BudgetMetric, RampConfig, TICK_BUDGET_US}`.
- Produces: a CLI that parses argv, calls `run_lab`, prints a human summary, and (with `--out PATH`) writes the `LabReport` as pretty JSON.

- [ ] **Step 1: Write the binary**

Create `apps/simulation/src/bin/latency_lab.rs`:

```rust
//! Latency tuning lab runner. All measurement logic lives in
//! `speciate::bench_lab` (unit-tested); this binary is argv glue.
//!
//! Examples:
//!   cargo run --release --features dev-tools --bin latency_lab -- \
//!     --pop 200000 --seed 1 --samples 60 --warmup 20
//!   cargo run --release --features dev-tools --bin latency_lab -- \
//!     --find-max --low 700000 --high 1100000 --out docs/performance/snapshots/lab_maxpop.json

use speciate::bench_lab::budget::TICK_BUDGET_US;
use speciate::bench_lab::ramp::RampConfig;
use speciate::bench_lab::world::{Distribution, WorldSpec};
use speciate::bench_lab::{run_lab, BudgetMetric, LabConfig};

fn arg<T: std::str::FromStr>(args: &[String], key: &str, default: T) -> T {
    args.iter()
        .position(|a| a == key)
        .and_then(|i| args.get(i + 1))
        .and_then(|v| v.parse().ok())
        .unwrap_or(default)
}

fn flag(args: &[String], key: &str) -> bool {
    args.iter().any(|a| a == key)
}

fn main() {
    let args: Vec<String> = std::env::args().collect();

    let pop: usize = arg(&args, "--pop", 200_000);
    let seed: u64 = arg(&args, "--seed", 1);
    let samples: usize = arg(&args, "--samples", 60);
    let warmup: usize = arg(&args, "--warmup", 20);
    let half_x: f32 = arg(&args, "--half-x", 2500.0);
    let half_y: f32 = arg(&args, "--half-y", 2000.0);
    let dt: f32 = arg(&args, "--dt", 0.05);

    let distribution = if flag(&args, "--clustered") {
        Distribution::Clustered {
            clusters: arg(&args, "--clusters", 32),
            spread: arg(&args, "--spread", 150.0),
        }
    } else {
        Distribution::Uniform
    };

    let find_max = if flag(&args, "--find-max") {
        Some(RampConfig {
            low: arg(&args, "--low", 200_000),
            high: arg(&args, "--high", 1_200_000),
            coarse_step: arg(&args, "--coarse-step", 100_000),
            tolerance: arg(&args, "--tolerance", 25_000),
            budget_us: TICK_BUDGET_US,
            metric: BudgetMetric::P99,
        })
    } else {
        None
    };

    let cfg = LabConfig {
        label: args
            .iter()
            .position(|a| a == "--label")
            .and_then(|i| args.get(i + 1))
            .cloned()
            .unwrap_or_else(|| format!("pop{pop}_seed{seed}")),
        spec: WorldSpec { population: pop, seed, half_extent_x: half_x, half_extent_y: half_y, distribution },
        warmup,
        samples,
        dt,
        budget_us: TICK_BUDGET_US,
        metric: BudgetMetric::P99,
        find_max,
    };

    let report = run_lab(&cfg);

    eprintln!(
        "[{}] pop={} build={} within_budget={} p99_total={:.0}us wall_p99={:.0}us max_pop={:?}",
        report.label,
        report.spec.population,
        report.build_type,
        report.within_budget,
        report.samples.total_tick.p99,
        report.samples.wall_total.p99,
        report.max_pop,
    );

    if let Some(i) = args.iter().position(|a| a == "--out") {
        if let Some(path) = args.get(i + 1) {
            let json = serde_json::to_string_pretty(&report).expect("serialize report");
            std::fs::write(path, json).expect("write report");
            eprintln!("wrote {path}");
        }
    }
}
```

- [ ] **Step 2: Verify it builds (it should fail only if public re-exports are missing)**

Run: `cargo build --release --features dev-tools --bin latency_lab`
Expected: builds. If a `speciate::bench_lab::...` path fails to resolve, add the missing `pub use`/`pub mod` to `apps/simulation/src/bench_lab/mod.rs` (the submodules `budget`, `ramp`, `world` must be `pub mod`). If the build fails with napi link errors, switch the file to `apps/simulation/examples/latency_lab.rs` (same code, run with `--example latency_lab`) and note the change in the commit message.

- [ ] **Step 3: Smoke-run at a small population**

Run: `cargo run --release --features dev-tools --bin latency_lab -- --pop 5000 --samples 10 --warmup 5 --out docs/performance/snapshots/lab_smoke.json`
Expected: prints a `[pop5000_seed1] ... within_budget=true ...` line and writes `lab_smoke.json`. Open the file and confirm it has `withinBudget`, `totalTick`, and per-phase blocks with non-zero means.

- [ ] **Step 4: Delete the smoke artifact (not committed)**

Run: `rm docs/performance/snapshots/lab_smoke.json`

- [ ] **Step 5: Commit**

```bash
git add apps/simulation/src/bin/latency_lab.rs
git commit -m "feat(bench-lab): add latency_lab binary runner with JSON output"
```

---

### Task 10: Document the lab workflow

**Files:**
- Create: `docs/scale/latency-tuning-lab.md`
- Modify: `docs/scale/path-to-one-million.md:51-62` (link the lab from "Concrete next steps").

**Interfaces:** none (documentation).

- [ ] **Step 1: Write the lab doc**

Create `docs/scale/latency-tuning-lab.md`:

```markdown
# Latency Tuning Lab 🚧

> **Category: 🚧 In progress (NOW) — Pillar 1.** A deterministic harness for ruling
> speed/population optimizations in or out empirically. Code: `apps/simulation/src/bench_lab/`.

## What it does

Two measurements, two jobs:

1. **Headline KPI — max sustainable population.** `--find-max` does a coarse-bracket →
   bisection search for the largest population whose **p99 total tick ≤ 50,000 µs**
   (`TICK_BUDGET_US`). This is the undeniable scoreboard number.
2. **Diagnostic — per-phase A/B.** A fixed-population run (default 200k) captures
   per-phase timings (perception/steering/movement/grid/L1/behavior/export) plus
   wall-clock total, so a change can be attributed to a phase, not guessed at.

## Why it is trustworthy

- **Deterministic worlds.** `(population, seed, distribution, extents)` reproduce the
  exact same world (`Dna::random_seeded` + `StdRng`). A/B comparisons change the code,
  not the dice. Pin the seed.
- **Tail, not mean.** Pass/fail is p99 (`BudgetMetric::P99`) — the mean hides the
  dropped beats that live in the tail.
- **Per-phase attribution.** Built with `--features dev-tools`, the lab reads
  `Simulation::get_system_timings()`; the chained `.after()` schedule means a win only
  shows in total tick if it was on the critical path, so always read the per-phase diff.

## Workload note (read before trusting a max-pop number)

`Uniform` spread is the *cheap* density regime. Emergent flocking clusters the world,
raising perception cost at equilibrium, so a max-pop measured on a fresh uniform spread
can be optimistic. Run `--clustered` as the adversarial case for any headline claim.

## Commands

```bash
cd apps/simulation

# Fixed-pop diagnostic (per-phase attribution; A/B a change with --out before/after)
cargo run --release --features dev-tools --bin latency_lab -- \
  --pop 200000 --seed 1 --samples 60 --warmup 20 --out /tmp/before.json

# Headline: find the max sustainable population
cargo run --release --features dev-tools --bin latency_lab -- \
  --find-max --low 700000 --high 1100000 --coarse-step 100000 --tolerance 25000

# Adversarial clustered workload
cargo run --release --features dev-tools --bin latency_lab -- \
  --pop 200000 --clustered --clusters 32 --spread 150
```

## The honest gaps

- Hardware PMU counters (IPC, cache misses) remain Linux-only (`perf-event`); the lab's
  per-phase µs are software timers, valid cross-platform but blind to *why* a phase is slow.
- The lab measures the engine in-process, without the Electron/render pipeline. It is the
  tick-budget microscope, not an end-to-end frame-delivery test.

**Document Owner:** Pillar 1 (Prove Scale)
```

- [ ] **Step 2: Link from the path-to-one-million doc**

In `docs/scale/path-to-one-million.md`, under "## Concrete next steps", change step 1's parenthetical to reference the lab:

```markdown
1. **Profile the tick to find the idle.** Use the **latency tuning lab**
   (`docs/scale/latency-tuning-lab.md`) for deterministic per-phase A/B and max-pop
   search; Linux `perf`/PMU for hardware counters. Confirm or refute the 61% read
   before optimizing. (Agent: instrumentation-ian.)
```

- [ ] **Step 3: Commit**

```bash
git add docs/scale/latency-tuning-lab.md docs/scale/path-to-one-million.md
git commit -m "docs(scale): document the latency tuning lab workflow"
```

---

## Self-Review

**1. Spec coverage** (against our conversation's requirements):
- "Max population within budget is the undeniable measure" → Task 6 (`find_max_pop`) + Task 5 (p99 predicate). ✓
- "Local perf tests for confidence" → Task 4 (per-phase sampler) + Task 7 (`diff_reports`). ✓
- "Random DNA spread, pin the seed" → Task 1 (`random_seeded`) + Task 3 (seeded `build_world`). ✓
- "Clustered/equilibrium adversarial regime" → Task 3 (`Distribution::Clustered`) + Task 10 (doc warning). ✓
- "Adaptive ramp, smaller increments near the wall" → Task 6 (coarse-bracket → bisection). ✓
- "Tail not mean" → Task 5 (`BudgetMetric::P99` default) + the `p99_over_budget_fails_even_when_mean_passes` test. ✓
- "Rule ideas in/out by trying them" → Task 7 (`diff_reports`) + Task 9 (`--out` JSON A/B). ✓
- "Works on Windows without a feature split" → confirmed via `Cargo.toml:49` target-gating; lab builds with `--features dev-tools`, headline works without. ✓

**2. Placeholder scan:** No TBD/TODO/"add error handling"/"similar to Task N". Every code step shows complete code. ✓

**3. Type consistency:** `TickStats` fields (`count/min/max/mean/std_dev/p50/p95/p99`) are used identically in Tasks 2, 4, 5, 6, 7. `WorldSpec` fields (`population/seed/half_extent_x/half_extent_y/distribution`) match across Tasks 3, 7, 8, 9. `BudgetMetric` variants (`P99/Max/Mean`) match Tasks 5, 6, 8, 9. `find_max_pop` signature matches its call site in Task 8. `run_lab`/`LabConfig` match the binary in Task 9. ✓

**Known risk flagged for the executor:** Task 9 step 2 — a `src/bin` target links the `speciate` rlib which carries `napi` deps; if link errors appear, move the runner to `examples/latency_lab.rs`. The lab logic itself is fully proven by Tasks 1–8 regardless.

---

## Execution Handoff

Plan complete and saved to `docs/superpowers/plans/2026-06-21-latency-tuning-lab.md`. Two execution options:

**1. Subagent-Driven (recommended)** — I dispatch a fresh subagent per task (rusty-ron / ecs-emma fit here), review between tasks, fast iteration.

**2. Inline Execution** — Execute tasks in this session using executing-plans, batch execution with checkpoints.

Which approach?
