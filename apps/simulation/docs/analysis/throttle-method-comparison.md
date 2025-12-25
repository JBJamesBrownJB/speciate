# Throttle Method Hardware Analysis

## Question

For 200K entities updated every tick with frequency throttling (divisor=8), which approach is fastest?

**Approach A: Bitwise AND (power-of-2 divisors only)**
```rust
let bucket_mask = divisor - 1;  // precomputed once per tick
let current_bucket = tick & bucket_mask;

// Per entity (200K times):
if (entity.index() as usize) & bucket_mask != current_bucket {
    return;
}
```

**Approach B: Ticket Component (any divisor)**
```rust
let current_bucket = tick % divisor;  // computed once per tick

// Per entity (200K times):
if creature.ticket != current_bucket {
    return;
}
```

**Approach C: Modulo (current implementation)**
```rust
let current_bucket = tick % divisor;  // computed once per tick

// Per entity (200K times):
if (entity.index() as usize) % divisor != current_bucket {
    return;
}
```

## Hardware-Level Predictions

### Instruction Costs (x86-64)

| Operation | Assembly | Latency | Throughput | Notes |
|-----------|----------|---------|------------|-------|
| AND (A) | `and reg, reg` | 1 cycle | 0.25 cycles | Single ALU op |
| DIV (C) | `div reg` | 20-40 cycles | 6-10 cycles | **VERY EXPENSIVE** |
| MOD (C) | `div` + `idiv` | 20-40 cycles | 6-10 cycles | **Uses division** |
| Memory Load (B) | `mov reg, [mem]` | 4-5 cycles (L1) | 0.5 cycles | **If in cache** |
| Compare | `cmp reg, reg` | 1 cycle | 0.25 cycles | Same for all |
| Branch | `jne` | 1 cycle (predicted) | 0.5 cycles | Same for all |

### Critical Path Analysis

**Approach A (Bitwise):**
```
entity.index()  →  AND  →  CMP  →  Branch
   (register)    (1c)   (1c)    (1c)
   Total: ~3 cycles (best case)
```

**Approach B (Ticket):**
```
Load ticket  →  CMP  →  Branch
  (4-5c L1)    (1c)    (1c)
  Total: ~6-7 cycles (if cache hit)
```

**Approach C (Modulo):**
```
entity.index()  →  MOD  →  CMP  →  Branch
   (register)    (20-40c!) (1c)   (1c)
   Total: ~22-42 cycles (WORST)
```

### Cache Effects

**Approach A:**
- **Pros:** Entity struct already loaded for query iteration (zero extra cache pressure)
- **Cons:** None
- **Archetype impact:** None (uses existing Entity field)

**Approach B:**
- **Pros:** Ticket is u8 (tiny), likely packed into same cache line as Position
- **Cons:** Adds 1 byte to every creature (increases archetype size slightly)
- **Archetype impact:** Minimal (1 byte per entity = 200KB for 200K creatures)

**Approach C:**
- **Pros:** Entity struct already loaded (zero extra memory)
- **Cons:** **Division instruction is CPU-bound, not memory-bound**
- **Archetype impact:** None (uses existing Entity field)

### Branch Prediction

All three approaches have **identical** branch patterns:
- 7/8 entities skip (divisor=8)
- 1/8 entities process
- Pattern is predictable (sequential entity IDs)
- **Branch predictor will be equally effective**

## Prediction

**Expected Performance Ranking (200K entities, divisor=8):**

1. **Approach A (Bitwise): ~600 μs/tick**
   - Fastest: AND is 1 cycle, no division, no memory load
   - Constraint: Only works for power-of-2 divisors (2, 4, 8, 16)

2. **Approach B (Ticket): ~1.2 ms/tick**
   - Moderate: Memory load is 4-5 cycles (L1 hit)
   - Ticket likely in same cache line as Position (good locality)
   - Works for any divisor

3. **Approach C (Modulo): ~4-8 ms/tick**
   - **SLOWEST: Division is 20-40 cycles PER ENTITY**
   - 200K entities × 20-40 cycles = disaster
   - Current implementation is probably the bottleneck

**Speedup estimates:**
- A vs C: **7-13x faster**
- B vs C: **3-7x faster**
- A vs B: **2x faster**

## Memory Overhead

| Approach | Bytes Added | Total Overhead (200K) |
|----------|-------------|----------------------|
| A (Bitwise) | 0 | 0 KB |
| B (Ticket) | 1 byte/entity | ~195 KB |
| C (Modulo) | 0 | 0 KB |

## Recommendation

**Short-term (immediate fix):**
- **Switch to Approach A (Bitwise AND)** for perception_divisor
- Restrict divisor to power-of-2 values in FreqConfig validation
- Expected gain: **7-13x speedup** over current modulo implementation

**Long-term (flexibility):**
- If non-power-of-2 divisors are required, use **Approach B (Ticket)**
- Memory cost is negligible (195KB for 200K creatures)
- Still **3-7x faster** than modulo

**Never do:**
- Keep Approach C (Modulo) - it's a performance trap

## Testing Protocol

Run `/home/dev/dev/speciate/apps/simulation/scripts/analyze_throttle_hardware.sh`:

1. **Timing (user-space):** Measures actual tick duration
2. **perf stat:** Confirms instruction counts and IPC
3. **Assembly inspection:** Verifies AND vs DIV codegen

**Key metrics to validate prediction:**
- IPC (Instructions Per Cycle): A > B >> C
- Cycles per entity: A (~3) < B (~6-7) << C (~25-45)
- L1 miss rate: Similar for all (branch prediction dominates)

## Related Decisions

- Power-of-2 restriction is acceptable for frequency throttling
- Current divisor values (2, 4, 8, 16) already satisfy this constraint
- No biological reason to require divisor=3, 5, 7, etc.
