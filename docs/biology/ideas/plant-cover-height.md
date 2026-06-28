# Plant Height — DNA-Gated, Growth-Dependent Cover

## Problem / Opportunity

Plants today are essentially flat resource points — biomass to be eaten. They have no vertical dimension, so they can't function as **cover**. Giving plants a **height** that emerges from their own DNA and their growth stage turns flora from passive food into active terrain: tall, mature, dense stands become hiding spots and ambush lanes; seedlings and grazed-down patches offer little. Vegetation becomes a living, changing concealment map rather than scenery.

## Proposed Solution

Add a **height** property to plants, driven by two factors:

1. **DNA-gated potential** — a plant's genetics set its *maximum* height / cover potential (a grass-type tops out low; a tree-type tops out tall). This is the plant analogue of the creature `size_gene` — a heritable trait, not a hardcoded constant per species.
2. **Growth-dependent realization** — a plant only *reaches* its potential as it grows/matures. A young or recently-grazed plant is short and offers little cover; left to mature it rises toward its genetic ceiling. Height tracks the plant's growth/biomass state over time.

Effective cover at a location is then a function of the plants there: roughly `cover = f(plant_height, plant_density)` — tall *and* dense gives the most concealment. That per-location cover value is what the **cover/concealment** idea consumes to reduce creature conspicuousness.

This keeps the project's DNA-driven thesis consistent on the flora side: plant form **emerges** from genes + environment, and that form has real downstream consequences (cover) rather than being decorative.

## Golden Zone

Indirect — the optimization payoff lives in the linked concealment system (cheap "hidden in cover" perception skips). On the plant side the opportunity is **emergent ecology for free**:

- **Grazing pressure shapes the cover map:** herbivores eating plants down lowers local height → reduces cover → exposes whoever was hiding there. Cover availability becomes a dynamic consequence of the herbivore population, not a static layer. Over-grazed regions become exposed killing grounds; ungrazed thickets become refuges — emergent, unscripted.
- **Selection loop:** if tall plants give cover that helps creatures survive nearby, and grazers preferentially crop short/accessible plants, there's a plausible co-evolutionary pressure on plant height worth exploring later.

## Trade-offs

- **Plant data growth:** every plant gains height/growth-stage state and (if DNA-gated) a genetic max-height field — more per-plant memory and update cost at the scales this engine targets (plants can be numerous). Must stay SoA-friendly and cheap to update.
- **Cover aggregation cost:** turning per-plant height into a per-location cover value needs a spatial roll-up (e.g. per-grid-cell cover scalar) refreshed as plants grow/are eaten — another structure to maintain. Worth it only if reads are O(1) for the concealment consumer.
- **Growth-rate tuning:** how fast plants recover height after grazing controls how quickly refuges regenerate — a sensitive ecological dial.
- **Rendering:** height likely wants a visual cue (taller/denser sprites, or the depth/altitude encoding already sketched) so players can read where cover is — otherwise concealment feels arbitrary. Coordinate with the visuals depth/altitude idea.

## Expert Input

Not yet consulted — logged on request as a raw idea. Natural future consults: **botanist-betsy** for realistic height/growth/biomass relationships and DNA-gated max-height ranges per plant archetype; **zoologist-tom** for the grazing↔cover↔predation loop; **environment-eddy** for how cover folds into procedural terrain/biome generation.

## Dependencies

- A plant growth / biomass model that can carry a height/maturity state over time (some plant lifecycle already exists; this extends it with a vertical axis).
- A plant DNA / gene mechanism (parallels creature DNA) to gate maximum height heritably — may need new plant-side genetics if none exists yet.
- A spatial aggregation so creature-side systems can read local cover cheaply.

## Related Ideas

- `docs/biology/ideas/cover-concealment.md` — the consumer: plant height/density feeds the cover value that reduces creature conspicuousness. **These two are a pair.**
- `docs/biology/done/conspicuousness-visibility.md`, `docs/biology/todo/motion-gated-conspicuousness.md` — the conspicuousness chain that concealment (and thus this) plugs into.
- `docs/visuals/ideas/depth-altitude-sprite-encoding.md` — a rendering approach that could be extended to show plant height / cover density.
- `docs/biology/ideas/herbivore-competition.md`, `docs/biology/ideas/feeding-vigilance.md` — grazing-pressure systems that would dynamically reshape the cover map.

## Open Questions

- Is plant height a brand-new plant gene, or derived from an existing plant-size/biomass trait?
- Continuous height, or a few discrete cover tiers (seedling / mature / overgrown)?
- How does height relate to biomass — strictly coupled (taller = more food) or decoupled (a tall thin reed vs a low dense bush)?
- Does grazing reduce height immediately, or only via biomass depletion over time?
- How does per-plant height aggregate into per-location cover — sum, max, density-weighted?
- Should cover effectiveness be relative to the hiding creature's size (1 m grass hides a mouse, not a giant)? (Mirror open question in the concealment idea.)

---
*Captured: 2026-06-28*
