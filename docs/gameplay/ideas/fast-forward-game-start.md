# Fast-Forward Game Start

## Problem / Opportunity

New game worlds start "cold" - creatures spawn at initial positions without established territories, population dynamics, or interesting interactions. Players see the awkward "setup" phase rather than a living ecosystem.

A 30-60 second visible fast-forward during game start would:
- Let players watch the world "come alive" at high speed
- Ensure populations have spread, established territories, reached equilibrium
- Create immediate engagement (watching is more interesting than loading screens)
- Deliver a mature ecosystem without requiring actual wait time

## Proposed Solution

When starting a new game:

1. **Seed Phase:** Spawn known-viable creature populations appropriate for each biome
2. **Fast-Forward Phase:** Run simulation at accelerated speed (target: 100x+ at start, slowing as population grows)
3. **Visual Display:** Show the simulation running visibly (not a loading screen)
4. **Handoff:** Transition to normal speed when ecosystem reaches maturity metrics

### Implementation Phases

**Phase 1: Headless Burn-In (Immediate)**
Run ~1000 ticks before displaying anything. Player sees established world on first frame. Zero code changes needed - just delay renderer attachment.

**Phase 2: Low-Fidelity Dispersal Mode**
During fast-forward, skip expensive systems that aren't needed for basic spreading:
- Skip perception/steering (creatures spread without collision physics)
- Keep movement/reproduction (populations grow and disperse)
- Keep energy consumption (natural selection still operates)

This is biologically acceptable - dispersal phase doesn't need precision.

**Phase 3: Combined with Stochastic Vision**
Reduce perception updates from 10% to 1% per tick during fast-forward. Combined with Phase 2, achieves ~10x effective speedup.

### Speed Modes: Seamless Transitions

Fast-forward uses the same dual-mode system as Time Bubble:

| Phase | Speed | Mode | Visual |
|-------|-------|------|--------|
| Start | 100x+ | Real Simulation | Creatures spreading fast (visible) |
| Peak | 1000x+ | Population Genetics | Blur, swirling particles, stats |
| Decel | 100x → 1x | Real Simulation | Creatures "crystallize", slow to normal |

**Player Experience:**
1. World seeds with 5K creatures
2. Speed ramps up - watch creatures spread across biomes
3. Past threshold - blur transition, stats overlay (generations ticking)
4. After N generations - speed ramps down
5. Evolved creatures emerge from blur - now at moderate fast-forward
6. Continues slowing until real-time - gameplay begins

The visual of creatures "emerging" from the blur as time slows is the payoff - player sees the evolved population materialize.

See `docs/biology/ideas/population-genetics-algorithm.md` for the statistical model.

### What Players See

During the 30-60 second fast-forward:
- **Early:** Creatures spreading across biomes at visible high speed
- **Peak:** Shimmering blur with stats (population, generations, mean traits)
- **Late:** Creatures crystallizing out, still fast, slowing down
- **End:** Normal speed, mature evolved ecosystem

## Golden Zone

**Low-Fidelity Mode as Dispersal Simulation**
Skipping perception/steering during initialization isn't just an optimization - it models real dispersal behavior where creatures spread out before settling into territories. Young animals dispersing from birth areas don't engage in full territorial behaviors.

**Satiated Skip Acceleration**
During fast-forward, creatures that have eaten recently skip prey detection. Performance win + biologically accurate (full animals rest, don't hunt).

## Trade-offs

| Benefit | Cost |
|---------|------|
| Mature ecosystem from moment one | 30-60 sec before gameplay |
| Engaging visual experience | Must be visually interesting at high speed |
| Biologically plausible (dispersal phase) | Simplified physics may cause minor artifacts |
| Significant engineering value (tests system at scale) | Sprint investment required |

### Technical Constraints

**Key insight:** Fast-forward starts with ~5K creatures, not 360K.

| Population | Est. Tick Time | Max Hz | Speedup vs 34Hz |
|------------|----------------|--------|-----------------|
| 5K (seed) | ~0.4ms | ~2500 Hz | **73x** |
| 50K | ~4ms | ~250 Hz | 7x |
| 200K | ~16ms | ~62 Hz | 2x |
| 360K | ~29ms | ~34 Hz | 1x (baseline) |

**100x speedup IS feasible** at small populations. Speed naturally decreases as creatures multiply - which creates a visually interesting "time slowing down" effect as the world fills up.

Current bottlenecks (at scale):
- Steering: 32% of tick time (can skip in dispersal mode)
- Movement: 21% (must keep - creatures need to move)
- Perception: 20% (can reduce to 1% sampling)
- Spatial Grid: 14% (can rebuild less frequently)

## Expert Input

**Instrumentation Consultation (2025-12-28):**

Original analysis assumed 360K creatures (current scale). Key correction: fast-forward starts with small seed population (~5K), making 100x feasible at start.

Recommended approach hierarchy:
1. **Headless burn-in** (trivial) - just run ticks before attaching renderer
2. **Low-fidelity dispersal** (optional) - skip perception/steering for additional gains
3. **Natural deceleration** - speed drops automatically as population grows

Performance validation required before merge:
- IPC >4.0 (maintain parallelization efficiency)
- Verify tick times scale as expected with population

## Dependencies

- Biome seeding system (spawn appropriate creatures per biome)
- Known-viable creature templates (pre-tested DNA combinations)
- Ecosystem maturity metrics (when to stop fast-forward)
- Optional: Visual time-lapse effects for renderer

## Related Ideas

- `docs/biology/ideas/population-genetics-algorithm.md` - **Core engine for hyper-speed phase**
- `docs/gameplay/ideas/time-bubble.md` - Same dual-mode architecture for localized speedup
- `docs/biology/ideas/game-director.md` - Director manages mature ecosystem after fast-forward
- `docs/gameplay/ideas/game-phases.md` - Fast-forward leads into "Survive" phase
- `docs/performance/ideas/dynamic-system-frequency.md` - Variable system update rates

## Open Questions

- What defines "mature ecosystem"? Population stability? Territorial coverage? Biodiversity threshold?
- Should players be able to skip fast-forward (sacrificing ecosystem quality)?
- Should fast-forward speed be configurable (let player choose how "aged" their world is)?
- How do we handle edge cases where ecosystem fails to stabilize?

---
*Captured: 2025-12-28*
