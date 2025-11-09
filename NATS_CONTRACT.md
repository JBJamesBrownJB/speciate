# NATS Message Contract Specification

## Version: 1.0.0

**Status:** Active
**Last Updated:** 2025-11-05
**Effective Date:** Sprint 6

---

## Overview

This document defines the contract for simulation data streaming between the **Simulation Service** (Rust) and **Broadcaster Service** (TypeScript) via NATS messaging.

### Contract Purpose

- Ensure type-safe, reliable data exchange between microservices
- Document schema versioning and evolution strategy
- Define breaking change policies
- Provide single source of truth for message structure

---

## Message Transport

### Protocol

- **Format:** MessagePack (binary serialization)
- **Transport:** NATS pub/sub
- **Subject:** `speciate.crits.transform`
- **Frequency:** ~20 Hz (20 messages per second)
- **Quality of Service:** At-most-once delivery

### Libraries

| Service | Language | Library | Version |
|---------|----------|---------|---------|
| Simulation | Rust | `rmp-serde` | 1.3.0 |
| Broadcaster | TypeScript | `@msgpack/msgpack` | ^3.0.0 |

---

## Schema Definition

### SimulationFrame

The top-level message structure representing a single simulation tick.

```typescript
interface SimulationFrame {
  tick: number;              // Simulation tick counter (uint64)
  timestamp: string;         // ISO 8601 timestamp (UTC)
  crits: CritTransform[];  // Array of crit states
}
```

**Rust Definition:**
```rust
#[derive(Debug, Serialize)]
struct SimulationFrame {
    tick: u64,
    timestamp: String,
    crits: Vec<CritTransform>,
}
```

#### Field Specifications

| Field | Type | Range | Required | Description |
|-------|------|-------|----------|-------------|
| `tick` | uint64 | 0 to 2^53-1 (safe) | Yes | Monotonically increasing tick counter |
| `timestamp` | string | ISO 8601 | Yes | Server timestamp when frame was created |
| `crits` | array | 0 to 100,000 | Yes | Array of crit transforms (can be empty) |

**Safe Integer Range:**
JavaScript can safely represent integers up to 2^53-1 (9,007,199,254,740,991). Tick values exceeding this will lose precision.

---

### CritTransform

Individual crit state data for rendering and interpolation.

```typescript
interface CritTransform {
  id: number;        // Crit unique identifier (uint64)
  x: number;         // Position X in world coordinates (f32)
  y: number;         // Position Y in world coordinates (f32)
  vx: number;        // Velocity X (meters/second) (f32)
  vy: number;        // Velocity Y (meters/second) (f32)
  rotation: number;  // Rotation in radians 0 to 2π (f32)
}
```

**Rust Definition:**
```rust
#[derive(Debug, Serialize)]
struct CritTransform {
    id: u64,
    x: f32,
    y: f32,
    vx: f32,
    vy: f32,
    rotation: f32,
}
```

#### Field Specifications

| Field | Type | Range | Required | Description |
|-------|------|-------|----------|-------------|
| `id` | uint64 | 0 to 2^53-1 (safe) | Yes | Stable agent identifier |
| `x` | float32 | -∞ to +∞ | Yes | World X coordinate (meters) |
| `y` | float32 | -∞ to +∞ | Yes | World Y coordinate (meters) |
| `vx` | float32 | -100 to +100 | Yes | X velocity component (m/s) |
| `vy` | float32 | -100 to +100 | Yes | Y velocity component (m/s) |
| `rotation` | float32 | 0 to 2π | Yes | Rotation angle (radians) |

**Notes:**
- Position coordinates are unbounded but typically within simulation bounds
- Velocity components are physically constrained by agent capabilities
- Rotation is normalized to 0-2π range but may wrap

---

## Type Mapping

### Rust → MessagePack → TypeScript

| Rust Type | MessagePack Type | TypeScript Type | Notes |
|-----------|------------------|-----------------|-------|
| `u64` | uint 64 | `number` | ⚠️ Safe up to 2^53-1 |
| `f32` | float 32 | `number` | ✅ Always safe (upcast to f64) |
| `String` | str | `string` | ✅ Always safe |
| `Vec<T>` | array | `T[]` | ✅ Always safe |

### Integer Precision Warning

**Critical:** JavaScript `number` is IEEE 754 double (64-bit float), which can only represent integers up to 2^53-1 without loss of precision.

**Impact:**
- `tick` values > 9,007,199,254,740,991 will corrupt
- `id` values > 9,007,199,254,740,991 will corrupt

**Mitigation:**
- Runtime validation warns when values approach limits
- Simulation unlikely to exceed limits in practice
- Future: Consider string representation for large integers

---

## Validation Rules

### Publisher (Simulation Service)

**Must Guarantee:**
1. All required fields present in every message
2. `tick` is monotonically increasing
3. `timestamp` is valid ISO 8601 format
4. `agents` array contains only valid AgentTransform objects
5. No NaN or Infinity values in numeric fields
6. Agent IDs are stable across frames (same ID = same agent)

**Should Guarantee:**
7. `tick` and `id` values stay below 2^53-1
8. `rotation` values normalized to 0-2π
9. Position/velocity within simulation bounds

### Subscriber (Broadcaster Service)

**Must Validate:**
1. MessagePack decode succeeds
2. Decoded object matches SimulationFrame structure
3. All required fields present with correct types
4. No NaN values in numeric fields
5. `agents` is an array

**Should Validate:**
6. `tick` is monotonically increasing (detect dropped messages)
7. Warn when `tick` or `id` exceeds safe integer range
8. Log validation failures for monitoring

---

## Error Handling

### Decoding Errors

**Causes:**
- Corrupted MessagePack binary
- Truncated messages
- Invalid MessagePack format
- Out-of-memory conditions

**Handling:**
- Log error with details
- Emit error event (don't crash)
- Skip message, continue processing
- Increment error metrics

### Validation Errors

**Causes:**
- Missing required fields
- Wrong field types
- NaN/Infinity values
- Schema mismatch

**Handling:**
- Log error with frame details
- Emit error event
- Skip message, continue processing
- Increment validation failure metrics
- Alert if sustained high error rate

---

## Versioning Strategy

### Semantic Versioning

Contract versions follow semantic versioning: `MAJOR.MINOR.PATCH`

- **MAJOR:** Breaking changes (incompatible schema)
- **MINOR:** Backward-compatible additions (new optional fields)
- **PATCH:** Documentation fixes, clarifications

### Version Field (Future)

**Planned for v2.0.0:**
```typescript
interface SimulationFrame {
  schema_version: number;  // Contract version (1, 2, 3, ...)
  // ... existing fields
}
```

### Breaking Change Policy

**Before introducing breaking changes:**
1. Announce change 2 weeks in advance
2. Update documentation with migration guide
3. Implement backward compatibility shims if possible
4. Bump MAJOR version
5. Update both services simultaneously

**Examples of breaking changes:**
- Removing required fields
- Changing field types
- Renaming fields
- Changing field semantics

### Non-Breaking Changes

**Can be added without version bump:**
- New optional fields
- Documentation improvements
- Expanded validation rules (if backward compatible)

**Must be coordinated:**
- Publisher adds optional field
- Subscriber updated to handle optional field
- No disruption to existing consumers

---

## Performance Characteristics

### Message Size

**Typical Frame (1000 agents):**
- MessagePack: ~60 KB
- JSON (for comparison): ~85 KB
- **Savings: ~30%**

**Large Frame (10,000 agents):**
- MessagePack: ~600 KB
- JSON (for comparison): ~850 KB

### Throughput

**At 20 Hz:**
- 1,000 agents: 1.2 MB/s
- 10,000 agents: 12 MB/s

**Network Requirements:**
- Single consumer: 1-15 MB/s
- 100 consumers: 100-1500 MB/s (broadcast amplification)

### Latency

**Typical Processing:**
- Simulation serialize: <1ms
- NATS transport: 1-5ms (localhost)
- Broadcaster decode: <1ms
- Broadcaster validate: <1ms
- **Total: 3-8ms**

---

## Monitoring & Observability

### Recommended Metrics

**Publisher (Simulation):**
- `simulation_frames_published_total` (counter)
- `simulation_frame_serialize_duration_seconds` (histogram)
- `simulation_frame_size_bytes` (histogram)
- `simulation_agent_count` (gauge)

**Subscriber (Broadcaster):**
- `broadcaster_frames_received_total` (counter)
- `broadcaster_decode_errors_total` (counter)
- `broadcaster_validation_errors_total` (counter)
- `broadcaster_frame_decode_duration_seconds` (histogram)
- `broadcaster_integer_overflow_warnings_total` (counter)

### Alerts

**Critical:**
- Sustained decode error rate > 1%
- No messages received for >5 seconds
- Integer overflow warnings

**Warning:**
- Decode latency p99 > 10ms
- Validation failure rate > 0.1%
- Message size > 1MB

---

## Future Enhancements

### Planned Additions (v2.0)

```typescript
interface SimulationFrame {
  schema_version: number;    // Version field
  tick: number;
  timestamp: string;
  delta_time: number;        // Time since last frame (ms)
  agents: AgentTransform[];
}

interface AgentTransform {
  id: number;
  x: number;
  y: number;
  vx: number;
  vy: number;
  rotation: number;
  scale: number;             // Body size/radius
  species: number;           // Species ID
  health: number;            // Health (0-1 normalized)
  energy: number;            // Energy (0-1 normalized)
}
```

### Under Consideration

- Delta encoding (only send changes)
- Compression (gzip/snappy)
- Viewport-based filtering
- Multi-subject routing (spatial partitioning)
- Checksums/integrity verification

---

## Migration Guide

### From JSON to MessagePack (v1.0.0)

**Broadcaster Service:**
1. Add `@msgpack/msgpack` dependency
2. Replace `JSON.parse(new TextDecoder().decode(data))` with `decode(data)`
3. Add validation functions (`isValidSimulationFrame`)
4. Test with edge cases (corrupted data, missing fields)

**No changes required for Simulation Service** - already publishing MessagePack.

---

## References

- **MessagePack Specification:** https://msgpack.org/
- **NATS Documentation:** https://docs.nats.io/
- **Project Architecture:** `/workspace/ARCHITECTURE.md`
- **Sprint 6 Documentation:** `/workspace/SPRINT_DOCS/SPRINT_6_STREAMING_PIPELINE.md`

---

## Contact

**Questions or Issues:**
- Technical Lead: Architect Andy
- Simulation Service: Backend Sam
- Broadcaster Service: Broadcaster Brian

**Revision History:**
- v1.0.0 (2025-11-05): Initial contract specification
