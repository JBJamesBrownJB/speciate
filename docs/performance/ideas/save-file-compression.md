# Save File Compression

**Status:** Idea
**Category:** Persistence Optimizations

## Problem

Large save files (10MB+ at scale).

## Solution

Use gzip/zstd compression on save files.

## Expected Benefit

50-70% size reduction.

## Notes

bincode already compact for Rust-only saves.
