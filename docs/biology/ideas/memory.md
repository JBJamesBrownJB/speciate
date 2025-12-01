# Perception Memory System

**Status:** 💡 Idea (not yet planned)

---

## Core Concept

**Replace "clear and rebuild" perception with "push and displace" - giving creatures emergent memory.**

Current approach: Clear neighbor list each tick, rebuild from scratch.

Proposed approach: Fixed-capacity buffer where new neighbors push out old ones (LIFO). If nothing new enters perception, old data persists.

---

## Why This Matters

### Emergent Memory Duration

Memory duration becomes a function of environment, not a parameter:

| Environment | New Neighbor Rate | Memory Effect |
|-------------|-------------------|---------------|
| Crowded area | High | Constantly refreshed, always current |
| Open terrain | Medium | Moderate persistence |
| Isolated | Low | Long-term recall of past encounters |

No timers. No decay functions. No staleness checks. Just buffer physics.

### Biological Basis

Working memory in animals operates similarly:
- Fixed capacity (Miller's "7 ± 2" in humans)
- New stimuli displace old
- Quiet environments allow rumination on past events
- Busy environments force attention to present

---

## The Ghost Problem

A remembered neighbor might be dead, moved, or irrelevant. Three approaches:

### Option A: Accept Ghosts (Recommended)

Let creatures act on stale information. Benefits:

- **Natural search patterns** - Creature heads to where food WAS, finds nothing, re-scans
- **Realistic mistakes** - "Predator was behind me" → vigilant wrong direction → flanked
- **Emergent uncertainty** - Creates genuine unpredictability in behavior
- **Zero overhead** - No validation checks

Animals constantly act on outdated information. This is a feature, not a bug.

### Option B: Lazy Validation

Check entity existence/proximity when brain accesses memory. More "correct" but:
- Adds per-access overhead
- Removes interesting failure modes
- Over-engineers the simple case

### Option C: Snapshot Position

Store last-known position with each memory entry. Allows "head toward last known location" even if entity despawned. Middle ground - some overhead, preserves ghost behavior for movement.

---

## DNA Traits

### working_memory_capacity: u8

**Range:** 4-32 entries

**Trade-offs:**
- Low capacity = reactive, present-focused, easily surprised, cheap
- High capacity = remembers threats, holds grudges, better planning, expensive

**Biological basis:** Neural tissue for working memory has metabolic cost. Prey animals often have lower capacity but faster refresh (live in the moment). Predators often have higher capacity (track multiple prey, remember territory).

### memory_type_bias (Future)

Weight allocation across memory types. Some creatures prioritize threat memory over food memory. DNA-driven attention allocation.

---

## Type-Separated Buffers

Different neighbor types have independent buffers with separate capacities:

| Buffer | Base Capacity | Push Rate | Rationale |
|--------|---------------|-----------|-----------|
| `threats` | 4-8 | High priority | Survival-critical, always freshest |
| `food` | 8-16 | Medium | Remember resource locations |
| `mates` | 2-4 | Low | Few but significant |
| `conspecifics` | 8-24 | Variable | Flocking, social awareness |
| `obstacles` | 16-32 | Low | Static, cheap to remember many |

### Emergent Attention Displacement

In a threat-rich environment:
- Threats constantly pushing in
- Threat buffer stays current
- Food buffer stagnates (attention elsewhere)
- Creature "forgets" food locations when stressed

This models attention - you can't focus on everything. Danger displaces other concerns.

---

## Expected Emergent Behaviors

### Territorial Memory
Creature in stable territory builds up obstacle/landmark memory. Navigates efficiently. Displaced to new area → disoriented until buffer rebuilds.

### Threat Persistence
Saw predator 30 seconds ago, nothing since. Still remembers. Stays vigilant. Creates realistic wariness after encounters.

### Food Route Learning
Creature that found food remembers location. Returns later. If food respawns in same area, reinforces memory. Emergent foraging routes.

### Social Bonds
Low mate-buffer turnover in stable pairs. High turnover in promiscuous species. Memory duration affects pair bonding without explicit "relationship" code.

### Panic Flooding
Sudden threat surge floods all buffers. Creature temporarily "forgets" everything except danger. Post-danger, must rebuild awareness. Realistic post-flight disorientation.

---

## Integration Points

### With Stochastic Vision (todo/dna-driven-fov.md)

Vision updates are rate-limited by neural_speed. Memory persists between updates. Slow updaters rely more on memory; fast updaters live in present.

### With State Machine

Different states might weight buffer access differently:
- FLEEING: Only reads threat buffer
- SEEKING_FOOD: Prioritizes food buffer
- WANDERING: Reads all buffers for opportunistic detection

### With FOV

Creatures with narrow FOV miss more neighbors per scan. Memory compensates - "I saw something to my left" persists even when looking right.

---

## Open Questions

1. **Should buffer capacity be per-type or global?** Per-type is cleaner but more DNA genes. Global with type priority queue is simpler.

2. **Position snapshots worth it?** Storing last-known (x, y) enables "return to location" behavior but doubles memory footprint.

3. **Should threats always displace other types?** Priority override where threat detection can evict non-threat memories regardless of buffer.

---

## Source

Design discussion, 2025-12-01
