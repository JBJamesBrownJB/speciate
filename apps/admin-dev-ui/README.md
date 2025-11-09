# Speciate Dev Admin UI

**Local testing control panel for the Speciate simulation**

This is a lightweight web application that allows you to spawn creatures, run test scenarios, and control the simulation for rapid visual testing during development.

## ⚠️ Development Only

This app is **FOR LOCAL TESTING ONLY** and should **NEVER** be deployed to production. It communicates directly with NATS and has no authentication or security measures.

---

## Quick Start

### 1. Start NATS (if not already running)

```bash
cd /workspace/infrastructure/local
docker compose up -d nats
```

NATS should be available at:
- `localhost:4222` (TCP client connections)
- `localhost:9224` (WebSocket - used by this admin UI)

### 2. Start the Simulation (with dev-commands feature)

```bash
cd /workspace/apps/simulation
cargo run --features dev-commands
```

The simulation will log:
```
[DEV] Dev commands enabled - listening on dev.sim.*
```

### 3. Serve the Admin UI

From this directory (`/workspace/apps/admin-dev-ui`):

```bash
# Option 1: Python
python3 -m http.server 8000

# Option 2: Node.js (npx)
npx serve -p 8000

# Option 3: PHP
php -S localhost:8000
```

### 4. Open in Browser

Navigate to:
```
http://localhost:8000
```

You should see "Connected" in green at the top if NATS is running.

---

## Features

### 🎯 Quick Scenarios

Pre-configured test scenarios for rapid visual testing:

- **Two Seekers Intercept** - Test collision avoidance during head-on approach
- **Wanderer Crowd** - 15 wanderers testing personal space and flocking
- **Seeker + Obstacles** - Navigation through a field of catatonic obstacles
- **Ring of Death** - 8 creatures seeking center (multi-body collision stress test)

Click any scenario button to instantly spawn the configured creatures.

### ✏️ Manual Spawn

Spawn individual creatures with custom parameters:

- **Position** - X, Y coordinates in meters
- **Behavior** - Seeking, Wandering, or Catatonic (obstacle)
- **Target** - Target position for seekers
- **Energy** - Initial energy (optional)
- **Max Speed** - Maximum speed in m/s (optional)

### 🔧 Utilities

- **Clear All Creatures** - Remove all creatures from the simulation (instant reset)
- **Speed Control** - Adjust simulation speed from 0.25x (slow-mo) to 5x (fast-forward)

### 📊 Activity Log

Real-time log of all commands sent and their status. Logs are also output to browser console.

---

## How It Works

### Architecture Flow

```
Admin UI (JavaScript)
    ↓ publish NATS message (JSON) via WebSocket (ws://localhost:9224)
NATS Broker (Docker container)
    ↓ dev.sim.> wildcard subscription (TCP localhost:4222)
Simulation Dev Command Listener (Rust thread)
    ↓ crossbeam channel
Bevy ECS System (process_dev_commands_system)
    ↓ spawn/modify entities
Simulation World
    ↓ NATS publisher (speciate.crits.transform)
Portal UI (PixiJS)
    ✓ Visual feedback!
```

### NATS Subjects

All dev commands are published to subjects under `dev.sim.*`:

- `dev.sim.spawn` - Spawn creature
- `dev.sim.clear` - Clear all creatures
- `dev.sim.speed` - Adjust simulation speed

### Message Format

Commands are sent as JSON:

**Spawn:**
```json
{
  "type": "Spawn",
  "x": 100.0,
  "y": 50.0,
  "behavior": "seeking",
  "target_x": 200.0,
  "target_y": 200.0,
  "energy": 100.0,
  "max_speed": 20.0
}
```

**Clear:**
```json
{
  "type": "Clear"
}
```

**Speed:**
```json
{
  "type": "Speed",
  "multiplier": 2.0
}
```

---

## Tech Stack

- **Vanilla HTML/CSS/JavaScript** - No framework overhead
- **NATS.ws** - WebSocket client for NATS (loaded from CDN)
- **ES Modules** - Modern JavaScript module system

---

## File Structure

```
admin-dev-ui/
├── index.html       # Main UI layout
├── styles.css       # Styling (dark theme)
├── app.js           # Main application logic
├── nats-client.js   # NATS WebSocket client
├── scenarios.js     # Scenario templates
└── README.md        # This file
```

---

## Troubleshooting

### "Disconnected" Status (Red)

**Problem:** Admin UI can't connect to NATS WebSocket.

**Solutions:**
1. Check NATS is running: `docker ps | grep nats`
2. Verify NATS WebSocket is active:
   ```bash
   docker logs speciate-nats | grep -i websocket
   # Expected: [INF] Listening for websocket clients on ws://0.0.0.0:9224
   ```
3. Test WebSocket port from browser console (F12):
   ```javascript
   new WebSocket('ws://localhost:9224')
   ```
4. Verify NATS config is loaded:
   ```bash
   docker exec speciate-nats cat /etc/nats/nats-server.conf
   # Should show: websocket { port: 9224 ... }
   ```
5. Check browser console for detailed connection errors (F12)

### Commands Not Working

**Problem:** UI is connected but creatures don't spawn.

**Solutions:**
1. Verify simulation is running with `--features dev-commands`:
   ```bash
   cargo run --features dev-commands
   ```
2. Check simulation logs for `[DEV] Dev commands enabled`
3. Check simulation logs for `[DEV] Command received: ...`
4. Verify Portal is connected and rendering (localhost:5173)

### Browser Compatibility

**Requirements:**
- Modern browser with ES modules support
- Chrome 61+, Firefox 60+, Safari 11+, Edge 79+

**CORS Issues:**
- Must serve from HTTP server (not `file://`)
- NATS.ws uses WebSocket (no CORS restrictions)

---

## Development Notes

### Adding New Scenarios

Edit `scenarios.js` and add a new function:

```javascript
export const scenarios = {
    // ... existing scenarios

    myNewScenario: () => {
        return [
            {
                type: "Spawn",
                x: 0.0,
                y: 0.0,
                behavior: "wandering"
            }
            // ... more spawns
        ];
    }
};
```

Then add a button in `index.html`:

```html
<button class="scenario-btn" data-scenario="myNewScenario">
    <span class="icon">✨</span>
    <span class="label">My New Scenario</span>
    <span class="desc">Description here</span>
</button>
```

### Adding New Commands

1. Add command type to Rust enum in `apps/simulation/src/dev_commands/commands.rs`
2. Add handler in `apps/simulation/src/dev_commands/systems.rs`
3. Add publish function in `nats-client.js`
4. Add UI controls in `index.html` and `app.js`

---

## Security Notes

🔒 **This app bypasses ALL security and sends commands directly to the simulation.**

- **No authentication** - Anyone on localhost can send commands
- **No validation** - Commands are trusted implicitly
- **No rate limiting** - Can spam unlimited commands
- **Feature flag required** - Simulation must explicitly enable `dev-commands`
- **Production builds** - Dev commands are completely compiled out in release

**Never enable this in production!**

---

## License

Part of the Speciate project. For development use only.
