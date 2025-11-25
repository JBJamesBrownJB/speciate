# Sprint 14: Frontend GPU Interpolation & Organic Animation

**Branch:** `feat/sprint-14-interpolation-perception`
**Duration:** 3 days (focused scope)
**Status:** IN PROGRESS

---

## Sprint Goal

**MORE DETAIL IN:** SPRINT_PLAN_sprint-14-interpolation-perception.md

**Achieve buttery-smooth 60 FPS frontend rendering** through GPU-accelerated interpolation:
1. Validate 22.2Hz tick rate (✅ achieved in Sprint 13)
2. GPU vertex shader interpolation (smooth position/rotation)
3. Organic wiggle animation (procedural, biologically plausible)
4. Performance validation (60 FPS @ 1M entities target)

**Backend ECS optimizations (Vision refactor, Uber-struct, Vec2, Parallelization) → Sprint 15**

---

## Phase Checklist

### Phase 1: Validate Tick Rate (22.2Hz) - Day 1 ✅ COMPLETE
- [x] Discovered: 22.2Hz achieved in Sprint 13 NAPI migration
- [x] Validated: Hardcoded in simulation_engine.rs:37
- [x] Confirmed: All systems use DeltaTime resource
- [x] Result: 22.2Hz provides ~45ms tick budget (sufficient for 150K-200K target)

### Phase 2: Frontend Interpolation (60Hz) - Days 2-3 🎮 GPU SHADER APPROACH
**Owner:** shader-sarah (Dr. Sarah Boid - GPU/Shader Specialist)
**Status:** CREATURES VISIBLE! Movement/spawn debugging remaining
**Spec:** `docs/visuals/shader-smooth-and-wiggle.md`

#### Phase 2A: Custom PixiJS Geometry Setup ✅ COMPLETE
- [x] Design interleaved Float32Array buffer layout (7 floats: startX/Y, endX/Y, startRot, endRot, size)
- [x] Implement custom PixiJS Geometry with instanced rendering
- [x] Create buffer update strategy (swap END→START on tick)
- [x] InterpolationBufferManager with 17 passing tests
- [x] Double buffering for GPU stall prevention

#### Phase 2B: Vertex Shader Interpolation (Kinematic Smoothing) 🎉 CREATURES VISIBLE
- [x] Implement GLSL vertex shader with mix(aStartPos, aEndPos, uInterpolation)
- [x] Implement rotation interpolation with shortest-path angle wrapping
- [x] Handle edge case: rotation wraparound (350° → 10° = 20° CW, not 340° CCW)
- [x] Handle edge case: entity spawn/despawn (buffer resizing in BufferManager)
- [x] PixiJS v8 API migration complete (UniformGroup pattern)
- [x] All 249 tests passing
- [x] **MILESTONE: Creatures render on screen!**
- [ ] Debug: Creatures not moving (tick integration)
- [ ] Debug: New spawns not appearing
- [ ] Validate smooth interpolation visually
- [ ] Profile: CPU <0.5ms per frame, GPU <0.2ms per frame
- [ ] Cross-GPU testing (Intel/NVIDIA/AMD)

#### Phase 2C: Organic Wiggle Animation
- [ ] Add uGameTime uniform for sine wave animation
- [ ] Implement procedural wiggle: sin(time - uv.y * lag) * amplitude
- [ ] Ensure tail wiggles more than head (uv.y gradient)
- [ ] (Nice-to-have) Dynamic coupling: wiggle frequency scales with velocity
- [ ] Verify ZERO performance regression vs Phase 2B
- [ ] Visual QA: creatures "swim" organically at various zoom levels

#### Phase 2D: Performance Validation & Polish
- [ ] Verify 60 FPS stable @ 1 million entities
- [ ] Confirm no visual stuttering or "rubber banding"
- [ ] Profile WebGL shader performance
- [ ] Cross-platform compatibility testing
- [ ] Collaborate with Instrumentation-Ian on GPU metrics for Dev-UI
- [ ] Document shader architecture and API

---

**Backend ECS work (Phases 3-5) moved to Sprint 15:**
- Phase 3: Uber-Struct Refactor
- Phase 4A-D: Vision Split Queries, Changed<T> Filters, Vec2 Migration, Parallelization
- Phase 5: Performance Validation @ 150K-200K creatures

See: `SPRINT_DOCS/SPRINT_15_PLAN/SPRINT_PLAN_sprint-15-ecs-optimizations.md`

---

## Success Criteria

**Frontend Performance:**
- [ ] 60 FPS stable @ 200K creatures
- [ ] <0.5ms CPU overhead per frame for interpolation
- [ ] <0.2ms GPU overhead for vertex shader
- [ ] Zero visual stuttering or rubber banding
- [ ] Smooth zoom at high entity counts

**Visual Quality:**
- [ ] Creatures move fluidly between simulation ticks
- [ ] Organic wiggle animation looks biologically plausible
- [ ] Rotation interpolation handles angle wraparound correctly
- [ ] No "teleporting" or visual artifacts

**Technical:**
- [ ] Custom PixiJS geometry with interleaved buffers implemented
- [ ] GLSL shaders working across Intel/NVIDIA/AMD GPUs
- [ ] Zero-copy NAPI buffer integration maintained
- [ ] GPU metrics integrated into Dev-UI

**Backend ECS optimizations → Sprint 15**

---

## Current Tasks

**Last Updated:** 2025-11-25 End of Day

### Completed Today
- ✅ PixiJS v8 UniformGroup fix - creatures now render!
- ✅ All 249 tests passing
- ✅ TypeScript compilation clean

### Next Session (Priority Order)
1. **Debug creature movement** - creatures visible but static
   - Check if onSimulationTick() is being called with new creature data
   - Verify interpolationAlpha is advancing in render()
   - Confirm camera uniforms are updating correctly

2. **Debug spawn visibility** - new spawns don't appear
   - Trace creature count through pipeline
   - Verify geometry.instanceCount increments
   - Check buffer updates on spawn

3. **Visual validation** - once movement works
   - Verify smooth interpolation (no teleporting)
   - Test rotation wraparound
   - Check zoom/pan behavior

### Blockers
None - path forward is clear (debug integration, not architecture)
