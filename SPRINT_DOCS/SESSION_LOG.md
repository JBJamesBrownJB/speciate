# Sprint 15: Session Log

**Branch:** `feat/sprint-15-ecs-optimizations`
**Sprint Start:** 2025-11-28
**Status:** IN PROGRESS

---

## 2025-11-28: Sprint Initialization

**Completed:**
- ✅ Pre-flight checks passed (clean working directory, main branch, no conflicts)
- ✅ Renamed SPRINT_15_PLAN → SPRINT_DOCS
- ✅ Branch created: `feat/sprint-15-ecs-optimizations`
- ✅ SPRINT_BACKLOG.md initialized
- ✅ Session log initialized

**Development Environment Verified:**
- Rust: 1.91.1 (ed61e7d7e 2025-11-07)
- Node: v24.11.1
- npm: 11.6.2

**Sprint Context:**
- Prerequisites: Sprint 14 complete (GPU interpolation @ 165 FPS)
- Frontend ready for high entity counts (200K+)
- Backend is the bottleneck (vision system Vec allocations)
- Target: Scale to 150K-200K creatures @ 22.2Hz stable

**Next Steps:**
- Begin Phase 1: Uber-Struct Refactor
- Review SPRINT_PLAN_sprint-15-ecs-optimizations.md for detailed implementation steps
- Follow TDD (Red-Green-Refactor) workflow for all changes
- Engage ecs-emma for ECS architecture design

---

## 2025-11-28: Performance Metrics Added to Plan

**Updated:**
- ✅ Added comprehensive performance metrics to SPRINT_PLAN
- ✅ Added expected gains per phase to SPRINT_BACKLOG
- ✅ Added baseline metrics (current 50K creature state)
- ✅ Added cumulative capacity visualization

**Key Metrics Documented:**
- Baseline: 50K creatures @ ~35ms tick (vision = 57% of frame)
- Phase 1: +20% capacity (cache locality)
- Phase 2A: +100% capacity (zero Vec allocations) **CRITICAL**
- Phase 2B: +25% capacity (Changed<T> + SIMD)
- Phase 2C: +33% capacity (parallelization)
- Final Target: 200K creatures @ <50ms tick

**Confidence Levels:**
- 150K @ 22.2Hz: 90% confidence
- 200K @ 22.2Hz: 60% confidence

---

## 2025-11-28: Metrics Corrected from Actual Snapshots

**Issue:** Initial metrics were estimated, not from actual data.

**Actual Baseline (from `5k_wanderers_2025-11-28T14-33-50.json`):**
- 5K active wanderers = **50ms tick** (AT budget limit, not 50K @ 35ms!)
- Perception = **34ms (67%)** of frame (not 57%)
- Movement = 13ms (26%)
- Avoidance = 3.4ms (7%)
- Max active creatures = **~5K** (not 50K!)

**Root Cause:** O(N²) perception system scales quadratically:
- 5K → 34ms (25M comparisons)
- 10K → ~136ms (100M comparisons)
- 20K → ~544ms (400M comparisons)

**Revised Expectations:**
- Phase 2A (split queries): 6K → 15-20K active (+200%)
- Phase 2D (stochastic vision): 40-50K → 100-200K active (+100-300%)

**Key Insight:** Stochastic vision (only 10% creatures update per tick) is now a Phase 2D requirement, not optional.

---

## 2025-11-28: Phase 1 Changed to Validation Trial

**Decision:** Before investing in uber-struct refactor, validate that archetype churn is actually a measurable problem.

**Trial Design:**
- **Scenario A (Stable):** 2.5K wanderers, no behavior changes
- **Scenario B (Churning):** 2.5K creatures with constant behavior transitions

**Metrics to Compare:**
- Tick time difference
- Archetype count growth
- IPC / cache miss rates

**Decision Point:**
- If B >> A (>20% slower) → Proceed to Phase 1b (uber-struct)
- If B ≈ A (<10% difference) → Skip to Phase 2A (vision optimization)

**Rationale:** Real wins come from Phase 2 vision optimization. Uber-struct may provide only modest gains (5-10%). Before spending a day on implementation, prove it's worth the effort.

**Next:** User to design behavior change mechanism for Scenario B.

---

## 2025-11-28: Phase 1 Complete - Brain Component Implemented

**Completed:**
- ✅ Removed dead `Catatonic` marker component from core/components.rs
- ✅ Created `Brain` component with `BrainMode` enum (Normal, Cycling, Dormant)
- ✅ Populated `behavior_transition_system` with Brain-driven decision logic
- ✅ Added `test_archetype_stability_with_cycling_brain` regression test
- ✅ All 149 tests passing

**New Files:**
- `apps/simulation/src/simulation/creatures/components/brain.rs`

**Modified Files:**
- `apps/simulation/src/simulation/core/components.rs` - Removed Catatonic
- `apps/simulation/src/simulation/creatures/components/mod.rs` - Added brain module
- `apps/simulation/src/simulation/creatures/builder.rs` - Added Brain to CritBundle
- `apps/simulation/src/simulation/creatures/behaviors/transitions.rs` - Brain-driven decisions
- `apps/simulation/src/simulation/tests.rs` - Archetype stability regression test
- Multiple re-export files (lib.rs, components.rs)

**Architecture:**
- Brain centralizes decision-making (receives perception, life stats, outputs behavior)
- `BrainMode::Normal` - Standard decision logic (future: perception-based)
- `BrainMode::Cycling` - Forces behavior cycling every cooldown period (for testing)
- `BrainMode::Dormant` - No decisions (static behavior)
- Current architecture already archetype-stable (BehaviorMode is an enum, not add/remove components)

**Key Insight:**
The original uber-struct trial was unnecessary because the current architecture already avoids archetype churn - BehaviorMode is an enum inside CreatureState, not a separate component being added/removed. The Brain component reinforces this pattern and provides:
1. Regression test to prevent future accidental churn
2. Centralized decision-making foundation
3. Future DNA integration point

**Next:** Phase 1.5 - Brain timing architecture refactor

---

## 2025-11-28: Phase 1.5 Complete - Brain Timing Architecture

**Problem Identified:**
- Brain had separate `decision_cooldown_ms` timing
- Vision will use stochastic timing (Phase 2D)
- These would compound, causing up to 2x intended reaction latency

**Solution Implemented:**
- Added `has_fresh_vision: bool` field to Brain component
- Perception system sets `brain.has_fresh_vision = true` after updating
- Brain system checks flag for Normal mode, clears it after processing
- Cycling mode still uses cooldown timer (for testing)

**Key Design Decision:**
Originally planned to use `VisionUpdated` marker component, but this caused archetype churn (add/remove each tick). Using a boolean field inside Brain avoids this entirely.

**Files Modified:**
- `creatures/components/brain.rs` - Added `has_fresh_vision` field
- `perception/systems.rs` - Sets flag after perception update
- `creatures/behaviors/transitions.rs` - Uses flag instead of marker

**Benefits:**
- Zero archetype churn (field mutation, not component add/remove)
- Single timing source (Vision drives Brain decisions)
- Prepares for Phase 2D stochastic vision

**All 149 tests passing**, including archetype stability regression test.

**Next:** Phase 1.5 - Brain timing architecture revision

---

## 2025-11-28: Phase 1.5 Revised - Dynamic Brain Cooldown

**Problem with Original Implementation:**
The `has_fresh_vision` approach was wrong because:
- Brain doesn't just decide based on perception
- Brain also considers hunger, health, energy, age
- Tying Brain to perception updates would delay internal-state-driven decisions

**Revised Implementation (Zoologist Consultation):**

Brain now uses **dynamic cooldown** that scales with creature state:
- **Age:** Power law (exponent 2.5) - older creatures think slower
- **Energy:** Quadratic - low energy slows thinking
- **Panic override:** Immediate threats bypass cooldown entirely

**Key Changes:**
1. Removed `has_fresh_vision` field from Brain
2. Added `effective_cooldown_ms(age, energy)` method
3. Added `should_panic()` function for immediate threat response
4. Updated `can_decide()` signature to include age/energy
5. Removed perception→brain coupling in perception/systems.rs
6. Added 4 new tests (153 total passing)

**New Specification System:**
- Created `docs/spec/` folder for live feature documentation
- Added `brain-spec.md` documenting current Brain implementation
- Updated `CLAUDE.md` to require spec updates when implementing features

**Files Modified:**
- `creatures/components/brain.rs` - Dynamic cooldown, panic override
- `creatures/behaviors/transitions.rs` - Uses new Brain API
- `perception/systems.rs` - Removed Brain coupling
- `CLAUDE.md` - Added spec documentation requirement
- `docs/spec/brain-spec.md` - NEW - Brain system specification

**Constants (hardcoded, DNA integration deferred):**
- `BASE_COOLDOWN_MS = 150.0`
- `AGE_SENSITIVITY = 2.0`
- `PANIC_THRESHOLD = 2.0` (body size multiplier)

**Next:** Phase 2A - Vision split queries (the critical optimization)

---

## 2025-11-28: Catatonic Marker Cleanup + Cycling Trial

**Issue:** Build error from leftover `Catatonic` marker component references in `trials/loader.rs`.

**Root Cause:** Phase 1 removed the `Catatonic` marker component from `core/components.rs`, but `trials/loader.rs` still referenced it.

**Fixed:**
- ✅ Line 133: Changed `world.spawn((bundle, Catatonic))` to `world.spawn(bundle)`
- ✅ Line 188: Updated test query to use `CreatureState` with behavior filter
- ✅ Line 392: Updated `catatonic_count` to filter by `BehaviorMode::Catatonic`

**New: Cycling Creature Type**
- ✅ Added `CreatureType::Cycling` to `trials/mod.rs`
- ✅ Updated `trials/loader.rs` to spawn cycling creatures with `with_cycling_brain()`
- ✅ Created `trials/cycling-brain-stress.toml` - 2.5K cycling creatures (50x50 grid)

**Test Results:**
- All 153 simulation tests passing
- One pre-existing flaky test in NAPI save state integration (unrelated)

**Files Modified:**
- `trials/loader.rs` - Fixed Catatonic references, added Cycling creature type
- `trials/mod.rs` - Added `CreatureType::Cycling`
- `trials/cycling-brain-stress.toml` - NEW - 2.5K cycling creatures trial

**Next:** Phase 2A - Vision split queries (the critical optimization)

---

## 2025-11-28: Perception System Specification Added

**Created:** `docs/spec/perception-spec.md`

**Documents:**
- Perception component (range, nearby entities)
- AvoidanceBehavior component (personal space, panic threshold)
- Constants from movement/constants.rs
- `update_perception_system` O(N²) algorithm
- Edge-to-edge distance calculation
- Current performance baseline (34ms @ 5K = 67% of tick)
- Scaling problem (O(N²) quadratic)
- Future optimization phases (2A-2D)

**Key Insights Captured:**
- Catatonic creatures skip perception (but ARE perceived)
- Brain runs independently of perception timing
- Vec allocations happen every tick (optimization target)

---
