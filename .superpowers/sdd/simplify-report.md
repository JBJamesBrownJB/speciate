# bench-lab simplify/harden report

## Item 1: Zero-samples guard — DONE

Test `sample_ticks_clamps_zero_samples_to_one` written first (RED: count was 0). Implemented `let samples = samples.max(1);` at top of `sample_ticks`. Test went GREEN. Prevents `wall_total.count == 0` and the all-zero `TickStats` that would cause `within_budget` to spuriously return `true`.

File: `apps/simulation/src/bench_lab/sampler.rs`

## Item 2: percentile precondition — DONE

Added `debug_assert!(sorted.windows(2).all(|w| w[0] <= w[1]), "percentile requires sorted input");` at the top of `percentile()`, before the match. All 4 existing stats tests remain green. No test needed (debug-only assertion).

File: `apps/simulation/src/bench_lab/stats.rs`

## Item 3: Clustered-distribution coverage — DONE

Added `clustered_distribution_positions_are_localized` test alongside the existing count-only test. Uses `population: 120, clusters: 4, spread: 50.0` with a fixed seed (42). Assertions:
- Each intra-cluster pairwise diameter (cluster 0 and cluster 1 sampled via `step_by(4)`) is within `2 * spread * sqrt(2)` — the worst-case diagonal of a `2*spread × 2*spread` bounding box.
- The total x-span across all creatures exceeds `2 * spread`, proving the clusters are separated (not all collapsed to one point).

Deterministic via fixed seed; no flakiness risk.

File: `apps/simulation/src/bench_lab/world.rs`

## Item 4: DRY frequency setter — DONE

`bevy_app.rs` `SetSystemFrequency` branch replaced the inline clamp + match block with a single delegation call to `self.simulation.set_system_frequency(system.as_str(), divisor)`. The `eprintln!` confirmation log is preserved. The now-unused `FreqConfig` import was removed from the `use` line. The contexts are identical (same clamp, same field mapping, same `u8` type) — clean drop-in.

Files: `apps/simulation/src/ipc/bridge/bevy_app.rs`

## Item 5: bench_lab tidy — SCANNED, no changes needed

Scanned all 8 `bench_lab/*.rs` files. No dead imports, duplicated helpers, or copy-paste candidates found. `cargo check` reported zero warnings. Files are already lean from previous sprint work.

## Test results

`cargo test --lib`: **470 passed; 0 failed**
`cargo test --lib --features dev-tools`: **577 passed; 0 failed**
