# Biology Consultation Notes

Log of zoologist-tom consultations for biological accuracy in A-Life simulation.

---

## 2025-12-28 | Drive Integration Architecture

**Topic:** Multi-sensory drive integration for macro navigation

**Question:** How do real animals neurologically integrate multi-sensory inputs into movement decisions?

**Findings:**

Real animal brains use **layered integration**, not a single strategy:

### Two-Tier System

**Tier 0: Emergency (Priority Override)**
- Brainstem-driven, reflexive
- Immediate threats bypass higher processing
- No weighted averaging - pure priority override
- Example: Deer seeing wolf doesn't weigh hunger vs fear - fear wins absolutely

**Tier 1: Motivated (Weighted Sum with State Modulation)**
- Hypothalamus modulates drive weights based on internal state
- Hunger increases food-seeking weight 3-5x (ghrelin hormone)
- Cortisol increases vigilance, decreases exploration
- Fatigue increases rest-seeking
- Basal ganglia handles action selection (winner-take-all for incompatible, blending for compatible)

### DNA-Encoded Base Weights

Different species use different integration strategies based on ecological niche:

| Species Type | Strategy | Biological Reason |
|-------------|----------|-------------------|
| Prey animals (deer, rabbits) | Fear-dominant hierarchy | Survival requires instant flight |
| Apex predators (lions, sharks) | Hunger-dominant, low fear weight | Few threats, focus on feeding |
| Social animals (wolves, dolphins) | Strong social drive integration | Cooperation essential |
| Opportunists (crows, rats) | High flexibility, rapid weight switching | Exploit varied environments |

### Golden Zone Discovery: Freeze Response

When flee directions cancel (cornered prey), output is near-zero. This produces **tonic immobility** (freeze response) - biologically accurate AND computationally efficient (no movement to process).

### Emergent Behaviors from Architecture

- **Satiation blindness:** Low hunger → weak hunt drive → predator ignores nearby prey
- **Desperation foraging:** Starving creature has amplified food-seeking, approaches risky areas
- **Personality types:** DNA weight variation creates cautious vs bold individuals
- **Context-dependent flocking:** Danger increases social drive weight (safety in numbers)

### Recommendation Applied

Implemented in the ABC Super Sprint (see `sprint_summaries/abc-super-sprint_summary.md`):
- Two-tier system (Emergency/Motivated) - Phase B implements Motivated only
- Contribution arrays per category (flee/approach/disperse)
- DriveContribution with source, tier, vector, magnitude
- DNA weights (uniform in Phase B, modulated in future)

---
