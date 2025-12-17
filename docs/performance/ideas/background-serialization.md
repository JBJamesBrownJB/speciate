# Background Serialization

**Status:** Idea
**Category:** Persistence Optimizations

## Problem

Save operation blocks main thread, freezes game.

## Solution

Run save in separate thread (async write).

## Notes

Non-blocking saves. Electron main process handles async I/O.
