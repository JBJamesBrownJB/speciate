# Procedural Gait Synthesis

## Problem / Opportunity

Current creatures move as sliding circles with no visual character. A 0.5m zebra-sized creature and a 5.0m elephant-sized creature appear identical in motion - both glide smoothly without expressing their physical differences. This breaks immersion and wastes an opportunity to create emergent predator-prey signaling through movement patterns.

**Opportunity:** GPU-accelerated vertex shader animation can inject biological movement character into 20K+ creatures at near-zero CPU cost, creating both visual realism AND gameplay-relevant information.

## Proposed Solution

**Core Concept:** Generate procedural gait patterns in the vertex shader based on creature size and movement speed, following allometric scaling laws. Instead of baked animations, compute a "GaitSignal" (0.0-1.0 sine wave) that drives vertex deformation in real-time.

### Visual Character Examples

- **Small creature (0.5m, "Zebra"):** High-frequency twitchy steps (3x faster), sharp galloping motion with flight phase, minimal lateral sway
- **Large creature (5.0m, "Elephant"):** Low-frequency lumbering stride, heavy pendulum sway, deep vertical bob
- **Medium speed:** Smooth undulation with tail-lag phase (head leads, tail follows)
- **Exhausted creature:** Irregular jitter, degraded gait smoothness (visible fatigue signal)

### Shader Architecture ("Uber-Gait" System)

Single unified shader handles all gait styles via blending uniforms:

**Per-creature instance attributes:**
- `aSize` (float): Creature scale (0.5-5.0)
- `aSpeed` (float): Current velocity magnitude
- `aEnergy` (float): Energy ratio (0.0-1.0)
- `aHealth` (float): Health ratio (0.0-1.0)
- `aGaitPhase` (float): Random offset (0.0-1.0, prevents lockstep animation)
- `aGaitStyle` (enum): 0=swim, 1=walk, 2=gallop (future extensibility)

**Global uniforms:**
- `uGameTime` (float): Continuous time for animation cycles
- `uBaseFrequency` (float): Reference step frequency (e.g., 5.0 Hz)
- `uBaseAmplitude` (float): Reference wiggle magnitude

**Vertex shader output:** Deformed mesh with procedural bob/sway/wiggle before world transform

### Mathematical Foundation (Allometric Scaling)

**Step Frequency Scaling:**
```
actual_frequency = base_freq × (ref_size / creature_size)^0.33 × (speed / max_speed)
```
- Biological basis: Muscle twitch rate scales with surface-area-to-volume ratio
- Result: 0.5m creature steps ~1.5x faster than 1.0m, 5.0m creature ~0.6x speed

**Amplitude Scaling:**
```
actual_amplitude = base_amplitude × (speed / max_speed)^0.67
```
- Biological basis: Stride power scales sublinearly with velocity
- Result: Faster movement = more pronounced bob/sway

**Phase Lag (Spine S-curve):**
```
local_phase = gait_phase - (uv.y × 3.0)
```
- Head (uv.y=0): Full frequency, zero lag
- Tail (uv.y=1): Maximum delay, creates traveling wave
- Result: Natural undulation along body axis

**Fatigue Jitter:**
```
jitter = noise(time) × (1.0 - energy) × (1.0 - health)
```
- Healthy creature: Smooth gait (no jitter)
- Exhausted creature: Irregular wobble (visible weakness)

### Gait Style Blending (Shadertoy-Inspired)

**Gallop (high-speed asymmetric):**
```glsl
float gallop = max(0.0, sin(time * freq));  // Clipped sine (ground contact)
pos.y += gallop * amplitude * 0.5;  // Sharp bounce
```

**Lumber (low-speed pendulum):**
```glsl
float walk_cycle = sin(time * freq);
float sway_cycle = cos(time * freq * 0.5);  // Half frequency
pos.y += abs(walk_cycle) * amplitude * 0.2;  // Vertical bob
pos.x += sway_cycle * amplitude * 0.3;  // Lateral sway
```

**Scuttle (insect/rodent jitter):**
```glsl
float jitter = sin(time * 50.0) * 0.05;  // High frequency, low amplitude
pos.x += jitter;
```

**Blending strategy:** Use `mix()` between styles based on speed thresholds or future DNA-encoded gait preference.

## Golden Zone

**This is a quintessential Golden Zone opportunity:** The performance optimization (GPU vertex deformation) IS the biological feature (allometric gait scaling).

### Emergent Behavior (Free Gameplay)

**1. Size Estimation from Gait Frequency**
- Predators learn to estimate prey size by watching step frequency
- Slow gait = large creature (potentially dangerous or not worth energy)
- Fast gait = small creature (easy target if caught)
- Creates visual hunting strategy without explicit perception code

**2. Fatigue Detection from Gait Irregularity**
- Smooth gait = healthy, vigorous prey (hard target)
- Jerky gait = exhausted/injured prey (EASY target)
- Predators develop preference for irregular gaiters
- Emergent dynamic: Tired creatures become preferentially hunted

**3. Species Recognition**
- Different gait styles (gallop vs lumber vs scuttle) become species signatures
- Predators specialize on specific gait patterns
- Prey evolve gait mimicry (small aggressive species mimic large lumbering gait)

**4. Arousal/Threat State Signaling**
- Large amplitude = alert, energetic, dangerous
- Small amplitude = relaxed, vulnerable
- Frozen (zero amplitude) = camouflage attempt

**Performance win:** GPU shader runs on 20K+ creatures with <1ms overhead (trivial vertex math)
**Gameplay win:** Rich predator-prey signaling emerges from realistic physics

### Why This is Golden Zone

| Traditional Approach | Golden Zone Approach |
|---------------------|---------------------|
| Bake animations (memory/CPU cost) | Procedural shader (zero CPU, minimal GPU) |
| Hardcode "tired" animation state | Fatigue emerges from energy parameter |
| Explicit "size detection" AI | Size emerges from gait frequency |
| Separate predator hunting logic | Hunting emerges from gait analysis |

**One system serves multiple purposes:**
- Visual realism (biological accuracy)
- Performance optimization (GPU parallelism)
- Gameplay mechanics (predator-prey signaling)
- Emergent ecology (gait-based niches)

## Trade-offs

**Benefits:**
- Biologically accurate allometric scaling (free from physics)
- Infinite scalability (GPU parallel processing)
- Zero CPU overhead (no animation state machines)
- Emergent predator-prey dynamics (gait = information)
- Visual variety without animator intervention

**Costs:**
- Requires mesh creatures (not simple circle sprites) - needs geometry
- Shader complexity (gait blending, phase lag, fatigue modulation)
- Tuning overhead (biological parameters must match real locomotion data)
- Perception system integration (predators need to "read" gait signals)

**Key Trade-off:** Mesh rendering cost vs sprite simplicity
- Sprites: Cheap, but lifeless (no vertex deformation)
- Meshes: Moderate GPU cost, but enables organic animation
- Decision: Already committed to mesh-based rendering (interpolation system exists)

## Expert Input

### Shader-sarah (Technical Feasibility)

**Architecture:** Seamless integration with existing `InterpolatedCreatureRenderer` (no redesign needed). Use shared shader with instance attributes.

**Performance:** Adding per-vertex gait deformation costs only 1-2ms for vertex shader at 20K creatures (comfortable headroom within 165 FPS budget).

**Mesh Density:** 8 vertices per creature (2 rings, 4 points each) balances detail vs GPU cost. Adequate for visible undulation without excessive geometry.

**Critical Gaps Identified:**
1. Uniform set lacks biological grounding - need frequency coupled to speed/size, not arbitrary
2. Gait signal parameterization underspecified - need tail-lag phase, amplitude scaling exponents
3. Missing per-creature desynchronization - pure `uTime` creates lockstep animation (use `aGaitPhase` offset)

**Implementation Pattern:** Expand buffer layout from 7 → 9 floats to include `aGaitPhase` and `aSpeedMagnitude`. Use traveling wave formula with allometric modulation.

**Gotchas:**
- Rotation interaction: Apply gait in local space BEFORE world rotation
- Clipping: Excessive amplitude causes mesh self-intersection
- Aliasing: High-frequency gaits need sufficient mesh density

### Zoologist-tom (Biological Validation)

**Allometric Scaling Confirmed:** Size-frequency relationship is biological law, not suggestion:
- Step frequency ∝ size^(-0.33)
- Stride length ∝ size^(0.67)
- Top speed ∝ size^(0.25)

**Real-world validation:**
- Insect (0.01m): 20-40 Hz leg beat
- Mouse (0.1m): 10-15 Hz
- Wolf (1.0m): 2-3 Hz
- Elephant (4m): 0.8-1.2 Hz

**Speed Affects BOTH Frequency AND Amplitude:**
- Frequency: f ∝ speed^1.0 (linear)
- Amplitude: A ∝ speed^0.67 (sublinear)
- High amplitude + high frequency = maximum speed

**Critical Missing Parameters:**
1. **Energy state (fatigue gait degradation):**
   - Tired creatures show irregular gait (Perlin noise modulation)
   - Frequency drops 20% when exhausted
   - Visible to predators as "easy prey" signal

2. **Directional phase lag (head-to-tail S-curve):**
   - Head leads, tail follows with 3-radian phase delay
   - Increases realism 10x (prevents rigid-body appearance)

3. **Terrain reaction (future):**
   - Uphill: Higher frequency, lower amplitude (struggling)
   - Downhill: Lower frequency, higher amplitude (confident)

4. **Acceleration overshoot (springy startup):**
   - Sudden acceleration creates brief frequency spike
   - Mimics "spring into action" biology

**Gait as Fitness Signal (Confirmed):**
- Parasites/disease → neuromuscular dysfunction → irregular gait
- Malnourishment → weak muscles → jerky movement
- Injury → compensation → asymmetrical gait
- Old age → slower, stiffer gait

**In nature:** Predators preferentially target irregular gaiters (sick, injured, old). This behavior emerges for free in simulation if perception system reads gait jitter.

### Shadertoy/Spore Research (Implementation Examples)

**Spore's Procedural Gait (Conceptual Gold Standard):**
- Used harmonic oscillators instead of keyframe animations
- Heavy creatures: Low-frequency, high-amplitude (deep center-of-mass dip)
- Light creatures: High-frequency, low-amplitude (twitchy jitter)
- Formula: `pow(sin(x), k)` to sharpen movement (gallop "flight phase")

**Shadertoy Patterns (Direct Code References):**
- "Procedural Walk Cycle" by iq: `pow(sin(x), k)` for sharp stepping
- "Happy Jumping" by iq: Velocity-driven squash/stretch (lumbering effect)
- "Vertex Displacement" examples: GPU-skinned instancing for thousands of units

**Gait Math Breakdown:**
- Gallop: `max(0.0, sin(t))` - clipped sine simulates ground contact
- Lumber: `sin(t) + cos(t/2)` - vertical bob + lateral sway (figure-8 motion)
- Scuttle: `sin(t * 50) * 0.05` - high frequency, low amplitude

**Uber-Gait Strategy:** Don't write three shaders. Write one with `uGaitStyle` uniform (0=swim, 1=walk, 2=gallop). Blend between styles using `mix()`.

## Dependencies

**Must exist first:**
- Mesh-based creature rendering (sprites won't support vertex deformation)
- Per-creature instance attributes pipeline (size, speed, energy already passed to GPU)
- PixiJS shader integration (custom vertex shaders working with renderer)

**Nice to have (not blockers):**
- Perception system that reads gait patterns (for predator-prey signaling)
- DNA-encoded gait preferences (walk vs gallop species archetypes)
- Terrain system (for slope-dependent gait modulation)

**Current status check:**
- Mesh rendering: Exists (see `shader-smooth-and-wiggle.md` - interleaved buffer with 8 floats/vertex)
- GPU interpolation: Implemented (Sprint 14, double-buffer architecture)
- Instance attributes: Partial (position/rotation exist, need to add energy/health/phase)

## Related Ideas

**Implemented:**
- `docs/visuals/ideas/shader-animation.md` - Breathing/undulation/micro-movement (Phase 2: Undulation is THIS idea)
- `docs/visuals/ideas/shader-smooth-and-wiggle.md` - GPU interpolation foundation (provides architecture for gait shader)
- `docs/biology/done/movement-physics.md` - Allometric scaling laws for speed/acceleration/turn rate (gait extends these laws to animation)

**Synergies:**
- Perlin locomotion noise (existing): Could blend with gait jitter for compound effect
- Size-based turning: Visual gait matches physical agility (small = twitchy gait + tight turns)
- Fatigue system (future): Energy state modulates both physics AND gait appearance

**Conflicts:**
- None identified. Gait shader is additive (enhances existing mesh rendering without replacing systems)

## Open Questions

**For shader-sarah (Technical):**
1. Should we use `aGaitStyle` enum now, or defer multi-style blending to later sprint?
2. Mesh density: Is 8 vertices sufficient for visible tail-lag, or do we need 12-16?
3. Buffer expansion: Add 2 floats (energy, phase) to existing layout, or rebuild entire buffer structure?

**For zoologist-tom (Biological):**
1. Swimming frequency vs velocity: Linear relationship or sublinear? (affects shader formula)
2. Tail-lag phase delay: How many radians for realistic S-curve? (current guess: 3.0)
3. Realistic frequency bounds: What's idle frequency (0.3-0.5 Hz?) and max-speed frequency (3-5 Hz?)
4. Fatigue jitter threshold: At what energy level does gait become visibly irregular? (current guess: <50%)

**For integration (Design):**
1. Should predators explicitly "perceive" gait jitter, or emerge from visual detection of irregular movement?
2. DNA genes for gait: Add explicit gait-style genes, or derive everything from existing size/speed genes?
3. Terrain integration: Defer slope-dependent gait to post-terrain-system sprint?

## Implementation Phases (Suggested Roadmap)

**Phase 1: Core Allometric Gait (Sprint 16-17)**
- Size-frequency scaling (size^-0.33)
- Speed modulation (linear frequency, 0.67 amplitude)
- Phase lag along spine (uv.y based)
- Validation: 10 creatures (0.5m-5.0m), confirm gait matches allometric law

**Phase 2: Fatigue Integration (Sprint 18)**
- Energy-ratio modulation of frequency/amplitude
- Jitter based on (1 - health_ratio)
- Validation: Animated tired creature, visual smoothness degradation

**Phase 3: Predator-Prey Perception (Sprint 19+)**
- Perception system reads gait frequency + jitter
- Predators estimate size/fitness from gait patterns
- Emergent hunting strategies
- Validation: Predators preferentially target irregular gaiters

**Phase 4: Multi-Style Blending (Future)**
- Implement gallop/lumber/scuttle distinct styles
- DNA-encoded gait preference
- Speed-threshold-based automatic style switching

## Success Criteria

- [ ] 0.5m creatures wiggle ~1.5x faster than 1.0m creatures (allometric validation)
- [ ] 5.0m creatures show deep lumbering bob, small creatures show twitchy scuttle
- [ ] Exhausted creatures (energy <30%) show visible gait irregularity
- [ ] Shader overhead <2ms GPU time @ 20K creatures
- [ ] No two creatures move identically (gait phase desynchronization)
- [ ] Tail lags behind head (visible S-curve undulation)
- [ ] Stationary creatures (speed <0.01) show no gait (idle breathing only)

## Next Steps

1. **Consult shader-sarah:** Implement prototype shader with size-frequency-speed formula
2. **Consult zoologist-tom:** Validate allometric exponents against real animal data
3. **Expand buffer layout:** Add `aEnergy`, `aGaitPhase` to instance attributes
4. **Write TDD tests:** Gait signal generation (frequency, amplitude, jitter calculations)
5. **Visual validation:** Render side-by-side 0.5m vs 5.0m creatures, confirm 1.5x frequency difference

---

*Captured: 2025-12-28*
*Category: Visuals (GPU shader animation)*
*Expert Consultations: shader-sarah (a4f4b19), zoologist-tom (a8050ae)*
*Related: shader-animation.md (Phase 2), shader-smooth-and-wiggle.md (architecture), movement-physics.md (allometric laws)*
