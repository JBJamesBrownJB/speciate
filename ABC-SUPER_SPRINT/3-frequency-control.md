# Sprint: System Update Frequency (Phase C) - COMPLETE

## Outcome

Runtime-adjustable Hz for cognitive systems using entity-ID bucketing with bitwise AND optimization.

**Depends on:** Phase A (Dual Grid) - for early-exit optimization only

**Reference:** `docs/performance/todo/system-update-frequency.md`

**Status:** ✅ COMPLETE

---

## Core Concept

Frequency control uses **entity ID bucketing** with **power-of-2 bitwise optimization**:

```rust
// FrequencyThrottle helper (core/frequency_throttle.rs)
pub struct FrequencyThrottle {
    bucket_mask: usize,
    current_bucket: usize,
}

impl FrequencyThrottle {
    pub fn new(divisor: u8, tick: u64) -> Self {
        let divisor = divisor as usize;
        Self {
            bucket_mask: divisor - 1,
            current_bucket: (tick as usize) & (divisor - 1),
        }
    }

    #[inline(always)]
    pub fn should_process(&self, entity_index: u32) -> bool {
        (entity_index as usize) & self.bucket_mask == self.current_bucket
    }
}
```

**Why bitwise AND instead of modulo:**
- Modulo (`%`): ~30 CPU cycles
- Bitwise AND (`&`): ~1 CPU cycle
- Requires power-of-2 divisors (2, 4, 8)

**Why minimum divisor is 2 (no "full rate" option):**
- Cache line false sharing: shared variables across Rayon workers cause variance at divisor=1
- Branch prediction pollution: even "always skip" branches affect predictor
- Solution: Always throttle (minimum divisor=2) eliminates these issues

**Why entity-ID instead of L1 cell position:**
- **No visual artifacts**: Creatures in same cell don't update simultaneously
- **Decoupled from grid**: Frequency control independent of spatial architecture
- **Uniform distribution**: Entity IDs naturally spread across buckets

---

## Systems Controlled

| System | Throttled? | Divisor Options | Rationale |
|--------|------------|-----------------|-----------|
| Perception | Yes | 2, 4, 8 | Stale data acceptable (reaction time) |
| Behavior Transition | Yes | 2, 4, 8 | Decision-making, not physics |
| Steering | **NO** | Removed | Steering throttling caused jerky movement |
| Movement | **NO** | N/A | Physics integration requires every-tick |
| L0/L1 Grid Rebuild | **NO** | N/A | Perception accuracy depends on current positions |

---

## Implementation Summary

### FreqConfig Resource
**File:** `core/components.rs`

```rust
#[derive(Resource, Clone, Copy, Debug)]
pub struct FreqConfig {
    pub perception_divisor: u8,
    pub behavior_divisor: u8,
    pub steering_divisor: u8,  // Always 1 (steering throttling removed)
}

impl Default for FreqConfig {
    fn default() -> Self {
        Self {
            perception_divisor: 2,  // Minimum 2 - no "off" option
            behavior_divisor: 2,    // Minimum 2 - no "off" option
            steering_divisor: 1,    // Keep 1 (steering throttling removed)
        }
    }
}

impl FreqConfig {
    /// Clamp divisor to power-of-2: 2, 4, or 8 (minimum 2, no "off" option).
    /// Power-of-2 required for bitwise AND optimization in FrequencyThrottle.
    pub fn clamp_power_of_2(divisor: u8) -> u8 {
        match divisor {
            0..=2 => 2,
            3..=4 => 4,
            _ => 8,
        }
    }
}
```

### FrequencyThrottle Helper
**File:** `core/frequency_throttle.rs`

Reusable helper extracted to eliminate code duplication between perception and behavior systems.

### System Usage Pattern
**Files:** `perception/systems.rs`, `behaviors/transitions/systems.rs`

```rust
// Create throttle once per system invocation
let throttle = FrequencyThrottle::new(freq.perception_divisor, physics_tick.get());

// Inside parallel loop
entities.par_iter_mut().for_each(|(entity, ...)| {
    if !throttle.should_process(entity.index()) {
        return;  // Skip this tick, keep stale data
    }
    // Normal processing
});
```

### Debug Target Bypass
**File:** `perception/systems.rs`

Debug target creature bypasses throttling to prevent visualization flashing:

```rust
#[cfg(feature = "dev-tools")]
let bypass_throttle = is_debug_target;
#[cfg(not(feature = "dev-tools"))]
let bypass_throttle = false;

if !bypass_throttle && !throttle.should_process(entity.index()) {
    return;
}
```

### Dev-UI Controls
**File:** `apps/dev-ui/src/components/SystemTimingsPanel.tsx`

Dropdown selectors (not sliders) for power-of-2 options:

```tsx
const DIVISOR_OPTIONS = [2, 4, 8] as const;

<select value={divisor} onChange={(e) => onChange(Number(e.target.value))}>
  {DIVISOR_OPTIONS.map(d => (
    <option key={d} value={d}>÷{d}</option>
  ))}
</select>
```

---

## Additional Optimizations

### select_nth_unstable for Neighbor Selection
**File:** `perception/systems.rs`

Replaced 40-line manual max-heap with stdlib's `select_nth_unstable`:

```rust
// Get K closest neighbors using partial sort (O(n) vs O(n log k))
let k = MAX_PERCEIVED_NEIGHBORS.min(candidates.len());
if k > 0 {
    if candidates.len() > k {
        candidates.select_nth_unstable_by(k - 1, |a, b| {
            a.0.total_cmp(&b.0)  // NaN-safe comparison
        });
        candidates.truncate(k);
    }

    for (_, neighbor) in candidates.drain(..) {
        neighbor_cache.add_neighbor(neighbor);
    }
}
```

**Benchmark results (1.7x faster):**

| Candidates | Heap (ns) | select_nth (ns) | Speedup |
|------------|-----------|-----------------|---------|
| 15         | 160.4     | 88.8            | 1.8x    |
| 20         | 200.1     | 116.5           | 1.7x    |
| 30         | 259.2     | 151.4           | 1.7x    |

---

## Performance Results

**Perception:** Visible latency reduction when throttling (heavy per-entity work)

**Behavior Transition:** Timing appears unchanged due to fixed overhead (Vec collection dominates trivial per-entity work). Throttle IS working but savings masked by overhead.

---

## Validation ✅

- [x] Bitwise AND replaces modulo (30 cycles → 1 cycle)
- [x] Power-of-2 clamping at IPC boundary
- [x] Dev-UI dropdown controls (2, 4, 8 options)
- [x] Debug target bypasses throttle (no visualization flashing)
- [x] select_nth_unstable replaces manual heap (1.7x faster)
- [x] Tests for FrequencyThrottle distribution
- [x] Tests for clamp_power_of_2

---

## Files Modified

| File | Change |
|------|--------|
| `core/components.rs` | FreqConfig resource, clamp_power_of_2 |
| `core/frequency_throttle.rs` | FrequencyThrottle helper (NEW) |
| `core/mod.rs` | Export FrequencyThrottle |
| `perception/systems.rs` | Throttle + select_nth_unstable + debug bypass |
| `behaviors/transitions/systems.rs` | Throttle using FrequencyThrottle |
| `napi_addon/simulation_engine.rs` | IPC clamp at boundary |
| `apps/dev-ui/.../SystemTimingsPanel.tsx` | Dropdown controls |
| `apps/dev-ui/src/index.css` | Dropdown styling |
