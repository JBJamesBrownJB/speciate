# Per-System Update Frequency Control

**Status:** ✅ Implemented (Phase C)
**Location:** `apps/simulation/src/simulation/core/frequency_throttle.rs`, `perception/systems.rs`, `behaviors/transitions/systems.rs`

---

## What It Does

Runtime-adjustable update frequency for cognitive ECS systems (perception, behavior) using entity-ID bucketing with bitwise AND optimization.

---

## Why It Exists

**Performance scaling:** Cognitive systems (perception, behavior) can tolerate stale data. By processing only a fraction of entities per tick, we reduce CPU usage proportionally while maintaining smooth simulation.

**Why NOT physics systems:** Movement and spatial grid rebuild must run every tick - physics integration requires temporal consistency, and perception accuracy depends on current positions.

---

## Key Design Decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| Bucketing strategy | Entity-ID | No visual artifacts (creatures in same cell don't update together), decoupled from spatial grid |
| Divisor values | Power-of-2 (2, 4, 8) | Enables bitwise AND (1 cycle) vs modulo (30 cycles) |
| Minimum divisor | 2 (no "full rate") | Cache line contention and branch prediction issues at divisor=1 caused 20% latency variance |
| Steering throttling | Removed | Caused jerky movement, not worth the savings |

---

## Implementation Location

| Component | File |
|-----------|------|
| FreqConfig resource | `core/components.rs:144-172` |
| FrequencyThrottle helper | `core/frequency_throttle.rs` |
| Perception throttling | `perception/systems.rs:103-126` |
| Behavior throttling | `behaviors/transitions/systems.rs:23-33` |
| Dev-UI controls | `apps/dev-ui/src/components/SystemTimingsPanel.tsx:186-220` |
| IPC boundary clamp | `napi_addon/simulation_engine.rs` (setSystemFrequency) |

---

## How It Works

Entity-ID bucketing distributes entities evenly across ticks:
- Each tick processes entities where `entity.index() & (divisor-1) == tick & (divisor-1)`
- Power-of-2 divisors enable bitwise AND (1 cycle vs 30 cycles for modulo)
- Entities keep stale data when skipped (don't clear neighbor cache)

---

## Systems Controlled

| System | Throttled? | Rationale |
|--------|------------|-----------|
| Perception | Yes | Stale data acceptable (reaction time) |
| Behavior Transition | Yes | Decision-making, not physics |
| Steering | No | Removed - caused jerky movement |
| Movement | Never | Physics integration requires every-tick |
| Spatial Grid | Never | Perception accuracy depends on current positions |

---

## Integration

- **Dev-UI:** Dropdown selectors (÷2, ÷4, ÷8) below perception/behavior sparklines
- **IPC:** `setSystemFrequency(system, divisor)` command, clamped to power-of-2
- **Debug target:** Bypasses throttling to prevent visualization flashing

---

## Additional Optimization

**select_nth_unstable:** During Phase C, also replaced 40-line manual max-heap with stdlib's `select_nth_unstable` for neighbor selection. Benchmark showed 1.7x speedup in typical 15-30 candidate range.

---

## Future Work

**Behavior timing opacity:** Behavior transition latency appears unchanged when throttling because Vec collection overhead (200K entities) dominates the trivial per-entity work. Throttle IS working, just not visible in timing metrics.
