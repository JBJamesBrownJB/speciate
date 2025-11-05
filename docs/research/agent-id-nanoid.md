# Agent ID: Globally Unique Identifiers with NanoId

**Date**: 2025-11-05
**Status**: Research / Future Enhancement
**Priority**: Medium

## Problem Statement

Currently, agent IDs in the simulation use a simple monotonic counter (`u32`). While this provides stable IDs within a single simulation instance, it has limitations:

1. **Not globally unique**: IDs reset to 1 when simulation restarts
2. **Collision risk**: Multiple simulation instances would generate overlapping IDs
3. **No temporal/spatial distribution**: Sequential IDs can create hotspots in distributed systems
4. **Limited scalability**: u32 caps at ~4.3 billion entities

## Current Implementation

```rust
// apps/simulation/src/simulation/systems.rs
let id = self.next_id;  // Simple counter starting at 1
self.next_id += 1;
```

Attached as `AgentId(u32)` component to each entity at spawn time.

## Proposed Solution: NanoId

[NanoId](https://github.com/ai/nanoid) is a tiny, secure, URL-friendly unique ID generator.

### Benefits

- **Globally unique**: Collision probability is negligible (< 1% in ~100 years generating 1000 IDs/hour)
- **Compact**: Default 21 characters, much shorter than UUID (36 chars)
- **URL-safe**: Uses `A-Za-z0-9_-` alphabet
- **Fast**: ~2x faster than UUID
- **Rust support**: [`nanoid`](https://crates.io/crates/nanoid) crate available

### Example

```rust
use nanoid::nanoid;

let id = nanoid!(); // "V1StGXR8_Z5jdHi6B-myT"
```

### Migration Path

1. **Phase 1** (Current Sprint): Keep `u32` for MVP/walking skeleton
2. **Phase 2** (Post-MVP): Migrate to NanoId
   - Change `AgentId(u32)` to `AgentId(String)` or `AgentId([u8; 21])`
   - Update NATS contract to use string IDs
   - Update database schema (if persisting agents)
   - Update frontend to handle string IDs

### Considerations

#### Performance Impact
- **Memory**: `String` (24 bytes) vs `u32` (4 bytes) = 20 bytes overhead per agent
  - With 10,000 agents: ~200 KB additional memory
  - Negligible for modern systems
- **Serialization**: String IDs add ~21 bytes to JSON vs 1-5 bytes for u32
  - With 10,000 agents at 20 Hz: ~4.2 MB/s vs ~200 KB/s
  - May require NATS message compression

#### Database Indexing
- String primary keys are slightly slower to index than integers
- Consider using `CHAR(21)` or `BINARY(16)` for optimal DB performance
- PostgreSQL: `id CHAR(21) PRIMARY KEY`

#### Alternative: Snowflake IDs
If deterministic/sortable IDs are needed:
- Twitter Snowflake: 64-bit IDs with timestamp + worker ID + sequence
- Rust crate: [`snowflake`](https://crates.io/crates/snowflake)
- More complex but provides temporal ordering

## Decision Record

**For Sprint 6 (Walking Skeleton)**: Use simple `u32` counter
- ✅ Simpler to implement
- ✅ Smaller message size
- ✅ Faster serialization
- ⚠️ Single-instance only

**Post-MVP**: Evaluate NanoId vs Snowflake vs UUIDv7
- Depends on multi-instance requirements
- Depends on message size constraints
- Depends on need for temporal ordering

## References

- [NanoId GitHub](https://github.com/ai/nanoid)
- [Rust nanoid crate](https://crates.io/crates/nanoid)
- [NanoId collision calculator](https://zelark.github.io/nano-id-cc/)
- [UUIDv7 (time-ordered)](https://www.ietf.org/archive/id/draft-peabody-dispatch-new-uuid-format-04.html)
