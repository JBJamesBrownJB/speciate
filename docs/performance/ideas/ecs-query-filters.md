# ECS Query Filters

**Status:** Idea
**Category:** Simulation Optimizations

## Problem

Systems iterate ALL entities every frame, even unchanged ones.

## Solution

Use Bevy `Changed<>` and `With<>` filters to skip static entities.

## Expected Benefit

25-30% throughput improvement.

## Notes

Maintain determinism for replay/save.
