# Technical Review: Procedural Shader Gait Synthesizer

**Reviewed by:** Sarah (Principal Architect)
**Date:** 2025-12-28
**Status:** FEASIBLE with critical refinements required

---

## Executive Summary

Your Procedural Shader Gait Synthesizer is **technically sound** and represents a promising path forward for animated creature movement at 20K+ scale. However, the current uniform set and gait signal generation need biological grounding and architectural refinement.

**Key Findings:**
- ✅ **Technically feasible** for PixiJS/WebGL with instanced rendering
- ✅ **Performance-viable** at 20K+ creatures (per-vertex deformation is GPU-bound, not a blocker)
- ✅ **Golden Zone potential** (size → gait emergent behavior)
- ⚠️ **Missing:** Biological grounding for gait parameters
- ⚠️ **Risk:** Current uniform proposal may create "robotic" motion without proper speed-coupling

---

## 1. Technical Feasibility Assessment

### 1.1 Architecture Compatibility: EXCELLENT

Your approach aligns perfectly with the existing PixiJS rendering pipeline:

**Current State:**
- `InterpolatedCreatureRenderer` uses custom `Shader` + instanced geometry (PixiJS v8)
- Per-creature attributes already in buffer: `aStartPos`, `aEndPos`, `aStartRot`, `aEndRot`, `aSize`
- Vertex shader already handles transformation (world → NDC) and rotation
- Fragment shader is trivial (texture lookup)

**Gait Shader Integration:**
1. **Vertex Stage:** Deform quad vertices based on gait signal
2. **Instance Attributes:** Reuse existing `aSize` for gait scaling
3. **Uniforms:** Add `uTime` + optional per-tick `uGameTick`
4. **Performance:** Zero additional draw calls (all deformation in existing vertex shader)

**Verdict:** Seamless. No architectural changes needed.

### 1.2 Per-Vertex Deformation Performance: VIABLE

**Mesh Requirements:**
- Current: Simple quad (4 vertices per creature)
- Needed for organic wiggle: 8-16 vertices per creature (spine segments)

**Cost Analysis (WebGL2 at 20K creatures):**

| Vertices/Creature | Total Verts | Processing/Frame | GPU Bandwidth | Estimate |
|-------------------|------------|-----------------|---------------|----------|
| 4 (current) | 80K | 80K vertex shader | 640K floats | <1ms |
| 8 (body segments) | 160K | 160K vertex shader | 1.28M floats | 1-2ms |
| 16 (detailed spine) | 320K | 320K vertex shader | 2.56M floats | 2-4ms |

**Recommendation:** Start with **8 vertices per creature** (two rings: head + tail with 4 points each). This gives:
- Enough detail for believable undulation
- Minimal GPU overhead (~2ms for deformation)
- Room to expand to 16 later without redesign

**Gotchas:**
- Modern GPUs: 4M+ vertices/frame is comfortable
- Bandwidth: Float32Array buffer updates still bounded by NAPI transfer (mitigated by double-buffering)
- Vertex Cache: Ensure vertices are laid out predictably (tightly packed in buffer)

---

## 2. Uniform Set Analysis

### 2.1 Current Proposal: INSUFFICIENT

You proposed:
```glsl
uniform float uTime;        // Global game time
uniform float uSpeed;       // Current velocity magnitude
uniform float uSize;        // Creature scale (0.5-5.0)
uniform float uRandom;      // Per-creature seed
```

**Problems:**

1. **`uTime` is ambiguous:** Is this wall-clock time or in-game ticks? At 22.2Hz backend + 60Hz frontend rendering, you have a **fractional tick value** needed for synchronized animation.

2. **`uSpeed` lacks frequency coupling:** A naive approach would be:
   ```glsl
   float freq = uSpeed;  // BAD: creates jittery, non-biological motion
   float wiggle = sin(uGameTime * freq - spine_position * lag);
   ```
   This disconnects motion frequency from body physics.

3. **`uSize` alone doesn't drive gait:** Size affects both amplitude AND frequency. Need separate signals for:
   - Base oscillation frequency (coupled to speed)
   - Amplitude scaling (coupled to size)
   - Wave propagation lag (tail delay relative to head)

4. **`uRandom` doesn't prevent lockstep:** 20K creatures with same `uTime` + `uSize` will move in perfect lockstep. You need:
   - Per-creature **phase offset** (built into shader, not uniform)
   - Per-creature **offset time** (to desynchronize creatures spawned simultaneously)

### 2.2 Revised Uniform Set: RECOMMENDED

```glsl
uniform float uGameTime;       // In-game time (seconds), from simulation tick
uniform float uTickFraction;   // 0-1 interpolation within current tick (for smooth frontend)
uniform float uSimTickRate;    // Simulation tick rate (Hz), e.g. 22.2
uniform float uBaseFreq;       // Base wiggle frequency (Hz), e.g. 1.5

// Per-creature (via instance attribute, not uniform)
in float aGaitPhase;           // 0-1 phase offset, embedded in geometry/buffer
in float aSpeedMagnitude;      // Current velocity magnitude (m/s)
in float aBodyLength;          // Size from BodySize component
```

**Why this works:**
- `uGameTime + uTickFraction` gives smooth frontend interpolation
- `uBaseFreq` is biologically grounded (species trait)
- `aGaitPhase` + creature ID prevents lockstep
- `aSpeedMagnitude` drives frequency coupling (small = slow twitches, large = flowing)
- `aBodyLength` scales amplitude and wave lag

---

## 3. Gait Signal Generation: BIOLOGICAL GROUNDING REQUIRED

### 3.1 Current Proposal Issues

You suggested:
```glsl
gaitSignal = sin(time * freq - distance * lag) * amplitude;
```

This is **structurally correct** (traveling wave), but the **parameterization is wrong**:
- No speed coupling (swimming at 10 m/s looks same as 1 m/s)
- No size-dependent frequency adjustment
- No amplitude scaling with body size
- No tail-lag phase relationship

### 3.2 Biological Reality (consult zoologist-tom)

**CRITICAL:** Before finalizing, I need to ask Tom:

1. **Fish/snake swimming frequency:** How does velocity relate to undulation frequency in real organisms?
   - Example hypothesis: frequency ∝ speed^0.3 (sublinear scaling)?
   - Or is it linear?

2. **Tail-lag relationship:** What's the realistic phase delay between head and tail?
   - Example: Head completes one cycle in 1.0s, tail completes in 1.2s?
   - Is this fixed per-species, or speed-dependent?

3. **Amplitude scaling:** How does body size affect wiggle magnitude?
   - Larger creature = larger absolute displacement?
   - Smaller creature = proportionally larger relative wiggle?

4. **Frequency bounds:** What are realistic min/max undulation frequencies?
   - Idle/drift: 0.3-0.5 Hz?
   - Cruising: 1-2 Hz?
   - Max speed: 3-5 Hz?

**RECOMMENDATION:** Document these in `docs/biology/done/procedural-animation.md` after Tom consults.

### 3.3 Proposed Gait Formula (PENDING BIOLOGICAL VALIDATION)

```glsl
// Pseudocode - awaiting Tom's biological parameters

float baseFreq = 1.5;                      // Species trait (Hz)
float speedScaling = sqrt(aSpeedMagnitude) // Speed couples to frequency

// Frequency increases with speed (non-linearly)
float animFreq = baseFreq * (0.5 + 1.5 * speedScaling); // 0.75x to 3x modulation

// Wave position along spine (0=head, 1=tail)
float spinePos = vUV.y;  // Assuming vUV.y is normalized spine position

// Traveling wave: sin(time * freq - position * lag)
// Tail lags behind head by ~20-30% phase
float wavePhase = uGameTime * animFreq - spinePos * 3.0;

// Add per-creature phase offset to prevent lockstep
wavePhase += aGaitPhase * 6.28318;

// Amplitude scales with body size
float ampMin = 0.1;
float ampMax = 0.8;
float amplitude = mix(ampMin, ampMax, aBodyLength);

// Final deformation
float gaitSignal = sin(wavePhase) * amplitude;
```

**This requires:**
- Tom's validation of frequency/speed relationships
- Empirical testing against real fish/snake footage
- Adjustable constants (currently hardcoded, should be uniforms for tuning)

---

## 4. Integration with PixiJS Pipeline

### 4.1 Mesh Topology Change

**Current:**
```typescript
geometry.topology = 'triangle-strip';
(geometry as any).vertexCount = 4;  // One quad
```

**For gait animation:**
```typescript
// Option A: Enhanced quad (8 vertices: 2 rings × 4 points)
geometry.topology = 'triangle-strip';
(geometry as any).vertexCount = 8;

// Option B: Index-based rendering (more complex, better for LOD)
// geometry.topology = 'triangle-list';
// Add index buffer mapping 8 vertices → triangles
```

**Recommendation:** Start with **Option A** (8-vertex quad) – simpler, sufficient detail.

### 4.2 Buffer Layout Change

**Current `InterpolationBufferManager` layout:**
```
FLOATS_PER_CREATURE = 7
[aStartPos(2) | aEndPos(2) | aStartRot(1) | aEndRot(1) | aSize(1)]
```

**Proposed addition (for gait):**
```
FLOATS_PER_CREATURE = 9  // +2 for phase, speed
[aStartPos(2) | aEndPos(2) | aStartRot(1) | aEndRot(1) |
 aSize(1) | aGaitPhase(1) | aSpeedMagnitude(1)]
```

**Cost:** +2 floats × 20K creatures = 160KB additional buffer per frame (negligible).

**Implementation:**
1. Modify `InterpolationBufferManager` to expand stride
2. Export `aGaitPhase` + `aSpeedMagnitude` from Rust IPC
3. Update vertex shader to accept new attributes

### 4.3 Shader Code Template

```glsl
// New in vertex shader
in float aGaitPhase;
in float aSpeedMagnitude;

// Existing uniforms (keep these)
uniform float uInterpolation;
uniform float uGameTime;
uniform float uTickFraction;

// New uniforms
uniform float uSimTickRate;
uniform float uBaseFreq;

void main() {
  // Existing code: compute worldPos, rotation, etc.
  vec2 worldPos = mix(aStartPos, aEndPos, uInterpolation);

  // NEW: Compute gait signal
  float smoothTime = uGameTime + (uTickFraction / uSimTickRate);
  float freq = uBaseFreq * (0.5 + 1.5 * sqrt(aSpeedMagnitude));
  float wavePhase = smoothTime * freq - vUV.y * 3.0 + aGaitPhase * 6.28;
  float amplitude = 0.3 + 0.5 * aSize;  // Size scales amplitude

  // Deform quad vertex based on gait signal
  float gaitOffset = sin(wavePhase) * amplitude;
  vec2 gaitDisplace = vec2(0.0, gaitOffset);  // Vertical wobble

  // Apply gait + rotation + camera transform (as before)
  vec2 localPos = (aQuadVertex - 0.5) * vec2(aSize, aSize);
  localPos += gaitDisplace;  // Add gait deformation

  // ... rest of vertex shader (rotation, world transform, etc.)
}
```

---

## 5. Performance Implications

### 5.1 GPU Cost Breakdown

| Component | Cost/Vertex | 20K × 8 verts | Notes |
|-----------|------------|---------------|-------|
| Position interpolation | 1 op | <0.1ms | Existing |
| Rotation + local transform | 6 ops | <0.5ms | Existing |
| Gait computation | 8 ops | 1-2ms | NEW (sin, sqrt, arithmetic) |
| Camera transform | 4 ops | <0.5ms | Existing |
| Total | 19 ops | 2-3ms | Comfortable headroom at 60Hz |

**Verdict:** Gait computation adds **1-2ms to vertex shader**, well within budget.

### 5.2 CPU Cost (Rust Backend)

**New exports needed:**
- `aGaitPhase`: Deterministic per-creature (seed-based, computed once)
- `aSpeedMagnitude`: Already available in `Velocity` component

**Cost:** Negligible (<0.1ms to export two floats per creature).

### 5.3 Memory Cost

- Geometry buffer expansion: 160KB
- Shader complexity: Minimal (no new uniforms affecting texture/sampler count)

**Verdict:** No memory pressure.

---

## 6. Critical Gotchas & Mitigations

### 6.1 Gotcha: Deformed Quad Clipping

**Problem:** If gait deformation pushes vertices outside quad bounds, you'll see "poking" beyond the sprite texture.

**Mitigation:**
1. **Constrain amplitude:** Ensure `amplitude * max_sin(wavePhase) < 0.3 * aSize` (30% of creature radius)
2. **Mesh density:** Use 8+ vertices so deformation is distributed (not concentrated at corners)
3. **Test visual:** Render creatures at different sizes/speeds and validate no clipping

### 6.2 Gotcha: Frequency Aliasing

**Problem:** At high simulation tick rate (22.2Hz) with low wiggle frequency (0.3Hz), you may see **temporal aliasing** if gait updates at fixed tick intervals.

**Mitigation:**
- Frontend renders at 60Hz with continuous `uTime` (smooth curve)
- Backend simulation tick is hidden (gait computes via `uGameTime`, not discrete ticks)
- No frame-rate dependency (unlike naive per-tick increments)

**Verdict:** This design avoids aliasing by using continuous time, not tick counts.

### 6.3 Gotcha: Rotation Interpolation + Deformation

**Problem:** If a creature rotates while wiggling, the gait deformation direction rotates with it. This might look unnatural (spine always wiggles in world-space Y, not creature-local Y).

**Options:**
1. **World-space deformation:** Wiggle along absolute Y axis (ignores creature rotation)
2. **Local-space deformation:** Wiggle perpendicular to creature direction (current approach)
3. **Hybrid:** Wiggle mostly local, with dampening toward world-space

**Recommendation:** Start with **local-space** (perpendicular to velocity direction). Validate against footage of real swimming creatures.

### 6.4 Gotcha: Instanced Rendering Attribute Bandwidth

**Problem:** Expanding attributes requires re-binding geometry. If done every frame, could cause GPU stalls.

**Mitigation (Already Handled):**
- PixiJS's `Geometry` system batches attribute updates
- Double-buffering in `InterpolationBufferManager` prevents GPU stalls
- Single `buffer.update()` call per frame (not per creature)

**Verdict:** No issue with existing architecture.

---

## 7. Golden Zone Opportunity

**Size → Gait Feel (Emergent Behavior)**

Your intuition about Golden Zone is **correct**. The current design creates free emergent behavior:

```
Size → Amplitude scaling (larger = more amplitude)
Size → Frequency modulation (via speed-coupling: larger → faster top speed)
Result: Small creatures twitch rapidly, large creatures flow gracefully
```

This is **biologically accurate** AND **computationally free** (size already in buffer).

**Example Emergence:**
- Small creature (0.2m): High freq (2-4 Hz), low amplitude → frantic jittering
- Medium creature (1.0m): Base freq (1.5 Hz), med amplitude → natural swimming
- Large creature (3.0m): Low freq (0.8 Hz), high amplitude → ponderous flowing

This is not "arbitrary frame skipping" (bad Golden Zone). It's **physically meaningful**.

---

## 8. Implementation Roadmap

### Phase 2C-1: Biological Grounding (PREREQUISITE)

1. **Consult zoologist-tom:**
   - Fish/snake swimming frequency vs. velocity relationship
   - Tail-lag phase timing
   - Amplitude scaling with body size
   - Realistic frequency bounds

2. **Document in:** `docs/biology/done/procedural-animation.md`

3. **Create test footage:** Render 3 reference creatures (small/medium/large) with various speeds and validate against real animal footage

### Phase 2C-2: Shader Implementation (TDD)

1. **Red:** Write failing test for gait signal generation
   - Test that creatures of different sizes have different wiggle frequencies
   - Test that stationary creatures have slow, low-amplitude wiggle
   - Test that fast-moving creatures have high-frequency wiggle

2. **Green:** Implement minimal shader code (use hardcoded constants first)

3. **Refactor:** Extract constants to uniforms, add biology-driven parameterization

### Phase 2C-3: Buffer Integration (TDD)

1. **Red:** Test that `aGaitPhase` and `aSpeedMagnitude` are exported to shader

2. **Green:** Expand `InterpolationBufferManager` to include new attributes

3. **Refactor:** Validate no frame-rate drops at 20K creatures

### Phase 2C-4: Visual Validation (Manual)

1. Render creatures at various sizes/speeds
2. Compare against reference footage (Tom provides)
3. Tune amplitude/frequency constants for naturalness
4. Test edge cases: very small creatures, very large creatures, zero velocity

---

## 9. Instancing Approach: FINAL RECOMMENDATION

**Question:** Should we use instanced rendering with shared shaders, or per-creature instances?

**Answer:** **Shared shader with instanced geometry (current approach).**

**Why:**
- Single shader compilation
- Single draw call per frame (batched 20K creatures)
- Attribute data controls per-creature variation
- **Perfect for your use case**

**Alternative (not recommended):**
- Per-creature shader instances: Would require 20K separate shaders (memory explosion, compile stalls)
- Dynamic shader generation: Overcomplicated, no benefit

**Verdict:** Your architectural instinct is correct. Stick with shared shader + instance attributes.

---

## 10. Refinements Summary

### Must-Do (Blocking)
1. **Consult Tom** for biological gait parameters
2. **Expand buffer layout** from 7 → 9 floats (add phase + speed)
3. **Write shader code** with proper frequency/amplitude coupling

### Should-Do (Important)
1. **Test mesh density:** Validate 8 vertices is sufficient (no clipping, natural deformation)
2. **Tune constants:** Amplitude bounds, frequency scaling factors
3. **Create reference footage:** Real creatures vs. shader creatures side-by-side

### Nice-To-Do (Polish)
1. **LOD system:** Reduce vertex count for distant creatures
2. **Behavior-driven gait:** Seeking behavior → faster wiggle frequency
3. **Environmental effects:** Water density → damping of wiggle
4. **Directional deformation:** Wiggle perpendicular to movement direction

---

## 11. Key Decisions & Questions

### Decision: Vertex Count Per Creature
- **Recommendation:** 8 vertices (2 rings, 4 points each)
- **Rationale:** Balance between visual quality and GPU cost
- **Alternative:** 4 (current) or 16 (higher quality, ~2x cost)

### Decision: Frequency Coupling
- **Recommendation:** `freq = baseFreq * (0.5 + 1.5 * sqrt(speed))`
- **Rationale:** Sublinear speed scaling (natural, matches fish locomotion)
- **Alternative:** Linear `speed`, exponential, or piecewise function

### Decision: Amplitude Scaling
- **Recommendation:** `amplitude = 0.3 + 0.5 * (size - 0.1) / 4.9`
- **Rationale:** Larger creatures have larger absolute displacement
- **Alternative:** Proportional to size (might look too extreme for small creatures)

### Critical Question for Tom
**What is the realistic relationship between swimming speed and undulation frequency in fish/snakes?**

This single parameter drives everything. Without it, we risk creating motion that looks "robotic" rather than alive.

---

## Conclusion

Your Procedural Shader Gait Synthesizer is **sound architecture** with **excellent performance characteristics**. The main risk is **biological grounding** – without consulting Tom, the gait parameters might not look natural.

**Next Steps:**
1. Interview zoologist-tom on swimming locomotion parameters
2. Document findings in `docs/biology/done/procedural-animation.md`
3. Write tests (TDD) for gait signal generation
4. Implement shader with biology-driven parameterization
5. Validate visual output against reference footage

**Estimated Effort:** 3-4 days of focused work (including Tom's consultation).

**Expected Outcome:** 20K creatures on-screen, each with natural, size-dependent, speed-coupled movement – rendering at 60Hz with zero visual jitter.

---

**Sarah (Principal Architect, The Lab)**
