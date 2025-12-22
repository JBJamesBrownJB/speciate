# Hunger Gating (Golden Zone Opportunity)

## Status: DEFERRED

Identified during Phase A planning. Performance optimization with rich emergent behavior.

---

## Concept

Well-fed creatures skip PREY classification entirely, reducing perception workload while creating biologically accurate satiation behavior.

```rust
if my_hunger < SATIATION_THRESHOLD {
    skip_prey_detection  // Performance win
    // Biology: satiated predators genuinely ignore prey
}
```

## Performance Win

- Skip prey classification for well-fed creatures
- Reduces L1 cell processing when predators are resting
- Post-meal predators become "simple" entities (only avoid threats, don't hunt)

## Free Biological Behavior

**Satiated predators ignore prey.** This is universal:
- Lions rest 20 hours/day post-meal
- Crocodiles can go months without eating after large meal
- Snakes become completely dormant during digestion

**Result:** Prey can safely graze near sleeping/digesting predators.

## Starvation Reduces Fussiness

**Key insight:** The inverse is also true and biologically important.

```rust
// Normal: giant ignores mouse (size domination)
let effective_threshold = if my_hunger > DESPERATE_THRESHOLD {
    my_mass * 0.01  // Starving: lower threshold, target smaller prey
} else {
    my_mass * 0.05  // Normal: standard size threshold
};
```

**When starving:**
- Giants start targeting mice (any calories matter)
- Predators become less selective
- Creates desperation behavior

**Real examples:**
- Starving wolves hunt rodents (normally too small)
- Famished bears eat berries instead of salmon
- Desperate lions scavenge instead of hunting

## Emergent Behaviors

1. **Temporal rhythms:** Hunting hours vs rest hours
2. **Safe zones:** Prey grazes near satiated predators
3. **Desperation hunts:** Starving predators chase anything
4. **Scavenging:** Very hungry predators approach CROWDED cells (carrion)
5. **Risk assessment:** Prey learns which predators are dangerous (hungry vs full)

## Entertainment Value: High

Players observe:
- Predators lounging after successful hunt
- Prey boldly grazing near "sleeping" predators
- Starving predators frantically chasing tiny prey
- Ecosystem rhythm of hunt → rest → hunt

## Implementation Notes

### Hunger Levels

```rust
const SATIATION_THRESHOLD: f32 = 0.3;   // Below = skip prey detection
const DESPERATE_THRESHOLD: f32 = 0.8;   // Above = reduced fussiness
```

### DNA Variation

- `metabolism_rate`: Affects how quickly hunger changes
- `desperation_threshold`: Some species more/less desperate

### Interaction with Size Domination

Starvation modifies the size domination threshold:
- Normal giant: threshold = mass * 0.05 = ignores mice
- Starving giant: threshold = mass * 0.01 = notices mice

## Dependencies

- Hunger/energy system (CreatureState.energy)
- Basic perception system (Phase A)

## Related

- See `docs/biology/todo/motion-detection.md` for another golden zone opportunity
- See `docs/biology/todo/crowding-affinity.md` for social behavior
