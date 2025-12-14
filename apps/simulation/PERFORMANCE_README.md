# Performance Optimization Guide

## TL;DR - The Hidden Gem

**Problem:** Perception component is 192 bytes (3 cachelines), destroying L1 cache locality.

**Solution:** Split into hot (16 bytes) + cold (168 bytes) components.

**Expected:** 22-24ms → 18-20ms tick time @ 200K creatures.

---

## Documents

1. **PERFORMANCE_ANALYSIS.md** - Deep technical analysis
   - Hardware counter interpretation
   - Memory layout issues
   - Cache miss attribution
   - SIMD opportunities
   - Component size audit

2. **OPTIMIZATION_ROADMAP.md** - Implementation guide
   - Exact code changes for each phase
   - File-by-file diffs
   - Validation protocol
   - Success criteria

3. **scripts/perf_baseline.sh** - Measurement tool
   - Hardware counter collection
   - IPC calculation
   - Cache miss rate analysis
   - Before/after comparison

4. **scripts/analyze_component_sizes.sh** - Memory layout analyzer
   - Component size report
   - Cacheline calculation
   - Memory impact @ 200K creatures

---

## Quick Start

### 1. Establish Baseline

```bash
cd /home/dev/dev/speciate/apps/simulation
cargo build --release

# Make scripts executable
chmod +x scripts/*.sh

# Run baseline measurement
./scripts/perf_baseline.sh 200000 10
```

**Expected Output:**
```
IPC: 2.5-4.0 (good for memory-bound workload)
L1 Miss Rate: 5-10% (target: < 5%)
LLC Miss Rate: 1-3% (target: < 1%)
```

### 2. Analyze Component Sizes

```bash
./scripts/analyze_component_sizes.sh
```

**Expected Output:**
```
Perception: 192 bytes ⚠️ BLOATED!
  Spans 3 cachelines
  35 MB @ 200K creatures
```

### 3. Implement Phase 1 (Perception Split)

Follow `OPTIMIZATION_ROADMAP.md` Phase 1:
- Split `Perception` into hot + cold components
- Update perception system to write both
- Update avoidance system to read both

### 4. Validate Improvement

```bash
cargo build --release
./scripts/perf_baseline.sh 200000 10
```

**Success Criteria:**
- L1 miss rate: < 5% (down from 5-10%)
- IPC: > 3.0 (up from 2.5-4.0)
- Tick time: < 21ms (down from 22-24ms)

---

## How to Read perf Output

### IPC (Instructions Per Cycle)

```
IPC < 0.8:  MEMORY BOUND - CPU stalled waiting on RAM
            ACTION: Reduce cache misses (component size, access pattern)

IPC 0.8-2.0: MODERATE - Mixed workload
             ACTION: Balance CPU and memory optimizations

IPC > 2.0:   EXCELLENT - Good SIMD/cache usage
             ACTION: Focus on algorithmic improvements
```

### Cache Miss Rates

```
L1 Miss > 5%:  LOCALITY VIOLATION - Components too large or scattered
               ACTION: Shrink hot components, improve iteration order

LLC Miss > 1%: RANDOM ACCESS - Pointer chasing, HashMap abuse
               ACTION: Use contiguous arrays, avoid indirection
```

### Branch Misprediction

```
Branch Miss > 10%: UNPREDICTABLE LOGIC - Confusing CPU branch predictor
                   ACTION: Reduce if/else in hot loops, use branchless math
```

---

## Advanced Profiling

### Hotspot Analysis (GUI)

```bash
# Record L1 cache misses with call stacks
perf record --call-graph dwarf -e L1-dcache-load-misses \
    timeout 30s ./target/release/simulation

# Visualize in Hotspot
hotspot perf.data
```

**What to Look For:**
1. Red bars on `perception::update_perception_system`
2. High weight on line accessing `perception.neighbors`
3. Confirm cache misses come from large array access

### Flamegraph (Alternative to Hotspot)

```bash
# Install tools
cargo install flamegraph

# Generate flamegraph
cargo flamegraph --release -- --creatures 200000 --ticks 100

# Open flamegraph.svg in browser
firefox flamegraph.svg
```

### samply (Firefox Profiler)

```bash
# Install samply
cargo install samply

# Record profile
samply record ./target/release/simulation --creatures 200000 --ticks 100

# Upload to profiler.firefox.com and analyze
```

---

## Common Issues

### "perf not found"

```bash
sudo apt-get install linux-tools-common linux-tools-generic linux-tools-$(uname -r)
```

### "Permission denied" (perf_event_paranoid)

```bash
# Temporary (until reboot)
sudo sysctl -w kernel.perf_event_paranoid=-1

# Permanent
echo 'kernel.perf_event_paranoid = -1' | sudo tee -a /etc/sysctl.conf
```

### "Binary not found"

```bash
cargo build --release
# Binary should be at: ./target/release/simulation
```

---

## Optimization Phases

### Phase 1: Perception Split (CRITICAL)
- **Effort:** 4-6 hours
- **Risk:** LOW
- **Expected:** 2-4ms gain
- **Files:** 4 files to modify
- **Details:** See `OPTIMIZATION_ROADMAP.md`

### Phase 2: Thread-Local Optimization
- **Effort:** 2-3 hours
- **Risk:** LOW
- **Expected:** 0.5-1ms gain
- **Files:** 1 file to modify
- **Details:** See `OPTIMIZATION_ROADMAP.md`

### Phase 3: Component Size Audit
- **Effort:** 4-8 hours
- **Risk:** MEDIUM
- **Expected:** TBD
- **Files:** Varies
- **Details:** See `OPTIMIZATION_ROADMAP.md`

---

## Success Metrics

### Current State
```
Tick Time: 22-24ms @ 200K creatures
IPC: 2.5-4.0
L1 Miss Rate: 5-10%
LLC Miss Rate: 1-3%
```

### Target State (After Phase 1)
```
Tick Time: < 20ms @ 200K creatures
IPC: > 3.0
L1 Miss Rate: < 5%
LLC Miss Rate: < 1%
```

### Stretch Goal (After All Phases)
```
Tick Time: < 18ms @ 200K creatures
IPC: > 3.5
L1 Miss Rate: < 3%
LLC Miss Rate: < 0.5%
```

---

## References

**Linux Performance Analysis:**
- Brendan Gregg's Linux Perf Tools: http://www.brendangregg.com/perf.html
- Intel Optimization Manual: https://software.intel.com/content/www/us/en/develop/articles/intel-sdm.html

**Cache-Friendly Data Structures:**
- Data-Oriented Design: https://www.dataorienteddesign.com/dodbook/
- Pitfalls of Object Oriented Programming (Tony Albrecht): https://harmful.cat-v.org/software/OO_programming/_pdf/Pitfalls_of_Object_Oriented_Programming_GCAP_09.pdf

**ECS Optimization:**
- Bevy ECS Performance Guide: https://bevyengine.org/learn/book/performance/
- Unity ECS Best Practices: https://docs.unity3d.com/Packages/com.unity.entities@1.0/manual/index.html

---

## Questions?

**For performance analysis questions:**
- Check `PERFORMANCE_ANALYSIS.md` for technical details
- Run `./scripts/perf_baseline.sh` to see your baseline

**For implementation questions:**
- Check `OPTIMIZATION_ROADMAP.md` for step-by-step guide
- Read code comments in modified files

**For profiling tool questions:**
- `perf --help`
- `man perf-stat`
- `man perf-record`
