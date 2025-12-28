# Game Director (God System)

## Core Mission

**The Director exists to make sure the player has a good time.**

Everything else flows from this. It's not about simulation accuracy, biological realism, or emergent purity. It's about entertainment.

## Problem / Opportunity

Open-ended ecosystem simulations can fall into states that aren't fun:
- Mass extinctions → empty, boring worlds
- Dead zones → nothing to watch or interact with
- Stagnation → no drama, no interesting events
- Imbalance → frustrating or confusing dynamics

The simulation needs a **god** that can intervene whenever the player's experience is at risk.

## Proposed Solution

A **God System** - an explicit, high-level game director that monitors simulation state and can directly intervene to:

1. **Prevent/recover from extinction events** - Spawn creatures, boost populations, introduce migrants
2. **Create narrative events** - Trigger migrations, conflicts, environmental disasters, dramatic moments
3. **Balance ecosystem health** - Adjust populations, introduce species, cull overpopulation
4. **Eliminate dead zones** - Detect boring regions and inject activity

### This is NOT Natural Simulation

The Director is an **explicit game system**, not a simulation of natural processes. It can:
- Spawn creatures from nothing when needed
- Trigger events that wouldn't emerge naturally
- Override simulation rules for narrative purposes
- Make "unfair" interventions to serve player experience

Natural environmental systems (disease, weather, refugia) are a **separate concept** that could complement but not replace the Director.

### Visibility Spectrum

Some interventions visible, some invisible:

| Intervention Type | Visibility | Example |
|-------------------|------------|---------|
| Dramatic events | Visible | "A migration wave arrives from the north!" |
| Population boosts | Invisible | Quietly spawn creatures in unobserved areas |
| Extinction prevention | Semi-visible | "A rare survivor was spotted..." |
| Dead zone fixes | Invisible | Inject activity where player isn't looking |
| Narrative moments | Visible | Predator-prey showdowns, territorial conflicts |

### Director Triggers

The Director monitors and responds to:

**Population Metrics**
- Species count drops below threshold → spawn/migrate
- Single species dominates → introduce competitor/disease event
- Total creature count too low → boost reproduction rates

**Spatial Analysis**
- Region has no activity for X ticks → inject creatures/resources
- Player hasn't seen interesting event recently → stage something nearby

**Narrative Pacing**
- Too long since dramatic event → trigger conflict/migration/disaster
- Player approaching extinction threshold → create "last stand" moment

## Golden Zone

N/A - This is a **pure gameplay/narrative feature**, not a performance optimization. The Director adds CPU cost (monitoring, event generation) rather than reducing it.

However, the Director could **leverage** Golden Zone optimizations:
- Use activity phase data to time events (attack during prey sleep cycle)
- Use satiation data for narrative (hungry predator encounters)

## Trade-offs

| Benefit | Cost |
|---------|------|
| Guaranteed interesting gameplay | Breaks simulation purity |
| Prevents frustrating dead worlds | Players may detect "artificial" interventions |
| Enables narrative storytelling | Requires extensive tuning |
| Safety net for edge cases | May reduce emergent surprise |

### Simulation Purists vs Entertainment
Some players want pure emergence; others want curated experience. Consider: Director intensity slider (Off / Subtle / Active / Dramatic)?

### Detection Risk
If players notice the Director's hand, immersion breaks. Invisible interventions must be truly invisible - spawn off-screen, in fog of war, during scene transitions.

## Expert Input

**Zoologist Consultation (2025-12-28):**

The zoologist recommended reframing as natural environmental systems (refugia, disease, dispersal). This is a **valid separate idea** for emergent ecosystem stability, but distinct from the Director concept.

The Director is explicitly artificial - a game system that prioritizes player experience over simulation accuracy.

**Note:** Natural systems and Director are complementary:
- Natural systems handle routine ecosystem dynamics
- Director handles edge cases, narrative events, and entertainment guarantees

## Dependencies

- Population monitoring system (track species counts, distribution)
- Spatial activity tracking (detect dead zones)
- Event system (trigger and display narrative moments)
- Fog of war / visibility system (for invisible spawning)

## Related Ideas

- `docs/gameplay/ideas/game-phases.md` - Director behavior may vary by game phase
- `docs/gameplay/ideas/fast-forward-game-start.md` - Director manages post-fast-forward world
- Future: Natural environmental systems (separate idea - disease, weather, refugia)

## Open Questions

- How aggressive should the Director be? (Player preference slider?)
- Should players ever be told the Director intervened?
- How does Director interact with player-caused extinctions? (Respect or reverse?)
- What's the cooldown between dramatic events?
- Should Director have "story arcs" or just reactive interventions?

---
*Captured: 2025-12-28*
