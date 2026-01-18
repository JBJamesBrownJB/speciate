# Zero-Copy Serialization (FlatBuffers/Cap'n Proto)

**Status:** Superseded - Zero-copy achieved via NAPI Float32Array (Sprint 13)

**Note:** This FlatBuffers approach was not implemented. Instead, NAPI-RS with direct Float32Array buffer sharing achieved zero-copy IPC. See `docs/performance/done/buffer-transfer-baseline.md` and `docs/architecture/electron-architecture.md`.
**Category:** IPC Optimizations

## Problem

MessagePack requires Node.js to allocate memory for buffer read, then allocate objects to decode (copy-and-parse).

## Solution

Migrate to FlatBuffers or Cap'n Proto for zero-copy reads where binary payload IS the in-memory object.

## Benefits

Access `creatures[0].x` without decoding entire frame. Reduces Electron-side deserialization overhead.

## Trade-offs

- Requires schema definition file (`.fbs`), code generation step in build
- Violates Phase 1 "schema-free" simplicity

## Timeline

Later towards release day on Steam (when schema stabilizes). MessagePack serialization (3ms) is NOT current bottleneck—IO blocking is.

## Consultant Recommendation

Stick with MessagePack for Phase 1. Optimize right bottleneck first (background writer thread).
