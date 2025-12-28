# Pheromone Mate Attraction Side-Effect

## Problem / Opportunity

Apex pheromone deterrence (see `apex-pheromone-harvesting.md`) creates a strategic problem: pheromones serve dual biological functions (territorial marking AND mate advertisement). Using predator scent for protection should have a realistic and dangerous consequence.

**This is not a separate feature - it is the inevitable biological side-effect** of deploying apex pheromones.

## Proposed Solution

When players apply apex predator pheromones to machines or self, they broadcast **both** territorial warning signals **and** mate attraction signals simultaneously.

### Biological Mechanism

Pheromones encode multiple messages in real species:

| Species | Pheromone Function | Dual Effect |
|---------|-------------------|-------------|
| Tiger | Urine marks territory | Also advertises reproductive status |
| Wolf | Scent marks boundaries | Also attracts potential mates |
| Rhinoceros | Dung piles (middens) | Territory + mating availability |

**Chemical reality:** These signals are not separable. The same volatile compounds that deter rivals also attract mates. When you deploy apex pheromones, you are impersonating an apex predator - both its threat (to prey) and its allure (to conspecifics).

### Gameplay Consequence

**Strong deterrent = Strong attractant**

| Pheromone Potency | Prey Deterrence | Apex Mate Attraction |
|-------------------|-----------------|----------------------|
| Weak (crude extract) | Small radius, short duration | Low risk (faint signal) |
| Medium (refined) | Medium radius, hours duration | Medium risk (detectable by searching mates) |
| Strong (synthetic apex) | Large radius, day+ duration | High risk (acts as mating beacon) |

**Strategic dilemma:**
- Maximum protection requires strong pheromones
- Strong pheromones summon the most dangerous creatures (apex predators in mating state)
- Player must balance safety from weak creatures vs. danger from apex

### Behavioral Responses

When apex predator detects pheromone (especially reproductive-stage individuals):

**Mating-ready apex:**
1. Attracted to pheromone source (long-range detection)
2. Investigates cautiously (is this a real mate or rival?)
3. If "fake" (no actual creature emitting), may mark over it (territorial override)
4. May patrol area (believes rival/mate is nearby)

**Territorial apex:**
1. Detects "rival" pheromone in its territory
2. Approaches aggressively (eliminate competitor)
3. Attacks machines or player as "scent source"
4. May mark over pheromone (pheromone protection broken)

**Result:** The strongest deterrents occasionally summon the most dangerous threats.

## Golden Zone

| Optimization | Free Biological Behavior |
|--------------|--------------------------|
| Reproductive state gating | Only mating-ready apex creatures respond to mate signals (skip attraction for non-reproductive individuals) |
| Territory distance check | Apex predators far from their territory ignore distant mate calls (range-based culling) |
| Confidence threshold | Weak pheromones ignored by apex (skip processing if signal strength < threshold) |

**State-based skipping:**
```
IF apex.reproductive_state != MATING_READY:
    SKIP mate attraction computation
ELSE IF distance_to_home_territory > MAX_MATE_SEARCH_RANGE:
    SKIP (won't travel that far for mate)
ELSE IF pheromone.potency < apex.detection_threshold:
    SKIP (signal too weak to detect)
```

Result: Most apex creatures ignore weak/distant pheromones (reduced computation), but occasionally one responds (creates gameplay tension).

## Trade-offs

**Risk scales with reward:**
- Strongest protection (apex pheromones) attracts strongest threats
- Weak protection (low-tier pheromones) attracts nothing dangerous
- Player must assess: is this area worth the risk?

**Temporal unpredictability:**
- Reproductive seasons/states determine attraction likelihood
- Player cannot predict when apex will respond (emergent from DNA/breeding cycles)
- Successful defense one day ≠ safe tomorrow

**Counterplay options:**
1. Use species-specific pheromones (narrow deterrence, narrow attraction)
2. Apply weak concentrations (moderate protection, low attraction risk)
3. Combine with thumpers (multi-modal defense reduces pheromone potency needed)
4. Monitor apex territories (collect from distant populations less likely to respond)

**Scent masking strategy:**
- If player can harvest pheromones from multiple apex species, layering incompatible scents may "confuse" signals
- Trade-off: reduced deterrence potency (mixed signals weaker) vs. reduced mate attraction (no clear identity)

## Expert Input

### Zoologist (zoologist-tom)

**This is biologically inevitable:**
> "Pheromones are multifunctional by nature. You cannot separate territorial and sexual signals - they use overlapping chemical compounds. Deploying apex pheromones WILL attract mates. This is not a design choice; it's chemistry."

**Recommended DNA traits to gate behavior:**

| Gene | Range | Effect on Mate Attraction |
|------|-------|---------------------------|
| `reproductive_readiness` | 0.0-1.0 | Only high values respond to mate signals |
| `territorial_aggression` | 0.0-1.0 | High values attack "rival" scent sources |
| `mate_search_range` | 50-500m | Distance apex will travel for mate call |

**Emergent seasonal patterns:**
If creatures have breeding seasons (DNA-driven), mate attraction risk will fluctuate naturally:
- Spring: high reproductive readiness → pheromones very dangerous
- Winter: low reproductive readiness → pheromones mostly safe
- Player must learn ecology to predict risk

### Integration with Existing Systems

**Chemical scent system** (`chemical-scent.md`):
- Pheromone signals already support "type" tags (alarm, territorial, mating)
- Apex pheromone deployment emits both `TERRITORIAL` and `MATING` tags simultaneously
- Creatures with receptor traits process both signals based on internal state

**Perception system:**
- Mate-seeking apex uses chemical scent to navigate toward source
- Territorial apex uses scent to locate "rival"
- Both behaviors use existing pathfinding/approach systems

## Dependencies

- Apex pheromone harvesting system (provides deployable pheromones)
- Chemical scent architecture (signal propagation and reception)
- Creature reproductive state (DNA-driven mating readiness)
- Territorial behavior (apex patrol/defend territory)

## Related Ideas

- `apex-pheromone-harvesting.md` - Primary mechanic that triggers this side-effect (required)
- `chemical-scent.md` - Foundation pheromone system (required)
- `mating-calls.md` - Alternative mate attraction signal (vocal vs. chemical)
- `attack.md` - Territorial aggression behavior (how apex responds to "rival")

## Open Questions

- Should pheromone "aging" reduce mate attraction faster than deterrence? (old scent = less attractive but still threatening)
- Can apex creatures "call back" when detecting mate pheromones? (vocal + chemical signals combine)
- Should player receive warning (visual/audio cue) when apex is attracted?
- Do multiple pheromone sources (player + multiple machines) amplify attraction or dilute it?
- Can player research "neutered" pheromones (territorial only, no mating signal) at high tech tier?

---
*Captured: 2025-12-28*
*Note: This is a side-effect of apex pheromone use, not a standalone feature*
