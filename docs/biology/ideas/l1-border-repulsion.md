# L1 Cell Border Repulsion Zone

## Problem / Opportunity

Hard boundary clamping creates unnatural "wall bouncing" behavior at map edges. Creatures should naturally avoid edges before hitting them, creating organic population distribution away from boundaries.

## Proposed Solution

Mark L1 cells at map borders as "repulsion zones" that apply a constant outward force to creatures within them. This creates a soft boundary layer where creatures are gently pushed away from edges before reaching the hard physics clamp.

**Mechanism:**
- Border L1 cells flagged at initialization (static, never recomputed)
- Creatures in border cells receive repulsion force toward map center
- Force is constant within border cells (not gradient-based)
- Hard boundary remains as failsafe for fast-moving creatures

**Creature perception integration:**
- When creatures scan L1 cells, border cells are categorized as "avoid" zones
- This feeds into drive simplex decision-making for wander target selection

## Golden Zone

**Edge Hunting Emergent Behavior:**

If predators have weaker border repulsion than prey (scaled by predator_score), predators can push prey toward edges where prey movement is constrained. This mirrors real wolf, lion, and orca hunting tactics - herding prey toward terrain traps.

| Optimization | Biological Behavior |
|--------------|---------------------|
| Border cells have zero resources | Animals avoid unproductive edges - skip resource queries in border cells |
| Border repulsion scales with creature size | Larger animals need more space, naturally avoid edges more |
| Skip perception checks for border cells after first encounter | Learned territory awareness |

## Trade-offs

- **Corner oscillation risk:** Creatures pushed into corners experience multi-directional repulsion - mitigated by constant (not additive) force model
- **Population centering:** Creates higher density in map center - may need edge resources to counter
- **Fast creature penetration:** Very fast creatures may overshoot into border before force applies - hard boundary failsafe handles this

## Expert Input

**Zoologist-tom consultation (2025-12-29):**

- Biologically plausible: "Edge effect" and ecotone wariness well-documented across species
- Animals have innate anxiety responses to unfamiliar/exposed terrain (amygdala-driven)
- Soft repulsion more realistic than hard walls - boundaries in nature are gradient zones
- Recommended Golden Zone: Predator/prey force differential creates edge hunting dynamics for free
- Suggested future enhancement: DNA-driven `edge_aversion` trait for niche differentiation (edge-tolerant species could specialize in shoreline/cliff resources)

## Dependencies

- L1 spatial grid must be functional with stable coordinates
- Creature L1 cell perception (planned in drive-simplex-plan-v2)
- Drive simplex system for wander target selection

## Related Ideas

- `docs/biology/ideas/crowd-tolerance-dna.md` - Related spatial preference system
- `docs/gameplay/ideas/repulsion-field.md` - Player equipment (different concept)

## Open Questions

- Should corner cells have special handling (repulsion from two directions)?
- What is the optimal border zone width (1 or 2 L1 cells)?
- Should `edge_aversion` DNA trait be added now or deferred?

---
*Captured: 2025-12-29*
