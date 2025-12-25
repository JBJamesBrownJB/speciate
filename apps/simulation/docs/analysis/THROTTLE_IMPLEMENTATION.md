# Throttle Implementation Guide

## The Winner: Bitwise AND (Approach A)

Replace the current modulo implementation with bitwise AND for ~7-13x speedup.

## Before (Current - SLOW)
```rust
// systems.rs - perception system
let divisor = freq.perception_divisor.max(1) as usize;
let current_bucket = (physics_tick.get() as usize) % divisor;  // ← Once per tick (OK)

entities.par_iter_mut().for_each(|(entity, ...)| {
    // 200K times per tick:
    if (entity.index() as usize) % divisor != current_bucket {  // ← DIVISION (20-40 cycles!)
        return;
    }
    // perception logic...
});
```

**Problem:** `%` operator compiles to `div` instruction (20-40 cycles per entity × 200K = disaster)

## After (Fast - Bitwise AND)
```rust
// systems.rs - perception system
let divisor = freq.perception_divisor.max(1) as usize;
let bucket_mask = divisor - 1;  // ← Precompute mask (power-of-2 optimization)
let current_bucket = (physics_tick.get() as usize) & bucket_mask;  // ← Bitwise AND (1 cycle)

entities.par_iter_mut().for_each(|(entity, ...)| {
    // 200K times per tick:
    if (entity.index() as usize) & bucket_mask != current_bucket {  // ← AND (1 cycle!)
        return;
    }
    // perception logic...
});
```

**Benefit:** `&` (AND) is 1 cycle vs `%` (modulo) is 20-40 cycles → **20-40x faster per check**

## Validation Required

Add power-of-2 constraint to FreqConfig:

```rust
// ipc/sim_command.rs or wherever FreqConfig is validated
impl FreqConfig {
    pub fn validate(&self) -> Result<(), String> {
        // Perception divisor must be power of 2 for bitwise optimization
        if !self.perception_divisor.is_power_of_two() {
            return Err(format!(
                "perception_divisor must be power of 2 (got {}). Valid: 1, 2, 4, 8, 16, 32",
                self.perception_divisor
            ));
        }
        // ... other validations
        Ok(())
    }
}
```

**Current values already comply:**
- Default divisor: 8 (power of 2 ✓)
- Common values: 1, 2, 4, 8, 16 (all powers of 2 ✓)

## Files to Modify

1. `/home/dev/dev/speciate/apps/simulation/src/simulation/perception/systems.rs`
   - Line ~105: Change modulo to bitwise AND

2. `/home/dev/dev/speciate/apps/simulation/src/ipc/sim_command.rs` (or wherever FreqConfig lives)
   - Add `is_power_of_two()` validation for perception_divisor

3. (Optional) Apply same optimization to other throttled systems:
   - Behavior system
   - Steering system
   - Any other frequency-throttled loops

## Test Before/After

```bash
# Before (current implementation)
cd /home/dev/dev/speciate/apps/simulation
cargo build --release --features dev-tools
# Run simulation, check dev-ui "perception" timing

# After (bitwise AND)
# Implement changes above
cargo build --release --features dev-tools
# Run simulation, check dev-ui "perception" timing
# Expected: 7-13x faster
```

## Hardware Verification

```bash
cd /home/dev/dev/speciate/apps/simulation
chmod +x scripts/analyze_throttle_hardware.sh
./scripts/analyze_throttle_hardware.sh
```

**Look for:**
- IPC (Instructions Per Cycle): Should improve significantly
- Cycles per entity: Should drop from ~25-45 to ~3-5
- No increase in L1 cache misses

## Why Not Ticket Component?

**Ticket Approach (Approach B):**
- Adds 1 byte per entity (195KB for 200K creatures)
- Requires memory load (4-5 cycles vs 1 cycle for AND)
- Only **3-7x faster** than modulo (vs 7-13x for bitwise)

**When to use Ticket:**
- If you need non-power-of-2 divisors (e.g., divisor=3, 5, 7)
- Currently no biological reason to use non-power-of-2 values

**Bottom line:** Bitwise AND is faster AND simpler (no extra component).

## Implementation Checklist

- [ ] Replace `% divisor` with `& bucket_mask` in perception system (line ~120)
- [ ] Precompute `bucket_mask = divisor - 1` outside parallel loop (line ~105)
- [ ] Add `is_power_of_two()` validation to FreqConfig
- [ ] Run hardware analysis script to confirm speedup
- [ ] Update dev-ui metrics to show perception timing improvement
- [ ] (Optional) Apply to behavior/steering systems
- [ ] Document decision in Sprint 16 notes
