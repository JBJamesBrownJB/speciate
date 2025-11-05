# Broadcaster Service

WebSocket broadcaster service that relays NATS simulation data to browser clients in real-time.

## Overview

The Broadcaster subscribes to simulation data from NATS and broadcasts it to all connected WebSocket clients. This is the **walking skeleton** implementation - a simple pass-through relay with no filtering or optimization.

## Architecture

```
┌─────────────┐
│ Simulation  │ (Rust, 20 Hz)
│  Publisher  │
└──────┬──────┘
       │ publishes to
       │ "speciate.agents.transform"
       ↓
┌─────────────┐
│    NATS     │ (Message Broker)
│   Server    │
└──────┬──────┘
       │ subscribes
       ↓
┌─────────────┐
│ Broadcaster │ (Node.js) ← YOU ARE HERE
│  Subscriber │
└──────┬──────┘
       │ WebSocket
       ↓
┌─────────────┐
│   Portal    │ (Browser)
│   Clients   │
└─────────────┘
```

## Quick Start

### Prerequisites

- Node.js 20+
- NATS server running (see `infrastructure/local/README.md`)

### Installation

```bash
npm install
```

### Development

```bash
# Start in watch mode (auto-reload on changes)
npm run dev
```

### Production

```bash
# Build TypeScript
npm run build

# Start service
npm start
```

## Configuration

Configuration is done via environment variables:

| Variable    | Default                  | Description                    |
|-------------|--------------------------|--------------------------------|
| `NATS_URL`  | `nats://nats:4222`       | NATS server URL (Docker DNS)   |
| `WS_PORT`   | `8080`                   | WebSocket server port          |
| `LOG_LEVEL` | `info`                   | Log level (debug, info, warn, error) |

### Docker Networking

The broadcaster runs inside the **devcontainer** and connects to NATS via **Docker DNS**:

- **From devcontainer:** Use `nats://nats:4222` (Docker DNS resolves to NATS container)
- **From host:** Use `nats://localhost:4222` (if running outside Docker)

The devcontainer is configured to join the `speciate-local` network (see `.devcontainer/docker-compose.yml`), which allows it to reach the NATS container by hostname.

### Example

```bash
# Inside devcontainer (default)
npm run dev

# Override NATS URL
NATS_URL=nats://nats-server:4222 WS_PORT=9000 npm start
```

## Testing

```bash
# Run all tests
npm test

# Run tests in watch mode
npm run test:watch

# Run tests with coverage
npm run test:coverage

# nats cli to sub to msgs
nats sub -s nats://nats:4222 speciate.agents.transform
```

## Manual Verification

### 1. Start NATS

```bash
cd infrastructure/local
docker compose up -d
```

### 2. Start Broadcaster

```bash
cd apps/broadcaster
npm run dev
```

You should see:

```
[INFO] ============================================================
[INFO] Broadcaster Service Starting
[INFO] ============================================================
[INFO] NATS Server: nats://nats:4222
[INFO] NATS Subject: speciate.agents.transform
[INFO] WebSocket Port: 8080
[INFO] WebSocket Path: /stream
[INFO] Connected to NATS successfully!
[INFO] WebSocket server listening on port 8080 (path: /stream)
[INFO] ============================================================
```

### 3. Test WebSocket Connection

In another terminal, use `websocat` to test the connection:

```bash
# Install websocat if needed
# macOS: brew install websocat
# Linux: cargo install websocat

websocat ws://localhost:8080/stream
```

### 4. Start Simulation

In another terminal:

```bash
cd apps/simulation
cargo run
```

### 5. Verify Messages

You should see JSON messages flowing through `websocat` at ~20 Hz:

```json
{"tick":1,"timestamp":"2025-11-05T12:00:00.000Z","agents":[{"id":1,"x":45.23,"y":78.91,"vx":2.15,"vy":-0.87,"rotation":1.57}]}
{"tick":2,"timestamp":"2025-11-05T12:00:00.050Z","agents":[{"id":1,"x":45.33,"y":78.87,"vx":2.15,"vy":-0.87,"rotation":1.57}]}
...
```

## Module Structure

- **`src/config.ts`** - Configuration management (env vars)
- **`src/types.ts`** - TypeScript interfaces for data structures
- **`src/nats-subscriber.ts`** - NATS connection and subscription (EventEmitter)
- **`src/websocket-server.ts`** - WebSocket server and client management
- **`src/broadcaster.ts`** - Coordination between NATS and WebSocket
- **`src/logger.ts`** - Simple logging utility
- **`src/index.ts`** - Main entry point with graceful shutdown

## Message Format

See `SPRINT_DOCS/NATS_CONTRACT.md` for the complete message specification.

### SimulationFrame

```typescript
interface SimulationFrame {
  tick: number;           // Simulation tick counter
  timestamp: string;      // ISO 8601 timestamp (UTC)
  agents: AgentTransform[];
}

interface AgentTransform {
  id: number;       // Stable agent ID
  x: number;        // Position X (world coordinates)
  y: number;        // Position Y (world coordinates)
  vx: number;       // Velocity X
  vy: number;       // Velocity Y
  rotation: number; // Rotation in radians (0 to 2π)
}
```

## Resilience Features

- **Auto-reconnect to NATS** - Automatic reconnection with exponential backoff (built into NATS client)
- **Graceful shutdown** - Handles SIGTERM/SIGINT, closes connections cleanly
- **Error handling** - Continues operation even if individual messages fail
- **Client cleanup** - Removes disconnected WebSocket clients automatically

## Performance

- **Target:** 20 Hz message throughput
- **Latency:** <50ms (typically 5-20ms on localhost)
- **Clients:** Designed to handle dozens to hundreds of concurrent WebSocket connections

## Logging

Logs include:

- Service startup/shutdown
- NATS connection events (connect, disconnect, reconnect)
- WebSocket client connections/disconnections
- Periodic stats (every 100 ticks): tick number, client count, agent count
- Errors and warnings

Example log output:

```
[INFO] Broadcaster Service Ready
[INFO] Listening for simulation data from NATS...
[Broadcaster] NATS connected
[WebSocketServer] Client connected (total: 1)
[Broadcaster] Tick 100, Clients: 1, Agents: 50
[WebSocketServer] Client disconnected (total: 0)
```

## Troubleshooting

### "Error: connect ECONNREFUSED" or "NatsError: CONNECTION_REFUSED"

**Cause:** NATS server is not running, or broadcaster can't reach it.

**Solutions:**

1. **Start NATS server:**
   ```bash
   cd infrastructure/local
   docker compose up -d
   ```

2. **Verify NATS is accessible from devcontainer:**
   ```bash
   curl http://nats:8222/varz
   ```

   If this fails, rebuild the devcontainer:
   - In VS Code: `Cmd+Shift+P` → "Dev Containers: Rebuild Container"

3. **Check NATS URL configuration:**
   - Inside devcontainer: Use `nats://nats:4222` (default)
   - On host machine: Use `nats://localhost:4222`

### "WebSocket connection failed"

Broadcaster service is not running or port 8080 is in use. Check:

```bash
lsof -i :8080
```

### No messages received

Simulation is not running or not publishing. Start it with:

```bash
cd apps/simulation
cargo run
```

## Future Enhancements

This is the **walking skeleton** - intentionally minimal. Future sprints may add:

- **Delta compression** - Only send changed agents
- **Viewport culling** - Send only agents in client's viewport
- **Client interest management** - Subscribe to specific spatial regions
- **Message batching** - Combine multiple frames for efficiency
- **Metrics endpoint** - Prometheus metrics for monitoring
- **Authentication** - Validate WebSocket clients
- **Rate limiting** - Prevent client abuse

## Contributing

This service follows **Test-Driven Development (TDD)**:

1. Write tests first
2. Implement minimum code to pass tests
3. Refactor for clarity

All new features must have >85% test coverage.

## License

MIT
