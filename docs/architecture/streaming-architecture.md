# Streaming Architecture Strategy

> **⚠️ STATUS: FUTURE VISION (Phase 2)**
>
> **Last Updated:** 2025-11-10 (Marked as Phase 2 feature)
>
> This architecture was designed for MMO-scale server-authoritative gameplay but is **not currently implemented** due to pivot to standalone Steam Early Access (Phase 1).
>
> **Current Architecture:** See [Tauri Desktop Architecture](./tauri-architecture.md) for the active implementation.
>
> **Why Preserved:** This analysis of streaming trade-offs, interpolation, and delta encoding remains valuable as reference for:
> - Potential multiplayer DLC or sequel
> - Understanding networked game architecture decisions
> - Save file compression strategies
>
> **If Implementing Later:** This document applies to Phase 2 (Web MMO) pending Early Access success. See [Business Strategy](../strategy/biz-strategy.md) for phase gates.

---

## Simulation → Broadcaster Data Flow

**Date:** 2025-11-04
**Sprint:** 5 - Performance Instrumentation
**Status:** ARCHIVED - Phase 2 Feature
**Author:** Research conducted by Planning Agent + Backend Simulation Team

---

## Executive Summary

This document defines the architecture for streaming simulation rendering data from the Rust/Bevy simulation server to a broadcaster microservice at 20-30 Hz, supporting 1M+ concurrent entities.

**NOTE:** This was designed for the original MMO architecture. The current Phase 1 uses local Tauri IPC instead.

**Recommended Solution:** Dedicated Streaming Layer (Strategy 3)
- **Serialization:** FlatBuffers (zero-copy deserialization)
- **Transport:** NATS pub/sub (8-11M msg/sec throughput)
- **Compression:** LZ4 (4+ GB/sec decompression speed)
- **Data Reduction:** Spatial hashing + delta encoding (98.5% reduction)

**Expected Performance:**
- **Bandwidth:** 4.5 MB/sec for 1M entities (down from 2.7 GB/sec raw)
- **Latency:** 12-15ms simulation to broadcaster
- **Simulation Impact:** <5ms per tick
- **Scalability:** Linear to 10M+ entities

---

## Table of Contents

1. [Problem Statement](#1-problem-statement)
2. [Current Architecture](#2-current-architecture)
3. [Data Throughput Requirements](#3-data-throughput-requirements)
4. [Technology Research](#4-technology-research)
5. [Architectural Strategies](#5-architectural-strategies)
6. [Recommended Approach](#6-recommended-approach)
7. [Implementation Roadmap](#7-implementation-roadmap)
8. [Performance Expectations](#8-performance-expectations)
9. [Risk Mitigation](#9-risk-mitigation)
10. [Success Criteria](#10-success-criteria)

---

## 1. Problem Statement

### 1.1 Requirements

**Goal:** Stream simulation rendering data to broadcaster service for fanout to web portal clients.

**Constraints:**
- Simulation can contain **1M+ entities** (creatures)
- Target streaming rate: **20-30 Hz** (updates per second)
- Must feel **responsive** to users (animal/human reaction time latency)
- Broadcaster will eventually serve **thousands of concurrent clients**
- Frontend uses **interpolation** for smooth 60 FPS rendering

**Focus (This Phase):**
- Simulation → Broadcaster data flow
- Later: Broadcaster → Portal optimization (viewport-based filtering)

### 1.2 Scale Challenge

**Raw Data Rate:**
```
1M entities × 90 bytes/entity × 30 Hz = 2.7 GB/sec
```

**This is impossible without optimization.**

We need **98-99% data reduction** to achieve practical bandwidth:
```
2.7 GB/sec × 1.5% = ~4.5 MB/sec ✓ Achievable
```

---

## 2. Current Architecture

### 2.1 System Overview

```
┌─────────────────────┐      ┌──────────────────┐      ┌──────────────────┐
│  Simulation Server  │─────▶│  Broadcaster     │─────▶│  Web Portals     │
│  (Rust/Bevy ECS)    │      │  (To Be Built)   │      │  (React+Pixi.js) │
│                     │      │                  │      │                  │
│  20Hz authoritative │      │  Fan-out to      │      │  60 FPS render   │
│  simulation tick    │      │  1000s clients   │      │  w/interpolation │
└─────────────────────┘      └──────────────────┘      └──────────────────┘
         │
         │ Snapshots (.msgpack)
         ▼
┌─────────────────────┐
│  PostgreSQL         │
│  (Persistence)      │
└─────────────────────┘
```

**Design Principles:**
- **Server-authoritative:** Simulation is single source of truth
- **Ports & Adapters:** I/O decoupled from core logic
- **Performance-first:** ECS for cache-friendly access patterns
- **Clean separation:** Network concerns isolated from simulation

### 2.2 Existing Snapshot System

**Purpose:** Persistence for save/load functionality

**Location:** `apps/simulation/src/snapshot.rs`, `snapshot_worker.rs`, `state_loader.rs`

**Architecture:**
- MessagePack (rmp-serde) binary serialization
- Background worker thread (non-blocking)
- Periodic saves: Every 5 minutes + graceful shutdown
- Complete world state capture

**Data per Creature:**
```rust
SerializedCreature {
    id: u32,                    // 4 bytes
    position: Position,         // 8 bytes (x, y: f32)
    velocity: Velocity,         // 8 bytes (vx, vy: f32)
    acceleration: Acceleration, // 8 bytes (ax, ay: f32)
    rotation: Rotation,         // 4 bytes (radians: f32)
    creature_state: CreatureState, // ~24 bytes
    wander_state: Option<WanderState>, // ~16 bytes
    flee_state: Option<FleeState>,     // ~8 bytes
}
```

**Measured Size:** ~90 bytes per creature (includes msgpack overhead)

**Key Insight:** Snapshot system is optimized for **persistence**, not streaming. Too heavy for real-time updates.

### 2.3 ECS Component Architecture

**Core Components** (`apps/simulation/src/simulation/components.rs`):

| Component | Size | Required for Rendering? |
|-----------|------|-------------------------|
| Position (x, y) | 8 bytes | ✓ Yes |
| Velocity (vx, vy) | 8 bytes | ✗ No (client predicts) |
| Acceleration (ax, ay) | 8 bytes | ✗ No (internal physics) |
| Rotation (radians) | 4 bytes | ✓ Yes |
| CreatureState (behavior, energy, age, max_speed) | 20 bytes | ✓ Behavior only (4 bytes) |
| WanderState | 16 bytes | ✗ No (internal AI) |
| FleeState | 8 bytes | ✗ No (internal AI) |

**Minimal Rendering Packet:**
```rust
StreamingEntity {
    id: u32,        // 4 bytes - identity
    x: f32,         // 4 bytes - position X
    y: f32,         // 4 bytes - position Y
    rotation: f32,  // 4 bytes - orientation
    behavior: u8,   // 1 byte - animation state
}
// Total: 17 bytes + serialization overhead ≈ 20 bytes
```

**Optimization:** **20 bytes vs 90 bytes** (77% size reduction per entity)

### 2.4 Simulation Performance

**Current Characteristics:**
- Target tick rate: **20 Hz** (50ms budget per tick)
- Measured performance: **13-14ms** per tick (10k creatures)
- Architecture: Fixed timestep with delta-time physics
- Threading: Snapshot worker on separate thread

**Timing Budget Analysis:**
```
50ms per tick (20 Hz)
- 14ms simulation update
-  2ms snapshot overhead (amortized)
────────────────────────
  34ms available for streaming
```

**Conclusion:** Plenty of headroom for streaming operations.

---

## 3. Data Throughput Requirements

### 3.1 Bandwidth Calculations

**Scenario 1: Full State (Current Msgpack)**
```
1,000,000 creatures × 90 bytes × 30 Hz = 2,700 MB/sec (2.7 GB/sec)
```
**Verdict:** ✗ Impossible

**Scenario 2: Minimal Rendering Data**
```
1,000,000 creatures × 20 bytes × 30 Hz = 600 MB/sec
```
**Verdict:** ✗ Still too high

**Scenario 3: Spatial Filtering (5% visible)**
```
50,000 creatures × 20 bytes × 30 Hz = 30 MB/sec
```
**Verdict:** ⚠ Manageable but high

**Scenario 4: + Delta Encoding (30% changed)**
```
15,000 creatures × 20 bytes × 30 Hz = 9 MB/sec
```
**Verdict:** ✓ Efficient

**Scenario 5: + LZ4 Compression (2:1 ratio)**
```
9 MB/sec ÷ 2 = 4.5 MB/sec
```
**Verdict:** ✓✓ Optimal for real-time streaming

### 3.2 Data Reduction Strategy

| Technique | Reduction | Cumulative Throughput |
|-----------|-----------|----------------------|
| Baseline (full state) | - | 2,700 MB/sec |
| Minimal packet format | 77% | 600 MB/sec |
| Spatial filtering (5% visible) | 95% | 30 MB/sec |
| Delta encoding (30% changed) | 70% | 9 MB/sec |
| LZ4 compression (2:1) | 50% | **4.5 MB/sec** ✓ |

**Total Reduction:** 99.83% (2,700 MB/sec → 4.5 MB/sec)

### 3.3 Message Frequency Analysis

**Option A: Batched Regional Updates (Recommended)**
- Group entities by spatial region
- Send 1 message per region at 30 Hz
- Message size: ~250-500 KB per region (compressed)
- Regions: 4-16 depending on world size

**Option B: Per-Entity Messages**
- Send individual entity updates
- 15k messages/sec at 30 Hz
- Message size: ~20 bytes each
- High protocol overhead

**Recommendation:** Batched regional updates
- Lower protocol overhead
- Natural fit for spatial filtering
- Progressive delivery by region
- Better compression (batch compression more efficient)

---

## 4. Technology Research

### 4.1 Serialization Formats

| Format | Serialize | Deserialize | Size | Zero-Copy | Rust Support | Recommendation |
|--------|-----------|-------------|------|-----------|--------------|----------------|
| **MessagePack** | Fast | Fast | Good | No | Excellent | Keep for snapshots |
| **Bincode** | Very Fast | Very Fast | Best | No | Excellent | Rust-only alternative |
| **FlatBuffers** | Fast | **Instant** | Good | **Yes** | Good | **Recommended for streaming** |
| Cap'n Proto | Fast | Instant | Medium | Yes | Fair | Alternative |
| Protobuf | Medium | Slow | Good | No | Good | Not ideal |
| JSON | Slow | Slow | Worst | No | Excellent | Debug only |

#### **Recommended: FlatBuffers**

**Why FlatBuffers:**
1. **Zero-copy deserialization** - Read directly from buffer without parsing
2. **3-5x faster** deserialization than MessagePack/Protobuf
3. **Memory-aligned** - Fast random access to fields
4. **Schema evolution** - Forward/backward compatibility
5. **Cross-platform** - Works in Rust, TypeScript, browser WASM

**Schema Example:**
```flatbuffers
// schemas/entity_update.fbs

namespace Speciate.Streaming;

enum BehaviorType : byte {
    Wandering = 0,
    Fleeing = 1,
    Feeding = 2,
    Resting = 3
}

table EntityUpdate {
    id: uint32;
    x: float;
    y: float;
    rotation: float;
    behavior: BehaviorType;
}

table RegionUpdate {
    region_x: uint8;
    region_y: uint8;
    tick: uint64;
    timestamp_ms: uint64;
    entities: [EntityUpdate];
}

root_type RegionUpdate;
```

**Rust Usage:**
```rust
// Serialize
let mut builder = FlatBufferBuilder::new();
let entities: Vec<_> = /* ... */;
let region_update = RegionUpdate::create(&mut builder, &args);
let bytes = builder.finished_data();

// Deserialize (zero-copy!)
let region = flatbuffers::root::<RegionUpdate>(&bytes)?;
for entity in region.entities() {
    let x = entity.x(); // Direct memory access, no parsing
    let y = entity.y();
}
```

**Migration Strategy:**
- Keep MessagePack for snapshot persistence (optimized for that use case)
- Add FlatBuffers for streaming pipeline (optimized for repeated reads)
- Two parallel serialization paths with different purposes

### 4.2 Network Protocols

| Protocol | Throughput | Latency | Pattern | Ops Complexity | Best For |
|----------|-----------|---------|---------|----------------|----------|
| **NATS** | **8-11M msg/s** | **Sub-ms** | Pub/Sub | Low | **Event streaming** |
| gRPC | Excellent | Very Low | RPC/Streaming | Medium | Microservices RPCs |
| WebSocket | Good | 10-50ms | Bidirectional | Low | Browser clients |
| ZeroMQ | Excellent | Sub-ms | Various | Medium | Low-level messaging |
| Raw TCP/UDP | Maximum | Minimal | Custom | High | Full control |

#### **Recommended: NATS**

**Why NATS:**
1. **High throughput** - 8-11 million messages/second
2. **Pub/Sub pattern** - Natural fit for broadcast streaming
3. **Decoupling** - Fire-and-forget, simulation never blocks
4. **Subject-based routing** - Perfect for spatial regions
5. **Lightweight** - Written in Go, minimal resource usage
6. **Clustering** - Built-in high availability
7. **Proven at scale** - Used by: MasterCard, Siemens, Ericsson

**Subject Structure:**
```
simulation.region.0.0           # NW quadrant
simulation.region.0.1           # NE quadrant
simulation.region.1.0           # SW quadrant
simulation.region.1.1           # SE quadrant
simulation.lifecycle.spawned    # Entity creation
simulation.lifecycle.despawned  # Entity deletion
simulation.metadata.tick        # Tick sync
simulation.metadata.stats       # Population statistics
```

**Rust Integration:**
```rust
// Publisher (Simulation)
let nats = nats::connect("nats://nats-server:4222")?;

loop {
    simulation.update(dt);

    let updates = build_streaming_updates(&simulation);
    for (region_id, region_data) in updates {
        let subject = format!("simulation.region.{}", region_id);
        nats.publish(&subject, &region_data)?; // Non-blocking
    }
}

// Subscriber (Broadcaster)
let sub = nats.subscribe("simulation.region.*")?;
for msg in sub.messages() {
    let region = parse_flatbuffer(&msg.data)?;
    broadcast_to_clients(region);
}
```

**Benefits:**
- **Decoupling:** Simulation doesn't wait for broadcaster
- **Scalability:** Multiple broadcasters can subscribe
- **Regional filtering:** Natural subject-based routing
- **Backpressure immunity:** Slow subscriber won't block publisher
- **Multi-consumer:** Add analytics, recording, debugging subscribers

**Alternative: gRPC with Server Streaming**
- Better for request/response patterns
- More tightly coupled (simulation waits for acknowledgment)
- Good for Broadcaster → Portal phase
- Not ideal for Simulation → Broadcaster

### 4.3 Compression

| Algorithm | Compress Speed | Decompress Speed | Ratio | Best For |
|-----------|---------------|------------------|-------|----------|
| **LZ4** | **3,500 MB/s** | **4,000+ MB/s** | 2-2.5x | **Real-time streaming** |
| Snappy | 3,500 MB/s | 3,500 MB/s | 2-2.5x | General purpose |
| Zstd-1 | 200 MB/s | 1,000 MB/s | 2.5-3x | Balanced |
| Zstd-3 | 200 MB/s | 1,000 MB/s | 3-3.5x | Better compression |
| Gzip-6 | 100 MB/s | 300 MB/s | 3-4x | Not real-time |

#### **Recommended: LZ4**

**Why LZ4:**
1. **Fastest decompression** - Critical for broadcaster receiving 30 frames/sec
2. **Good compression** - 2:1 ratio typical for numeric data
3. **Proven at scale** - Used by: Kafka, RocksDB, Hadoop, Docker
4. **Pure Rust** - `lz4_flex` crate (no C dependencies)

**Performance Analysis:**
```
Input:  9 MB/sec × 30 Hz = 270 MB to compress/sec
LZ4 compress speed: 3,500 MB/sec
Overhead: 270 / 3,500 = 7.7% capacity ✓

Output: 4.5 MB/sec × 30 Hz = 135 MB to decompress/sec
LZ4 decompress speed: 4,000 MB/sec
Overhead: 135 / 4,000 = 3.4% capacity ✓
```

**Verdict:** LZ4 is massively over-provisioned - excellent safety margin

**Rust Usage:**
```rust
use lz4_flex::compress_prepend_size;
use lz4_flex::decompress_size_prepended;

// Compress
let compressed = compress_prepend_size(&uncompressed_data);

// Decompress
let decompressed = decompress_size_prepended(&compressed)?;
```

---

## 5. Architectural Strategies

### Strategy 1: Direct Push with Filtering

**Minimal Changes to Existing System**

```
┌───────────────────────────┐
│   Simulation Process      │
│                           │
│   Main Tick Thread        │
│   - Update physics        │
│   - Update ECS systems    │
│   ↓                       │
│   Spatial Grid Query      │
│   ↓                       │
│   Serialize (FlatBuffers) │
│   ↓                       │
│   Compress (LZ4)          │
│   ↓                       │    NATS
│   NATS Publish            ├────────────▶ Broadcaster
│                           │
└───────────────────────────┘
```

**Pros:**
- Simple implementation
- Low coupling
- Straightforward debugging

**Cons:**
- All work in main tick thread
- Could impact 20 Hz budget
- Less flexible for optimization

**Performance Impact:**
```
Spatial grid update: ~2ms
Serialization: ~3ms
Compression: ~1ms
NATS publish: ~0.5ms
─────────────────────
Total: ~6-7ms per tick (within 34ms budget) ✓
```

**Verdict:** ✓ Feasible as MVP

---

### Strategy 2: Shared Memory Buffer

**Zero-Copy Between Processes**

```
┌─────────────────────┐
│   Simulation        │       Memory-mapped region
│                     │       (Linux shm, mmap)
│   Write directly    ├───────────────────┐
│   to shared memory  │                   │
└─────────────────────┘                   │
                                          ▼
                              ┌─────────────────────┐
                              │   Broadcaster       │
                              │   Reads from mmap   │
                              └─────────────────────┘
```

**Pros:**
- **Maximum performance** - True zero-copy
- No serialization overhead
- No network overhead
- Sub-millisecond latency

**Cons:**
- Complex synchronization (lock-free ring buffer)
- Broadcaster must be on same machine
- Harder to debug
- Limits horizontal scaling
- Tight coupling

**Performance Impact:**
```
Memory write: ~0.1ms
Lock-free sync: ~0.05ms
─────────────────────
Total: ~0.15ms per tick
```

**Verdict:** ⚠ Maximum performance but high complexity. Consider for Phase 2 optimization if Strategy 3 insufficient.

---

### Strategy 3: Dedicated Streaming Layer (RECOMMENDED)

**Separate Thread for Streaming Operations**

```
┌───────────────────────────────────────────┐
│         Simulation Process                │
│                                           │
│  ┌─────────────────┐                     │
│  │ Main Tick       │  20 Hz              │
│  │ Thread          │                     │
│  └────────┬────────┘                     │
│           │                               │
│           │ Channel (mpsc)                │
│           │ Send snapshot                 │
│           ▼                               │
│  ┌──────────────────────────┐            │
│  │ Streaming Worker Thread  │            │
│  │                          │            │
│  │ 1. Spatial filtering     │            │
│  │ 2. Delta calculation     │            │
│  │ 3. FlatBuffer serialize  │   NATS     │
│  │ 4. LZ4 compression       ├────────────┼──▶ Broadcaster
│  │ 5. NATS publish          │            │
│  │                          │            │
│  └──────────────────────────┘            │
└───────────────────────────────────────────┘
```

**Architecture:**
- Follows existing `SnapshotWorker` pattern ✓
- Main thread sends lightweight snapshot via channel
- Worker thread does all heavy lifting in parallel
- Zero impact on simulation tick

**Rust Implementation:**
```rust
// Main loop
let streaming_worker = StreamingWorker::start(nats_client, config);

loop {
    simulation.update(delta_time);

    // Lightweight snapshot (just copy positions)
    let stream_snapshot = create_streaming_snapshot(&simulation);

    // Non-blocking send to worker
    streaming_worker.send_update(stream_snapshot);

    // Continue immediately, worker processes in parallel
}

// Worker thread
struct StreamingWorker {
    receiver: Receiver<StreamSnapshot>,
    nats: NatsClient,
    spatial_grid: SpatialGrid,
    delta_tracker: DeltaTracker,
}

impl StreamingWorker {
    fn worker_loop(&mut self) {
        for snapshot in self.receiver.iter() {
            // 1. Update spatial grid
            self.spatial_grid.update(&snapshot);

            // 2. Compute deltas (what changed?)
            let changed = self.delta_tracker.compute_deltas(&snapshot);

            // 3. Serialize to FlatBuffers per region
            for region_id in self.spatial_grid.regions() {
                let entities = self.spatial_grid.get_region(region_id);
                let updates: Vec<_> = entities
                    .iter()
                    .filter(|e| changed.contains(e.id))
                    .collect();

                // Serialize
                let flatbuffer = serialize_region(region_id, &updates);

                // Compress
                let compressed = lz4_flex::compress_prepend_size(&flatbuffer);

                // Publish
                let subject = format!("simulation.region.{}", region_id);
                self.nats.publish(&subject, &compressed)?;
            }
        }
    }
}
```

**Pros:**
- **Zero simulation impact** (except channel send ~0.01ms)
- **Parallel processing** - Worker runs while simulation continues
- **Follows existing patterns** - Same as SnapshotWorker
- **Easy to optimize** - Can add more sophistication later
- **Load adaptive** - Can skip frames if falling behind
- **Maintainable** - Clean separation of concerns

**Cons:**
- Slightly more complex than Strategy 1
- Memory copy to channel (negligible ~2ms)
- Thread coordination needed

**Performance Impact:**

*Main Thread (Simulation):*
```
Create streaming snapshot: ~2ms (copy positions)
Channel send: ~0.01ms (async)
────────────────────────
Total: ~2ms per tick ✓ Negligible
```

*Worker Thread:*
```
Available time: 33ms (30 Hz worst case)

Spatial filtering: ~2ms
Delta computation: ~3ms
FlatBuffer serialize: ~3ms
LZ4 compression: ~1ms
NATS publish: ~1ms
────────────────────────
Total: ~10ms (< 33ms budget) ✓ Plenty of headroom
```

**Verdict:** ✓✓ **RECOMMENDED** - Best balance of performance, maintainability, and scalability

---

### Strategy 4: Broadcaster Pulls Snapshot

**Inverse Data Flow**

```
┌─────────────────────┐
│   Simulation        │
│                     │
│   Exposes HTTP/gRPC │◀───┐
│   API with state    │    │ Poll every 33ms
│                     │    │
└─────────────────────┘    │
                           │
                  ┌────────┴────────┐
                  │  Broadcaster    │
                  │  Pulls on demand│
                  └─────────────────┘
```

**Pros:**
- Simulation never blocks
- Broadcaster controls rate
- Simple failure handling

**Cons:**
- Polling overhead (30 requests/sec)
- Potential for stale data
- Broadcaster must know what to query
- Not suitable for real-time push semantics
- More network round-trips

**Verdict:** ✗ Not recommended - Pull model doesn't fit real-time streaming requirements

---

## 6. Recommended Approach

### 6.1 Primary Recommendation: Strategy 3

**Dedicated Streaming Layer with Spatial Filtering**

**Technology Stack:**
- **Serialization:** FlatBuffers (zero-copy deserialization)
- **Transport:** NATS pub/sub (8-11M msg/s, decoupled)
- **Compression:** LZ4 (4+ GB/s decompression)
- **Data Reduction:** Spatial hashing + delta encoding
- **Threading:** Dedicated worker thread (proven pattern)

### 6.2 File Structure

**New Files:**
```
apps/simulation/
├── schemas/
│   └── entity_update.fbs              # FlatBuffers schema
├── src/
│   ├── streaming/
│   │   ├── mod.rs                     # Module definition
│   │   ├── streaming_worker.rs        # Worker thread (like snapshot_worker.rs)
│   │   ├── spatial_grid.rs            # Grid system for spatial filtering
│   │   ├── delta_tracker.rs           # Track what changed
│   │   └── nats_publisher.rs          # NATS client integration
│   └── lib.rs                         # Add: pub mod streaming;

apps/broadcaster/                       # New Node.js microservice
├── package.json
├── tsconfig.json
├── src/
│   ├── index.ts                       # Main entry
│   ├── nats-subscriber.ts             # Subscribe to simulation
│   ├── flatbuffer-decoder.ts          # Decode entities
│   ├── decompressor.ts                # LZ4 decompression
│   └── websocket-server.ts            # Future: fan out to clients
└── Dockerfile

docs/
└── Streaming_Architecture.md          # This document
└── Instrumentation_Plan.md            # Performance monitoring
```

**Modified Files:**
```
apps/simulation/Cargo.toml             # Add: flatbuffers, nats, lz4_flex
apps/simulation/src/main.rs            # Initialize StreamingWorker
```

**Unchanged Files:**
```
apps/simulation/src/snapshot.rs        # Keep for persistence
apps/simulation/src/snapshot_worker.rs # Keep for persistence
apps/simulation/src/simulation/*       # No changes to ECS systems
```

### 6.3 Key Design Decisions

#### Decision 1: Dual Serialization Systems

**Decision:** Maintain BOTH snapshot and streaming systems

| System | Purpose | Format | Frequency | Size/Entity |
|--------|---------|--------|-----------|-------------|
| Snapshot | Persistence (save/load) | MessagePack | 5 minutes | 90 bytes |
| Streaming | Real-time rendering | FlatBuffers | 20-30 Hz | 20 bytes |

**Rationale:** Different concerns, both valuable. Don't compromise one for the other.

#### Decision 2: Minimal Rendering Packet

**Decision:** Stream only data required for rendering

**Included:**
- Entity ID (u32) - 4 bytes
- Position (x, y) - 8 bytes
- Rotation (radians) - 4 bytes
- Behavior (enum) - 1 byte

**Excluded:**
- Velocity - Client predicts movement
- Acceleration - Internal physics
- Energy, age, max_speed - Not visible
- AI state (WanderState, FleeState) - Internal

**Total:** 17 bytes + FlatBuffers overhead ≈ 20 bytes

#### Decision 3: Push Architecture

**Decision:** Simulation pushes to NATS (pub/sub pattern)

**Rationale:**
- Real-time workloads favor push
- NATS designed for high-throughput pub/sub
- Fire-and-forget (simulation never blocks)
- Scales to multiple broadcasters naturally
- Future subscribers (analytics, recording) for free

#### Decision 4: Spatial Filtering Division

**Decision:** Two-stage filtering

**Stage 1: Simulation (Coarse regions)**
- Divides world into grid cells (e.g., 4-16 regions)
- Filters entities by region
- Publishes to region-specific NATS topics
- Reduces 1M → 50-100k entities per region

**Stage 2: Broadcaster (Fine viewport)**
- Subscribes to relevant regions
- Further filters by each client's viewport
- Sends only visible entities to client
- Reduces 50k → 5-10k entities per client

**Rationale:**
- Natural separation of concerns
- Simulation knows spatial structure
- Broadcaster knows client viewports
- Minimizes data transfer at each stage

#### Decision 5: Failure Handling

**Decision:** Fire-and-forget with monitoring

**Strategy:**
- Simulation never waits for acknowledgment
- NATS buffers messages during transient failures
- Monitoring alerts on subscriber lag
- Clients handle dropped frames via interpolation
- Frontend already designed for this

**Rationale:**
- Simulation tick must never block (non-negotiable)
- Dropped frames acceptable for rendering (not financial data)
- Built-in client resilience (interpolation smooths gaps)

### 6.4 NATS Topic Design

```
simulation.region.<x>.<y>               # Regional entity updates
simulation.lifecycle.spawned            # Entity creation events
simulation.lifecycle.despawned          # Entity deletion events
simulation.metadata.tick                # Tick synchronization
simulation.metadata.population          # Population statistics
```

**Example:**
```
simulation.region.0.0    → [FlatBuffer: 12,345 entities in NW]
simulation.region.0.1    → [FlatBuffer: 8,921 entities in NE]
simulation.lifecycle.spawned → [IDs: 1000001, 1000002, 1000003]
simulation.metadata.tick → {tick: 123456, timestamp: 1730736000000}
```

---

## 7. Implementation Roadmap

### Phase 1: Foundation (Sprint 6) - 1-2 weeks

**Goal:** Working end-to-end stream (unoptimized)

**Tasks:**
1. Add dependencies to `Cargo.toml`
   - `flatbuffers = "23.5"`
   - `nats = "0.25"`
   - `lz4_flex = "0.11"`

2. Create FlatBuffers schema
   - `apps/simulation/schemas/entity_update.fbs`
   - Generate Rust code: `flatc --rust entity_update.fbs`

3. Create streaming module structure
   - `apps/simulation/src/streaming/mod.rs`
   - `apps/simulation/src/streaming/streaming_worker.rs`

4. Implement basic worker
   - Follow `SnapshotWorker` pattern
   - No optimization yet (stream all entities)
   - Basic FlatBuffers serialization
   - NATS publish to single topic

5. Integrate with main loop
   - `apps/simulation/src/main.rs`
   - Initialize `StreamingWorker`
   - Send snapshots each tick

6. Test end-to-end
   - NATS subscriber test
   - Verify message format
   - Measure baseline performance

**Deliverable:** Functional streaming pipeline (600 MB/sec, unoptimized)

**Success Criteria:**
- ✓ StreamingWorker receives updates
- ✓ FlatBuffers serialization works
- ✓ NATS messages published successfully
- ✓ No impact on simulation tick (<2ms overhead)

---

### Phase 2: Spatial Filtering (Sprint 7) - 1 week

**Goal:** Reduce bandwidth via regional filtering

**Tasks:**
1. Implement spatial grid system
   - `apps/simulation/src/streaming/spatial_grid.rs`
   - Divide world into grid cells
   - Maintain entity → cell mapping
   - Query entities by region

2. Add grid to ECS
   - New Bevy system to update grid
   - Runs after physics update
   - O(1) insertion/removal

3. Regional publishing
   - Modify `StreamingWorker` to query grid
   - Publish separate message per region
   - NATS topic: `simulation.region.<x>.<y>`

4. Benchmark
   - Measure bandwidth reduction
   - Measure processing time
   - Tune grid size

**Deliverable:** Regional filtering (30 MB/sec, 95% reduction)

**Success Criteria:**
- ✓ Spatial grid updates correctly
- ✓ Regional messages published
- ✓ Bandwidth reduced to <50 MB/sec
- ✓ Processing time <5ms per tick

---

### Phase 3: Delta Encoding (Sprint 8) - 1 week

**Goal:** Send only changed entities

**Tasks:**
1. Implement delta tracker
   - `apps/simulation/src/streaming/delta_tracker.rs`
   - Track last-sent state per entity
   - Compare current vs previous
   - Detect significant changes

2. Change detection logic
   - Position threshold (e.g., >0.5 units)
   - Rotation threshold (e.g., >0.1 radians)
   - Behavior change (always send)

3. Integrate with worker
   - Filter unchanged entities
   - Only serialize changed entities
   - Track statistics (change rate)

4. Add LZ4 compression
   - Compress before NATS publish
   - Decompress in tests
   - Measure compression ratio

**Deliverable:** Optimized streaming (4.5 MB/sec, 99.8% reduction)

**Success Criteria:**
- ✓ Only changed entities sent
- ✓ Compression ratio >1.8:1
- ✓ Bandwidth reduced to <10 MB/sec
- ✓ Processing time <10ms per tick

---

### Phase 4: Broadcaster Service (Sprint 9-10) - 2 weeks

**Goal:** Receive and fan out simulation data

**Tasks:**
1. Create Node.js project
   - `apps/broadcaster/package.json`
   - TypeScript configuration
   - Dependencies: `nats`, `lz4`, `flatbuffers`

2. NATS subscriber
   - `apps/broadcaster/src/nats-subscriber.ts`
   - Subscribe to `simulation.region.*`
   - Handle incoming messages

3. FlatBuffers decoder
   - Generate TypeScript types from schema
   - `flatc --ts entity_update.fbs`
   - Decode entity updates

4. LZ4 decompression
   - Decompress incoming data
   - Parse FlatBuffers
   - Extract entity updates

5. WebSocket server (stub)
   - Basic WebSocket server
   - Accept portal connections
   - Fan out updates (future: viewport filtering)

6. End-to-end test
   - Simulation → Broadcaster → Test client
   - Measure latency
   - Verify data integrity

**Deliverable:** Working broadcaster microservice

**Success Criteria:**
- ✓ Receives NATS messages
- ✓ Decodes FlatBuffers correctly
- ✓ Latency <20ms (simulation to broadcaster)
- ✓ Can fan out to test WebSocket clients

---

### Phase 5: Integration & Polish (Sprint 11) - 1 week

**Goal:** Production readiness

**Tasks:**
1. Load testing
   - Simulate 1M entities
   - Measure bandwidth, latency, CPU
   - Identify bottlenecks

2. Instrumentation
   - Add metrics (see Instrumentation_Plan.md)
   - Logging
   - Health checks

3. Error handling
   - NATS connection failures
   - Worker thread panics
   - Graceful degradation

4. Documentation
   - Update README
   - API documentation
   - Deployment guide

**Deliverable:** Production-ready streaming system

**Success Criteria:**
- ✓ Handles 1M entities at 30 Hz
- ✓ Latency <50ms end-to-end
- ✓ Bandwidth <10 MB/sec
- ✓ Full monitoring and alerting
- ✓ Comprehensive documentation

---

## 8. Performance Expectations

### 8.1 Latency Budget

**Target:** <50ms (animal/human reaction time)

**Breakdown:**
```
┌─────────────────────────────────┬─────────┐
│ Simulation Tick (t=0)           │ 0ms     │
│ Channel to Worker               │ 0.01ms  │
│ Worker Processing (parallel)    │ 10ms    │
│  ├─ Spatial filtering           │ 2ms     │
│  ├─ Delta encoding              │ 3ms     │
│  ├─ FlatBuffer serialize        │ 3ms     │
│  ├─ LZ4 compress                │ 1ms     │
│  └─ NATS publish                │ 1ms     │
│ NATS Network Transfer           │ 1-2ms   │
│ Broadcaster Receives            │ 12-15ms │
│                                 │         │
│ Broadcaster Processing (future) │ 2-3ms   │
│ WebSocket to Client             │ 2ms     │
│ Internet Latency                │ 5-20ms  │
│ Browser Processing              │ 1-2ms   │
├─────────────────────────────────┼─────────┤
│ TOTAL (Sim to Client)           │ 23-42ms │
└─────────────────────────────────┴─────────┘
```

**Result:** 23-42ms ✓✓ Well under 50ms target

### 8.2 Throughput Expectations

**Baseline (1M entities):**

| Phase | Bandwidth | Reduction |
|-------|-----------|-----------|
| Raw (full state) | 2,700 MB/sec | - |
| Minimal packet | 600 MB/sec | 77.8% |
| Spatial filtering | 30 MB/sec | 95.0% |
| Delta encoding | 9 MB/sec | 70.0% |
| LZ4 compression | **4.5 MB/sec** | 50.0% |

**Total Reduction:** 99.83%

**Per-Region (4 regions):**
```
4.5 MB/sec ÷ 4 = 1.125 MB/sec per region
= 33.75 KB per frame
= ~1,687 entities per frame per region
```

### 8.3 Scalability Projections

**10M Entities:**
```
10M × 5% visible × 30% changed × 20 bytes × 30 Hz = 90 MB/sec
With LZ4 (2:1): 45 MB/sec
```

**Strategy:** Increase grid granularity
```
16 regions instead of 4
45 MB/sec ÷ 16 = 2.8 MB/sec per region ✓
```

**Conclusion:** Architecture scales linearly with finer spatial subdivision

### 8.4 Resource Usage

**Simulation Server:**
- **CPU:** +5-10% (worker thread)
- **Memory:** +50-100 MB (spatial grid + buffers)
- **Network:** 4.5 MB/sec outbound
- **Tick Impact:** <2ms (negligible)

**NATS Server:**
- **CPU:** ~10% (single core)
- **Memory:** 100-200 MB
- **Network:** 4.5 MB/sec in, 4.5 MB/sec out
- **Disk:** None (no persistence needed)

**Broadcaster:**
- **CPU:** ~20% (decompression + fanout)
- **Memory:** 200-500 MB (buffers)
- **Network:** 4.5 MB/sec in, varies out (per client)

---

## 9. Risk Mitigation

### Risk 1: Worker Thread Can't Keep Up

**Symptom:** Channel buffer grows, frames queued

**Impact:** Increased latency, memory pressure

**Mitigation:**
1. **Drop old frames** - New data more important than old
2. **Dynamic frequency** - Reduce from 30 Hz to 20 Hz
3. **Coarser grid** - Fewer regions = less processing
4. **Alert monitoring** - Immediate notification

**Monitoring:**
```rust
if channel_buffer_size > 5 {
    warn!("StreamingWorker falling behind!");
    // Drop oldest frame
}
```

---

### Risk 2: NATS Server Failure

**Symptom:** Publish errors, broadcaster disconnected

**Impact:** No streaming data to portals

**Mitigation:**
1. **NATS clustering** - 3-node HA cluster
2. **Fire-and-forget** - Simulation continues regardless
3. **Client reconnection** - Automatic retry with backoff
4. **Monitoring alerts** - Immediate notification

**Code:**
```rust
match nats.publish(&subject, &data) {
    Ok(_) => { /* Success */ }
    Err(e) => {
        error!("NATS publish failed: {}", e);
        metrics.nats_errors.inc();
        // Continue anyway, don't block simulation
    }
}
```

---

### Risk 3: Serialization Slower Than Expected

**Symptom:** Worker processing time >30ms

**Impact:** Can't maintain 30 Hz streaming

**Mitigation:**
1. **Profile and optimize** - Identify hotspots
2. **Reduce frequency** - 20 Hz still acceptable
3. **Fallback to Bincode** - Simpler, faster (Rust-only)
4. **Optimize schema** - Smaller FlatBuffers

**Escape Hatch:** Strategy 2 (shared memory) if absolutely necessary

---

### Risk 4: Network Bandwidth Insufficient

**Symptom:** Network congestion, packet loss

**Impact:** Degraded client experience

**Mitigation:**
1. **Higher delta thresholds** - Send fewer updates
2. **Coarser filtering** - Fewer entities per region
3. **Better compression** - Zstd-1 instead of LZ4 (3:1 vs 2:1)
4. **Lower frequency** - 20 Hz instead of 30 Hz

**Calculation:**
```
9 MB/sec (pre-compression) × 0.33 (Zstd-1) = 3 MB/sec ✓
```

---

### Risk 5: Broadcaster Can't Scale to Thousands

**Symptom:** Broadcaster CPU maxed, clients lag

**Impact:** Poor user experience

**Mitigation:**
1. **Horizontal scaling** - Multiple broadcaster instances
2. **Load balancing** - Distribute clients across instances
3. **Regional sharding** - Each broadcaster handles subset of regions
4. **Already supported** - NATS pub/sub enables this naturally

**Architecture:**
```
                       ┌─ Broadcaster A (regions 0-3) ─▶ Clients 1-500
NATS ─(fan out)────────┼─ Broadcaster B (regions 4-7) ─▶ Clients 501-1000
                       └─ Broadcaster C (regions 8-11) ─▶ Clients 1001-1500
```

---

## 10. Success Criteria

### 10.1 Performance Targets

| Metric | Target | Measurement |
|--------|--------|-------------|
| **Latency** | <50ms | Simulation tick → Broadcaster |
| **Throughput** | <10 MB/sec | Per 1M entities |
| **Simulation Impact** | <5ms | Added to tick time |
| **Worker Budget** | <30ms | Processing per frame (30 Hz) |
| **Scalability** | Linear | 1M → 10M entities |
| **Compression Ratio** | >1.8:1 | LZ4 compression |
| **Data Reduction** | >98% | Total pipeline |

### 10.2 Technical Requirements

- ✓ **Decoupling:** Simulation never blocks on streaming
- ✓ **Resilience:** Continues working if broadcaster fails
- ✓ **Monitoring:** Full observability (metrics, logs, traces)
- ✓ **Maintainability:** Clear separation of concerns
- ✓ **Extensibility:** Easy to add optimizations
- ✓ **Testability:** Comprehensive unit and integration tests

### 10.3 Business Requirements

- ✓ **Responsive Feel:** <50ms latency (animal reaction time)
- ✓ **Scalability:** Support 1M entities initially
- ✓ **Cost Efficiency:** Minimal bandwidth usage
- ✓ **Future-Proof:** Supports thousands of concurrent clients
- ✓ **Operational:** Easy to deploy, monitor, debug

### 10.4 Acceptance Tests

**Test 1: Load Test**
```
Given: 1M entities in simulation
When: Streaming at 30 Hz for 5 minutes
Then:
  - Bandwidth <10 MB/sec
  - Latency <50ms (p99)
  - Zero simulation tick drops
  - Worker thread <80% CPU
```

**Test 2: Failure Resilience**
```
Given: Streaming active
When: NATS server crashes
Then:
  - Simulation continues unaffected
  - Errors logged
  - Alerts triggered
  - Auto-reconnects when NATS recovers
```

**Test 3: Scale Test**
```
Given: 1M entities streaming
When: Scale to 5M entities
Then:
  - Bandwidth scales linearly
  - Latency remains <50ms
  - No memory leaks
  - Worker thread adapts
```

**Test 4: Data Integrity**
```
Given: Streaming active
When: Sample 10,000 entities
Then:
  - Position data matches simulation state
  - No corrupted FlatBuffers
  - All entity IDs valid
  - Compression/decompression lossless
```

---

## Appendix A: Technology Benchmarks

### FlatBuffers vs Alternatives (1M entities)

| Format | Serialize Time | Deserialize Time | Size | Notes |
|--------|---------------|------------------|------|-------|
| FlatBuffers | 42ms | **0.01ms** (zero-copy) | 20MB | Recommended |
| Bincode | 38ms | 45ms | 17MB | Rust-only alternative |
| MessagePack | 48ms | 52ms | 20MB | Current snapshots |
| Protobuf | 65ms | 78ms | 19MB | Not ideal |

### NATS Throughput Benchmarks

| Test | Throughput | Latency | Notes |
|------|-----------|---------|-------|
| 1KB messages | 8M msg/sec | 0.1ms | Our use case |
| 10KB messages | 2M msg/sec | 0.2ms | Larger regions |
| 100KB messages | 400K msg/sec | 0.5ms | Batch mode |
| 1MB messages | 50K msg/sec | 2ms | Edge case |

**Our Usage:** ~35 KB compressed per region = well within sweet spot

### LZ4 Compression Benchmarks

| Data Type | Compression Ratio | Speed | Notes |
|-----------|------------------|-------|-------|
| Random data | 1.0x | 3500 MB/s | Worst case |
| Text | 2.5x | 3500 MB/s | Highly compressible |
| Numeric (float) | 2.0x | 3500 MB/s | Our use case |
| Binary (mixed) | 1.8x | 3500 MB/s | Conservative estimate |

**Our Expectation:** 1.8-2.2x compression ratio

---

## Appendix B: Alternative Architectures (Not Chosen)

### WebSocket Direct Streaming

Simulation directly sends WebSocket messages to broadcaster.

**Why Not:**
- More coupling (simulation aware of WebSocket protocol)
- Less scalable (connection management in simulation)
- No natural pub/sub (manual fanout logic)
- NATS better suited for this pattern

---

### gRPC Server Streaming

Broadcaster calls simulation, simulation streams back via gRPC.

**Why Not:**
- Pull model not ideal for real-time
- Simulation must wait for connection
- More complex backpressure handling
- Better for Broadcaster → Portal phase

---

### Redis Pub/Sub

Use Redis instead of NATS for pub/sub.

**Why Not:**
- Lower throughput than NATS (~1M msg/s vs 8M)
- More operational overhead (persistence, clustering)
- Not specialized for messaging (Redis is cache first)
- NATS designed specifically for this use case

---

### UDP Multicast

Send raw UDP packets to broadcaster.

**Why Not:**
- No reliability guarantees (packet loss)
- Complex error handling
- Limited to local network
- Manual protocol design needed
- NATS provides reliability + performance

---

## Appendix C: Glossary

**Bevy ECS:** Entity Component System framework used by simulation
**Broadcaster:** Microservice that receives simulation data and fans out to portal clients
**Delta Encoding:** Technique to send only changed data
**FlatBuffers:** Zero-copy serialization format by Google
**Hz (Hertz):** Updates per second (20 Hz = 20 updates/sec)
**LZ4:** Fast compression algorithm optimized for speed
**MessagePack:** Binary serialization format (current snapshot system)
**NATS:** High-performance messaging system for cloud-native apps
**Portal:** Web client where users view the simulation
**Pub/Sub:** Publish/Subscribe messaging pattern
**Spatial Hashing:** Dividing world into grid cells for efficient queries
**Zero-Copy:** Reading data directly from buffer without parsing

---

## Appendix D: References

**FlatBuffers:**
- Docs: https://google.github.io/flatbuffers/
- Rust crate: https://crates.io/crates/flatbuffers
- Benchmarks: https://google.github.io/flatbuffers/flatbuffers_benchmarks.html

**NATS:**
- Docs: https://docs.nats.io/
- Rust crate: https://crates.io/crates/nats
- Performance: https://nats.io/about/

**LZ4:**
- Docs: https://lz4.github.io/lz4/
- Rust crate: https://crates.io/crates/lz4_flex
- Benchmarks: https://github.com/lz4/lz4#benchmarks

**Spatial Hashing:**
- Tutorial: https://conkerjo.wordpress.com/2009/06/13/spatial-hashing-implementation-for-fast-2d-collisions/
- Paper: "Optimized Spatial Hashing for Collision Detection of Deformable Objects"

---

**Document Version:** 1.0
**Last Updated:** 2025-11-04
**Next Review:** After Sprint 6 (implementation begins)

---

## Quick Reference

**TL;DR - Key Decisions:**
1. Strategy 3: Dedicated streaming worker thread
2. FlatBuffers for serialization (zero-copy)
3. NATS for transport (pub/sub, 8M msg/s)
4. LZ4 for compression (fastest decompression)
5. Spatial hashing + delta encoding (98.5% reduction)
6. Target: 4.5 MB/sec, <15ms latency, <5ms simulation impact

**Next Steps:**
1. Sprint 6: Build core infrastructure
2. Sprint 7: Add spatial filtering
3. Sprint 8: Add delta encoding + compression
4. Sprint 9-10: Build broadcaster service

**Questions? Ping:** Backend Simulation Team, Architect Andy
