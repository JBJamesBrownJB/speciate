# Fused Steering System

**Status:** Done
**Location:** `apps/simulation/src/simulation/creatures/steering/`

## What It Does

Four steering systems (wander, seek, avoidance, flee) fused into one unified system with a single query and single iteration.

**Performance gain from:**
- Single query setup instead of 4
- Single Vec::collect() for Rayon instead of 4
- Single Rayon sync barrier instead of 4
- Better cache utilization (each entity's components loaded once)

## Before/After

```
BEFORE (4 queries, 4 iterations):
┌──────────┐   ┌──────────┐   ┌──────────┐   ┌──────────┐
│  Wander  │ → │   Seek   │ → │  Avoid   │ → │   Flee   │
└──────────┘   └──────────┘   └──────────┘   └──────────┘
   Query 1       Query 2        Query 3        Query 4

AFTER (1 query, 1 iteration):
┌─────────────────────────────────────────────────────────┐
│                   update_steering                        │
│  wander() + seek() + avoid() + flee() → Acceleration    │
└─────────────────────────────────────────────────────────┘
                        Query 1
```

## Folder Structure

```
creatures/steering/
├── mod.rs          # exports update_steering_system
├── system.rs       # THE fused system (one query, calls pure fns)
├── wander.rs       # calculate_wander_force() + tests
├── seek.rs         # calculate_seek_force() + tests
├── avoidance.rs    # calculate_avoidance_force() + tests
└── flee.rs         # calculate_flee_force() + tests
```

## Pure Function Pattern

Each behavior file exports a pure function that the fused system calls:
- `calculate_wander()` - Territory wandering behavior
- `calculate_arrival()` - Seek with arrival (slowing near target)
- `calculate_avoidance_force()` - Neighbor avoidance
- `calculate_flee_force()` - Flee from threats (stub)

## References

- Main system: `creatures/steering/system.rs`
- System registration: `simulation/core/simulation.rs`
