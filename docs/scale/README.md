# 🚧 Scale — In Progress (NOW · Pillar 1)

> **Category: 🚧 IN PROGRESS (NOW).** This is an active **NOW-tier** pillar —
> work being built right now, not an idea, a backlog item, or a finished log. It
> is the home of **Pillar 1 — Prove Scale**.
>
> **Legend:** 📖 REFERENCE · 💡 IDEAS · 🚧 IN PROGRESS (NOW) · 📋 PLANNED · ✅ DONE · 🌙 DREAMLAND
>
> Cross-links: the authoritative NOW/NEXT/DREAM tiering is in
> [`../ROADMAP.md`](../ROADMAP.md) (see Pillar 1). The category convention is
> defined in [`../documentation-standards.md`](../documentation-standards.md). Closely related: the optimization
> lifecycle in [`../performance/`](../performance/) (perf = *how we make it
> fast*; scale = *the Pillar 1 NOW deliverables that prove it*).

---

# Pillar 1 - Prove Scale

> Can a Rust + Bevy ECS engine, fed to a web frontend over a zero-copy seam, credibly drive populations and worlds that most simulations cannot touch?

This folder is the home of **Pillar 1 (NOW tier)**: proving that the engine scales. It collects the metrics specification, the deliverables plan, and the documentation for the test, measurement, and CI infrastructure that turns "we got a big number once" into "we can demonstrate this number on demand, on every commit, on more than one OS."

For where this pillar sits among the four, see [`../ROADMAP.md`](../ROADMAP.md).

---

## Status

[![Target](https://img.shields.io/badge/target-1M%20creatures-blue)](../ROADMAP.md)
[![Linux](https://img.shields.io/badge/Linux-500K%20achieved-success)](../ROADMAP.md)
[![Windows](https://img.shields.io/badge/Windows-20K%20experimental-orange)](../ROADMAP.md)

> **These badges are static placeholders.** They are hand-set shields.io images, not live measurements. Making them *live* - regenerated from real benchmark runs in CI - is one of this pillar's deliverables (see Cross-OS CI below).

| Platform | Population | Status |
|----------|-----------|--------|
| **Target / stretch** | **1,000,000 creatures** | The art of the possible - the headline this engine is built to chase. |
| **Linux** | **500,000 creatures** | Actually tested. Validated achieved state. |
| **Windows** | **20,000 creatures** | Experimental. **Not officially supported.** Population ceiling far below Linux; root cause unknown and under investigation. |

The gap between Linux 500K and Windows 20K is itself a finding, not a footnote. Closing it (or explaining it) is in scope for this pillar.

---

## Why this pillar exists

Scale claims are the easiest thing to overstate and the easiest thing for an engineer to disprove. The honesty mandate for this project is simple: **engineers are reading.** So Pillar 1 is not "make the number bigger" - it is "make the number *defensible*":

- A **deterministic** simulation so a run can be reproduced and a regression can be bisected.
- A **metrics framework** that measures the right things (cache behavior, archetype stability, per-system tick cost) rather than just frames-per-second.
- A **live dashboard** so the numbers are observable, not anecdotal.
- **Cross-OS CI** so the badges above stop being promises and start being evidence.

The validated -> target -> stretch ladder (Linux 500K -> 1M target) is the whole point: show what is real, show where we are headed, and never blur the two.

---

## Deliverables

### 1. Deterministic test framework
A reproducible-run harness so that a given seed and configuration produces the same simulation every time. Determinism is the precondition for everything else in this pillar: without it, a "regression" is indistinguishable from noise, and a benchmark number cannot be trusted across machines or across commits.

### 2. Metrics framework + live dashboard
ECS-aware instrumentation that exposes Data-Oriented Design behavior - archetype/table layout, cache hit/miss patterns, per-system tick budget against the tick deadline - surfaced on a **live dashboard** (developer-facing; this belongs in `dev-ui`, never in the player-facing portal). The goal is to answer "is our data layout helping or hurting?" at a glance, in real time, at population.

- [`METRICS_DELIVERABLES.md`](./METRICS_DELIVERABLES.md) - the deliverables summary: what the metrics system covers and the implementation plan.
- [`ecs-metrics-specification.md`](./ecs-metrics-specification.md) - the full specification: which metrics, why, and the <1ms-per-tick collection budget they must respect.

### 3. Windows + Linux CI
Continuous integration that builds and runs the scale benchmarks on **both** Linux and Windows, captures the achieved population/throughput, and **regenerates the status badges from real runs.** This is what converts the static placeholders above into live status, and it is the structural fix for the Windows 20K mystery: a Windows runner that exercises the engine on every commit turns "root cause unknown" into "root cause observable."

---

## Known constraint: instrumentation is Linux-centric

The current hardware-counter instrumentation is **Linux-only by construction.** Cache-miss / IPC / hardware-event collection is built on `perf_event` and gated with `#[cfg(target_os = "linux")]`; on every other platform it compiles to a no-op stub.

See [`../../apps/simulation/src/instrumentation/hardware_metrics.rs`](../../apps/simulation/src/instrumentation/hardware_metrics.rs) - the `#[cfg(target_os = "linux")]` perf-event path versus the `#[cfg(not(target_os = "linux"))]` stub.

Implication for Pillar 1: the metrics story is strong on Linux (perf / eBPF / hardware counters) but **does not yet exist on Windows.** Part of this pillar's work is therefore a Windows-native counter path (or, at minimum, graceful degradation so the dashboard and CI still report meaningful timing/throughput metrics on Windows without the Linux-only hardware events). This is also directly relevant to investigating the Windows 20K ceiling - we currently have the least visibility on the platform where the engine performs worst.

---

## How we got the numbers we have

The achieved Linux 500K figure rests on the engine's optimization work, documented in the playbook:

- [`../architecture/ecs-optimization-playbook.md`](../architecture/ecs-optimization-playbook.md) - the ECS optimization techniques (Rayon movement parallelization ~6.3x across all cores, the two-level spatial grid at L0 20m / L1 60m, power-of-2 frequency throttling, and capability-marker ECS for archetype stability) that make the population numbers possible.

For the engine architecture as a whole, and the zero-copy NAPI Float32Array seam that lets Rust throughput reach the web frontend without a serialization tax, see [`../architecture/`](../architecture/) and [`../architecture/rust-js-thesis.md`](../architecture/rust-js-thesis.md).

---

## In this folder

| Document | Purpose |
|----------|---------|
| [`METRICS_DELIVERABLES.md`](./METRICS_DELIVERABLES.md) | Metrics system deliverables summary and implementation plan. |
| [`ecs-metrics-specification.md`](./ecs-metrics-specification.md) | Full ECS metrics specification (DOD-focused, cache-conscious). |
