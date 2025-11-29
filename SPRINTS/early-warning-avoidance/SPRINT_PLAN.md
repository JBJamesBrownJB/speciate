# Sprint: Early Warning Avoidance - Trajectory Tweaking

**Theme:** Add gentle, long-range course corrections to create smoother, more realistic creature navigation

**Goal:** Implement speed-adaptive early warning zones that trigger gentle avoidance forces at 3-5× personal_space distance, eliminating jerky last-second swerves and creating smooth, organic trajectories.

**Prerequisites:** None (builds on existing avoidance system)

**Expected Duration:** 3-4 hours

**Target Behavior:** Creatures begin gentle trajectory tweaks at 8-9m distance instead of hard swerves at 2.5m

---

## High-Level Phases

### Phase 1: Add Perception Constants
**Outcome:** Three new constants in `perception/constants.rs` defining early warning zone parameters

**Key Decisions:**
- `EARLY_WARNING_MULTIPLIER`: 3.0 (base zone is 3× personal_space)
- `SPEED_WARNING_FACTOR`: 0.5 (fast creatures get extra lead time)
- `EARLY_WARNING_FORCE_RATIO`: 0.15 (gentle nudge, 15% of AVOIDANCE_FORCE)

### Phase 2: Implement Speed-Adaptive Warning Distance
**Outcome:** Avoidance system calculates dynamic early warning zone based on current speed

**Key Decisions:**
- Formula: `base_warning + (speed_ratio × personal_space × factor)`
- Slow creatures (2 m/s): 8.0m early warning
- Fast creatures (8 m/s): 8.75m early warning
- Provides stopping distance safety margin

### Phase 3: Add Early Warning Force Application
**Outcome:** New force zone applied before existing personal_space logic

**Key Decisions:**
- Mutually exclusive zones (early OR personal OR panic, not multiple)
- Weak force (5.25N for 35N avoidance base) for gentle nudging
- Energy modulation applies to early warning (hungry creatures still push through)

### Phase 4: Testing & Validation
**Outcome:** Four test scenarios validating smooth navigation, speed adaptation, and energy modulation

**Key Decisions:**
- Static crowd: Gentle spacing emerges over time
- High-speed approach: Fast creatures react earlier than slow
- Dense navigation: Smooth weaving vs jerky swerves
- Energy variation: Hungry creatures tolerate closer early encounters

---

## Guidance Notes

### Biological Context

**Real animal behavior:** Birds, fish, and pedestrians start gentle trajectory adjustments 3-10 body lengths away, well before close encounters.

**Examples:**
- Birds in murmurations: React at 7-10 body lengths (smooth flocking)
- Fish schools: React at 3-4 body lengths (coordinated swimming)
- Human pedestrians: React 2-3 seconds before collision (~5-10m at walking speed)

**Biological Principle:** "Animals optimize energy by avoiding unnecessary hard maneuvers." Early, gentle corrections cost less energy than late, hard swerves.

### Technical Context

**Three-Zone Force Gradient:**
```
Early Warning (8-9m):   5.25N gentle nudge → smooth trajectory tweak
Personal Space (2.5m):  35N inverse-square → active avoidance
Panic Zone (1.25m):     90N maximum force → emergency swerve
```

**Current Problem:** All avoidance happens at 2.5m → jerky movement, late reactions, unrealistic behavior

**Solution:** Gradual force ramp starting at 8-9m → smooth paths, realistic spacing, energy-efficient navigation

### Performance Impact

**Computational cost:**
- One additional distance check per neighbor
- One `velocity.length()` call per creature per frame
- One `max_speed` calculation (can be cached if needed)

**Expected overhead:** <5% on avoidance system (~0.1ms @ 10K creatures)

**Optimization opportunities:**
- Cache `max_speed` in creature component (calculate once at spawn)
- Cache `speed_ratio` in movement system (updated each tick)
- Early exit if speed near zero (stationary creatures don't need speed extension)

### Integration Notes

**Energy modulation preserved:**
- Early warning zone scales with energy-modulated personal space
- Formula: `effective_early_warning = base_early_warning × (0.4 + 0.6 × energy_fraction)`
- Starving creatures tolerate closer early encounters (matches personal space behavior)

**Force accumulation pattern maintained:**
- Early warning force adds to seek, wander, etc.
- Natural path blending emerges from multiple forces
- No special handling needed

**Panic override still works:**
- Panic threshold unchanged (50% of personal_space = 1.25m)
- Emergency swerves still activate when collision imminent

---

## Success Criteria

- [ ] Constants added to `perception/constants.rs` with tests
- [ ] Speed-adaptive warning distance calculation implemented
- [ ] Early warning force applied correctly (before personal_space check)
- [ ] Static crowd test: Gentle spacing emerges (no jerky movements)
- [ ] High-speed test: Fast creatures react earlier than slow creatures
- [ ] Dense navigation test: Smooth weaving through crowds
- [ ] Energy test: Hungry creatures tolerate closer early proximity
- [ ] All existing avoidance tests still pass (no behavioral regression)
- [ ] Documentation updated in `avoidance-behavior.md` and `perception-system.md`

---

## Implementation Files

**Constants:**
- `apps/simulation/src/simulation/perception/constants.rs`

**Logic:**
- `apps/simulation/src/simulation/creatures/behaviors/avoidance.rs` (or avoidance module)

**Documentation:**
- `docs/biology/done/avoidance-behavior.md` - Add "Early Warning Zone" section
- `docs/biology/done/perception-system.md` - Update constants table
- `docs/biology/done/movement-physics.md` - Add force hierarchy update

**Tests:**
- Add test scenarios to existing avoidance test suite
- Validate three zones operate correctly
- Verify energy modulation applies to early warning

---

## Future Enhancements

### DNA Integration (Post-DNA System)

**Gene: `early_warning_sensitivity` (0.5-2.0, default 1.0)**

**Trade-offs:**
- High sensitivity (2.0): Wide early warning → smooth paths BUT overly cautious, longer travel times
- Low sensitivity (0.5): Narrow early warning → aggressive navigation BUT higher collision risk

**Archetypes:**
- Cautious prey: High sensitivity, wide berth around others
- Aggressive predator: Low sensitivity, charges through crowds
- Efficient forager: Medium sensitivity, balanced

### Trajectory Prediction (Advanced)

**Concept:** Only apply early avoidance if on collision course

**Algorithm:**
1. Calculate time-to-closest-approach (TCA) from velocity vectors
2. If TCA < 2 seconds AND closest_distance < personal_space: apply early force
3. Otherwise: ignore (creature will pass by safely)

**Benefit:** Ignores creatures moving away or passing perpendicularly
**Cost:** Additional vector math per neighbor (~15% more expensive)

---

**See Also:**
- `docs/biology/todo/early-warning-avoidance.md` - Detailed biological rationale and design
- `docs/biology/done/avoidance-behavior.md` - Current avoidance system
- `docs/biology/done/perception-system.md` - Perception constants
