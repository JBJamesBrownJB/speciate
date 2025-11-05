# Local Infrastructure Stack

**Sprint:** Sprint 6 - Streaming Pipeline
**Purpose:** NATS message broker for real-time simulation streaming

**Note:** Observability (Prometheus/Grafana) deferred to future sprint. Focus is on proving the streaming pipeline first.

---

## Overview

This Docker Compose stack provides the **NATS message broker** that enables real-time streaming from the Rust simulation to browser clients via the Broadcaster service.

### Architecture Flow
```
Simulation (Rust) → NATS (port 4222) → Broadcaster (Node.js) → Portal (WebSocket)
      20 Hz              Message Broker          port 8080           Browser
```

---

## Services

### NATS (Message Broker)
- **Port 4222:** Client connections (Simulation, Broadcaster)
- **Port 8222:** Monitoring HTTP endpoint
- **Subject:** `speciate.agents.transform` (simulation data)
- **Rate:** 20 Hz (50ms per frame)
- **Format:** JSON messages with agent positions, velocities, rotations

**Monitoring Endpoints:**
- Health: `http://localhost:8222/healthz`
- Stats: `http://localhost:8222/varz` (JSON)
- Connections: `http://localhost:8222/connz` (JSON)

---

## Quick Start

### 1. Start NATS (Infrastructure)

```bash
cd infrastructure/local
docker compose up -d
```

**Verify NATS is Running:**
```bash
docker compose ps
# Should show: speciate-nats   Up

curl http://localhost:8222/varz
# Should return JSON with server stats
```

### 2. Start the Broadcaster (WebSocket Service)

**From devcontainer:**
```bash
cd /workspace/apps/broadcaster
npm install  # First time only
npm run dev
```

**Expected output:**
```
[INFO] Broadcaster Service Starting
[INFO] NATS Server: nats://nats:4222
[INFO] Connected to NATS successfully!
[INFO] WebSocket server listening on port 8080 (path: /stream)
```

**Note:** The broadcaster runs inside the devcontainer and connects to NATS via Docker DNS (`nats:4222`). The devcontainer must be configured to join the `speciate-local` network (already set up in `.devcontainer/docker-compose.yml`).

### 3. Start the Simulation (Data Publisher)

**New terminal:**
```bash
cd /workspace/apps/simulation
cargo run
```

**Expected output:**
```
[INFO] Simulation starting...
[INFO] Connected to NATS at nats://localhost:4222
[INFO] Spawned 5 agents
[Tick 0] Update: 2.50ms | Publish: 0.15ms
[Tick 1] Update: 2.45ms | Publish: 0.18ms
...
```

The simulation publishes agent data to NATS at 20 Hz (every 50ms).

### 4. Test the WebSocket Stream

**New terminal (use `websocat` or browser console):**
```bash
# Install websocat if needed: brew install websocat
websocat ws://localhost:8080/stream
```

**Expected output:** JSON frames streaming at ~20 Hz:
```json
{"tick":1,"timestamp":"2025-11-05T14:00:00.050Z","agents":[{"id":1,"x":45.2,"y":78.9,"vx":2.1,"vy":-0.8,"rotation":1.57}]}
{"tick":2,"timestamp":"2025-11-05T14:00:00.100Z","agents":[{"id":1,"x":45.3,"y":78.85,"vx":2.1,"vy":-0.8,"rotation":1.57}]}
...
```

---

## Complete Setup (All Steps)

### Prerequisites
- Docker and Docker Compose installed
- VS Code with Dev Containers extension (for devcontainer)
- Rust toolchain (for simulation)
- Node.js 20+ (for broadcaster)

### Step-by-Step Setup

1. **Start NATS infrastructure:**
   ```bash
   cd infrastructure/local
   docker compose up -d
   ```

2. **Verify NATS is accessible:**
   ```bash
   # From host
   curl http://localhost:8222/varz

   # From devcontainer (after rebuild)
   curl http://nats:8222/varz
   ```

3. **Rebuild devcontainer** (if first time or after network config changes):
   - In VS Code: `Cmd+Shift+P` → "Dev Containers: Rebuild Container"
   - Wait for rebuild to complete

4. **Start Broadcaster** (Terminal 1 in devcontainer):
   ```bash
   cd /workspace/apps/broadcaster
   npm install
   npm run dev
   ```

5. **Start Simulation** (Terminal 2 in devcontainer):
   ```bash
   cd /workspace/apps/simulation
   cargo run
   ```

6. **Test WebSocket** (Terminal 3 - from host or devcontainer):
   ```bash
   websocat ws://localhost:8080/stream
   ```

You should see JSON frames streaming at ~20 Hz with agent positions and velocities.

---

## Configuration

### Environment Variables

**Broadcaster** (`apps/broadcaster`):
```bash
NATS_URL=nats://nats:4222    # NATS server URL (default: nats:4222 in Docker)
WS_PORT=8080                 # WebSocket server port (default: 8080)
LOG_LEVEL=info               # Logging level (default: info)
```

**Simulation** (`apps/simulation`):
```bash
NATS_URL=nats://localhost:4222  # NATS server URL (default: localhost:4222)
```

### Docker Networks

The system uses two Docker networks:

1. **`speciate-local`** - NATS container network (created by `infrastructure/local/docker-compose.yml`)
2. **`speciate_devcontainer_default`** - Devcontainer network (joins `speciate-local` via `.devcontainer/docker-compose.yml`)

This allows the broadcaster (running in devcontainer) to reach NATS using the hostname `nats` (Docker DNS resolution).

---

## Viewing Logs

```bash
# View NATS logs
docker compose logs -f nats

# View all logs
docker compose logs -f
```

---

## Stopping the Stack

```bash
# Stop and remove containers
docker compose down

# Full reset (removes network too)
docker compose down -v

# Keep containers running in background
docker compose up -d
```

---

## Troubleshooting

### NATS won't start
```bash
# Check logs
docker compose logs nats

# Common issues:
# - Port 4222 already in use: lsof -i :4222
# - Port 8222 already in use: lsof -i :8222
# - Docker daemon not running: sudo systemctl start docker
```

### Can't connect to NATS from host
```bash
# Verify NATS is healthy
docker compose ps

# Check if ports are accessible from host
curl http://localhost:8222/varz

# Test NATS client port
telnet localhost 4222
```

### Can't connect to NATS from devcontainer
```bash
# From devcontainer, test Docker DNS resolution
curl http://nats:8222/varz

# If this fails, rebuild devcontainer:
# VS Code: Cmd+Shift+P → "Dev Containers: Rebuild Container"
```

### Broadcaster can't connect to NATS
```bash
# Error: CONNECTION_REFUSED at 127.0.0.1:4222
# Solution: Use nats:4222 instead (Docker DNS)

# Set environment variable:
export NATS_URL=nats://nats:4222

# Or update apps/broadcaster/src/config.ts default value
```

### WebSocket connection refused (port 8080)
```bash
# Verify broadcaster is running
cd /workspace/apps/broadcaster
npm run dev

# Check if port is listening
netstat -tuln | grep 8080

# Check if devcontainer port is forwarded
# VS Code should auto-forward port 8080
```

---

## Architecture Diagram

```
┌─────────────────────┐
│   Simulation        │ (Rust, 20 Hz)
│  apps/simulation    │ publishes: speciate.agents.transform
└─────────┬───────────┘
          │ NATS Client (localhost:4222)
          ↓
┌─────────────────────┐
│   NATS Broker       │ (Docker container)
│  speciate-nats      │ port 4222 (client), 8222 (monitoring)
│  Network:           │
│  speciate-local     │
└─────────┬───────────┘
          │ NATS Subscriber (nats:4222)
          ↓
┌─────────────────────┐
│   Broadcaster       │ (Node.js, TypeScript)
│  apps/broadcaster   │ WebSocket server on :8080/stream
│  (in devcontainer)  │
└─────────┬───────────┘
          │ WebSocket
          ↓
┌─────────────────────┐
│   Portal / Client   │ (Browser, 60 FPS)
│  (Future)           │ ws://localhost:8080/stream
└─────────────────────┘
```

---

## Data Flow Example

```
1. Simulation (Tick 42):
   Position: (45.2, 78.9), Velocity: (2.1, -0.8)
   ↓ Publishes to NATS

2. NATS receives message on subject: speciate.agents.transform
   {
     "tick": 42,
     "timestamp": "2025-11-05T14:00:02.100Z",
     "agents": [
       {"id": 1, "x": 45.2, "y": 78.9, "vx": 2.1, "vy": -0.8, "rotation": 1.57}
     ]
   }
   ↓ Broadcasts to subscribers

3. Broadcaster receives message, relays to WebSocket clients
   ↓ WebSocket send

4. Portal (browser) receives frame, interpolates to 60 FPS
```

---

## Useful Commands

```bash
# Restart NATS
docker compose restart nats

# View resource usage
docker stats speciate-nats

# Execute command in NATS container
docker compose exec nats sh

# Clean up everything
docker compose down -v
docker system prune -f

# View NATS subscribers
curl http://localhost:8222/subsz | jq

# View NATS connections
curl http://localhost:8222/connz | jq

# Monitor message rate (requires NATS CLI in container)
docker exec -it speciate-nats nats sub "speciate.agents.transform" | pv -l -i 1
```

---

## Message Contract

See **[SPRINT_DOCS/NATS_CONTRACT.md](../../SPRINT_DOCS/NATS_CONTRACT.md)** for complete message format specification.

**Quick Reference:**
- **Subject:** `speciate.agents.transform`
- **Rate:** 20 Hz (every 50ms)
- **Format:** JSON
- **Fields:** tick, timestamp, agents[] (id, x, y, vx, vy, rotation)

---

## Performance Targets

- **Simulation → NATS:** <1ms publish time (non-blocking async)
- **NATS → Broadcaster:** <5ms latency on localhost
- **Broadcaster → Portal:** <10ms WebSocket relay
- **Total Latency:** <20ms end-to-end (simulation to browser)
- **Agent Count:** Target 1,000-10,000 agents at 20 Hz

---

## Next Steps

### Current Status (Sprint 6)
- ✅ NATS infrastructure running
- ✅ Simulation publishing at 20 Hz
- ✅ Broadcaster relaying to WebSocket
- ⏸️ Portal UI (next sprint)

### Future Enhancements (Sprint 7+)
1. **Observability:** Add Prometheus + Grafana for monitoring
2. **Optimization:** MessagePack serialization (30-40% smaller payloads)
3. **Filtering:** Viewport-based spatial culling
4. **Persistence:** JetStream for message persistence and replay
