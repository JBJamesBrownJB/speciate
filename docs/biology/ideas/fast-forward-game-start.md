# Fast-Forward Game Start - Performance Analysis

**Status:** Idea (Feasibility Analysis)
**Analyzed by:** perf-pete (Performance Telemetry Specialist)
**Date:** 2025-12-28

## Concept

When starting a new game, seed the world with known viable creatures appropriate for each biome, then run the simulation at ultra-speed (10x-100x normal) for 30-60 seconds real-time. Player would see the simulation running visibly during this acceleration phase.

**Goal:** Ensure player doesn't start with a "dead" or "boring" world - populations have had time to spread, establish territories, and reach interesting dynamics.

## Current Performance Baseline (360K Creatures @ 20Hz)

**Source:** `/home/dev/dev/speciate/docs/performance/snapshots/NOW.json`

```
Total Tick:       29.3ms avg (p95: 29.9ms)
Movement:          6.0ms (21%)
Perception:        5.9ms (20%)
Steering:          9.4ms (32%)
Spatial Grid:      4.2ms (14%)
Behavior:          2.0ms (7%)
Export Positions:  1.6ms (5%)
```

**Key Constraints:**
- Current tick budget: 50ms @ 20Hz (25ms avg utilization = 50% headroom)
- Target tick rate: 20Hz (production)
- Validated capacity: 360K creatures sustained

## Fast-Forward Feasibility Analysis

### Q1: What are the performance implications of running at ultra-speed?

**Answer:** Ultra-speed is NOT a rendering problem. It's a physics/logic budget problem.

**Breakdown:**

| Speed Multiplier | Effective Tick Rate | Required Tick Budget | Feasibility |
|------------------|---------------------|----------------------|-------------|
| 1x (Normal)      | 20 Hz               | 50ms                 | ✅ CURRENT  |
| 10x              | 200 Hz              | 5ms                  | ❌ IMPOSSIBLE (29ms current) |
| 100x             | 2000 Hz             | 0.5ms                | ❌ IMPOSSIBLE |

**Reality Check:** We cannot run faster than 1/(tick_duration). At 29ms per tick, max achievable rate is ~34Hz (1.7x speedup), not 10x-100x.

**Alternative Interpretation: "Skip Rendering, Run Headless"**

If "fast-forward" means "disable rendering and run simulation-only," let's examine what we can skip:

| Component | Current Cost | Can Skip? | Headless Cost |
|-----------|--------------|-----------|---------------|
| Movement | 6.0ms | ❌ NO | 6.0ms |
| Perception | 5.9ms | ❌ NO | 5.9ms |
| Steering | 9.4ms | ❌ NO | 9.4ms |
| Spatial Grid | 4.2ms | ❌ NO | 4.2ms |
| Behavior | 2.0ms | ❌ NO | 2.0ms |
| Export Positions | 1.6ms | ✅ YES | 0ms |
| **Total** | **29.3ms** | | **27.7ms** |

**Headless Speedup:** 5.5% reduction (1.06x speedup) by skipping IPC export.

**Conclusion:** Disabling rendering saves almost nothing. The simulation itself is the bottleneck.

---

### Q2: What bottlenecks would we hit (perception? physics? rendering?)?

**Answer:** Perception and Steering are the primary bottlenecks (62% of tick time).

**Hot Path Analysis (360K creatures):**

```
1. Steering:          9.4ms (32%) - Dominant cost
2. Movement:          6.0ms (21%) - Physics integration + rotation
3. Perception:        5.9ms (20%) - FOV + spatial queries
4. Spatial Grid:      4.2ms (14%) - Rebuild every tick
5. Behavior:          2.0ms (7%)  - Brain decision logic
```

**Bottleneck Hierarchy:**

1. **Steering (9.4ms):** Fused avoidance + seeking + wandering forces (Sprint 20 optimization already applied)
2. **Movement (6.0ms):** Rayon-parallelized physics (Sprint 15 - already 6.3x optimized)
3. **Perception (5.9ms):** Spatial grid + FOV culling + neighbor caching

**Known Scaling Issues:**

- Perception: O(N) with spatial grid, but query radius scales with creature size (large creatures scan wider areas)
- Steering: Linear in neighbor count, but neighbor count scales with density
- Spatial Grid: O(N) rebuild, but hash lookups cause cache thrashing at high density

**From Sprint 15 Summary:**
> "50K+ creatures limited by O(N²) perception (Sprint 16/18 work)"

This was written for an older architecture. With spatial grid (now implemented), perception is O(N), but still dominates at scale.

---

### Q3: Can we skip certain systems during fast-forward without breaking simulation integrity?

**Answer:** Very limited skipping possible. Most systems are tightly coupled.

**System Dependency Graph:**

```
Spatial Grid ──→ Perception ──→ Steering ──→ Movement ──→ (repeat)
      ↑                                          │
      └──────────────────────────────────────────┘
           (positions changed, must rebuild)
```

**Skip Analysis:**

| System | Can Skip? | Impact if Skipped | Biological Integrity |
|--------|-----------|-------------------|----------------------|
| Movement | ❌ NO | Creatures freeze | Total failure |
| Spatial Grid | ❌ NO | Perception blindness | Total failure |
| Perception | ⚠️ MAYBE | No avoidance/seeking | Chaos (collisions, starvation) |
| Steering | ⚠️ MAYBE | No obstacle avoidance | Mass collisions |
| Behavior | ⚠️ MAYBE | No state transitions | Creatures stuck in current behavior |
| Export Positions | ✅ YES | No visual feedback | Fine (headless) |

**Partial Skip Strategies:**

1. **Perception Throttling (Already Implemented):**
   - See: `/home/dev/dev/speciate/docs/biology/done/perception-time-slicing.md`
   - Creatures update perception on staggered schedule (not every tick)
   - Already running in production
   - **Max speedup:** None (already active)

2. **Skip Behavior Transitions:**
   - Creatures stay in current behavior state (wandering, seeking, fleeing)
   - **Risk:** Predators won't switch to hunting, herbivores won't flee
   - **Biological impact:** SEVERE (food chain breaks down)
   - **Speedup:** ~7% (2ms saved)

3. **Skip Steering (Ballistic Movement):**
   - Creatures move in straight lines, no avoidance
   - **Risk:** Mass collisions, boundary violations
   - **Biological impact:** SEVERE (spatial distribution corrupted)
   - **Speedup:** ~32% (9.4ms saved)

**Verdict:** Cannot skip core simulation systems without breaking ecosystem dynamics. Only Export Positions is safe to skip (5.5% gain).

---

### Q4: Any Golden Zone Opportunities Here?

**Answer:** YES. Multiple Golden Zone opportunities exist by making fast-forward meaningful instead of fast.

#### Golden Zone Opportunity 1: Ecological "Burn-In" Period

**Concept:** Instead of speeding up time, run at normal speed but skip initial boring phases.

**Implementation:**
```rust
// Seed with high genetic diversity
spawn_initial_population(world, biome_config);

// Run 1000 ticks headless (50 seconds @ 20Hz)
for _ in 0..1000 {
    simulation.tick(PRODUCTION_DELTA_TIME);
}

// Player sees established ecosystem
start_rendering();
```

**Biological Win:** Ecosystems need time to establish predator/prey ratios, territorial boundaries, genetic mixing.

**Performance Win:** Headless mode skips export_positions (5.5% speedup = negligible).

**Player Experience:** "Your world is 50 seconds old" - populations have spread, weak genes eliminated, territories claimed.

**No artificial speedup required.**

---

#### Golden Zone Opportunity 2: Low-Fidelity Initialization (Coarse Simulation)

**Concept:** Run a simplified simulation during initialization, then switch to full-fidelity.

**Low-Fidelity Rules:**
- Skip avoidance (creatures pass through each other)
- Skip perception updates (use cached data)
- Skip behavior transitions (all creatures wander)
- Run at 2-4x normal tick rate (reduced system load)

**Biological Win:** Dispersal phase (creatures spreading out from spawn points) doesn't require full interaction physics.

**Performance Win:**
```
Skip Perception:  -5.9ms
Skip Steering:    -9.4ms
Skip Behavior:    -2.0ms
Total Saved:      17.3ms (59% reduction)
New Tick Budget:  12ms → 83Hz achievable (4x speedup)
```

**Player Experience:** Visible "settling" animation - creatures disperse from spawn points, world feels alive before gameplay starts.

**Risk:** Spatial distribution may be unrealistic (no predator/prey clustering). Acceptable for initialization.

---

#### Golden Zone Opportunity 3: Stochastic Vision Fast-Forward

**Concept:** During fast-forward, reduce perception frequency dramatically (update 1% of creatures per tick instead of 10%).

**Implementation:**
```rust
// Normal mode: 10% of creatures update perception per tick (current)
const NORMAL_PERCEPTION_UPDATE_RATE: f32 = 0.1;

// Fast-forward mode: 1% per tick (10x reduction)
const FASTFORWARD_PERCEPTION_UPDATE_RATE: f32 = 0.01;
```

**Biological Win:** During dispersal, precise threat detection isn't critical. Creatures can afford stale perception data.

**Performance Win:**
```
Perception:  5.9ms → 0.59ms (10x reduction)
Steering:    9.4ms → ~5ms (fewer neighbors cached, less avoidance work)
Total Saved: ~10ms
Speedup:     29.3ms → 19.3ms → 51Hz achievable (2.5x speedup)
```

**Player Experience:** Visible fast-forward (2.5x speed = 30 seconds becomes 12 seconds).

**Caveat:** Not 10x-100x speedup, but perceptually faster.

---

#### Golden Zone Opportunity 4: GPU-Accelerated Initialization (Future Work)

**Concept:** Offload physics integration to GPU compute shaders during initialization.

**Why This is Golden Zone:**
- Optimization: Parallel position updates on GPU (massive speedup)
- Biological: Final positions are identical to CPU physics (deterministic)

**Estimated Speedup:** 10x-50x for movement system (WebGPU compute shaders)

**Complexity:** HIGH (requires WebGPU compute pipeline, shader authoring)

**Sprint Estimate:** Sprint 25+ (after core gameplay complete)

---

## Recommended Implementation Path

### Phase 1: Headless Burn-In (Immediate - No Performance Work Required)

**What:** Run 500-1000 ticks headless after spawn, then start rendering.

**Code Change:**
```rust
// apps/simulation/src/napi_addon/simulation_engine.rs
#[napi]
pub fn initialize_world(&mut self, config: WorldConfig) {
    self.spawn_initial_population(config);

    // Burn-in period (headless)
    for _ in 0..1000 {
        self.app.tick(PRODUCTION_DELTA_TIME);
    }

    // Start IPC export
    self.rendering_enabled = true;
}
```

**Speedup:** None (runs at normal 20Hz).

**Benefit:** Player sees established ecosystem instead of initial spawn chaos.

**Dev-UI Metric:** Add `burn_in_progress` counter to show initialization status.

---

### Phase 2: Low-Fidelity Dispersal (Sprint 21 Candidate)

**What:** During burn-in, disable avoidance/perception to achieve 4x speedup.

**Implementation:**
```rust
pub fn fast_forward_mode(&mut self, enabled: bool) {
    self.world.resource_mut::<FastForwardMode>().0 = enabled;
}

// In perception system:
pub fn update_perception_system(
    fast_forward: Res<FastForwardMode>,
    // ... rest of query
) {
    if fast_forward.0 {
        return; // Skip perception during fast-forward
    }
    // ... normal perception logic
}
```

**Speedup:** 4x (29ms → ~12ms tick → 83Hz).

**Player Experience:** 1000 ticks @ 83Hz = 12 seconds instead of 50 seconds.

**Risk:** LOW (only used during initialization, not gameplay).

---

### Phase 3: Stochastic Vision Reduction (Sprint 22 Candidate)

**What:** Combine with Phase 2 to achieve 2.5x speedup without sacrificing all collision detection.

**Implementation:**
```rust
const FASTFORWARD_PERCEPTION_RATE: f32 = 0.01; // 1% per tick instead of 10%

if fast_forward.0 {
    perception_rate = FASTFORWARD_PERCEPTION_RATE;
}
```

**Speedup:** 2.5x on top of Phase 2 = 10x total (500 ticks in 5 seconds).

**Biological Integrity:** ACCEPTABLE (creatures still avoid each other, just with stale data).

---

## Performance Measurement Plan

### Baseline Metrics (Before Implementation)

**Tool:** `perf stat` Health Check

```bash
perf stat -e instructions,cycles,L1-dcache-load-misses,LLC-load-misses \
  timeout 60s ./apps/simulation/target/release/sim_app --headless --burn-in 1000
```

**Expected Output (Normal Mode):**
```
IPC:           4.25 (from Sprint 15 parallelization)
L1 Miss Rate:  ~3% (movement system optimized)
LLC Miss Rate: ~1% (spatial grid cache-friendly)
Duration:      50 seconds (1000 ticks @ 20Hz)
```

---

### Phase 2 Validation (Low-Fidelity Fast-Forward)

**Command:**
```bash
perf stat -e instructions,cycles,L1-dcache-load-misses,LLC-load-misses \
  timeout 60s ./apps/simulation/target/release/sim_app --headless --burn-in 1000 --fast-forward
```

**Success Criteria:**
```
IPC:           >4.0 (acceptable if slightly lower due to different code paths)
L1 Miss Rate:  <5% (should improve - less random access in perception)
Duration:      <15 seconds (4x speedup target)
Creatures:     Same final positions as normal mode (within epsilon for FP errors)
```

**Validation Test:**
```rust
#[test]
fn fast_forward_produces_same_dispersal() {
    let mut sim_normal = create_simulation();
    let mut sim_fast = create_simulation();

    sim_fast.enable_fast_forward(true);

    for _ in 0..1000 {
        sim_normal.tick(DELTA_TIME);
        sim_fast.tick(DELTA_TIME);
    }

    let positions_normal = sim_normal.get_all_positions();
    let positions_fast = sim_fast.get_all_positions();

    // Positions should diverge (different physics due to skipped avoidance)
    // But spatial distribution should be similar (creatures spread out)
    assert_spatial_distribution_similar(positions_normal, positions_fast, epsilon: 50.0);
}
```

---

### Dev-UI Integration

**New Metrics to Expose:**

```rust
pub struct FastForwardMetrics {
    pub enabled: bool,
    pub ticks_completed: u64,
    pub ticks_target: u64,
    pub effective_hz: f32,
    pub skipped_systems: Vec<String>, // ["perception", "steering"]
}
```

**Dev-UI Display (React Component):**

```tsx
// apps/dev-ui/src/components/FastForwardStatus.tsx
export const FastForwardStatus: React.FC = () => {
  const { enabled, ticks_completed, ticks_target, effective_hz } = useFastForwardMetrics();

  if (!enabled) return null;

  const progress = (ticks_completed / ticks_target) * 100;

  return (
    <div className="fast-forward-status">
      <h3>Fast-Forward Mode</h3>
      <ProgressBar value={progress} />
      <div>Ticks: {ticks_completed} / {ticks_target}</div>
      <div>Effective Rate: {effective_hz.toFixed(1)} Hz</div>
      <div className="warning">⚠️ Low-fidelity physics active</div>
    </div>
  );
};
```

---

## Final Verdict

### Can We Do 10x-100x Speedup?

**NO.** Not without GPU compute shaders (future work).

### Can We Do Meaningful Fast-Forward?

**YES.** Via Golden Zone strategies:

| Approach | Speedup | Biological Integrity | Complexity |
|----------|---------|----------------------|------------|
| Headless Burn-In | 1.05x | ✅ PERFECT | TRIVIAL |
| Low-Fidelity Dispersal | 4x | ⚠️ ACCEPTABLE (init only) | LOW |
| Stochastic Vision | 2.5x | ⚠️ ACCEPTABLE (init only) | LOW |
| **Combined (Phase 2+3)** | **10x** | ⚠️ ACCEPTABLE | **MEDIUM** |
| GPU Compute | 50x+ | ✅ PERFECT | VERY HIGH |

---

## Recommended Next Steps

1. **Immediate:** Implement Phase 1 (Headless Burn-In) - requires no performance work
2. **Sprint 21:** Implement Phase 2 (Low-Fidelity Fast-Forward) with TDD and perf validation
3. **Sprint 22:** Add Phase 3 (Stochastic Vision) if Phase 2 proves stable
4. **Future (Sprint 25+):** Investigate GPU compute shaders for true 100x speedup

---

## Golden Rule Reminder

**Every optimization claim must be backed by perf data.**

Before merging Phase 2 implementation:
- Run Health Check SOP (perf stat baseline)
- Capture before/after flamegraphs (samply)
- Validate in Dev-UI (effective Hz matches target)
- Document results in sprint summary

**No merge without measurement.**

---

**Files Referenced:**
- `/home/dev/dev/speciate/apps/simulation/src/napi_addon/simulation_engine.rs:39` (TARGET_SIMULATION_HZ)
- `/home/dev/dev/speciate/docs/performance/snapshots/NOW.json` (360K creature baseline)
- `/home/dev/dev/speciate/docs/biology/done/perception-time-slicing.md` (existing throttling)
- `/home/dev/dev/speciate/sprint_summaries/sprint-15-ecs-optimizations_summary.md` (Rayon parallelization)
