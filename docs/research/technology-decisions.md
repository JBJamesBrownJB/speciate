# Technology Decisions: Streaming Pipeline

**Sprint:** Sprint 6 - Streaming Pipeline
**Date:** 2025-11-05
**Status:** Approved - Walking Skeleton Focus

---

## Core Architecture

```
Simulation (Rust, 20Hz)
    ↓
NATS Core (fire-and-forget, sub-ms latency)
    ↓
Broadcaster (Node.js/TypeScript)
    ↓
WebSockets (WSS, binary MessagePack)
    ↓
Portal (Browser, 60 FPS rendering via interpolation)
```

---

## Technology Choices

### 1. NATS Core for Message Broker

**Choice:** NATS Core (not JetStream, not RabbitMQ, not Kafka)

**Rationale:**
- **Sub-millisecond latency:** 200-400μs typical, perfect for 20Hz simulation ticks
- **Fire-and-forget semantics:** Matches UDP-like philosophy (accept loss over backpressure)
- **Throughput:** 11-12M msg/sec far exceeds our needs (20 ticks/sec × 500k entities = 10M msgs/sec peak)
- **Simple operations:** No JVM, no complex configs, runs great in single Docker container
- **Rust client:** `async-nats` is mature, ergonomic, production-ready

**Subject Design:**
```
speciate.agents.{agent_id}.transform
```
Contains: position (x, y), orientation (radians), size (radius)

**Why Not JetStream?**
- No persistence needed (no replay/rewind requirement)
- Position updates are ephemeral (next tick overwrites)
- Reserve JetStream for future critical events (agent deaths, player actions)

---

### 2. WebSockets for Client Streaming

**Choice:** WebSockets with binary MessagePack encoding (not Server-Sent Events)

**Rationale:**
- **60 FPS target:** Requires <16ms frame budget; WebSocket overhead is 1-5ms (plenty of headroom)
- **Binary efficiency:** MessagePack reduces payload size ~40% vs JSON (critical for 500k entities)
- **Future-proof:** Supports bidirectional communication when player actions are added
- **Industry standard:** Real-time games universally use WebSockets

**Why Not SSE?**
- SSE overhead: 5-20ms (tight margins for 60 FPS)
- Text-based (JSON) adds parsing overhead every frame
- No bidirectional support (would need separate POST for player actions)

**Client-Side Strategy:**
- Server sends at 20Hz (50ms intervals)
- Client interpolates to 60 FPS (16.67ms intervals)
- Smooth animation despite lower server tick rate

---

### 3. Local Observability Stack

**Choice:** Prometheus + Grafana for metrics (not Cloud Monitoring)

**Rationale:**
- **Runs locally:** Docker-compose compatible, no cloud dependencies
- **Production-ready:** Same tools used in production (smooth transition)
- **Real-time visibility:** Essential for debugging simulation performance
- **Standard ecosystem:** Battle-tested exporters and dashboards

**Key Metrics:**
- **Simulation:** Tick rate (Hz), message publish rate (msg/sec), message size (bytes), NATS publish latency (ms)
- **NATS:** Throughput (msg/sec), connection count, memory usage
- **Broadcaster:** Active WebSocket connections, message queue depth, delivery latency
- **Portal:** Frame rate (FPS), message reception rate, client-perceived lag

**Why Instrumentation First?**
- 500k entities is unprecedented scale for us
- Entity state size is unknown (need to measure)
- Stream lag tolerance is unknown (need to observe)
- "You can't optimize what you don't measure"

---

## Requirements & Constraints

### Simulation Specifications
- **Tick Rate:** 20 Hz (50ms per tick)
- **Entity Count:** Up to 500,000 agents
- **Entity State:** Position (x, y), orientation (radians), size (radius)
- **Entity State Size:** Unknown yet (add to observability metrics)
- **Philosophy:** Simulation never stops, even if all clients disconnect

### Client Specifications
- **Initial Scale:** Just a few clients (learn over time via instrumentation)
- **Platform:** Desktop only (mobile deferred)
- **Region:** UK (where user is located)
- **Frame Rate:** 60 FPS rendering via client-side interpolation
- **Failure Tolerance:** Accept message loss (treat like UDP, prioritize performance)

### Persistence & Replay
- **Replay:** Not needed (no rewind functionality)
- **NATS Retention:** Real-time only (no JetStream durability)
- **Player Avatar Preservation:** Future concern (not sprint 6)

### Stream Lag Tolerance
- **Unknown yet:** Measure during implementation
- **Strategy:** Instrument client-perceived lag, set thresholds based on observed data

---

## Performance Targets

### Latency Targets (End-to-End)

| Pipeline Stage | Target | Why |
|----------------|--------|-----|
| Simulation → NATS | < 5ms | Minimize impact on simulation tick budget |
| NATS → Broadcaster | < 10ms | Consumer lag should be negligible |
| Broadcaster → Portal | < 20ms | Network + WebSocket framing |
| Portal Render | < 16ms | 60 FPS requires 16.67ms frame budget |
| **Total** | **< 60ms** | Three frames of lag (imperceptible) |

### Throughput Targets

- **Simulation:** 20 ticks/sec
- **Message Volume (Peak):** 20 ticks/sec × 500k entities = 10M msgs/sec
- **Message Volume (Typical):** Likely lower due to viewport culling (future optimization)

### Reliability Targets

- **Message Loss:** Acceptable (UDP-like semantics, next tick overwrites stale data)
- **NATS Uptime:** Single container (no HA needed for walking skeleton)
- **Broadcaster Uptime:** Single container (graceful reconnection on restart)
- **Simulation Uptime:** Never stops (clients may disconnect/reconnect)

---

## Technology Stack Summary

| Component | Technology | Deployment |
|-----------|-----------|------------|
| **Message Broker** | NATS Core (single container) | Docker Compose |
| **Rust Client** | `async-nats` crate | N/A (library) |
| **Broadcaster** | Node.js/TypeScript (single container) | Docker Compose |
| **Message Format** | MessagePack (binary) | N/A (protocol) |
| **Client Protocol** | WebSockets (WSS, port 443) | Docker Compose |
| **Metrics** | Prometheus (single container) | Docker Compose |
| **Dashboards** | Grafana (single container) | Docker Compose |
| **Logging** | Structured JSON to stdout | Docker Compose logs |

---

## Risks & Mitigations

### Risk 1: 500k Entities Overwhelms NATS
**Mitigation:**
- NATS can handle 11M msg/sec; our peak is 10M msg/sec (within limits)
- Monitor NATS throughput metric; alert if approaching 80% capacity
- Future: Viewport culling (only send visible entities to Broadcaster)

### Risk 2: Broadcaster Can't Keep Up (Queue Backlog)
**Mitigation:**
- Implement per-client queue limit (e.g., 100 messages)
- Drop oldest messages if queue full (fire-and-forget philosophy)
- Monitor queue depth metric; scale horizontally if sustained high depth

### Risk 3: Portal Rendering Lags (Low-End Devices)
**Mitigation:**
- Client-side frame pacing (drop frames if render queue exceeds threshold)
- Monitor client-side frame drop rate
- Future: Adaptive quality (reduce entity count on slow clients)

### Risk 4: Unknown Entity State Size Causes Bandwidth Issues
**Mitigation:**
- Instrument message size in Simulation (Prometheus histogram)
- Calculate bandwidth: message_size × 20 Hz × visible_entity_count
- Compress with MessagePack first; reassess if bandwidth exceeds 100 Mbps

---

## Walking Skeleton Philosophy

**Current Sprint Goal:** Prove the end-to-end pipeline works

**Not in Scope:**
- Multi-region deployment
- Kubernetes/cloud infrastructure
- Horizontal scaling (multiple Broadcaster replicas)
- Advanced features (viewport culling, player actions, avatar preservation)

**In Scope:**
- Single NATS container running locally
- Single Broadcaster container
- Prometheus + Grafana observability
- End-to-end latency < 60ms
- 60 FPS rendering in browser
- Comprehensive instrumentation (measure everything)

**Principle:** Start lean, instrument heavily, scale when data justifies it.

---

## Future Scaling (Brief Appendix)

When the walking skeleton proves viable and usage grows, consider:

- **Kubernetes Deployment:** Replace docker-compose with K8s manifests (StatefulSet for NATS, Deployment for Broadcaster)
- **Multi-Region:** NATS federation or geo-distributed clusters for global players
- **Horizontal Scaling:** Multiple Broadcaster replicas with load balancer and session affinity
- **Advanced Features:** Viewport culling, entity LOD, adaptive quality, player action handling
- **Critical Events:** JetStream for guaranteed delivery (agent deaths, player state persistence)

These are deferred until the local stack demonstrates success and metrics justify the complexity.

---

## Sign-Off

**Open Questions:** Answered ✅
**Technology Choices:** Approved ✅
**Focus:** Local docker-compose walking skeleton ✅
**Observability:** Prometheus + Grafana from day one ✅

**Ready for Implementation:** Yes

**Last Updated:** 2025-11-05
