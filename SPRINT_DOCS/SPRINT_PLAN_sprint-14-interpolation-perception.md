# Sprint 14: Interpolation, Vision Refactor & Data-Oriented Design

**Branch:** `feat/sprint-14-interpolation-perception`
**Status:** COMPLETE
**Prerequisites:** Sprint 13 complete (NAPI-RS Zero-Copy Migration)
**Duration:** 3 days (focused scope)

---

## Sprint Goal

**Achieve buttery-smooth 60 FPS frontend rendering** through GPU-accelerated interpolation:

1. **Validate tick rate** (achieved in Sprint 13) → 60Hz interpolated rendering
2. **GPU vertex shader interpolation** (smooth position/rotation, <0.5ms CPU overhead)
3. **Pre-allocated buffer system** (eliminates GC pressure during spawn/despawn)

**Organic wiggle animation moved to Sprint 16**

**Key Architecture:**
- Simulation tick rate defined in `simulation_engine.rs:37` (TARGET_SIMULATION_HZ)
- Custom PixiJS geometry with interleaved vertex buffers
- GPU-side interpolation (parallel execution on all entities)
- Zero-copy NAPI buffer integration

**Backend ECS optimizations moved to Sprint 15**

---

## Team

**Phase 2 Lead (GPU Interpolation & Wiggle):**
- **shader-sarah** (Dr. Sarah Boid) - GPU/Shader specialist
  - WebGL 2.0, GLSL ES 3.0, PixiJS custom geometry
  - Organic procedural animation expert
  - Target: 60 FPS @ 1M entities, <0.5ms CPU, <0.2ms GPU per frame

**Key Collaborators:**
- **zoologist-tom** - 🔥 PRIMARY CREATIVE PARTNER for Sarah
  - Biological motion patterns, creature locomotion physics
  - Natural movement consultation (fish swimming, snake slithering)
  - Ensures visual beauty matches ecological reality
- **frontend-fanny** - PixiJS integration, TypeScript buffer management
- **rusty-ron** - Backend NAPI zero-copy buffers, snapshot format
- **architect-andy** - Performance architecture, technical standards
- **instrumentation-ian** - GPU profiling, frame time metrics
- **pm-pam** - Sprint coordination, task breakdown

---

## Phase Overview

1. **Phase 1:** Validate Tick Rate - ✅ COMPLETE
2. **Phase 2:** Frontend GPU Interpolation - ✅ COMPLETE
   - 2A: Custom PixiJS Geometry Setup - ✅ COMPLETE
   - 2B: Vertex Shader Interpolation (Kinematic Smoothing) - ✅ COMPLETE

**Organic wiggle animation → Sprint 16**
**Backend ECS work (Vision refactor, Uber-struct, Vec2, Parallelization) → Sprint 15**

---

## Phase 1: Validate Tick Rate

**Duration:** Day 1 (COMPLETE - Discovery)

**Goal:** Confirm tick rate achieved in Sprint 13 NAPI migration

**Discovery:**
Sprint 13's NAPI-RS migration introduced tick rate constant in `simulation_engine.rs:37` (TARGET_SIMULATION_HZ). This replaced the old `config.rs` approach.

**Validation:**
- ✅ All systems use `DeltaTime` resource (no hardcoded assumptions)
- ✅ Tick rate stable in NAPI engine
- ✅ No changes needed - already optimal

**Success:** Tick rate stable, ready for Phase 2 interpolation

---

## Phase 2: Frontend Interpolation (60Hz) 🎮 GPU SHADER APPROACH

**Duration:** Days 2-3
**Owner:** shader-sarah (Dr. Sarah Boid)
**Status:** IN PROGRESS
**Technical Spec:** `docs/visuals/shader-smooth-and-wiggle.md`

**Goal:** GPU-accelerated 60Hz rendering with smooth position/rotation interpolation + organic wiggle animation

### Overview

This phase uses **GPU vertex shaders** instead of CPU-based JavaScript interpolation to achieve:
- 60 FPS @ 1 million entities
- <0.5ms CPU overhead per frame
- <0.2ms GPU overhead for interpolation shader
- Zero visual stuttering or "rubber banding"
- Organic procedural animation (wiggle) at near-zero cost

**Key Innovation:** Move interpolation math from CPU (12M ops/sec) to GPU (parallel execution on all entities simultaneously).

### Phase 2A: Custom PixiJS Geometry Setup

**Goal:** Create instanced rendering infrastructure with interleaved attribute buffers.

**Implementation:**
```typescript
// Interleaved Float32Array buffer layout per entity:
// [ startX, startY, endX, endY, startRot, endRot, uvX, uvY ]
const FLOATS_PER_VERTEX = 8;

// Custom PixiJS Geometry
const geometry = new Geometry()
  .addAttribute('aStartPos', buffer, 2, false, FLOAT, stride, 0)
  .addAttribute('aEndPos', buffer, 2, false, FLOAT, stride, 8)
  .addAttribute('aStartRot', buffer, 1, false, FLOAT, stride, 16)
  .addAttribute('aEndRot', buffer, 1, false, FLOAT, stride, 20)
  .addAttribute('aTextureCoord', buffer, 2, false, FLOAT, stride, 24);
```

**Update Strategy:**
- On Server Tick: Copy `end` data to `start`, load new server data into `end`, reset `uInterpolation` to 0
- On Render Frame (60Hz): Increment `uInterpolation` based on `deltaMS / tickIntervalMs`

**Collaboration:** Frontend-Fanny (PixiJS integration), Rusty-Ron (NAPI buffer format validation)

### Phase 2B: Vertex Shader Interpolation (Kinematic Smoothing)

**Goal:** Perfectly smooth linear movement masking low-frequency server updates.

**GLSL Vertex Shader:**
```glsl
// Attributes per entity
attribute vec2 aStartPos;
attribute vec2 aEndPos;
attribute float aStartRot;
attribute float aEndRot;
attribute vec2 aTextureCoord;

// Uniforms (updated every frame)
uniform float uInterpolation;  // 0.0 to 1.0
uniform mat3 uProjection;

void main() {
  // Position interpolation
  vec2 worldPos = mix(aStartPos, aEndPos, uInterpolation);

  // Rotation interpolation (shortest path, handles 350°→10° wraparound)
  float rotation = shortestPathAngle(aStartRot, aEndRot, uInterpolation);

  // Apply rotation + project
  vec2 rotatedPos = rotate(aLocalPos, rotation);
  gl_Position = vec4((uProjection * vec3(worldPos + rotatedPos, 1.0)).xy, 0.0, 1.0);
}
```

**Edge Cases:**
- ✅ Rotation wrapping (350° → 10° = 20° CW, not 340° CCW)
- ✅ Entity spawn/despawn (buffer resizing)
- ✅ Extrapolation when `uInterpolation > 1.0` (network lag)

**Collaboration:** Architect-Andy (performance validation), Instrumentation-Ian (GPU profiling)

---

## Testing Requirements

**Frontend Tests:**
- [x] GPU interpolation smooth at 60 FPS
- [x] No visual artifacts (rubber banding, stuttering)
- [x] Rotation interpolation handles wraparound correctly (350° → 10°)
- [x] Buffer updates synchronize correctly with simulation ticks
- [x] Pre-allocated buffers prevent GC pressure during spawn/despawn
- [x] Buffer capacity growth works correctly
- [ ] Performance stable @ 200K creatures (validation pending)
- [ ] Cross-platform GPU compatibility (Intel/NVIDIA/AMD)

**Integration Tests:**
- [x] Simulation ticks feed interpolation correctly
- [x] Zero-copy NAPI buffers work with custom geometry
- [x] Zoom smoothness maintained at high entity counts

---

## Frontend Architecture

### Key Files

**PixiJS Rendering (`apps/portal/`):**
- Custom geometry with interleaved vertex buffers
- Vertex shader for interpolation (`shaders/creature-interpolation.vert`)
- Fragment shader for creature rendering (`shaders/creature.frag`)
- Buffer management (TypeScript domain layer)
- NAPI buffer integration (zero-copy from Rust)

**Dev-UI Metrics (`apps/dev-ui/`):**
- GPU performance metrics (frame time, shader overhead)
- WebGL profiling integration
- Entity count displays
- Interpolation quality indicators

---

## Success Metrics

**Frontend Performance:**
- [x] 60 FPS stable @ 165fps achieved
- [x] <0.5ms CPU overhead per frame for interpolation
- [x] Zero visual stuttering or rubber banding
- [x] Smooth zoom at high entity counts

**Visual Quality:**
- [x] Creatures move fluidly between simulation ticks
- [x] Rotation interpolation handles angle wraparound correctly
- [x] No "teleporting" or visual artifacts

**Technical:**
- [x] Custom PixiJS geometry with interleaved buffers implemented
- [x] GLSL shaders working (ES 3.0)
- [x] Zero-copy NAPI buffer integration maintained
- [x] Pre-allocated buffer system prevents GC crashes
- [ ] Cross-GPU validation (Intel/NVIDIA/AMD) - pending

**Backend work (ECS optimizations) moved to Sprint 15**
**Organic wiggle animation moved to Sprint 16**

---

## Risks & Mitigations

**Risk:** GPU shader interpolation looks floaty or unnatural
- **Mitigation:** Linear lerp only (no easing), validate with 20K creatures first
- **Status:** RESOLVED - interpolation looks smooth

**Risk:** Cross-platform GPU compatibility issues
- **Mitigation:** Test on Intel/NVIDIA/AMD, use GLSL ES 3.0 (widely supported)
- **Status:** PENDING - needs cross-GPU validation

**Risk:** Buffer synchronization bugs (visual artifacts)
- **Mitigation:** TDD - write tests for buffer update logic
- **Status:** RESOLVED - 254 tests passing

---

## Future Work

**Sprint 15 (Backend ECS Optimizations):**
- Uber-struct refactor (stable archetypes, hot/cold split)
- Vision system optimization (remove Vec allocation bottleneck)
- Vec2 SIMD migration
- Parallelization (multi-core utilization)

**Sprint 16 (Organic Shader Animation):**
- Organic wiggle animation (procedural vertex deformation)
- Movement-coupled animation (speed affects wiggle intensity)
- Biological locomotion patterns (fish swimming, snake slithering)
- zoologist-tom collaboration for biological accuracy

**Sprint 17+ (Advanced Features):**
- DNA-driven `neural_speed` gene (0.5-2.0 multiplier, costs energy²)
- Spatial grid for O(1) vision queries
- Metabolic brain cost (fast reactions = high energy drain)
- Viewport culling (only update visible creatures)
- Variable LOD based on zoom level
- Advanced shader effects (shadows, lighting, water refraction)

---

## References

- **Sprint 13:** NAPI-RS migration (zero-copy buffers, tick rate constant)
- **Sprint 15 (Next):** Backend ECS optimizations
- **Sprint 16:** Organic shader animation (wiggle)
- **Shader spec:** `docs/visuals/shader-smooth-and-wiggle.md`
- **NAPI architecture:** `docs/architecture/napi-architecture.md`
- **Biology notes:** `docs/biology/biology-notes.md`
