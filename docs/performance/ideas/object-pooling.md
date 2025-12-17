# Object Pooling ("Ghost" Pool)

**Status:** Idea
**Category:** Memory Optimizations

## Problem

Dying and spawning creatures cause memory allocator churn.

## Solution

Recycle dead entities instead of spawn/despawn to prevent memory allocator churn.

## Notes

Leak Prevention: Automatic cleanup of interpolation history buffers (PreviousPositions) upon entity death.
