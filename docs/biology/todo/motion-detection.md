# Motion Detection (Golden Zone Opportunity)

## Status: DEFERRED → **recommended next 1M lever (2026-06-21)**

Identified during Phase A planning. High-value optimization that provides free emergent behavior.

**2026-06-21 update:** promoted to the recommended next perception cut on the path to 1M. Rationale (team perf analysis): it cuts the *fattest* tick phase (perception ~14.5 ms at 900K) by a *real* amount — at scale a large fraction of creatures are slow/idle, so skipping them is a big slice, plausibly enough to clear the ~2 ms p99 noise floor where the micro-trims (range-trim, fast_inv_sqrt) couldn't. Needs **no energy/hunger system** (just a velocity check), unlike hunger-gating. Double-value Golden Zone: perf win = "prey freeze = camouflage" gameplay. Validate in the latency lab (multi-seed, p99-aware, `cells_queried` as the causal signal). See `docs/scale/optimization-checklist.md` (T2.3) and `docs/scale/path-to-one-million.md`. First verify the existing L1 size-domination early-exit (`perception/systems.rs`) doesn't already overlap before building.

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
