# Port Allocation Plan

This document defines the port allocation strategy for all services in the Speciate ecosystem to prevent conflicts and ensure clear service communication.

## Port Allocation Table

| Service | Port(s) | Protocol | Purpose | Environment |
|---------|---------|----------|---------|-------------|
| **NATS** | 4222 | TCP | Client connections (Simulation, Broadcaster) | Docker |
| **NATS** | 9224 | WebSocket | Browser clients (Admin Dev UI) | Docker |
| **NATS** | 8222 | HTTP | Monitoring/metrics endpoint | Docker |
| **Portal** | 3000 | HTTP | Vite dev server (frontend UI) | Devcontainer |
| **Ledger** | 3001 | HTTP | REST API (economy service) | Devcontainer |
| **Admin Dev UI** | 8000 | HTTP | Dev control panel (local only) | Host/Devcontainer |
| **Broadcaster** | 8080 | WebSocket | Creature stream to Portal | Devcontainer |
| **Simulation** | N/A | N/A | Publishes to NATS only | Host/Devcontainer |

## Communication Paths

### 1. Simulation → NATS
- **Connection**: TCP to `localhost:4222`
- **Direction**: Publish only
- **Data**: Creature updates, world state, lifecycle events
- **Format**: NATS messages on subjects like `creatures.spawned`, `creatures.moved`

### 2. Broadcaster → NATS
- **Connection**: TCP to `nats:4222` (Docker DNS name)
- **Direction**: Subscribe only
- **Data**: Listens for creature updates
- **Format**: NATS subscription to `creatures.>`

### 3. Broadcaster → Portal
- **Connection**: WebSocket on `ws://localhost:8080`
- **Direction**: Server push (Broadcaster → Portal)
- **Data**: Filtered/transformed creature updates for rendering
- **Format**: JSON-encoded WebSocket frames

### 4. Portal → Ledger (Future)
- **Connection**: HTTP to `http://localhost:3001`
- **Direction**: REST API calls
- **Data**: Player actions, resource queries, economy transactions
- **Format**: JSON REST API

### 5. Admin Dev UI → NATS
- **Connection**: WebSocket to `ws://localhost:9224`
- **Direction**: Bidirectional (pub/sub)
- **Data**: Dev commands (spawn creatures, set speed), diagnostics
- **Format**: NATS WebSocket protocol
- **Note**: Development only - never exposed in production

## Network Configuration

### Docker Network: `speciate-local`
- **Driver**: Bridge
- **Purpose**: Allows devcontainer services to reach NATS via DNS name `nats`
- **Members**:
  - NATS container (automatically via docker-compose)
  - Devcontainer services (configured in `.devcontainer/docker-compose.yml`)

### Container DNS Resolution
Services running inside the devcontainer can reach NATS using the hostname `nats` because:
1. Devcontainer joins the `speciate-local` network
2. Docker's internal DNS resolves `nats` to the NATS container IP
3. No need for `localhost` or hardcoded IPs

### Host → Container Access
Services running on the host (or from the devcontainer to Docker services) use:
- `localhost:4222` for NATS TCP
- `localhost:9224` for NATS WebSocket
- `localhost:8222` for NATS monitoring

## Startup Sequence

The recommended startup order ensures services can connect to their dependencies:

```bash
# 1. Start infrastructure (NATS)
cd infrastructure/local
docker compose up -d

# Wait for NATS to be ready (check http://localhost:8222/varz)

# 2. Start backend services (can be parallel)
cd apps/broadcaster
npm run dev        # Connects to nats:4222

cd apps/simulation
cargo run --features dev-commands   # Connects to localhost:4222

# 3. Start frontend
cd apps/portal
npm run dev        # Serves on port 3000, connects to ws://localhost:8080

# 4. (Optional) Start admin dev UI
cd apps/admin-dev-ui
python3 -m http.server 8000   # Serves on port 8000, connects to ws://localhost:9224
```

## Port Conflicts (Historical)

### Resolved Conflicts
- **Ledger vs Broadcaster**: Early documentation referenced Ledger on port 8080, which conflicted with Broadcaster. **Resolution**: Ledger moved to 3001.
- **NATS WebSocket Port Conflict (4224 → 9224)**: During Sprint 6 setup, port 4224 caused persistent "address already in use" errors in Docker despite no visible process using it. Investigation revealed Docker daemon caching/phantom port reservation. **Resolution**: Changed WebSocket port to 9224 in both `nats-server.conf` and `docker-compose.yml`. Admin portal (`nats-client.js`) updated accordingly.

### Current Status
✅ No active port conflicts as of Sprint 6 Phase 3.

## Future Port Allocations

When adding new services, follow these guidelines:

### Observability Stack (Deferred to Later Sprint)
- **Prometheus**: 9091 (metrics scraping)
- **Grafana**: 3002 (changed from 3001 to avoid Ledger conflict)
- **Simulation Metrics**: 9092 (Prometheus endpoint in Rust simulation)

### Production Deployment
- All ports above are for **local development only**
- Production will use:
  - Cloud Run services (no exposed ports)
  - Cloud Load Balancer (HTTPS on 443)
  - Internal VPC networking for service-to-service communication

## Configuration Files

### NATS Configuration
**File**: `infrastructure/local/nats-server.conf`

```conf
server_name: speciate-nats
port: 4222           # TCP client connections
http_port: 8222      # HTTP monitoring

websocket {
  port: 9224         # WebSocket for browsers
  no_tls: true       # Local dev only (NEVER in production)
  compression: true
}
```

**Note**: The config file must be mounted using an **absolute host path** in `docker-compose.yml` when using Docker-outside-of-Docker (DooD):
```yaml
volumes:
  - /home/dev/dev/speciate/infrastructure/local/nats-server.conf:/etc/nats/nats-server.conf:ro
```
Relative paths (`./nats-server.conf`) will mount as directories instead of files.

### Portal Vite Configuration
**File**: `apps/portal/vite.config.ts`

```typescript
server: {
  port: 3000,
  strictPort: true,   // Fail if port 3000 is busy (don't auto-increment)
  host: '0.0.0.0',    // Expose on all interfaces
}
```

### Broadcaster Configuration
**File**: `apps/broadcaster/src/config.ts` (to be created)

```typescript
export const config = {
  wsPort: parseInt(process.env.WS_PORT || '8080'),
  natsUrl: process.env.NATS_URL || 'nats://nats:4222',
  healthPort: parseInt(process.env.HEALTH_PORT || '9090'),
};
```

## Troubleshooting

### "Port already in use" errors
```bash
# Check what's using a port
lsof -i :3000
sudo lsof -i :8080

# Find and kill the process (SAFELY - don't use killall!)
kill <PID>
```

### Cannot connect to NATS from devcontainer
- Verify devcontainer is on `speciate-local` network: `docker network inspect speciate-local`
- Check NATS is running: `docker compose ps` (should show `speciate-nats` as `Up`)
- Test connectivity: `ping nats` (from inside devcontainer)

### NATS WebSocket connection refused
- Check NATS config is mounted: `docker exec speciate-nats cat /etc/nats/nats-server.conf`
- Verify WebSocket port is exposed: `docker port speciate-nats 9224`
- Check WebSocket is active in logs: `docker logs speciate-nats | grep -i websocket`
- Test from browser console: `new WebSocket('ws://localhost:9224')`

## References
- **Docker Compose Config**: `infrastructure/local/docker-compose.yml`
- **NATS Documentation**: https://docs.nats.io/
- **Devcontainer Config**: `.devcontainer/docker-compose.yml`
- **Streaming Architecture**: `docs/architecture/streaming-architecture.md`
