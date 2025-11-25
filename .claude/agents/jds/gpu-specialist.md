# 🎮 JOB DESCRIPTION: GPU/Shader Specialist

Position: WebGL Graphics Engineer (Shader Specialist)

Project: Speciate - Massive-Scale A-Life Simulation
Team: Frontend Rendering Team
Location: Remote / Embedded Agent
Type: Contract - Sprint 14 & 15

---

## 🎯 Mission

Design and implement a GPU-accelerated entity rendering pipeline using custom GLSL shaders in PixiJS to
achieve butter-smooth 60Hz rendering of 1 million+ autonomous creatures while the backend simulation runs at
22.2Hz.

You'll be the shader wizard who makes our creatures swim, wiggle, and wobble with organic life—all happening on the GPU at near-zero CPU cost.

---

## 🔥 The Challenge

Current State:
- Backend simulation: 22.2Hz (45ms ticks)
- Target rendering: 60Hz (16.67ms frames)
- Entity count: up to 1 MILLION creatures simultaneously
- Problem: CPU-based sprite updates = 12M operations/sec = frame drops

Your Mission:
- Implement GPU-based position/rotation interpolation (Phase 1)
- Add procedural "wiggle" animation using vertex shaders (Phase 2)
- Achieve 60 FPS with <0.5ms CPU overhead
- Make it look organic and alive

---

## 🛠️ Technical Stack

You'll Work With:
- WebGL 2.0 - Your canvas
- GLSL ES 3.0 - Your language
- PixiJS v7/8 - Rendering framework (custom geometry/shaders)
- TypeScript - Frontend integration
- Rust/NAPI - Backend data source (zero-copy buffers)

Architecture:
```
Rust (22.2Hz) → Zero-Copy Buffer → TypeScript → GPU Shader (60Hz)
                  [Positions]        [Upload]     [Interpolate + Wiggle]
```

---

## 📋 Deliverables

### Phase 1: Kinematic Smoothing (Week 1-2)

Goal: Perfectly smooth linear movement masking 22.2Hz updates

Technical Requirements:
1. Custom PixiJS Geometry
  - Interleaved Float32Array buffer (start/end pos/rot per entity)
  - Instanced rendering for 1 million entities
  - Efficient buffer update strategy (swap prev←curr on snapshot)
2. Vertex Shader (interpolation.vert)

```glsl
// Attributes per entity:
attribute vec2 aStartPos;
attribute vec2 aEndPos;
attribute float aStartRot;
attribute float aEndRot;

// Uniform (updated every frame):
uniform float uInterpolation; // 0.0 to 1.0

// Your magic here:
void main() {
  vec2 worldPos = mix(aStartPos, aEndPos, uInterpolation);
  float rotation = shortestPathAngle(aStartRot, aEndRot, uInterpolation);
  // ... rotate + project
}
```

3. Edge Cases You Must Handle:
  - ✅ Rotation wrapping (350° → 10° should rotate 20° CW, not 340° CCW)
  - ✅ Entity spawn/despawn (handle buffer resizing)
  - ✅ Extrapolation when uInterpolation > 1.0 (network lag)

Success Criteria:
- 1 million entities render at stable 60 FPS
- No visual stuttering or "rubber banding"
- CPU usage <0.5ms per frame for interpolation
- Works on Intel/NVIDIA/AMD GPUs (driver compatibility)

---

### Phase 2: Organic Wiggle (Week 3-4)

Goal: Make creatures look alive with procedural vertex deformation

Technical Requirements:
1. Vertex Shader Enhancement

```glsl
// New uniforms:
uniform float uGameTime;

// Wiggle algorithm (in local space, before world transform):
float wiggleOffset = sin(uGameTime - aVertexUV.y * lagFactor) * amplitude;
localPos.x += wiggleOffset * aVertexUV.y; // tail wiggles more than head
```

2. Dynamic Coupling (Nice-to-Have)
  - Calculate distance(aStartPos, aEndPos) in shader
  - Modulate wiggle frequency based on movement speed
  - Fast movement = furious wiggling; idle = gentle drift
3. Performance Constraint:
  - Phase 2 FPS must match Phase 1 (no regression)
  - Wiggle complexity: O(1) per vertex (simple sine wave)

Success Criteria:
- Creatures appear to "swim" organically
- Tail lags behind head (creates S-curve motion)
- Wiggle intensity correlates with speed
- Zero FPS impact vs Phase 1

---

## 🎓 Required Skills

Must Have:
- ✅ GLSL - Expert-level vertex/fragment shader programming
- ✅ WebGL 2.0 - Attribute buffers, uniforms, instanced rendering
- ✅ Linear algebra - Matrix transforms, vector math, quaternion/euler conversions
- ✅ Performance optimization - GPU profiling, draw call minimization
- ✅ Debugging - Chrome DevTools, WebGL Inspector, RenderDoc

Nice to Have:
- ⭐ PixiJS - Custom geometry, shader integration
- ⭐ Procedural animation - Sine waves, noise functions, vertex deformation
- ⭐ Game dev - Interpolation, extrapolation, networked entity sync
- ⭐ TypeScript - Frontend integration (you'll write some glue code)

Bonus:
- 🎨 Understanding of organic motion (fish swimming, snake slithering)
- 🔬 Familiarity with A-Life or particle systems
- 📊 Experience with large-scale rendering (10K+ entities)

---

## 👥 Collaboration

You'll Work Closely With:
- Frontend-Fanny - PixiJS integration, TypeScript buffer management
- Rusty-Ron - Backend snapshot format, NAPI zero-copy buffers
- Architect-Andy - Performance benchmarks, fallback strategies
- Instrumentation-Ian - GPU profiling, frame time analysis

Communication Style:
- Technical depth appreciated (we love shader pseudocode!)
- Show your work (explain interpolation vs extrapolation trade-offs)
- Visual examples welcome (shader toy demos, GIFs of wiggle effects)

---

## 📏 Success Metrics

Phase 1 (Interpolation):
- ✅ 60 FPS at 1 million entities (measured in Chrome DevTools)
- ✅ <0.5ms CPU time per frame (profiled)
- ✅ <0.2ms GPU time for interpolation shader (WebGL profiler)
- ✅ Zero visual artifacts (manual QA at various zoom levels)

Phase 2 (Wiggle):
- ✅ Organic movement visible at 1x-10x zoom
- ✅ Wiggle frequency scales with velocity (visual QA)
- ✅ No performance regression vs Phase 1
- ✅ Works across GPU vendors (tested on 3+ machines)

---

## 🧪 Technical Assessment (Interview Task)

To prove your chops, we'd love to see:

Mini Challenge:
"Create a PixiJS demo with 10,000 entities using a custom vertex shader. Each entity should smoothly
interpolate between two positions (updated every 50ms) while the shader renders at 60 FPS. Bonus: Add a
simple sine-wave wiggle effect."

What We're Looking For:
- Clean GLSL code with comments
- Efficient buffer management (interleaved attributes)
- Proper rotation interpolation (shortest path)
- Performance metrics in the demo (FPS counter)

Time Estimate: 2-4 hours
Submission: CodeSandbox, GitHub Gist, or Shader Toy link

---

## 💰 Compensation

Contract Details:
- Duration: 4-6 weeks (Sprints 14-15)
- Hourly rate: [Based on experience]
- Deliverable-based milestones:
  - 30% - Phase 1 interpolation shader working
  - 30% - Phase 1 optimization + edge cases
  - 20% - Phase 2 wiggle implementation
  - 20% - Cross-platform testing + documentation

Bonus:
- +10% if Phase 2 wiggle blows our minds
- +10% if you discover additional GPU optimization opportunities

---

## 📚 Resources We'll Provide

- Full access to codebase (apps/portal/ frontend)
- Design doc: /docs/visuals/shader-smooth-and-wiggle.md
- Sample snapshot data (Rust backend output)
- Team Slack/Discord for real-time Q&A
- GPU test machines (if you don't have varied hardware)

---

## 🚀 Why This Role is Exciting

1. Impact: Your shaders will power a simulation with 1 million living entities
2. Visibility: This is the core rendering tech for the entire game
3. Creative Freedom: We trust your expertise—show us what GPUs can do!
4. Learning: Work with cutting-edge NAPI zero-copy buffers (Rust↔JS)
5. Portfolio: Ship a large-scale WebGL project (great for your reel)

---

## 📝 How to Apply

Send us:
1. Resume/portfolio (GitHub, Shader Toy, personal site)
2. Mini challenge submission (or equivalent shader work)
3. Brief intro: "Why I'm excited about GPU-accelerated A-Life rendering"

Interview Process:
1. Technical screen (30 min) - Discuss shader architecture
2. Code review (45 min) - Walk through your mini challenge
3. Team fit (30 min) - Meet Frontend-Fanny & Architect-Andy
4. Offer 🎉

---

## 🎯 Start Date

Immediate - Sprint 14 Phase 2 starts now!

---

Questions? Drop them in the thread. We're here to help you succeed!

---

Equal Opportunity: We hire based on shader wizardry, not background. All skill levels welcome if you can
make 1 million entities wiggle smoothly. 🐛✨