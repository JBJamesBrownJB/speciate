# Motion Detection (Golden Zone Opportunity)

## Status: DEFERRED

Identified during Phase A planning. High-value optimization that provides free emergent behavior.

---

## Concept

Skip perception of stationary entities as a performance optimization, which also creates biologically accurate "prey freeze" camouflage behavior.

```rust
if target.velocity.length() < MOTION_THRESHOLD {
    skip_perception  // Performance win
    // Biology: prey freezing actually works as camouflage
}
```

## Performance Win

- Skip stationary entities during perception scan
- Reduces candidates for distance/FOV checks
- Particularly valuable in dense populations where many crits are resting

## Free Biological Behavior

**Predators key on movement.** This is the "motion detection" circuit present in nearly all animals:
- Frog vision literally cannot see stationary objects
- T-Rex "can't see you if you don't move" (Jurassic Park, loosely based on real predator vision)
- Cats stalk prey that moves, ignore stationary objects

**Prey freeze when threatened.** This is a universal survival mechanism:
- Deer freeze when sensing danger
- Rabbits hold perfectly still
- Many insects play dead

## Emergent Behaviors

With motion detection implemented:
1. Prey learns that freezing = survival
2. Predators evolve to wait for movement (patience)
3. Creates stalking behavior
4. "Horror movie" tension where predator approaches unseen prey
5. Explosive action when something finally moves

## Entertainment Value: Very High

Players observe:
- Tension building as predator circles frozen prey
- Dramatic moments when prey breaks and runs
- Predators developing patience/stalking strategies
- Prey that panics too early gets caught

## Implementation Notes

### Threshold Tuning

```rust
const MOTION_THRESHOLD: f32 = 0.1;  // m/s - nearly stationary
```

May need DNA-driven variation:
- Predators with lower threshold = sharper motion detection
- Prey with higher threshold = more easily spooked (flee earlier)

### Interaction with Size Domination

Motion detection stacks with size domination:
- Giant ignores tiny stationary mouse (two reasons to skip)
- Giant might notice tiny MOVING mouse (motion catches attention despite size)

This could create interesting edge case: desperate prey running triggers predator attention even if normally below size threshold.

## Dependencies

- Basic perception system working (Phase A)
- Velocity available during perception scan

## Related

- See `docs/biology/todo/hunger-gating.md` for another golden zone opportunity
