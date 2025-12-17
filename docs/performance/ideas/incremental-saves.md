# Incremental Saves

**Status:** Idea
**Category:** Persistence Optimizations

## Problem

Full world serialization on every save (5-10s at scale).

## Solution

Only serialize changed entities since last save.

## Expected Benefit

Save time: 5-10s -> 500ms.

## Notes

Requires dirty entity tracking.
