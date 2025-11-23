# Dual-Tick Architecture Archive

**Status:** ABANDONED (Sprint 11, November 2025)

## What's Here

This directory contains documentation for the abandoned dual-tick simulation architecture, preserved for reference and learning.

### Contents

- `dual-tick-simulation.md` - Complete technical specification
- `sprint-11-dual-tick-architecture_summary.md` - Sprint 11 implementation summary

## Why Abandoned

**Core Issue:** Sequential execution on a single thread provides no performance benefit.

### The Problem

The dual-tick approach attempted to run:
- Physics systems at 30Hz (cheap, frequent updates)
- AI systems at 20Hz (expensive, infrequent updates)

**Expected benefit:** Lighter frames between AI ticks would improve throughput.

**Actual result:** When schedules align at the LCM (every 100ms), both run together creating the same spike as a single-tick system. Must budget for worst-case anyway.

### Key Insight

**Dual-tick only benefits true parallelism:**
- Requires separate threads
- Requires lock-free data structures
- Significant architectural complexity

**For single-threaded execution:** The complexity isn't worth zero performance gain.

## What We Learned

1. **Tick separation doesn't help on single thread**
   - Must still budget for worst-case combined load
   - No performance benefit from "lighter" frames

2. **Real solution is frontend interpolation**
   - Lower simulation tick rate (20Hz)
   - Frontend interpolates to 90Hz for smooth visuals
   - Same scaling benefit, simpler architecture

3. **True parallelism requires major changes**
   - Lock-free ECS access patterns
   - Thread-safe component access
   - Complex synchronization
   - Deferred for Phase 2 (MMO architecture)

## What We Did Instead

**Sprint 12+:** Frontend interpolation approach
- Simulation at 20Hz (current: 22Hz)
- Frontend lerp() between frames for 60+ FPS visuals
- Achieves same 150K-200K creature scaling goal
- Much simpler architecture

## References

- Sprint 11 Summary: Exploration and benchmarking
- Sprint 12 Plan: Interpolation implementation
- Sprint 13: NAPI migration (unrelated but concurrent)

---

**Lesson:** Always benchmark assumptions. Sequential dual-tick sounded good on paper but provided zero benefit in practice. Frontend interpolation achieves the same goal with 1/10th the complexity.

**Preserved:** 2025-11-23 (Sprint 13 cleanup)
