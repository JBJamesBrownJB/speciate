# Biology Consultation Log

This document logs consultations with the zoologist-tom agent to ensure biological realism in creature behavior and trait design.

---

## 2025-11-29 - Personal Space Energy Modulation

**Question:** How do real animals balance personal space with goal-seeking behavior? For example, when multiple animals compete for the same food source or resource, how do they manage to actually reach it despite territorial/personal space instincts? What biological mechanisms allow "tolerance reduction" when highly motivated (hungry, seeking mate, etc.)? Are there examples of social animals that reduce personal space requirements when focused on a goal?

**Zoologist Response:**

### Real-World Examples

**Vultures at Carrion**
- Flight spacing: 50-100 meters when soaring
- Feeding density: Body-contact at carcasses (200+ vultures in 20m²)
- Transformation: Minutes from territorial to shoulder-to-shoulder

**Wolves at Kills**
- Travel spacing: 2-5 meters (avoid accidental bites)
- Feeding: Shoulder-to-shoulder, zero spacing for dominants
- Subordinates wait at periphery (social hierarchy)

**Wildebeest at Water**
- Grazing: 5-10 meter spacing
- At water sources: Near-trampling density
- Drive to drink overrides anti-predator spacing

### Biological Mechanisms

**Hormonal Modulation:**
- Ghrelin (hunger hormone): Reduces territorial aggression by 40-60%
- Cortisol (stress): Dampens amygdala threat response
- Counter-intuitive: Moderate stress REDUCES personal space sensitivity

**Neurological:**
- Prefrontal cortex suppresses avoidance when goal-directed
- Dopamine reward prediction overrides avoidance pathway
- Metabolically expensive - requires high motivation

### Energy State Effects

| Satiation Level | Personal Space Tolerance | Behavior |
|-----------------|-------------------------|----------|
| >80% energy | Full space enforced | Territorial defense |
| 50-80% | Moderate tolerance | Brief proximity for high-value food |
| 20-50% | Significantly reduced | Crowding at food sources |
| <20% | Near-zero space | Accepts body contact, ignores threats |

### Implementation Decision

**Formula:** `effective_personal_space = base × (0.4 + 0.6 × energy_fraction)`

- At 100% energy: 1.0× modifier (full personal space)
- At 50% energy: 0.7× modifier (30% reduction)
- At 0% energy: 0.4× modifier (60% reduction)

**Rationale:**
- 60% reduction matches wolf feeding behavior
- 40% minimum prevents complete collapse
- Linear interpolation provides emergent complexity

**Files Modified:**
- `apps/simulation/src/simulation/perception/components.rs` - Added `effective_personal_space()` method
- `apps/simulation/src/simulation/creatures/behaviors/avoidance.rs` - Uses energy-driven space
- `apps/simulation/src/simulation/queries.rs` - Added CreatureState to AvoidanceQuery

**Implementation Status:** ✅ Complete

**Trade-offs:**
- Cost: Crowding increases disease transmission, injury risk
- Benefit: Access to contested high-value resources
- Niche: Creates "cautious" vs "bold" archetypes

**Future Work:**
- Phase 2A: Add `energy_sensitivity` gene (0.2-1.0 range)
- Creates individual variation: cautious creatures maintain boundaries when hungry, bold creatures collapse space when starving
- Enables evolutionary adaptation to resource scarcity

---

## Template for Future Consultations

```markdown
## [Date] - [Trait/System Name]

**Question:** [What you asked the zoologist]

**Zoologist Response:** [Summary of biological guidance]

**Implementation:** [What was built and where]

**Status:** Pending / In Progress / Complete

---
```
