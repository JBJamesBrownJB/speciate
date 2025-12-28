# Organic Shader Animation

**Status:** Idea (Sprint 17 proposal)
**Category:** GPU-accelerated procedural creature animation

---

## Goal

Replace static sprites with shader-driven organic movement (breathing, undulation, micro-movements) to make creatures feel alive without animator intervention.

**Prerequisites:** GPU interpolation working (Sprint 14)
**Target:** 200K creatures with unique organic motion @ 165 FPS

---

## High-Level Phases

### Phase 1: Breathing Shader

**Outcome:** Procedural expansion/contraction based on creature state (resting = slow breath, fleeing = rapid panting)

**Key Decisions:**
- Sine wave modulation with state-driven frequency
- Amplitude scales with body size (larger = more visible breathing)
- Link to energy level (low energy = shallow breathing)

### Phase 2: Undulation/Gait Shader

**Outcome:** Movement-synchronized body wave (walk cycle, slither, swim depending on locomotion)

**Detailed Specification:** See `procedural-gait-synthesis.md` for comprehensive design including:
- Allometric scaling (size^-0.33 frequency, biologically accurate)
- Multiple gait styles (gallop, lumber, scuttle)
- Fatigue/health modulation (gait irregularity = fitness signal)
- Golden Zone: Emergent predator-prey signaling via gait patterns

**Key Decisions:**
- Velocity-driven wave frequency (faster movement = faster undulation)
- Phase offset per creature for visual variety
- Directional wave orientation (head-to-tail for quadrupeds)

### Phase 3: Micro-Movement Noise

**Outcome:** Subtle Perlin noise overlay for natural jitter (never perfectly still like robots)

**Key Decisions:**
- Time-based seed for continuous variation
- Low amplitude (<1% scale) to avoid distraction
- Higher frequency when alert/stressed

---

## Biological Context

Real creatures are NEVER static:
- Breathing: 12-60 cycles/minute depending on size/activity
- Postural sway: Even stationary animals micro-adjust balance
- Alert behavior: Ear flicks, head movements, weight shifts

Shaders enable this at zero CPU cost - all computation on GPU in parallel.

---

## Technical Context

**Why Shaders?** Animating 200K creatures with skeletal rigs is CPU/memory prohibitive. Procedural shaders scale infinitely with zero overhead.

**Pattern:** Pass creature state (velocity, energy, behavior mode) as uniforms → shader computes organic deformation per-vertex.

**Performance:** Expect <1ms GPU overhead @ 200K creatures (trivial fragment shader math).

---

## Gameplay Impact

**Player Perception:** Motion is life - organic animation makes creatures feel like living beings instead of moving icons.

**Emergent Detail:** Herd movement becomes visually mesmerizing when each creature undulates independently.

**Stress Indication:** Rapid breathing/jitter during panic creates immediate visual feedback of creature state.

---

## Success Criteria

- [ ] Breathing shader synchronized with creature energy/behavior
- [ ] Undulation scales with velocity (stationary = no wave, sprinting = rapid)
- [ ] Micro-movement noise active on all creatures
- [ ] Shader overhead <1ms GPU time @ 200K creatures
- [ ] Visual variety - no two creatures move identically
- [ ] Smooth integration with existing GPU interpolation system
