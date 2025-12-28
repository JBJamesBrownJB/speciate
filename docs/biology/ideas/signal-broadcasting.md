# Signal Broadcasting: Reverse Perception Architecture

**Status:** Idea (Foundation for signal types)

**Related:** `seismic-impacts.md`, `mating-calls.md`, `flocking-calls.md`, `chemical-scent.md`, `grapevine.md`, `share-L1-knowledge.md`

## Problem / Opportunity

Currently, creatures only actively scan their environment (pull-based perception). Real animals also emit signals that propagate through the environment, allowing passive detection by others. This creates richer sensing and communication while leveraging work already done during spatial grid operations.

**Opportunity:** Do signal emission during movement phase when spatial work is already complete. Push signals to cells, not directly to creatures. Let reception be a separate sensing pass.

## Proposed Solution

### Dual-Layer Signal Propagation

Instead of creatures directly sensing each other, creatures emit signals to spatial grid cells. Other creatures read signals from cells they occupy.

**L0 Layer (Fine-grained, immediate vicinity):**
- Signals emitted to surrounding 9 L0 cells
- High fidelity: direction vector + strength + signal type
- Cells accumulate signals from all emitters
- Sequential distribution from cells to creature signal buffers
- Use case: "I feel vibrations from the northwest, something BIG"

**L1 Layer (Coarse-grained, broader area):**
- Extend existing BioSignature with ambient signal array
- Lower fidelity: aggregate strength + dominant direction (centroid)
- Signals decay over ticks (persist longer than L0)
- Use case: "The ground here feels disturbed, something passed through"

### Signal Data (No Entity ID)

Each signal carries:
- Direction vector (toward source) - enables creature to orient toward/away
- Strength (magnitude) - enables threat assessment
- Signal type tag (Seismic/Vocal/Chemical) - enables species-specific responses
- Age (ticks since emission) - enables decay

**Critically: NO entity ID.** Creatures sense *presence*, not *identity*. They must use other senses (vision) to identify the source. This is biologically accurate.

### Reception: Separate Sensing Pass

Signal reception is a dedicated phase after perception:
1. Creature reads own signal buffer (L0 direct signals)
2. Creature reads current L1 cell ambient signals
3. Brain processes: direction, strength, type
4. Triggers behavioral response (flee, investigate, ignore)

Reception is DNA-gated: some species can sense seismic (ground-dwelling), others can sense vocal (social species), etc.

### Signal Buffer Per Creature

Fixed 8-signal ring buffer (128 bytes):
- Strength-sorted: loudest signals processed first (attention priority)
- Oldest evicted when full (biological: memory fades)
- Universal component (all creatures have it; deaf ones ignore it)

## Golden Zone Opportunities

| Optimization | Free Biological Behavior |
|--------------|--------------------------|
| Stationary creatures emit no seismic | Ambush predators are seismically invisible |
| Small creatures emit weak signals | Mice are quiet; elephants shake ground |
| Freeze to improve reception | Prey freeze = skip movement processing |
| L1 cell batching | O(cells) not O(creatures squared) |
| Max 8 signals processed | Cognitive attention limit |
| Self-noise penalty (moving creatures sense poorly) | Must stop to listen |

**Key insight:** Freezing to improve reception (reduce self-noise) naturally reduces movement energy cost. The optimization (skip movement processing) IS the behavior (predator/prey freeze).

## Trade-offs

**Memory:** 128 bytes per creature for signal buffer (12.8 MB at 100K creatures) - acceptable for L3 cache

**Two-phase distribution:** Cannot safely write to other creatures' buffers from parallel context. Solution: emit to cells (parallel safe), then distribute to creatures (sequential).

**Reception cost:** While raw signal detection is free (passive transduction), creatures that actively process many signals pay cognitive energy cost.

## Expert Input

### Zoologist (zoologist-tom)

Real-world analogues validate dual-layer model:
- **L0 (fine, direct):** Spider web vibration, catfish electroreception, fish lateral line
- **L1 (coarse, ambient):** Elephants detecting distant herds through ground, salmon detecting home stream chemical signature

Trade-offs for seismic-sensing creatures:
- Ground contact required (airborne/jumping = no signal)
- Self-noise penalty: `reception_threshold += own_velocity * own_size`
- Sensory trade-off: High seismic sensitivity may correlate with reduced visual acuity

### ECS (ecs-emma)

Architecture recommendations:
- **Signal buffer:** Fixed 8-slot ring buffer, strength-sorted insertion, 16 bytes per signal
- **L1 ambient:** Extend BioSignature with 4-slot ambient array, decay factor per tick
- **Parallelization:** Cell-based emission (parallel) followed by sequential distribution to creature buffers
- **Memory layout:** SignalBuffer is cold data; excluded from movement queries for cache efficiency

## Dependencies

- Spatial grid (L0 and L1) must exist - **already implemented**
- BioSignature on L1 cells - **already implemented**
- DNA trait system for gating reception abilities

## Related Ideas

- `seismic-impacts.md` - First implementation (physics-derived, size x velocity)
- `mating-calls.md` - DNA-driven vocal reproduction signaling
- `flocking-calls.md` - Same-species coordination
- `chemical-scent.md` - Pheromone trails, territory marking
- `grapevine.md` - Signal broadcasting is the mechanism for this
- `share-L1-knowledge.md` - Signal broadcasting enables this
- `stigmergy.md` - Complementary (environmental modification vs real-time signals)

## Open Questions

- Should signal type decay rates be DNA-encoded (species variation) or global physics constants?
- How does reception interact with the brain/decision system? New input layer to neural network?
- Should creatures be able to "suppress" their emissions (stealth hunting)?

---
*Captured: 2025-12-28*
*Expert consultations: zoologist-tom, ecs-emma*
