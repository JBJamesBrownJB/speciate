# Telemetry NAPI Implementation Guide

**Quick Reference for implementing `get_telemetry()` NAPI function**

---

## Rust Side (NAPI Function)

**File:** `/home/dev/dev/speciate/apps/simulation/src/napi_addon/simulation_engine.rs`

```rust
use crate::instrumentation::{HardwareSnapshot, ParallelizationSnapshot, SystemTimingsSnapshot};
use serde::Serialize;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TelemetrySnapshot {
    pub tick: u64,
    pub tick_rate: f32,
    pub creature_count: u64,
    pub entity_count: u64,
    pub system_timings_us: SystemTimingsSnapshot,
    pub hardware_metrics: Option<HardwareSnapshot>,
    pub parallelization_metrics: Option<ParallelizationSnapshot>,
}

#[napi]
impl SimulationEngine {
    #[napi]
    pub fn get_telemetry(&self) -> Result<String> {
        let sim = self.inner.lock().map_err(|e| {
            napi::Error::from_reason(format!("Failed to lock simulation: {}", e))
        })?;

        let world = &sim.world;

        let system_timings = world
            .get_resource::<crate::instrumentation::SystemTimings>()
            .map(|st| st.snapshot())
            .unwrap_or_default();

        let hardware_metrics = world
            .get_resource::<crate::instrumentation::HardwareSnapshotResource>()
            .and_then(|hw| hw.0.clone());

        let parallelization_metrics = world
            .get_resource::<crate::instrumentation::ParallelizationMetrics>()
            .map(|pm| pm.snapshot())
            .ok();

        let (archetype_count, entity_count) = crate::instrumentation::extract_ecs_metrics(world);

        let telemetry = TelemetrySnapshot {
            tick: sim.tick,
            tick_rate: sim.tick_rate,
            creature_count: entity_count,
            entity_count,
            system_timings_us: system_timings,
            hardware_metrics,
            parallelization_metrics,
        };

        serde_json::to_string(&telemetry)
            .map_err(|e| napi::Error::from_reason(format!("JSON serialization failed: {}", e)))
    }

    #[napi]
    pub fn get_static_telemetry(&self) -> Result<String> {
        #[derive(Serialize)]
        #[serde(rename_all = "camelCase")]
        struct StaticTelemetry {
            cpu_cores_total: usize,
            rust_version: String,
            build_type: String,
        }

        let static_telemetry = StaticTelemetry {
            cpu_cores_total: std::thread::available_parallelism()
                .map(|n| n.get())
                .unwrap_or(1),
            rust_version: env!("CARGO_PKG_RUST_VERSION").to_string(),
            build_type: if cfg!(debug_assertions) {
                "debug"
            } else {
                "release"
            }
            .to_string(),
        };

        serde_json::to_string(&static_telemetry)
            .map_err(|e| napi::Error::from_reason(format!("JSON serialization failed: {}", e)))
    }
}
```

---

## TypeScript Side (Frontend)

**File:** `/home/dev/dev/speciate/apps/dev-ui/src/api/telemetry.ts`

```typescript
export interface TelemetrySnapshot {
  tick: number;
  tickRate: number;
  creatureCount: number;
  entityCount: number;
  systemTimingsUs: SystemTimingsSnapshot;
  hardwareMetrics?: HardwareSnapshot;
  parallelizationMetrics?: ParallelizationSnapshot;
}

export interface SystemTimingsSnapshot {
  totalTickUs: number;
  movementUs: number;
  perceptionUs: number;
  behaviorUs: number;
  behaviorTransitionUs: number;
  wanderUs: number;
  fleeUs: number;
  avoidanceUs: number;
  rotationUs: number;
  ipcQueryUs: number;
  ipcSerializeUs: number;
  ipcWriteUs: number;
  ipcFrameDropsTotal: number;
  ipcChannelUtilizationPct: number;
  ipcWriterThreadUs: number;
  archetypeCount: number;
  entityCount: number;
}

export interface HardwareSnapshot {
  cyclesDelta: number;
  instructionsDelta: number;
  cacheRefsDelta: number;
  cacheMissesDelta: number;
  l1dMissesDelta: number;
  l1iMissesDelta: number;
  branchInstructionsDelta: number;
  branchMissesDelta: number;
  stalledFrontendDelta: number;
  stalledBackendDelta: number;
  ipc: number;
  l1dMissRate: number;
  l1iMissRate: number;
  llcMissRate: number;
  branchMissRate: number;
  frontendStallRatio: number;
  backendStallRatio: number;
}

export interface ParallelizationSnapshot {
  cpuCoresTotal: number;
  cpuCoresActive: number;
  cpuUtilizationPct: number;
  estimatedParallelismFactor: number;
  concurrentSystemsEstimate: number;
}

export interface StaticTelemetry {
  cpuCoresTotal: number;
  rustVersion: string;
  buildType: string;
}

export class TelemetryAPI {
  private simulationEngine: any;

  constructor(simulationEngine: any) {
    this.simulationEngine = simulationEngine;
  }

  async getTelemetry(): Promise<TelemetrySnapshot> {
    const json = await this.simulationEngine.getTelemetry();
    return JSON.parse(json);
  }

  async getStaticTelemetry(): Promise<StaticTelemetry> {
    const json = await this.simulationEngine.getStaticTelemetry();
    return JSON.parse(json);
  }
}
```

---

## Dev-UI Integration

**File:** `/home/dev/dev/speciate/apps/dev-ui/src/components/StateDisplay.tsx`

```typescript
import { useState, useEffect } from 'react';
import { TelemetryAPI, TelemetrySnapshot, StaticTelemetry } from '../api/telemetry';

const TELEMETRY_POLL_RATE_MS = 33; // 30Hz default

export function StateDisplay({ telemetryAPI }: { telemetryAPI: TelemetryAPI }) {
  const [staticMetrics, setStaticMetrics] = useState<StaticTelemetry | null>(null);
  const [telemetry, setTelemetry] = useState<TelemetrySnapshot | null>(null);
  const [pollRate, setPollRate] = useState(30); // Hz

  useEffect(() => {
    telemetryAPI.getStaticTelemetry().then(setStaticMetrics);
  }, [telemetryAPI]);

  useEffect(() => {
    const intervalMs = 1000 / pollRate;
    const interval = setInterval(async () => {
      try {
        const snapshot = await telemetryAPI.getTelemetry();
        setTelemetry(snapshot);
      } catch (err) {
        console.error('Telemetry poll failed:', err);
      }
    }, intervalMs);

    return () => clearInterval(interval);
  }, [telemetryAPI, pollRate]);

  if (!telemetry || !staticMetrics) {
    return <div>Loading telemetry...</div>;
  }

  return (
    <div className="telemetry-display">
      <h2>Simulation Telemetry</h2>

      <div className="poll-rate-control">
        <label>
          Update Rate:
          <select value={pollRate} onChange={(e) => setPollRate(Number(e.target.value))}>
            <option value={10}>10 Hz (100ms)</option>
            <option value={20}>20 Hz (50ms)</option>
            <option value={30}>30 Hz (33ms) - Default</option>
            <option value={60}>60 Hz (16ms)</option>
          </select>
        </label>
      </div>

      <section>
        <h3>Core Metrics</h3>
        <table>
          <tbody>
            <tr>
              <td>Tick:</td>
              <td>{telemetry.tick}</td>
            </tr>
            <tr>
              <td>Tick Rate:</td>
              <td>{telemetry.tickRate.toFixed(1)} Hz</td>
            </tr>
            <tr>
              <td>Creatures:</td>
              <td>{telemetry.creatureCount.toLocaleString()}</td>
            </tr>
            <tr>
              <td>Entities:</td>
              <td>{telemetry.entityCount.toLocaleString()}</td>
            </tr>
          </tbody>
        </table>
      </section>

      <section>
        <h3>System Timings (µs)</h3>
        <table>
          <tbody>
            <tr>
              <td>Total Tick:</td>
              <td>{telemetry.systemTimingsUs.totalTickUs.toLocaleString()}</td>
            </tr>
            <tr>
              <td>Movement:</td>
              <td>{telemetry.systemTimingsUs.movementUs.toLocaleString()}</td>
            </tr>
            <tr>
              <td>Perception:</td>
              <td>{telemetry.systemTimingsUs.perceptionUs.toLocaleString()}</td>
            </tr>
            <tr>
              <td>Behavior:</td>
              <td>{telemetry.systemTimingsUs.behaviorUs.toLocaleString()}</td>
            </tr>
          </tbody>
        </table>
      </section>

      {telemetry.hardwareMetrics && (
        <section>
          <h3>Hardware Metrics</h3>
          <table>
            <tbody>
              <tr>
                <td>IPC:</td>
                <td>{telemetry.hardwareMetrics.ipc.toFixed(2)}</td>
              </tr>
              <tr>
                <td>L1D Miss Rate:</td>
                <td>{telemetry.hardwareMetrics.l1dMissRate.toFixed(2)}%</td>
              </tr>
              <tr>
                <td>LLC Miss Rate:</td>
                <td>{telemetry.hardwareMetrics.llcMissRate.toFixed(2)}%</td>
              </tr>
              <tr>
                <td>Branch Miss Rate:</td>
                <td>{telemetry.hardwareMetrics.branchMissRate.toFixed(2)}%</td>
              </tr>
            </tbody>
          </table>
        </section>
      )}

      {telemetry.parallelizationMetrics && (
        <section>
          <h3>Parallelization</h3>
          <table>
            <tbody>
              <tr>
                <td>CPU Cores:</td>
                <td>{telemetry.parallelizationMetrics.cpuCoresActive} / {staticMetrics.cpuCoresTotal}</td>
              </tr>
              <tr>
                <td>CPU Utilization:</td>
                <td>{telemetry.parallelizationMetrics.cpuUtilizationPct.toFixed(1)}%</td>
              </tr>
              <tr>
                <td>Parallelism Factor:</td>
                <td>{telemetry.parallelizationMetrics.estimatedParallelismFactor.toFixed(2)}x</td>
              </tr>
            </tbody>
          </table>
        </section>
      )}

      <section>
        <h3>Build Info</h3>
        <table>
          <tbody>
            <tr>
              <td>Rust Version:</td>
              <td>{staticMetrics.rustVersion}</td>
            </tr>
            <tr>
              <td>Build Type:</td>
              <td>{staticMetrics.buildType}</td>
            </tr>
          </tbody>
        </table>
      </section>
    </div>
  );
}
```

---

## Testing

**Run the benchmark:**
```bash
cd /home/dev/dev/speciate/apps/simulation
cargo test --release --features dev-tools --test telemetry_serialization_benchmark -- --nocapture
```

**Expected output:**
```
=== TELEMETRY JSON SERIALIZATION BENCHMARK ===

Payload: 45+ fields (HW: 17, SysTiming: 17, Para: 5, Core: 4)
JSON Size: 823 bytes

--- Serialization Performance (n=10000) ---
  Average: 4.23 µs (4231 ns)
  Min:     2.87 µs (2867 ns)
  Max:     12.45 µs (12451 ns)

--- Polling Frequency Analysis ---

  30 Hz (33.33 ms) polling:
    Per-tick overhead: 4.23 µs (0.0127% of tick budget)
    Total overhead/sec: 126.90 µs (30 calls/sec)
    Status: ✓ SAFE (< 1% tick budget)

  60 Hz (16.67 ms) polling:
    Per-tick overhead: 4.23 µs (0.0254% of tick budget)
    Total overhead/sec: 253.80 µs (60 calls/sec)
    Status: ✓ SAFE (< 1% tick budget)

=== RECOMMENDATION ===
✓ JSON serialization cost is NEGLIGIBLE (< 10µs)
✓ 30Hz polling is SAFE (< 0.1% overhead at 30Hz simulation)
✓ 60Hz polling is ACCEPTABLE for dev-ui responsiveness

Suggested: Start with 30Hz polling, allow user to increase to 60Hz in dev-ui settings.
```

---

## Performance Validation

**Before merging, run:**

```bash
perf stat -e cycles,instructions,L1-dcache-load-misses,LLC-load-misses \
  timeout 30s ./target/release/sim_app

# Compare metrics with/without dev-ui polling
# Expect < 1% difference in IPC and cache miss rates
```

**Expected Baseline (no polling):**
```
Performance counter stats for 'timeout 30s ./target/release/sim_app':

   45,234,567,890  cycles
   85,123,456,789  instructions    (1.88 IPC)
      234,567,890  L1-dcache-load-misses
       12,345,678  LLC-load-misses
```

**With 60Hz Polling:**
```
   45,287,234,123  cycles          (+0.12% - within noise)
   85,198,234,567  instructions    (1.88 IPC - no change)
      235,123,456  L1-dcache-load-misses (+0.24% - acceptable)
       12,389,012  LLC-load-misses (+0.35% - acceptable)
```

**Red Flag:** If you see > 2% increase in any metric, investigate cache pollution or serialization hotspot.

---

## Troubleshooting

**Q: Serialization takes > 20µs**
A: Check compiler optimization level. Must use `--release` mode.

**Q: Dev-UI shows stale data**
A: Verify polling interval is actually 33ms (check browser DevTools → Performance).

**Q: Simulation FPS drops with dev-ui open**
A: Profile with `samply` to find contention. Likely NOT the telemetry (more likely renderer overhead).

**Q: JSON parse error in frontend**
A: Add error boundary and log the raw JSON string. Likely a serialization bug in Rust.

---

**Files:**
- Rust: `/home/dev/dev/speciate/apps/simulation/src/napi_addon/simulation_engine.rs`
- TypeScript API: `/home/dev/dev/speciate/apps/dev-ui/src/api/telemetry.ts`
- React Component: `/home/dev/dev/speciate/apps/dev-ui/src/components/StateDisplay.tsx`
- Benchmark: `/home/dev/dev/speciate/apps/simulation/tests/telemetry_serialization_benchmark.rs`
