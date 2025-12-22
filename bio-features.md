# Biological Features Summary

Quick reference for creature behavior features - what's implemented, what's next, what's deferred.

---

## Feature Status

| Feature | Description | Status | Details |
|---------|-------------|--------|---------|
| **Movement Physics** | Kleiber scaling, allometric mass/speed/acceleration | DONE | `docs/biology/done/movement-physics.md` |
| **Bio-Physics** | Mass from body length, energy consumption scaling | DONE | `docs/biology/done/bio-physics.md` |
| **Basic DNA** | Genes for body size, perception range | DONE | `docs/biology/done/basic-dna.md` |
| **Perception System** | Vision cone, FOV, neighbor detection | DONE | `docs/biology/done/perception-system.md` |
| **Wandering Behavior** | Random exploration when no target | DONE | `docs/biology/done/wandering-behavior.md` |
| **Avoidance Behavior** | L0 collision avoidance (lateral steering) | DONE | `docs/biology/done/avoidance-behavior.md` |
| **Target Seeking** | Edge-to-edge seeking with arrival | DONE | `docs/biology/done/target-radius-seeking.md` |
| **Size-Based Turning** | Larger creatures turn slower | DONE | `docs/biology/done/size-based-turning.md` |
| **Brain Timing** | Decision delays, perception intervals | DONE | `docs/biology/done/brain-decision-timing.md` |
| **Perception Slicing** | Time-sliced perception for performance | DONE | `docs/biology/done/perception-time-slicing.md` |
| **Size Domination** | Giants ignore mice (5% mass threshold) | NEXT | `ABC-SUPER_SPRINT/1-dual-grid.md` |
| **L1 Classification** | EMPTY/THREAT/PREY/CROWDED cell tagging | NEXT | `ABC-SUPER_SPRINT/1-dual-grid.md` |
| **Drive Simplex** | Continuous drives replace behavior states | NEXT | `ABC-SUPER_SPRINT/2-simple-drive-simplex.md` |
| **Threat Velocity** | Flee urgency based on predator movement | NEXT | `ABC-SUPER_SPRINT/2-simple-drive-simplex.md` |
| **Frequency Control** | Runtime Hz adjustment for 500K scale | NEXT | `ABC-SUPER_SPRINT/3-frequency-control.md` |
| **Motion Detection** | Skip stationary entities (prey freeze) | LATER | `docs/biology/todo/motion-detection.md` |
| **Hunger Gating** | Satiated predators ignore prey | LATER | `docs/biology/todo/hunger-gating.md` |
| **Crowding Affinity** | DNA gene: solitary (-1) to social (+1) | LATER | `docs/biology/todo/crowding-affinity.md` |
| **Actual Predation** | Catching, eating, energy transfer, death | LATER | Post-ABC |
| **DNA-Driven FOV** | Perception range from genes | LATER | `docs/biology/todo/dna-driven-fov.md` |
| **DNA Speed/Accel** | Movement params from genes | LATER | `docs/biology/todo/speed-accel-dna-based.md` |
| **Influence Maps** | Spatial behavior gradients | LATER | `docs/biology/todo/influence-maps.md` |
