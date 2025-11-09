# Local Stack Setup: Docker Compose Walking Skeleton

**Sprint:** Sprint 6 - Streaming Pipeline
**Date:** 2025-11-05
**Focus:** Local development with production-grade observability

---

## Architecture Overview

```
┌─────────────────────────────────────────────────────────┐
│ Local Environment (Docker Compose)                      │
├─────────────────────────────────────────────────────────┤
│                                                           │
│  ┌─────────────────┐                                     │
│  │ Simulation      │  (runs on host, not containerized)  │
│  │ (Rust)          │  Publishes at 20Hz                  │
│  └────────┬────────┘                                     │
│           │                                               │
│           │ NATS protocol (publishes messages)           │
│           ▼                                               │
│  ┌─────────────────┐                                     │
│  │ NATS            │  Port 4222 (client)                 │
│  │ (single node)   │  Port 8222 (monitoring)             │
│  └────────┬────────┘                                     │
│           │                                               │
│           │ NATS subscription (consumes messages)        │
│           ▼                                               │
│  ┌─────────────────┐                                     │
│  │ Broadcaster     │  Port 8080 (WebSocket)              │
│  │ (Node.js/TS)    │  Port 9090 (Prometheus metrics)     │
│  └────────┬────────┘                                     │
│           │                                               │
│           │ WebSocket (WSS)                               │
│           │                                               │
└───────────┼─────────────────────────────────────────────┘
            │
            ▼
   ┌─────────────────┐
   │ Portal (Browser)│  Connects to ws://localhost:8080
   │ - Pixi.js       │  Renders at 60 FPS
   │ - Interpolation │
   └─────────────────┘

┌─────────────────────────────────────────────────────────┐
│ Observability Stack (Docker Compose)                    │
├─────────────────────────────────────────────────────────┤
│                                                           │
│  ┌─────────────────┐                                     │
│  │ Prometheus      │  Port 9091 (metrics scraping)       │
│  │ - Scrapes NATS  │  Scrapes: NATS, Broadcaster, Sim    │
│  │ - Scrapes BC    │                                     │
│  └────────┬────────┘                                     │
│           │                                               │
│  ┌────────▼────────┐                                     │
│  │ Grafana         │  Port 3000 (dashboards)             │
│  │ - Pre-configured│  Default: admin/admin               │
│  │ - Dashboards    │                                     │
│  └─────────────────┘                                     │
│                                                           │
└───────────────────────────────────────────────────────────┘
```

---

## Docker Compose Services

### 1. NATS (Message Broker)

**Image:** `nats:latest`
**Ports:**
- `4222`: Client connections (Simulation, Broadcaster)
- `8222`: Monitoring HTTP endpoint

**Configuration:**
- Single node (no clustering for walking skeleton)
- Fire-and-forget semantics (NATS Core, not JetStream)
- No persistence (messages are ephemeral)

**Health Check:**
- HTTP GET `http://localhost:8222/healthz`
- Returns 200 if healthy

**Monitoring:**
- Prometheus scrapes `http://localhost:8222/varz` via NATS exporter (or native endpoint)

---

### 2. Broadcaster (WebSocket Relay)

**Runtime:** Node.js 18+ / TypeScript
**Ports:**
- `8080`: WebSocket endpoint (Portal connects here)
- `9090`: Prometheus metrics endpoint

**Responsibilities:**
- Subscribe to NATS: `speciate.agents.*.*`
- Maintain WebSocket connections to Portal clients
- Forward NATS messages to connected clients (MessagePack encoding)
- Expose metrics: active connections, queue depth, delivery latency

**Configuration:**
- NATS URL: `nats://nats:4222` (docker-compose service name resolution)
- Single replica (no horizontal scaling yet)

**Health Check:**
- HTTP GET `http://localhost:8080/health`
- Returns 200 if: NATS connected, WebSocket listener active

**Graceful Shutdown:**
- On SIGTERM: Stop accepting new connections
- Allow existing connections to drain (30s timeout)
- Close NATS subscription cleanly

---

### 3. Prometheus (Metrics Collection)

**Image:** `prom/prometheus:latest`
**Ports:**
- `9091`: Prometheus UI and scraping endpoint

**Scrape Targets:**
- NATS: `http://nats:8222/varz` (every 5s)
- Broadcaster: `http://broadcaster:9090/metrics` (every 5s)
- Simulation: `http://host.docker.internal:9092/metrics` (every 5s)

**Retention:** 15 days (sufficient for local dev)

**Configuration File:** `prometheus.yml`
```yaml
global:
  scrape_interval: 5s
  evaluation_interval: 5s

scrape_configs:
  - job_name: 'nats'
    static_configs:
      - targets: ['nats:8222']

  - job_name: 'broadcaster'
    static_configs:
      - targets: ['broadcaster:9090']

  - job_name: 'simulation'
    static_configs:
      - targets: ['host.docker.internal:9092']
```

---

### 4. Grafana (Dashboards)

**Image:** `grafana/grafana:latest`
**Ports:**
- `3000`: Grafana UI (http://localhost:3000)

**Default Credentials:**
- Username: `admin`
- Password: `admin` (change on first login)

**Data Source:** Prometheus at `http://prometheus:9091`

**Pre-Configured Dashboards:**
- **Simulation Metrics:** Tick rate, message publish rate, message size, NATS publish latency
- **NATS Metrics:** Throughput, connection count, memory usage
- **Broadcaster Metrics:** Active connections, queue depth, delivery latency
- **End-to-End Latency:** Trace tick creation → Portal render (requires OpenTelemetry in future)

**Dashboard Provisioning:**
- Dashboards defined in `grafana/provisioning/dashboards/` (auto-loaded on startup)
- Data source defined in `grafana/provisioning/datasources/` (auto-configured)

---

## Simulation Integration

**Runs on Host (Not Containerized):**
- Rust simulation runs natively on host machine (better performance, easier debugging)
- Connects to NATS at `localhost:4222`

**NATS Publishing:**
```rust
use async_nats;

let client = async_nats::connect("localhost:4222").await?;

// Publish agent transform at 20Hz
loop {
    for agent in agents.iter() {
        let subject = format!("speciate.agents.{}.transform", agent.id);
        let payload = serialize_transform(agent); // MessagePack
        client.publish(subject, payload.into()).await?;
    }
    tokio::time::sleep(Duration::from_millis(50)).await; // 20Hz
}
```

**Prometheus Metrics Endpoint:**
- Expose HTTP server on port 9092: `/metrics`
- Metrics to expose:
  - `simulation_tick_rate_hz`: Current tick rate (gauge)
  - `simulation_message_publish_total`: Total messages published (counter)
  - `simulation_message_size_bytes`: Message size histogram
  - `simulation_nats_publish_duration_seconds`: Publish latency histogram

**Rust Crate:** Use `prometheus` crate for metrics
```rust
use prometheus::{Encoder, Gauge, Histogram, Registry};
```

---

## Portal Integration

**Runs in Browser (Not Containerized):**
- HTML/JavaScript app served by Broadcaster (or separate static server)
- Connects to WebSocket at `ws://localhost:8080`

**WebSocket Client:**
```javascript
const ws = new WebSocket('ws://localhost:8080');

ws.onmessage = (event) => {
  const message = msgpack.decode(event.data); // MessagePack decode
  updateAgent(message.agent_id, message.position, message.orientation, message.size);
};

ws.onerror = (error) => {
  console.error('WebSocket error:', error);
  // Exponential backoff reconnection logic
};
```

**Client-Side Interpolation:**
```javascript
// Receive updates at 20Hz (50ms intervals)
// Render at 60Hz (16.67ms intervals)

let lastUpdate = null;
let currentUpdate = null;

ws.onmessage = (event) => {
  lastUpdate = currentUpdate;
  currentUpdate = msgpack.decode(event.data);
};

function render(timestamp) {
  if (!lastUpdate || !currentUpdate) return;

  // Interpolate between lastUpdate and currentUpdate
  const alpha = (timestamp - currentUpdate.timestamp) / 50; // 50ms = 20Hz
  const interpolatedPosition = lerp(lastUpdate.position, currentUpdate.position, alpha);

  drawAgent(currentUpdate.agent_id, interpolatedPosition);

  requestAnimationFrame(render);
}

requestAnimationFrame(render);
```

---

## Key Metrics to Instrument

### Simulation Metrics (Prometheus)

| Metric | Type | Description |
|--------|------|-------------|
| `simulation_tick_rate_hz` | Gauge | Current simulation tick rate (should be ~20) |
| `simulation_entity_count` | Gauge | Current number of active entities |
| `simulation_message_publish_total` | Counter | Total messages published to NATS |
| `simulation_message_size_bytes` | Histogram | Distribution of message sizes |
| `simulation_nats_publish_duration_seconds` | Histogram | Time to publish to NATS (should be <5ms) |

### NATS Metrics (Native `/varz` Endpoint)

| Metric | Description |
|--------|-------------|
| `nats_server_mem_bytes` | NATS server memory usage |
| `nats_server_connections` | Current number of client connections |
| `nats_server_in_msgs_total` | Total inbound messages |
| `nats_server_out_msgs_total` | Total outbound messages |
| `nats_server_in_bytes_total` | Total inbound bytes |

### Broadcaster Metrics (Prometheus `/metrics`)

| Metric | Type | Description |
|--------|------|-------------|
| `broadcaster_websocket_connections` | Gauge | Current active WebSocket connections |
| `broadcaster_message_queue_depth` | Histogram | Per-client message queue depth |
| `broadcaster_message_delivery_total` | Counter | Total messages sent to clients |
| `broadcaster_message_delivery_duration_seconds` | Histogram | Time from NATS receive → WebSocket send |
| `broadcaster_reconnections_total` | Counter | Client reconnection count |

### Portal Metrics (Client-Side)

| Metric | Collection Method | Description |
|--------|-------------------|-------------|
| `portal_frame_rate_fps` | `requestAnimationFrame` delta | Current rendering frame rate |
| `portal_message_reception_rate_hz` | WebSocket message count | Messages received per second |
| `portal_perceived_lag_ms` | Server tick timestamp - client time | Client-perceived latency |
| `portal_frame_drops_total` | Missed `requestAnimationFrame` calls | Dropped frames (slow rendering) |

---

## Getting Started

### 1. Start Docker Compose Stack

```bash
cd infrastructure/local
docker-compose up -d
```

**Expected Output:**
```
Creating network "local_default" with default driver
Creating local_nats_1        ... done
Creating local_broadcaster_1 ... done
Creating local_prometheus_1  ... done
Creating local_grafana_1     ... done
```

**Verify Services:**
```bash
docker-compose ps
```

All services should show "Up" status.

### 2. Verify NATS is Running

```bash
curl http://localhost:8222/varz
```

**Expected:** JSON response with NATS server stats.

### 3. Start Simulation

```bash
cd backend/headless
cargo run --release
```

**Expected Console Output:**
```
[INFO] Simulation started at 20Hz
[INFO] Connected to NATS at localhost:4222
[INFO] Publishing to speciate.agents.*.transform
[INFO] Prometheus metrics at http://localhost:9092/metrics
```

### 4. Verify Simulation Metrics

```bash
curl http://localhost:9092/metrics | grep simulation
```

**Expected:**
```
simulation_tick_rate_hz 20.0
simulation_entity_count 500000
simulation_message_publish_total 10000000
```

### 5. Start Broadcaster

```bash
cd services/broadcaster
npm install
npm run dev
```

**Expected Console Output:**
```
[INFO] Broadcaster started on port 8080
[INFO] Connected to NATS at nats://nats:4222
[INFO] Subscribed to speciate.agents.*.*
[INFO] Prometheus metrics at http://localhost:9090/metrics
```

### 6. Open Portal in Browser

```
http://localhost:8080/portal
```

**Expected:**
- WebSocket connects to `ws://localhost:8080`
- Canvas renders agents at 60 FPS
- No visible stutter or lag

### 7. Open Grafana Dashboards

```
http://localhost:3000
```

**Login:** admin / admin

**Navigate to:** Dashboards → Speciate Streaming Pipeline

**Expected Panels:**
- Simulation tick rate (should be ~20 Hz)
- NATS throughput (should be ~10M msgs/sec)
- Broadcaster active connections (should be 1-3)
- End-to-end latency (should be <60ms)

---

## Performance Validation

### Expected Metrics (Walking Skeleton)

| Metric | Target | How to Verify |
|--------|--------|---------------|
| Simulation tick rate | 20 Hz | Grafana: `simulation_tick_rate_hz` panel |
| NATS publish latency | < 5ms | Grafana: `simulation_nats_publish_duration_seconds` P99 |
| Broadcaster queue depth | < 10 msgs | Grafana: `broadcaster_message_queue_depth` P95 |
| Portal frame rate | 60 FPS | Browser DevTools Performance tab |
| End-to-end latency | < 60ms | OpenTelemetry trace (future) or manual calculation |

### Troubleshooting Common Issues

**Simulation can't connect to NATS:**
- Verify NATS is running: `docker-compose ps`
- Check port 4222 is exposed: `docker-compose logs nats`

**Broadcaster queue depth growing:**
- Check NATS throughput: `curl http://localhost:8222/varz | jq .in_msgs`
- Check Broadcaster metrics: `curl http://localhost:9090/metrics | grep queue_depth`
- Possible cause: Broadcaster overwhelmed; consider reducing entity count or increasing tick interval

**Portal rendering stutters:**
- Check frame drop rate: Browser DevTools Console (`portal_frame_drops_total`)
- Check message reception rate: Should be ~20 msg/sec per visible entity
- Possible cause: Too many entities visible; implement viewport culling

**NATS memory growing unbounded:**
- Check NATS memory: `curl http://localhost:8222/varz | jq .mem`
- Possible cause: Slow consumer (Broadcaster not consuming fast enough)
- Solution: NATS will drop slow subscribers; Broadcaster should reconnect

---

## Future Enhancements (Brief Appendix)

When the walking skeleton proves viable and metrics justify scaling:

**Production Deployment:**
- Replace `docker-compose.yml` with Kubernetes manifests
- Deploy NATS as StatefulSet (3 replicas for HA)
- Deploy Broadcaster as Deployment (HPA-scaled based on connection count)
- Use Google Cloud Load Balancer for external traffic (TLS termination)

**Advanced Features:**
- **Viewport Culling:** Broadcaster only sends visible entities to each client
- **Entity LOD:** Reduce update frequency for distant entities
- **Adaptive Quality:** Lower entity count on slow clients
- **JetStream:** Critical events (agent deaths, player actions) use guaranteed delivery
- **Player Actions:** Bidirectional WebSocket for player control inputs

**Multi-Region:**
- NATS federation or geo-distributed clusters
- Broadcaster replicas in multiple regions (anycast routing)
- Reduced client latency for global players

These are deferred until local stack demonstrates success and usage patterns emerge.

---

## Summary

**Current State:** Walking skeleton with single NATS, single Broadcaster, Prometheus/Grafana observability

**Goal:** Prove end-to-end pipeline works at 20Hz, 500k entities, <60ms latency, 60 FPS rendering

**Philosophy:** Start simple, instrument everything, scale when data justifies it

**Next Sprint Tasks:**
1. Implement Simulation NATS publishing (2-3 days)
2. Implement Broadcaster (NATS → WebSocket relay) (2-3 days)
3. Implement Portal (WebSocket client + Pixi.js rendering) (3-4 days)
4. Set up Prometheus + Grafana dashboards (1-2 days)
5. Validate end-to-end performance (1 day)

**Ready for Implementation:** Yes ✅

**Last Updated:** 2025-11-05
