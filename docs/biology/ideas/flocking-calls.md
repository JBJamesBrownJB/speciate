# Flocking Calls: Same-Species Coordination Signals

**Status:** Idea (DNA-driven signal type)

**Depends on:** `signal-broadcasting.md` (foundation architecture)

## Problem / Opportunity

Flocking/schooling/herding creatures need to maintain group cohesion. Visual perception alone requires constant scanning for flock-mates. Flocking calls allow group members to stay connected even when visual contact is temporarily lost.

## Proposed Solution

### Emission: DNA-Driven Social Trait

Flocking calls require social DNA traits:

**Flock Caller Gene:**
- Enables emission of coordination signals
- Species-specific signature (prevents cross-species flocking)
- Low energy cost (frequent, low-intensity calls)

**Emission Pattern:**
- Continuous low-level emission while in flock state
- Intensity increases when separated from group
- "Lost" creatures call louder to rejoin

### Reception: Species-Matching

Creatures respond primarily to calls from same species:
- Species signature comparison (DNA-derived)
- Familiarity bonus: creatures may prefer calls from known flock-mates (memory integration)
- Non-flock species can detect calls but don't respond with cohesion behavior

### Integration with Existing Flocking

Current flocking uses visual perception of neighbors. Flocking calls supplement this:

| Situation | Visual | Calls | Behavior |
|-----------|--------|-------|----------|
| Flock together, visible | Used for alignment | Background comfort | Normal flocking |
| Flock together, occluded | Lost contact | Still connected | Maintain cohesion |
| Separated from flock | Can't see them | Hear distant calls | Navigate toward group |
| Alone, no calls heard | No visual | Silent | Wander (or distress call) |

### Alarm Calls (Extension)

A natural extension of flocking calls:
- When threatened, emit high-intensity alarm call
- Propagates through flock rapidly
- Triggers coordinated flight response
- Enables grapevine behavior from `grapevine.md`

## Golden Zone Opportunities

| Optimization | Free Biological Behavior |
|--------------|--------------------------|
| Only process calls from same species | Species isolation |
| Low-intensity continuous emission | Background flock awareness |
| Intensity scales with separation | Lost creatures call louder |
| Alarm calls trigger group response | Coordinated predator evasion |

**Alarm Cascade Golden Zone:**
- One creature spots predator, emits alarm
- Nearby creatures receive alarm, emit their own
- Cascade spreads faster than visual perception
- Creates realistic "wave" of fleeing behavior
- Performance: each creature only processes immediate signals, cascade emerges

## Trade-offs

**Predator eavesdropping:** Flocking calls reveal group location. Predators can home in on noisy flocks.

**Energy cost:** Continuous calling has ongoing energy cost. Solitary creatures save energy but lose group benefits.

**Noise interference:** Large flocks create call congestion. Direction information degrades in dense groups.

## Expert Input

### Zoologist (zoologist-tom)

Real-world flocking communication:
- **Geese:** Contact calls maintain formation during migration
- **Fish schools:** Lateral line + visual, no vocalizations (seismic equivalent?)
- **Prairie dogs:** Alarm calls include predator type information
- **Starlings:** Murmurations use visual cues primarily, calls for long-range

**Key insight:** Alarm calls often include information about threat type. A "hawk alarm" triggers different response than "snake alarm." Future extension: signal strength encodes urgency.

## Dependencies

- `signal-broadcasting.md` architecture (required)
- DNA system for social/flocking traits
- Existing flocking behavior system
- Species identification in DNA

## Related Ideas

- `signal-broadcasting.md` - Foundation architecture
- `mating-calls.md` - Similar vocal mechanism, different trigger
- `grapevine.md` - Flocking calls enable information sharing
- `share-L1-knowledge.md` - Flock members could share biosignature info

## Open Questions

- Should alarm calls encode threat type (predator vs environmental danger)?
- How does call frequency scale with flock size? (diminishing returns?)
- Can creatures learn to recognize individual flock-mate calls? (memory integration)
- Should there be "leader" calls that influence flock direction?

---
*Captured: 2025-12-28*
