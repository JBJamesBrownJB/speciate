# MMO Streaming Architecture (Archived)

**Status:** Archived - 2025-11-10
**Reason:** Strategic pivot to Steam Early Access (Phase 1)
**Archive Location:** `archive/mmo-streaming-v1` branch (to be created Sprint 7)
**Git Tag:** `v0.mmo-pivot` (to be created Sprint 7)

---

## What Was Archived

### Context

**Original Vision (Pre-Nov 2025):**
Speciate was designed as a server-authoritative MMO with:
- Browser-based clients (PixiJS WebGL)
- NATS message broker for pub/sub streaming
- Node.js Broadcaster microservice (WebSocket distribution)
- Complex streaming pipeline (quantization, delta encoding, interpolation)
- Player economy (DNA ownership, biomass trading)
- $19k/month infrastructure costs

**New Direction (Nov 2025):**
Strategic pivot to Steam Early Access standalone game:
- Desktop Tauri application (Rust + PixiJS bundled)
- Local IPC (lock-free snapshot queue, no network)
- Zero server costs
- Faster development (6-9 months vs. 12-18 months)
- Lower financial risk
- Validates concept before MMO investment

**See:** [docs/strategy/biz-strategy.md](../../strategy/biz-strategy.md) for full rationale

---

## Archived Components

### Code (To Be Archived in Sprint 7)

**Delete from `main` branch:**
- `apps/broadcaster/` - Node.js WebSocket distribution service
- `simulation/crates/nats_client/` - Rust NATS adapter
- Interpolation logic in `apps/portal/src/` - Client-side prediction/smoothing
- Quantization/delta encoding in simulation - Bandwidth optimization
- `infrastructure/local/docker-compose.yml` - NATS broker setup

**Preserve in Git history:**
- Create branch: `archive/mmo-streaming-v1`
- Tag: `v0.mmo-pivot` with context message
- Document in this file for future reference

### Architecture Documents (Marked "Future Vision")

**Updated with "Phase 2" tags:**
- [docs/architecture/streaming-architecture.md](../streaming-architecture.md) - NATS streaming design
- [docs/project-spec.md](../../project-spec.md) - Economy Ledger section
- NATS infrastructure documentation

**Still valuable because:**
- Analysis of streaming trade-offs (interpolation, delta encoding, backpressure)
- Understanding networked game architecture decisions
- Reference for potential Phase 2 (web MMO) implementation
- Save file compression strategies

---

## What This Code Did

### Broadcaster Microservice (Node.js)

**Purpose:** Distribute simulation state updates to thousands of web clients

**Architecture:**
```
Simulation (Rust, 20 Hz)
    → NATS (MessagePack binary)
    → Broadcaster (TypeScript, subscribe to NATS)
    → WebSocket (20 Hz broadcast to N clients)
    → Portal Clients (PixiJS, 60-90 FPS interpolation)
```

**Key Features:**
- NATS subscription to `simulation.state` subject
- WebSocket server on port 8080
- MessagePack deserialization
- JSON broadcasting to connected clients
- Health check endpoint (port 3001)
- Connection metrics (client count, message rate)

**Performance:**
- Handled 1000+ clients in testing
- 20 Hz broadcast rate (50ms intervals)
- ~50-100KB per frame (1000 creatures, quantized)

**Technology:**
- **Runtime:** Node.js 22.12+
- **WebSocket:** `ws` library
- **NATS Client:** `nats.js`
- **Serialization:** `@msgpack/msgpack`

**Location:** `apps/broadcaster/` (to be archived)

---

### NATS Streaming Pipeline

**Purpose:** High-throughput pub/sub messaging between simulation and broadcaster

**Why NATS?**
- 8-11M msg/sec capacity (handles massive scale)
- Pub/sub decoupling (simulation doesn't care about clients)
- Backpressure handling (clients can lag without blocking simulation)
- Multi-broadcaster support (horizontal scaling)

**Message Flow:**
1. Simulation writes state to `SnapshotPublisher` (Rust)
2. NATS client serializes to MessagePack
3. Publishes to `simulation.state` subject
4. Broadcaster(s) subscribe to subject
5. Deserialize MessagePack → JSON
6. Broadcast to WebSocket clients

**Optimizations:**
- MessagePack binary (30-40% smaller than JSON)
- Quantization (f32 → i16, 0.1 precision) = 98.5% bandwidth reduction
- Delta encoding (only send changed entities)
- LZ4 compression (4+ GB/sec decompression speed)
- Spatial hashing (only stream visible entities to each client)

**Performance Targets (Never Fully Implemented):**
- Raw: 2.7 GB/sec for 1M entities
- Optimized: 4.5 MB/sec (98.5% reduction)
- Latency: 12-15ms simulation → broadcaster
- Simulation impact: <5ms per tick

**Technology:**
- **Message Broker:** NATS 2.x (Docker container)
- **Rust Client:** `async-nats`
- **Node Client:** `nats.js`
- **Serialization:** FlatBuffers or MessagePack (debated, MessagePack chosen)

**Location:** `infrastructure/local/docker-compose.yml`, `simulation/crates/nats_client/`

**Reference:** [docs/architecture/streaming-architecture.md](../streaming-architecture.md)

---

### Client-Side Interpolation (PixiJS)

**Purpose:** Smooth 20 Hz server updates into 60-90 FPS rendering

**Problem:**
- Server sends state at 20 Hz (50ms intervals)
- Display must render at 60-90 FPS (11-16ms intervals)
- Without interpolation, creatures "teleport" every 50ms (jittery)

**Solution: Client-Side Prediction**

```typescript
// Store two states: old (t-1) and new (t)
let old_state = null;
let new_state = null;

// On WebSocket message (20 Hz)
socket.onmessage = (event) => {
    old_state = new_state;
    new_state = JSON.parse(event.data);
};

// On render frame (60-90 FPS)
app.ticker.add((delta) => {
    if (!old_state || !new_state) return;

    // Calculate interpolation factor (0.0 to 1.0)
    const alpha = calculateAlpha(old_state.timestamp, new_state.timestamp, now);

    // Smooth position between states
    sprite.x = lerp(old_state.x, new_state.x, alpha);
    sprite.y = lerp(old_state.y, new_state.y, alpha);
    sprite.rotation = lerpAngle(old_state.rotation, new_state.rotation, alpha);
});
```

**Challenges:**
- Lag compensation (client is always 50-100ms behind server)
- Extrapolation (predict beyond latest state when laggy)
- Entity spawn/despawn (gracefully add/remove sprites)
- Rotation wrapping (-π to π discontinuity)

**Technology:**
- **Lerp functions:** Linear interpolation for position
- **Slerp functions:** Spherical interpolation for rotation (unused, overkill for 2D)
- **Timestamp sync:** Client-server clock drift handling

**Location:** `apps/portal/src/` (to be deleted/simplified in Sprint 7)

---

### Quantization & Delta Encoding

**Purpose:** Reduce bandwidth from gigabytes to megabytes per second

**Quantization:**
Convert high-precision floats to low-precision integers:
```rust
// f32 (-50.7834) → i16 (-508) with 0.1 precision
fn quantize(value: f32, precision: f32) -> i16 {
    (value / precision).round() as i16
}

// i16 (-508) → f32 (-50.8)
fn dequantize(value: i16, precision: f32) -> f32 {
    value as f32 * precision
}
```

**Trade-offs:**
- Smaller payload (4 bytes → 2 bytes per coordinate)
- Precision loss (±0.05m error acceptable for rendering)
- Simpler for local coordinates (i16 range ±3,276m with 0.1 precision)

**Delta Encoding:**
Only send entities that changed since last frame:
```rust
let changed_entities = current_state
    .entities
    .iter()
    .filter(|e| e.hash() != previous_state.get(e.id).hash())
    .collect();
```

**Combined Impact:**
- 1M entities × 32 bytes = 32 MB per frame (raw)
- Quantization: 32 MB → 16 MB (50% reduction)
- Delta encoding: 16 MB → 0.8 MB (95% reduction, assuming 5% change rate)
- LZ4 compression: 0.8 MB → 0.2 MB (75% additional reduction)

**Total:** 32 MB → 0.2 MB = **99.4% bandwidth reduction**

**Location:** `simulation/crates/nats_client/` (never fully implemented, to be deleted)

---

## Why It Was Archived

### Financial Risk

**Original MMO Costs (Annual):**
- Server infrastructure: $180-220k/year (AWS/GCP for 10k+ concurrent players)
- CDN & assets: $24-36k/year
- Database & storage: $12-24k/year
- **Total:** ~$228k/year in recurring costs

**Risk:** If MMO launch fails, $228k/year burns through runway with no revenue.

**New Approach:**
- Steam Early Access: $0/year server costs
- Revenue from day 1 (pay-once model: $20-30)
- Proves concept before infrastructure investment
- **Break-even:** ~2,000 units sold at $25 (vs. $228k annual risk)

---

### Technical Complexity

**MMO Required:**
- NATS infrastructure (deployment, monitoring, scaling)
- Broadcaster horizontal scaling (region-based routing)
- Client-server clock synchronization
- Lag compensation & prediction
- Cheat prevention (server authority)
- Multi-tenant database (player assets, authentication)
- CDN for sprite assets (procedural generation)

**Tauri Requires:**
- Lock-free IPC (snapshot queue)
- Dual-tick scheduling (20 Hz AI, 90 Hz physics)
- Save/load to disk (bincode/MessagePack)
- Steam integration (achievements, cloud saves)

**Complexity Reduction:** 60-70% fewer systems to build/maintain

---

### Time to Market

**MMO Timeline:**
- 12-18 months to launch
- Infrastructure deployment (Terraform, Kubernetes)
- Security hardening (authentication, economy anti-cheat)
- Load testing (1000+ concurrent players)
- $0 revenue until launch (all upfront investment)

**Steam EA Timeline:**
- 6-9 months to launch
- No infrastructure (runs locally)
- No security concerns (single-player)
- Steam handles distribution
- Revenue starts at Early Access launch

**Difference:** 6-9 months faster to market

---

## If Resurrecting for Phase 2

### Prerequisites

**Business Validation:**
- ✅ Phase 1 success: 10,000+ units sold, 80%+ reviews, $200k+ revenue
- ✅ Community demand for multiplayer validated (Discord, Reddit requests)
- ✅ $200k+ funding secured (sales or investment)
- ✅ Team capacity for 12+ month MMO development

**See:** [docs/strategy/biz-strategy.md](../../strategy/biz-strategy.md) for phase gates

---

### Resurrection Steps

**1. Restore Archive Branch**
```bash
# Checkout archived code
git checkout archive/mmo-streaming-v1

# Create new feature branch
git checkout -b feature/phase2-mmo-resurrection main

# Cherry-pick relevant commits from archive
git cherry-pick <commit-hash>
```

**2. Update Dependencies**
- NATS client libraries (likely 2-3 years out of date by Phase 2)
- Bevy ECS version (simulation may be on newer Bevy)
- Node.js ecosystem (@nats.js, ws, @msgpack/msgpack)
- Docker base images

**3. Refactor for Current Bevy**
- Snapshot serialization system (Bevy resource patterns may have changed)
- ECS query syntax (Bevy query API evolves)
- Async runtime integration (tokio + Bevy schedule coordination)

**4. Infrastructure Setup**
```bash
# Deploy NATS cluster (GCP/AWS)
cd infrastructure/terraform
terraform init
terraform apply -var-file=production.tfvars

# Deploy Broadcaster (Kubernetes)
kubectl apply -f k8s/broadcaster-deployment.yaml

# Configure CDN for sprite assets
# TODO: Cloudflare or Google Cloud CDN setup
```

**5. Test Streaming Pipeline**
- Load test: 1000 creatures, 100 clients (validate 20 Hz stability)
- Stress test: 10k creatures, 1000 clients (identify bottlenecks)
- Latency test: Measure simulation → client round-trip (target <50ms)

**6. Implement Missing Optimizations**
- Delta encoding (only partially implemented in archive)
- Spatial hashing (region-based filtering)
- LZ4 compression (if bandwidth still high)

**7. Security Hardening**
- Authentication (JWT tokens, session management)
- Economy anti-cheat (server validates all transactions)
- Rate limiting (prevent DDoS on Broadcaster)
- Input validation (sanitize player commands)

**8. Player Economy Integration**
- Connect Broadcaster to Economy Ledger (REST API)
- Implement DNA ownership (PostgreSQL schema)
- Biomass trading (transaction validation)
- Speciation events (trigger on unique DNA discovery)

**9. Multi-Broadcaster Scaling**
- Region-based NATS subjects (`agents.world.region.{x}.{y}`)
- Load balancer (route clients to nearest Broadcaster)
- Horizontal scaling (add Broadcaster instances per region)

**10. Monitoring & Observability**
- Prometheus metrics (message rate, latency, client count)
- Grafana dashboards (real-time system health)
- Alerting (spike detection, outage notifications)

---

## Key Decisions to Revisit

### Serialization Format

**Archived Decision:** MessagePack (chosen over FlatBuffers)

**Rationale (2025):**
- MessagePack simpler to integrate
- FlatBuffers zero-copy benefit not critical for 20 Hz rate
- JSON debugging easier with MessagePack

**Revisit for Phase 2:**
- FlatBuffers may be better at MMO scale (1M entities, 10k clients)
- Zero-copy deserialization saves CPU on Broadcaster
- Schema evolution (FlatBuffers supports versioned schemas)

**Recommendation:** Benchmark both at Phase 2 scale before deciding

---

### Interpolation Strategy

**Archived Decision:** Linear interpolation (lerp)

**Alternative:** Hermite spline interpolation (smoother curves)

**Revisit for Phase 2:**
- Players may notice 20 Hz jitter at high zoom levels
- Hermite interpolation uses velocity for smoother prediction
- Trade-off: More CPU cost on client (acceptable for web?)

**Recommendation:** Test both in Phase 2 beta, gather player feedback

---

### NATS vs. Redis Streams

**Archived Decision:** NATS (chosen over Redis)

**Rationale (2025):**
- NATS designed for pub/sub (8-11M msg/sec)
- Redis Streams better for persistence (replay old messages)
- NATS lightweight, Redis heavy (memory footprint)

**Revisit for Phase 2:**
- Redis Streams may have improved performance by 2026-2027
- Persistence useful for debugging (replay simulation states)
- Cost comparison (managed NATS vs. managed Redis)

**Recommendation:** Re-evaluate in Phase 2 planning

---

## Documentation References

### Archived Architecture Docs

**Still Relevant:**
- [docs/architecture/streaming-architecture.md](../streaming-architecture.md) - Streaming pipeline design
- [docs/project-spec.md](../../project-spec.md) - Phase 2 sections (Economy Ledger, Microservices)

**Obsolete (Delete in Sprint 7):**
- `apps/broadcaster/README.md`
- `infrastructure/local/README.md` (NATS setup)

**Update in Sprint 7:**
- [README.md](../../../README.md) - Remove NATS quick start, add Tauri instructions
- [docs/project-spec.md](../../project-spec.md) - Mark Phase 2 sections clearly

---

### Lessons Learned

**What Worked:**
- NATS integration smooth (async-nats crate excellent)
- Broadcaster simple and reliable (Node.js + ws library stable)
- MessagePack serialization fast enough (no bottleneck)
- Interpolation on client smooth at 60 FPS

**What Didn't Work:**
- Quantization caused precision issues (±0.1m error visible at high zoom)
- Delta encoding complex (hash comparison expensive)
- Network lag unpredictable (client-side extrapolation needed)

**Takeaways for Phase 2:**
- Prioritize simplicity over premature optimization
- Measure actual bottlenecks before optimizing
- Player experience > theoretical bandwidth savings

---

## Conclusion

**The MMO architecture was sound but premature.**

By deferring to Phase 2, we:
- Eliminate $228k/year financial risk
- Validate A-Life concept in 6-9 months (vs. 12-18)
- Build community before infrastructure investment
- Use Steam EA revenue to fund Phase 2 properly

**If Phase 1 succeeds, we resurrect this architecture with:**
- Proven gameplay (players love it)
- Funding (Early Access sales)
- Community (built-in player base for MMO beta)

**If Phase 1 fails, we avoided $228k/year commitment on unproven concept.**

**Risk management: Maximize learning, minimize burn rate.**

---

**Archive Date:** 2025-11-10 (to be executed Sprint 7)
**Next Review:** 2026 Q3 (after Phase 1.5 launch, evaluate Phase 2 go/no-go)
**Owner:** pm-pam (project manager)
**Architect:** architect-andy
