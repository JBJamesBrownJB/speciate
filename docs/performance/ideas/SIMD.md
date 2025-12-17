# SIMD Optimization

**Status:** 📋 Backlog (Not Yet Implemented)
**Priority:** Low (Rayon parallelization already provides major speedup)

---

## What Is SIMD?

**Single Instruction, Multiple Data (SIMD)** allows a single CPU instruction to operate on multiple data elements simultaneously. Instead of processing one `f32` at a time, SIMD can process 4, 8, or 16 `f32` values in a single instruction.

**Example:** Adding 8 velocities to 8 positions in one instruction rather than 8 separate additions.

---

## Common Instruction Sets

| Instruction Set | Width | f32s per Op | CPU Support |
|-----------------|-------|-------------|-------------|
| **SSE** | 128-bit | 4 | All modern x86 (2000+) |
| **AVX** | 256-bit | 8 | Intel Sandy Bridge+ (2011+) |
| **AVX2** | 256-bit | 8 | Intel Haswell+ (2013+), AMD Excavator+ |
| **AVX-512** | 512-bit | 16 | Intel Skylake-X (2017+), select consumer CPUs |
| **NEON** | 128-bit | 4 | All ARM64 (Apple Silicon, Raspberry Pi 4+) |

**Reality:** Most modern desktops have AVX2. AVX-512 is datacenter/enthusiast only. Steam hardware survey confirms AVX2 as safe baseline for PC gaming.

---

## SIMD vs Rayon: Different Parallelism Levels

| Aspect | Rayon (Current) | SIMD (Potential) |
|--------|-----------------|------------------|
| **Parallelism Type** | Task parallelism | Data parallelism |
| **What Runs in Parallel** | Different creatures | Same operation, multiple values |
| **Scales With** | CPU cores (16 threads) | Register width (4-16 values) |
| **Implementation Effort** | Medium (collect + par_iter) | High (manual vectorization) |
| **Current Status** | ✅ Implemented (Sprint 15) | Not implemented |
| **Speedup Achieved** | 6.3x on movement | N/A |

**Key Insight:** Rayon and SIMD are complementary. Rayon parallelizes across creatures (threads), SIMD parallelizes within a single creature's math (registers). They can be combined.

---

## Where SIMD Would Help

### High-Impact Operations

1. **Position + Velocity integration** (bulk `f32` adds)
2. **Distance calculations** (squared distance between many entity pairs)
3. **Force accumulation** (summing multiple steering forces)
4. **Perception radius checks** (batch distance comparisons)

### Current Bottleneck Analysis

With Rayon already parallelizing movement (4.1ms at 10K creatures), the per-thread work is ~0.25ms. SIMD would accelerate this inner loop but diminishing returns apply:
- 4x theoretical speedup → 0.06ms per thread
- Actual gains typically 2-3x due to memory latency

**Conclusion:** SIMD is lower priority than spatial indexing and LOD systems.

---

## Rust SIMD Options

### 1. Auto-Vectorization (Current)

LLVM automatically vectorizes simple loops. Check with:
```bash
RUSTFLAGS="-C target-cpu=native" cargo build --release
```

### 2. `std::simd` (Nightly)

Rust's portable SIMD API (still unstable as of 2025):
- Requires `#![feature(portable_simd)]`
- Cross-platform abstraction
- May stabilize in 2025-2026

### 3. `wide` Crate (Stable)

Production-ready SIMD on stable Rust:
- `f32x4`, `f32x8` types
- AVX2/SSE fallback automatic
- Used by `glam` math library

### 4. `packed_simd2` (Stable)

Another stable option with broader type support.

---

## Detection and Fallback Strategy

### Compile-Time (Recommended)

Build multiple binaries or use feature flags:
```toml
[features]
avx2 = []
sse = []
```

### Runtime Detection

```rust
if is_x86_feature_detected!("avx2") {
    // Use AVX2 path
} else if is_x86_feature_detected!("sse4.1") {
    // Use SSE path
} else {
    // Scalar fallback
}
```

### Steam Distribution Strategy

- **Primary build:** AVX2 (covers 95%+ of gaming PCs)
- **Fallback build:** SSE-only (legacy systems)
- **Detection:** Check at startup, load appropriate DLL/so

---

## Implementation Priority

Given current architecture:

1. ✅ **Rayon parallelization** - Done (6.3x speedup)
2. 📋 **Spatial indexing** - Higher priority (O(n²) → O(n) perception)
3. 📋 **LOD systems** - Higher priority (reduce IPC payload)
4. 📋 **SIMD vectorization** - Lower priority (optimize inner loops)

**Recommendation:** Defer SIMD until after spatial indexing proves insufficient. The effort-to-gain ratio is unfavorable while larger architectural wins remain.

---

## When to Revisit

Consider SIMD optimization when:
- Creature count exceeds 100K
- Profiling shows per-thread physics > 1ms
- Spatial indexing and LOD are implemented
- Team has capacity for low-level optimization

---

## References

- Rust SIMD Guide: https://rust-lang.github.io/packed_simd/
- `wide` crate: https://docs.rs/wide
- Intel Intrinsics Guide: https://www.intel.com/content/www/us/en/docs/intrinsics-guide
- Current parallelization: `apps/simulation/src/simulation/movement/systems.rs:35-113`
