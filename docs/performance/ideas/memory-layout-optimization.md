# Memory Layout Optimization

**Status:** Idea
**Category:** Simulation Optimizations

## Problem

Cache misses from poorly aligned component data.

## Solution

Add `#[repr(C, align(16))]` for SIMD-friendly cache locality.

## Notes

Low-level optimization, measure before implementing.
