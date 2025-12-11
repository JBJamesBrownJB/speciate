# Mass & Force Calibration

**Status:** 📋 Planned for Sprint 16

## Problem

Force constants are arbitrary values that don't scale with creature size. A 2m creature uses the same force limits as a 0.5m creature, which is physically incorrect.

## Goal

Unify all force limits to derive from creature mass. Every behavior gets a fraction (0-1) of the creature's physical maximum force capability.

## Physics Constraint

**max_force is the PHYSICAL LIMIT - nothing can exceed it.**

- Mass scales with volume (length³)
- Force = mass × acceleration
- All behaviors use multipliers (0.0 - 1.0) of this limit

## Force Budget

Higher priority behaviors get higher fractions of max_force:

| Priority | Behavior | Fraction |
|----------|----------|----------|
| Emergency | Panic, Brake | 100% |
| High | Seek | 70% |
| Medium | Avoidance | 50% |
| Low | Homeward | 40% |
| Idle | Wander | 20% |

## Outcome

- Larger creatures exert proportionally more force
- Smaller creatures are relatively more agile (future: acceleration scaling)
- All behaviors respect the same physical limits
- Remove hardcoded force constants scattered across behaviors
