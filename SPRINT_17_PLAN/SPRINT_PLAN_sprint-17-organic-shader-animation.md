# Sprint 17: Organic Shader Animation & Visual Polish

**Branch:** `feat/sprint-17-organic-shader-animation` (to be created)
**Status:** PLANNED
**Prerequisites:** Sprint 15 complete (ECS Optimizations)
**Duration:** 3-4 days

---

## Sprint Goal

**Bring creatures to life** through GPU-accelerated organic animation:

1. **Organic wiggle animation** (procedural vertex deformation)
2. **Movement-coupled animation** (speed affects wiggle intensity)
3. **Biological locomotion** (fish swimming, snake slithering patterns)
4. **Visual polish** (advanced shader effects)

**Key Architecture:**
- Vertex shader procedural animation
- uGameTime uniform for continuous animation
- Movement speed coupling for natural behavior
- Zero CPU overhead (all GPU-side)

---

## Team

**Sprint Lead:**
- **shader-sarah** (Dr. Sarah Boid) - GPU/Shader specialist
  - WebGL 2.0, GLSL ES 3.0, procedural animation
  - Target: Organic motion at near-zero GPU cost

**Key Collaborators:**
- **zoologist-tom** - PRIMARY CREATIVE PARTNER
  - Biological motion patterns, creature locomotion physics
  - Natural movement consultation (fish swimming, snake slithering)
  - Ensures visual beauty matches ecological reality
- **frontend-fanny** - PixiJS integration
- **instrumentation-ian** - GPU profiling, performance validation
- **architect-andy** - Performance architecture
- **pm-pam** - Sprint coordination

---

## Phase Overview

1. **Phase 1:** Organic Wiggle Animation (Days 1-2)
2. **Phase 2:** Movement Coupling & Polish (Day 2-3)
3. **Phase 3:** Performance Validation (Day 3-4)

---

## Phase 1: Organic Wiggle Animation

**Duration:** Days 1-2
**Owner:** shader-sarah

**Goal:** Inject "life" using procedural vertex deformation (fish swimming, snake slithering).

### Enhanced Vertex Shader

```glsl
uniform float uGameTime;

void main() {
  // Calculate movement speed for dynamic coupling
  float moveSpeed = length(aEndPos - aStartPos) / 0.045;  // pixels/sec

  // Wiggle algorithm (in local space, before world transform)
  float lagFactor = 3.0;  // Tail lags behind head
  float amplitude = 5.0 * (moveSpeed / 100.0);  // Scale with speed
  float wiggleOffset = sin(uGameTime * 2.0 - aTextureCoord.y * lagFactor) * amplitude;

  vec2 localPos = aLocalVertexPos;
  localPos.x += wiggleOffset * aTextureCoord.y;  // head fixed, tail wiggles

  // ... rest of interpolation shader (position, rotation)
}
```

### Implementation Tasks

1. **Add uGameTime uniform** to existing interpolation shader
2. **Implement sin-wave wiggle** with texture coord-based lag
3. **Test S-curve motion** (tail trails behind head)
4. **Consult zoologist-tom** for biological accuracy

### Success Criteria

- Creatures appear to "swim" organically
- Tail lags behind head (S-curve motion)
- Wiggle visible at all zoom levels
- **ZERO performance regression** vs current interpolation

---

## Phase 2: Movement Coupling & Polish

**Duration:** Day 2-3

**Goal:** Make animation intensity correlate with creature movement speed.

### Dynamic Speed Coupling

```glsl
// Movement speed affects wiggle intensity
float speedFactor = clamp(moveSpeed / 100.0, 0.1, 2.0);
float amplitude = baseAmplitude * speedFactor;

// Stationary creatures have minimal wiggle
// Fast-moving creatures have pronounced S-curve
```

### Polish Tasks

1. **Speed-dependent amplitude** (fast = more wiggle)
2. **Idle animation** (subtle breathing/pulsing when stationary)
3. **Size-based frequency** (large creatures wiggle slower)
4. **Direction-aware wiggle** (perpendicular to movement)

### Zoologist Collaboration

Consult zoologist-tom for:
- Realistic wiggle frequencies by creature size
- Movement patterns for different creature types
- Biological constraints on motion amplitude

---

## Phase 3: Performance Validation

**Duration:** Day 3-4

**Goal:** Ensure organic animation adds zero measurable overhead.

### Performance Metrics

- 60 FPS stable @ 200K creatures (Chrome DevTools)
- GPU <0.2ms per frame (WebGL profiler)
- Zero visual artifacts at 1x-10x zoom
- Cross-GPU compatibility (Intel/NVIDIA/AMD)

### Testing Checklist

- [ ] Wiggle animation produces organic-looking motion
- [ ] Performance stable @ 200K creatures
- [ ] Cross-platform GPU compatibility (Intel/NVIDIA/AMD)
- [ ] No visual artifacts at extreme zoom levels
- [ ] Speed coupling feels natural
- [ ] Large creatures wiggle slower than small

### Benchmark Targets

| Entity Count | Frame Time Budget | Wiggle Overhead |
|--------------|------------------|-----------------|
| 50K | <8ms | <0.1ms |
| 100K | <12ms | <0.1ms |
| 200K | <16ms | <0.1ms |

---

## Testing Requirements

**Visual Tests:**
- [ ] Wiggle animation looks biologically plausible
- [ ] S-curve motion visible on moving creatures
- [ ] Speed coupling feels natural
- [ ] Idle creatures have subtle animation
- [ ] Large/small creatures animate differently

**Performance Tests:**
- [ ] Zero FPS regression vs non-wiggle shader
- [ ] GPU profiler shows <0.2ms overhead
- [ ] Stable across Intel/NVIDIA/AMD GPUs

**Integration Tests:**
- [ ] Works with existing interpolation
- [ ] Works with zoom/pan camera
- [ ] Works with spawn/despawn dynamics

---

## Success Metrics

**Visual Quality:**
- [ ] Creatures move fluidly between simulation ticks
- [ ] Organic wiggle animation looks biologically plausible
- [ ] Tail trails behind head during movement
- [ ] Speed-coupled animation feels natural
- [ ] Large creatures appear ponderous, small appear nimble

**Performance:**
- [ ] 60 FPS stable @ 200K creatures
- [ ] <0.2ms GPU overhead for wiggle shader
- [ ] Zero visual artifacts at any zoom level
- [ ] Cross-platform GPU compatibility

---

## Risks & Mitigations

**Risk:** Wiggle animation causes performance regression
- **Mitigation:** Profile GPU performance before/after, ensure <0.2ms overhead
- **Fallback:** Make wiggle optional via shader uniform

**Risk:** Animation looks artificial or "floaty"
- **Mitigation:** Collaborate with zoologist-tom for biological accuracy
- **Fallback:** Tune parameters based on playtesting

**Risk:** Cross-platform GPU compatibility issues
- **Mitigation:** Test on Intel/NVIDIA/AMD, use GLSL ES 3.0 (widely supported)
- **Fallback:** Disable wiggle on problematic GPUs

---

## Future Work (Sprint 17+)

- Advanced shader effects (shadows, lighting, water refraction)
- DNA-driven animation parameters (some creatures wiggle more)
- Environmental effects (current flow affects wiggle)
- Predator/prey specific animations (lunging, fleeing)
- Variable LOD animation (reduce wiggle at extreme zoom-out)

---

## References

- **Sprint 14:** GPU interpolation foundation (COMPLETE)
- **Sprint 15:** Backend ECS optimizations (prerequisite)
- **Shader spec:** `docs/visuals/shader-smooth-and-wiggle.md`
- **Biology notes:** `docs/biology/biology-notes.md`
