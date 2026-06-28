# Flight Initiation Distance (FID)

**Status:** 💡 Idea — reference logged, not yet designed
**Source:** YouTube — https://www.youtube.com/watch?v=d-rVcyi4kqs
**Closely related:** `threat-assesment.md` (the WHEN-to-flee trigger this would feed),
`feeding-vigilance.md`, `energy-vigilance.md`, `flee-noise.md`,
`done/conspicuousness-visibility.md` (the SEEING side — detection range vs flee range)

---

## The gold (why it's logged)

**Flight initiation distance** = the distance to an approaching threat at which an
animal *starts* to flee. It is not a reflex — it's an **economic optimum**, a cost/benefit
decision the animal is constantly solving:

- **Flee too LATE** → caught and eaten. Catastrophic, terminal cost.
- **Flee too EARLY** → you abandon whatever you were doing (eating, mating, resting),
  burn energy on an unnecessary sprint, and lose foraging/feeding time. Many small wasted
  early-flights add up to real fitness loss.

So there's a **margin of safety** the animal tunes: the optimum FID balances predation
risk against the opportunity cost of fleeing. This is the *decision rule* missing from our
current model — threat assessment scores "how threatened," but FID is the principled answer
to "at what point is fleeing worth it."

---

## Why it's relevant to Speciate

This is the **cost side** of the threat response, and it ties our existing pieces together:

- **Pairs with conspicuousness** (`done/conspicuousness-visibility.md`): conspicuousness sets
  how far away a creature is *detected*; FID sets how far away it *starts running*. A giant
  is now seen earlier — FID is the natural counterpart governing whether/when prey react.
- **Feeds threat-assessment** (`threat-assesment.md`): FID is essentially the threshold on
  `threat_weight`/tau at which the simplex tips from Approach/Rest into Flight. The video's
  economics give a principled basis for that threshold instead of a magic number.
- **DNA-tunable & emergent:** the optimum shifts with state and genes — exactly our model:
  - hungrier / higher-value food → *shorter* FID (hold longer, risk more — ghrelin vs
    cortisol tug-of-war, already noted in `feeding-vigilance.md`)
  - bolder genotype → shorter FID; timid → longer (the Boldness gene in `threat-assesment.md`)
  - already-fed / leptin-high → *longer* FID (flees readily; nothing to lose)
  - bigger / faster approaching threat (higher closing velocity, lower tau) → longer FID
- **Golden Zone potential:** a longer FID means fleeing from farther = less compute spent in
  close-range avoidance, and fewer actual chases — the optimal-economics behaviour is also
  the cheaper one.

---

## Captured terms to chase later
Economic/optimality model of escape; margin of safety; "assessment of approaching predator";
opportunity cost of fleeing; alert distance vs flight initiation distance (detection ≠
reaction); starting distance dependence.

---

## Next step (when picked up)
Watch the video in full and run a `dna-consult` (zoologist-tom) to turn FID into a concrete
flee-threshold function over (tau, size ratio, energy_fraction, boldness), wiring it as the
trigger between `threat-assesment.md` and the flee response. **Not now — logged only.**
