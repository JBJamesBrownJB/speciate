# Hardware Counter Reference Card

**Purpose:** Quick lookup for interpreting `perf stat` output during sprint A-B-C measurement.

**Analyst:** claude-perf
**Target Platform:** 16-core AMD/Intel x86_64 @ 3.2GHz

---

## Core Metrics

### IPC (Instructions Per Cycle)

**Formula:** `instructions / cycles`

**Interpretation:**

| IPC | Diagnosis | Action |
|-----|-----------|--------|
| < 0.8 | Memory-bound (CPU stalled on RAM) | Check LLC miss rate, optimize data layout |
| 0.8-1.5 | Memory-bound (L1/L2 cache pressure) | Check L1D miss rate, improve locality |
| 1.5-2.5 | Balanced (good cache usage) | Nominal - continue monitoring |
| 2.5-4.0 | SIMD/parallelism effective | Excellent - Rayon working well |
| > 4.0 | Superscalar execution saturated | CPU fully utilized, bottleneck elsewhere |

**Current Baseline (360K creatures):** 1.68 (memory-bound)

**Phase A Target:** 1.8-2.0 (early-exit reduces memory stalls)

**Phase B Target:** 2.0+ (L1 drive scan is compute-bound)

---

### L1 Data Cache Miss Rate

**Formula:** `L1-dcache-load-misses / L1-dcache-loads × 100`

**Interpretation:**

| Miss Rate | Diagnosis | Action |
|-----------|-----------|--------|
| < 1% | Excellent locality | Optimal - data fits in L1 |
| 1-3% | Good locality | Acceptable for ECS iteration |
| 3-5% | Moderate misses | Check struct layout, padding |
| 5-10% | Poor locality | Fat components, random access |
| > 10% | Pathological | Pointer chasing, HashMap abuse |

**Current Baseline:** 3.4% (acceptable for ECS iteration)

**Phase A Target (L1 aggregation):** < 1% (sequential scan, cache-friendly)

**Phase B Target (L1 drive scan):** < 3% (BioSignatures are small, 8 bytes)

---

### LLC (L3) Miss Rate

**Formula:** `LLC-load-misses / LLC-loads × 100`

**Interpretation:**

| Miss Rate | Diagnosis | Action |
|-----------|-----------|--------|
| < 0.5% | Excellent | Data resident in L3 |
| 0.5-2% | Good | Normal for large working sets |
| 2-5% | Moderate DRAM stalls | Working set > L3 size (check RSS) |
| > 5% | Heavy DRAM traffic | Reduce working set, use compression |

**Current Baseline:** 3.0% (high - 360K creatures × 200 bytes/creature = 72MB working set)

**Phase A Target:** 2.5% (early-exit reduces L0 scan frequency)

**L3 Cache Size (typical):** 32MB (shared across all cores)

**Working Set Estimate:**
- 360K creatures × 200 bytes/creature (components) = 72MB
- Spatial grid proxies: 360K × 24 bytes = 8.6MB
- Total: ~80MB (exceeds L3 → LLC misses expected)

---

### Branch Miss Rate

**Formula:** `branch-misses / branch-instructions × 100`

**Interpretation:**

| Miss Rate | Diagnosis | Action |
|-----------|-----------|--------|
| < 1% | Excellent | Predictable control flow |
| 1-3% | Good | Normal for complex logic |
| 3-5% | Moderate | Check conditionals, loops |
| > 5% | Poor prediction | Random branching, hash collisions |

**Current Baseline:** 0.02% (excellent - predictable ECS iteration)

**Target:** < 1% (maintain predictability)

---

## Advanced Metrics

### Frontend Stall Ratio

**Formula:** `stalled-cycles-frontend / cycles`

**Interpretation:**

| Ratio | Diagnosis | Action |
|-------|-----------|--------|
| < 20% | Nominal | CPU fed efficiently |
| 20-40% | Moderate stalls | Instruction cache pressure, branch mispredictions |
| > 40% | Heavy stalls | Check branch miss rate, I-cache misses |

**Current Baseline:** 42.7% (high - memory-bound workload)

**Why It's High:** Memory stalls (waiting on LLC misses) propagate to frontend stalls.

**Phase A Target:** 30-35% (early-exit reduces memory stalls)

---

### Backend Stall Ratio

**Formula:** `stalled-cycles-backend / cycles`

**Interpretation:**

| Ratio | Diagnosis | Action |
|-------|-----------|--------|
| 0% | Excellent | No execution unit saturation |
| < 20% | Good | Normal for compute-heavy code |
| > 20% | Execution bottleneck | Check for div/sqrt, long-latency ops |

**Current Baseline:** 0% (no backend stalls - memory-bound, not compute-bound)

---

## Perf Command Cheat Sheet

### Basic Health Check

```bash
perf stat -e instructions,cycles,L1-dcache-loads,L1-dcache-load-misses,LLC-loads,LLC-load-misses,branch-misses \
  timeout 10s ./target/release/sim_app
```

**Output:**
- IPC: `instructions / cycles`
- L1D Miss Rate: `L1-dcache-load-misses / L1-dcache-loads × 100`
- LLC Miss Rate: `LLC-load-misses / LLC-loads × 100`
- Branch Miss Rate: `branch-misses / branch-instructions × 100`

---

### Cache Profiling (Detailed)

```bash
perf stat -e L1-dcache-loads,L1-dcache-load-misses,L1-dcache-stores,L1-dcache-store-misses,\
L1-icache-loads,L1-icache-load-misses,LLC-loads,LLC-load-misses,LLC-stores,LLC-store-misses \
  timeout 10s ./target/release/sim_app
```

**Use Case:** Validate L1 aggregation or drive scan cache behavior.

---

### Hotspot Analysis (Flamegraph)

```bash
perf record --call-graph dwarf -F 999 -o profile.perf.data \
  timeout 10s ./target/release/sim_app

perf script -i profile.perf.data | \
  stackcollapse-perf.pl | \
  flamegraph.pl > flamegraph.svg
```

**Use Case:** Find exact function/loop causing cache misses or CPU cycles.

---

### Cache Miss Attribution (Source Lines)

```bash
perf record --call-graph dwarf -e L1-dcache-load-misses \
  timeout 10s ./target/release/sim_app

# Open in Hotspot GUI (maps misses to source code)
hotspot profile.perf.data
```

**Use Case:** Pinpoint exact line causing L1 misses (e.g., struct field access).

---

## Phase-Specific Checklists

### Phase A: L1 Aggregation

**Measure:**
```bash
perf stat -e cycles,instructions,L1-dcache-load-misses,LLC-load-misses \
  timeout 10s ./target/release/sim_app --creatures 360000
```

**Gates:**
- [ ] L1 aggregation time < 0.5ms (extract from telemetry)
- [ ] L1D miss rate < 1% (check perf output)
- [ ] IPC increases from 1.68 → 1.8+ (early-exit benefit)

---

### Phase A: Early-Exit Optimization

**Measure (Sparse):**
```bash
perf stat -e cycles,instructions,L1-dcache-load-misses \
  timeout 10s ./target/release/sim_app --spec sparse_test.toml
```

**Compare to Uniform:**
```bash
perf stat -e cycles,instructions,L1-dcache-load-misses \
  timeout 10s ./target/release/sim_app --creatures 10000
```

**Gates:**
- [ ] Sparse scenario: 50%+ reduction in perception time
- [ ] IPC: Sparse > Uniform (less memory-bound)
- [ ] L1 misses: Sparse < Uniform (fewer L0 scans)

---

### Phase C: Zero Overhead (divisor=1)

**Measure:**
```bash
# Baseline (no frequency control)
perf stat -r 5 -e cycles,instructions \
  timeout 10s ./target/release/sim_app --no-frequency-control

# Divisor=1 (with frequency control code)
perf stat -r 5 -e cycles,instructions \
  timeout 10s ./target/release/sim_app --perception-divisor 1
```

**Gates:**
- [ ] Cycles: Within 2% (use `-r 5` for 5 runs, check stddev)
- [ ] Instructions: Within 1% (compiler optimization check)

---

### Phase C: Throttling Scaling

**Measure:**
```bash
for DIV in 1 2 4 8; do
  perf stat -e cycles,instructions \
    timeout 10s ./target/release/sim_app --perception-divisor $DIV \
    2>&1 | tee divisor_$DIV.log
done
```

**Gates:**
- [ ] Divisor=2: 50% reduction in perception time
- [ ] Divisor=4: 75% reduction in perception time
- [ ] Linear scaling observed (plot divisor vs perception_us)

---

### Phase B: Drive Computation

**Measure:**
```bash
perf stat -e cycles,instructions,L1-dcache-loads,L1-dcache-load-misses \
  timeout 10s ./target/release/sim_app --enable-drive-system --creatures 360000
```

**Gates:**
- [ ] Drive computation < 2ms (extract from telemetry)
- [ ] IPC > 1.8 (compute-bound, not memory-bound)
- [ ] L1D miss rate < 3% (BioSignatures are cache-friendly)

---

### Phase B: Rayon Parallelization

**Measure:**
```bash
perf stat -e task-clock,context-switches -a \
  timeout 10s ./target/release/sim_app --enable-drive-system --creatures 360000
```

**Gates:**
- [ ] Task clock ~16× wall time (all cores engaged)
- [ ] Context switches < 10K (minimal thread overhead)
- [ ] IPC ~4.0 (matches movement system Rayon baseline)

---

## Interpretation Examples

### Example 1: Memory-Bound Workload

```
Performance counter stats:

    71,217,014,578      cycles
   119,615,669,600      instructions              #    1.68  insn per cycle
     2,847,804,922      L1-dcache-loads
         1,901,692      L1-dcache-load-misses     #    3.41% of all L1 loads
            87,389      LLC-load-misses           #    3.07% of all LLC loads
```

**Diagnosis:**
- IPC 1.68: Memory-bound (CPU waiting on cache)
- L1D 3.41%: Moderate misses (ECS iteration, acceptable)
- LLC 3.07%: DRAM stalls (working set > L3 cache)

**Action:** Early-exit optimization will reduce L0 scan frequency → fewer cache misses → higher IPC.

---

### Example 2: Compute-Bound Workload (Good)

```
Performance counter stats:

    20,000,000,000      cycles
    80,000,000,000      instructions              #    4.00  insn per cycle
     5,000,000,000      L1-dcache-loads
        50,000,000      L1-dcache-load-misses     #    1.00% of all L1 loads
            10,000      LLC-load-misses           #    0.20% of all LLC loads
```

**Diagnosis:**
- IPC 4.0: Excellent (Rayon parallelization effective)
- L1D 1.0%: Great locality (SIMD or tight loop)
- LLC 0.2%: Minimal DRAM access

**Result:** Optimal performance, no action needed.

---

### Example 3: Cache Thrashing (Bad)

```
Performance counter stats:

    30,000,000,000      cycles
    24,000,000,000      instructions              #    0.80  insn per cycle
    10,000,000,000      L1-dcache-loads
     1,000,000,000      L1-dcache-load-misses     #   10.00% of all L1 loads
        50,000,000      LLC-load-misses           #    5.00% of all LLC loads
```

**Diagnosis:**
- IPC 0.8: Severely memory-bound
- L1D 10%: Poor locality (random access, fat structs)
- LLC 5%: Heavy DRAM traffic

**Action:**
1. Profile with `perf record -e L1-dcache-load-misses`
2. Open in Hotspot → identify hot struct access
3. Refactor: pack frequently-accessed fields together
4. Consider component splitting (hot/cold data separation)

---

## Reference: Typical Cache Sizes

| Cache | Size (typical) | Latency | Bandwidth |
|-------|----------------|---------|-----------|
| L1D | 32KB per core | 4 cycles (~1.25ns) | 64 bytes/cycle |
| L1I | 32KB per core | 4 cycles (~1.25ns) | 32 bytes/cycle |
| L2 | 256KB per core | 12 cycles (~3.75ns) | 32 bytes/cycle |
| L3 (LLC) | 32MB shared | 40 cycles (~12.5ns) | 16 bytes/cycle |
| DRAM | 16GB+ | 200+ cycles (~60ns) | 2-4 bytes/cycle |

**Key Insight:** L1 miss → 3× penalty, LLC miss → 16× penalty.

---

## Anti-Patterns to Avoid

### 1. Premature Optimization

**Don't:** Optimize cache layout before profiling.

**Do:** Run `perf stat` baseline → identify bottleneck → optimize → re-measure.

---

### 2. Ignoring IPC

**Don't:** Focus only on wall-clock time (can be misleading).

**Do:** Check IPC first. If IPC < 1.5, it's a memory problem (optimize layout). If IPC > 2.5, it's CPU-bound (parallelize).

---

### 3. Trusting Compiler Magic

**Don't:** Assume compiler will optimize away modulo checks or branches.

**Do:** Validate with `perf stat` and `-r 5` for statistical significance.

---

### 4. Single-Run Measurements

**Don't:** Rely on one `perf stat` run (high variance).

**Do:** Use `-r 5` or `-r 10` for multiple runs, check stddev.

---

## Further Reading

- [Brendan Gregg's Perf Examples](http://www.brendangregg.com/perf.html)
- [Intel Optimization Manual](https://www.intel.com/content/www/us/en/developer/articles/technical/intel-sdm.html)
- [What Every Programmer Should Know About Memory](https://people.freebsd.org/~lstewart/articles/cpumemory.pdf)
- [Speciate Performance Docs](../done/rayon-parallelization.md)
