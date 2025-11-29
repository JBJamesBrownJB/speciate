# Brain & Decision Timing - Reaction Speeds

**Status:** 🟡 Partially Implemented (Core cooldown system working, DNA-driven variation pending)

**Code:** `apps/simulation/src/simulation/creatures/components/brain.rs`

---

## Core Concept

Animal brains operate in **discrete sampling cycles**, not continuous streams. Visual system integrates over 40-80ms windows (gamma oscillations 25-40 Hz).

**Implementation:** Brain component with dynamic cooldown determines when creature can make new decisions.

---

## Brain Component

**Status:** ✅ Implemented

```rust
pub struct Brain {
    pub mode: BrainMode,                  // Normal, Cycling, Dormant
    pub last_decision_time: f64,          // Seconds since simulation start
    base_cooldown_ms: f32,                // 150.0 ms (hardcoded)
}
```

**BrainMode:**
- `Normal`: Standard decision-making
- `Cycling`: Debug mode (rapid state changes for testing)
- `Dormant`: No decisions (hibernation/unconscious)

---

## Decision Cooldown System

**Status:** ✅ Implemented

**Base cooldown:** 150ms (all creatures currently identical)

**Age factor:**
```rust
age_factor = 1.0 + (age / MAX_AGE).powf(2.5) × 2.0
```
Older creatures react slower (neural degradation).

**Energy factor:**
```rust
energy_factor = 1.0 + (1.0 - energy_normalized).powf(2.0) × 1.5
```
Low-energy creatures react slower (fatigue).

**Effective cooldown:**
```rust
effective_ms = BASE_COOLDOWN_MS × age_factor × energy_factor
```

**Constants:** See `apps/simulation/src/simulation/creatures/components/brain.rs`
- `BASE_COOLDOWN_MS` - Base decision interval
- `AGE_SENSITIVITY` - How much age affects cooldown
- `MAX_AGE` - Age normalization factor

---

## Panic Override System

**Status:** ✅ Implemented

**Trigger:** Threat within `body_size × PANIC_THRESHOLD`

```rust
pub fn should_panic(&self, threat_distance: f32, body_size: f32) -> bool {
    threat_distance < (body_size × 2.0)  // PANIC_THRESHOLD = 2.0
}
```

**Behavior:** Bypass normal decision cooldown, immediately switch to fleeing.

**Biological rationale:** Fight-or-flight response overrides deliberative processing (amygdala hijack).

---

## Size-Based Reaction Time (NOT Implemented)

**Target formula (from biology validation):**
```rust
reaction_time_ms = 68.0 + ((body_length - 0.5).max(0.0) / 19.5) × 932.0
```

**Examples:**
- 0.5m creature: 68ms (fast, responsive)
- 1m creature: 92ms (baseline wolf-sized)
- 5m creature: 283ms (medium, deliberate)
- 10m creature: 519ms (slow but powerful)
- 20m creature: 1000ms (massive, ponderous)

**Current status:** ❌ All creatures use same 150ms base regardless of size

**Biological basis:**
- Larger bodies = longer neural pathways = slower total processing
- Neural conduction velocity ~100 m/s (constant)
- Reflects real prey/predator reaction speeds

---

## Neural Speed Gene (NOT Implemented)

**DNA trait:** `neural_speed: f32` (0.5-2.0, default 1.0)

**Formula:**
```rust
modified_reaction_ms = (base_ms / dna.neural_speed).clamp(30.0, 1000.0)
```

**Trade-offs:**
- Fast processing (2.0) = 50% quicker reactions BUT +10% metabolism, prone to false positives
- Slow processing (0.5) = 2× slower reactions BUT low energy, deliberate decisions

**Metabolism costs:**
- Base: +1% per 0.1 above 1.0
- Active: +3% per 0.1 above 1.0

---

## Systemic Trade-Offs

### Fast Reactions (Small Creatures + High neural_speed)
✅ Dodge predators quickly
✅ React to threats immediately
❌ High metabolic cost (constant vigilance)
❌ Can't overpower larger creatures

### Slow Reactions (Large Creatures + Low neural_speed)
✅ Low metabolic cost (less frequent processing)
✅ Powerful when they do act
❌ Vulnerable to fast attackers during commit
❌ Can't chase agile prey effectively

**No god-tier combinations** - physics enforces trade-offs.

---

## Why NOT Different Tick Rates per Creature?

1. **Computational complexity:** Different update schedules = synchronization nightmares
2. **Fairness issues:** Smaller creatures get more "turns" per second (gaming the system)
3. **Emergence breaks:** Interactions unpredictable when entities operate on different timescales

**Solution:** All creatures tick at same rate (20Hz AI tick), but individual reaction times emerge from DNA.

---

## Biological Validation

**50ms matches neural sampling:**
- Insects: 15-30ms
- Small prey (mouse): 50-80ms ← Our baseline
- Medium predators (wolf): 80-150ms
- Large herbivores (deer): 150-230ms
- Megafauna (elephant): 300-600ms+

**Reference:** Visual gamma oscillations 25-40 Hz (neural "frame rates")

---

## Implementation Status

### ✅ Implemented
- Brain component with mode system
- Dynamic cooldown (age/energy factors)
- `can_decide()` gating function
- Panic override system
- Serialization-safe (last_decision_time intentionally skipped)

### ❌ Not Implemented
- Size-based reaction time formula (68-1000ms range)
- `neural_speed` DNA gene (0.5-2.0)
- Metabolic costs for fast processing
- Archetype-based specialization (fast prey vs slow predators)

**Current:** All creatures identical 150ms base cooldown

**Location:** `apps/simulation/src/simulation/creatures/components/brain.rs:1-89`

---

## Integration with Vision System

**When combined with stochastic vision (planned):**
- Brain cooldown determines **when** creature can decide
- Vision cooldown determines **when** creature perceives
- Perception may be stale when brain makes decision (biologically realistic)

Example: Large slow creature (500ms brain + 500ms vision) makes decisions based on half-second-old data → creates deliberate, ponderous behavior.
